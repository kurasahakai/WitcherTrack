#![feature(iter_intersperse)]

use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::null_mut;
use std::{fs, slice};

use anyhow::{anyhow, Result};
use leptonica_sys::{
    boxCreate, boxDestroy, boxGetGeometry, boxaDestroy, boxaGetBox, boxaGetCount, kernelDestroy,
    lept_free, makeGaussianKernel, pixClipRectangle, pixConnCompBB, pixConvertRGBToGray,
    pixConvolveRGB, pixCreate, pixDestroy, pixDilateBrick, pixGetDepth, pixGetHeight, pixGetWidth,
    pixInvert, pixRasterop, pixReadMem, pixThresholdToBinary, pixWriteMemPng, Pix, L_CLONE,
    PIX_SRC,
};
use tesseract_sys::{
    TessBaseAPI, TessBaseAPICreate, TessBaseAPIDelete, TessBaseAPIGetUTF8Text, TessBaseAPIInit3,
    TessBaseAPISetImage2, TessDeleteText,
};

pub mod data;
pub mod screenshot;

const ENG_TRAINEDDATA_URL: &str =
    "https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata";

const TRAINEDDATA_PATH: &str = "./eng.traineddata";

/// Download english Tesseract trained data if it is not present.
pub fn download_trained_data() -> Result<()> {
    if Path::new(TRAINEDDATA_PATH).exists() {
        return Ok(());
    }

    let response = ureq::get(ENG_TRAINEDDATA_URL).call()?;
    let mut bytes = Vec::with_capacity(response.header("Content-Length").unwrap().parse()?);
    response.into_reader().read_to_end(&mut bytes)?;
    fs::write(TRAINEDDATA_PATH, bytes)?;

    Ok(())
}

/// Process picture to obtain something that's easy to extract OCR from.
///
/// # Safety
///
/// haha
pub unsafe fn preprocess(picture: Picture) -> Result<Picture> {
    let mut kern = makeGaussianKernel(2, 2, 0.3, 1.0);
    let picture = Picture::from(pixConvolveRGB(picture.pix, kern));
    kernelDestroy(&mut kern);

    // Convert to grayscale.
    let picture = Picture::from(pixConvertRGBToGray(picture.pix, 0.0, 0.0, 0.0));

    // Threshold the picture.
    let threshold = Picture::from(pixThresholdToBinary(picture.pix, 140));
    let target = Picture::from(pixCreate(
        pixGetWidth(threshold.pix),
        pixGetHeight(threshold.pix),
        pixGetDepth(threshold.pix),
    ));
    pixInvert(threshold.pix, threshold.pix);
    pixDilateBrick(threshold.pix, threshold.pix, 2, 2);

    let mut boxa = pixConnCompBB(threshold.pix, 4);
    for i in 0..boxaGetCount(boxa) {
        let boxx = boxaGetBox(boxa, i, L_CLONE);
        let mut p = [0i32; 4];
        boxGetGeometry(boxx, &mut p[0], &mut p[1], &mut p[2], &mut p[3]);
        let [x, y, w, h] = p;
        let area = w * h;
        let aspect_ratio = (w as f32) / (h as f32);

        if aspect_ratio < 3.5 && area < 3000 {
            pixRasterop(target.pix, x, y, w, h, PIX_SRC as _, threshold.pix, x, y);
        }
    }
    boxaDestroy(&mut boxa);
    pixInvert(target.pix, target.pix);

    Ok(target)
}

// RAII picture
pub struct Picture {
    pix: *mut Pix,
}

impl Picture {
    pub fn from_mem(mem: Vec<u8>) -> Self {
        Picture::from(unsafe { pixReadMem(mem.as_ptr(), mem.len()) })
    }

    pub fn pix(&self) -> *mut Pix {
        self.pix
    }

    pub fn is_null(&self) -> bool {
        self.pix.is_null()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        let mut ptr: *mut u8 = null_mut();
        let mut size = 0usize;
        unsafe { pixWriteMemPng(&mut ptr, &mut size, self.pix, 0.0) };
        data.extend(unsafe { slice::from_raw_parts(ptr, size) });
        unsafe { lept_free(ptr as *mut _) };

        data
    }

    /// Crop the center-leftmost part.
    pub fn into_cropped(self, width_pct: f32, height_pct: f32) -> Result<Self> {
        let pix = unsafe {
            let width = pixGetWidth(self.pix) as f32;
            let height = pixGetHeight(self.pix) as f32;

            if width == 0. || height == 0. {
                return Err(anyhow!("Width and height are {width} {height}, can't crop"));
            }

            let new_y = height * (1. - height_pct) / 2.;
            let new_width = width * width_pct;
            let new_height = height * height_pct;

            let mut boxx = boxCreate(0, new_y as i32, new_width as i32, new_height as i32);
            let pix = pixClipRectangle(self.pix, boxx, null_mut());
            boxDestroy(&mut boxx);
            pix
        };

        Ok(Self::from(pix))
    }
}

impl From<*mut Pix> for Picture {
    fn from(pix: *mut Pix) -> Self {
        Self { pix }
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            pixDestroy(&mut self.pix);
        }
    }
}

/// RAII wrapper around Tesseract API
pub struct OcrReader {
    handle: *mut TessBaseAPI,
}

unsafe impl Send for OcrReader {}
unsafe impl Sync for OcrReader {}

impl OcrReader {
    /// Construct a new instance.
    pub fn new() -> Result<Self> {
        let tessdata_path = CString::new(".").unwrap();
        let language = CString::new("eng").unwrap();

        let handle = unsafe { TessBaseAPICreate() };
        unsafe { TessBaseAPIInit3(handle, tessdata_path.as_ptr(), language.as_ptr()) };

        Ok(Self { handle })
    }

    /// Run OCR on a picture.
    pub fn get_ocr(&self, image: &Picture) -> Result<String> {
        unsafe { TessBaseAPISetImage2(self.handle, image.pix()) };

        let text = unsafe { TessBaseAPIGetUTF8Text(self.handle) };
        let text_str = unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned();

        unsafe { TessDeleteText(text) };

        Ok(text_str)
    }
}

impl Drop for OcrReader {
    fn drop(&mut self) {
        unsafe { TessBaseAPIDelete(self.handle) };
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use std::{fs, thread};

    use leptonica_sys::{boxCreate, boxDestroy, pixClipRectangle, pixWritePng};

    use super::*;
    use crate::data::text_preprocess;

    const X: i32 = 336;
    const Y: i32 = 15;
    const W: i32 = 1584;
    const H: i32 = 891;

    const CW: i32 = W / 2;
    const CH: i32 = H / 6;
    const CY: i32 = (H - CH) / 2;

    fn crop_pic(data: &[u8]) -> Picture {
        unsafe {
            let mut pix1 = pixReadMem(data.as_ptr(), data.len());
            let mut box1 = boxCreate(X, Y, W, H);
            let mut box2 = boxCreate(0, CY, CW, CH);
            let mut pix2 = pixClipRectangle(pix1, box1, null_mut());
            let pix3 = pixClipRectangle(pix2, box2, null_mut());
            boxDestroy(&mut box1);
            boxDestroy(&mut box2);
            pixDestroy(&mut pix1);
            pixDestroy(&mut pix2);
            pix3.into()
        }
    }

    fn ocr(ocr_reader: &OcrReader, path: &str) {
        let start = Instant::now();
        let data = fs::read(path).unwrap();
        let pic = crop_pic(&data);
        let elapsed = start.elapsed();
        println!("{path}: {:?} took {elapsed:?}", ocr_reader.get_ocr(&pic).map(text_preprocess));
    }

    fn preprocess_fn<P: AsRef<Path>>(path: P) {
        let path = path.as_ref();
        let filename = format!("prep-{}", path.file_name().unwrap().to_string_lossy());
        let mut dest_path = path.parent().unwrap().to_path_buf().join(filename);
        dest_path.set_extension("png");
        let dest_path = CString::new(dest_path.to_str().unwrap()).unwrap();

        let data = fs::read(path).unwrap();
        let pic = unsafe { preprocess(crop_pic(&data)).unwrap() };
        unsafe { pixWritePng(dest_path.as_ptr(), pic.pix, 0.) };
    }

    #[test]
    fn test_preprocess() {
        preprocess_fn("tests/fixtures/alchemy01.jpg");
        preprocess_fn("tests/fixtures/alchemy02.jpg");
        preprocess_fn("tests/fixtures/alchemy03.jpg");
        preprocess_fn("tests/fixtures/alchemy04.jpg");
        preprocess_fn("tests/fixtures/alchemy05.jpg");
        preprocess_fn("tests/fixtures/diagram01.jpg");
        preprocess_fn("tests/fixtures/diagram02.jpg");
        preprocess_fn("tests/fixtures/diagram03.jpg");
        preprocess_fn("tests/fixtures/diagram04.jpg");
        preprocess_fn("tests/fixtures/diagram05.jpg");
        preprocess_fn("tests/fixtures/diagram06.jpg");
        preprocess_fn("tests/fixtures/quest01.jpg");
        preprocess_fn("tests/fixtures/quest02.jpg");
        preprocess_fn("tests/fixtures/quest03.jpg");
    }

    #[test]
    fn test_ocr() {
        let ocr_reader = OcrReader::new().unwrap();
        ocr(&ocr_reader, "tests/fixtures/alchemy01.jpg");
        ocr(&ocr_reader, "tests/fixtures/alchemy02.jpg");
        ocr(&ocr_reader, "tests/fixtures/alchemy03.jpg");
        ocr(&ocr_reader, "tests/fixtures/alchemy04.jpg");
        ocr(&ocr_reader, "tests/fixtures/alchemy05.jpg");
        ocr(&ocr_reader, "tests/fixtures/diagram01.jpg");
        ocr(&ocr_reader, "tests/fixtures/diagram02.jpg");
        ocr(&ocr_reader, "tests/fixtures/diagram03.jpg");
        ocr(&ocr_reader, "tests/fixtures/diagram04.jpg");
        ocr(&ocr_reader, "tests/fixtures/diagram05.jpg");
        ocr(&ocr_reader, "tests/fixtures/diagram06.jpg");
        ocr(&ocr_reader, "tests/fixtures/quest01.jpg");
        ocr(&ocr_reader, "tests/fixtures/quest02.jpg");
        ocr(&ocr_reader, "tests/fixtures/quest03.jpg");
    }

    #[test]
    fn test_one() {
        preprocess_fn("tests/fixtures/alchemy01.jpg");
    }

    #[test]
    #[ignore]
    fn test_ocr_live() {
        download_trained_data().unwrap();
        let ocr_reader = OcrReader::new().unwrap();
        loop {
            let screenshot =
                screenshot::capture().and_then(|pic| unsafe { preprocess(pic) }).unwrap();

            let ocr = ocr_reader.get_ocr(&screenshot);
            println!("---\n{ocr:?}\n\n");
            fs::write("foo.png", screenshot.to_vec()).unwrap();

            thread::sleep(Duration::from_millis(2000));
        }
    }
}
