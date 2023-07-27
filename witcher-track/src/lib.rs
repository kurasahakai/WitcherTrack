#![feature(iter_intersperse)]

use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::null_mut;
use std::{fs, slice};

use anyhow::{anyhow, Result};
use leptonica_sys::{
    boxCreate, boxDestroy, boxGetGeometry, boxaDestroy, boxaGetBox, boxaGetCount, kernelDestroy,
    lept_free, makeGaussianKernel, pixClipRectangle, pixConnCompBB, pixConvertRGBToGray,
    pixConvolveRGB, pixCreate, pixDestroy, pixDilateBrick, pixGetDepth, pixGetHeight,
    pixGetRGBPixel, pixGetWidth, pixInvert, pixRasterop, pixReadMem, pixSetRGBPixel,
    pixThresholdToBinary, pixWriteMemPng, Pix, L_CLONE, PIX_SRC,
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

// https://github.com/ChevyRay/color_space/blob/master/src/hsv.rs#L34
fn rgb_to_hsv(r: i32, g: i32, b: i32) -> (u8, u8, u8) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let min = r.min(g.min(b));
    let max = r.max(g.max(b));
    let delta = max - min;

    let v = max;
    let s = match max > 1e-3 {
        true => delta / max,
        false => 0.0,
    };
    let h = match delta == 0.0 {
        true => 0.0,
        false => {
            if r == max {
                (g - b) / delta
            } else if g == max {
                2.0 + (b - r) / delta
            } else {
                4.0 + (r - g) / delta
            }
        },
    };
    let h = ((h * 60.0) + 360.0) % 360.0;

    let h = h * 255. / 360.;
    let s = s * 255.;
    let v = v * 255.;
    (h as u8, s as u8, v as u8)
}

/// Process picture to obtain something that's easy to extract OCR from.
///
/// # Safety
///
/// haha
pub unsafe fn preprocess(picture: Picture) -> Result<Picture> {
    // let mut kern = makeGaussianKernel(2, 2, 0.3, 1.0);
    // let picture = Picture::from(pixConvolveRGB(picture.pix, kern));
    // kernelDestroy(&mut kern);
    //
    // New approach

    for y in 0..pixGetHeight(picture.pix) {
        for x in 0..pixGetWidth(picture.pix) {
            let (mut r, mut g, mut b) = (0, 0, 0);
            pixGetRGBPixel(picture.pix, x, y, &mut r, &mut g, &mut b);
            let (h, s, v) = rgb_to_hsv(r, g, b);

            let is_in_range = (h > 20 && h < 50) && (s > 60 && s < 120) && (v > 200);
            if !is_in_range {
                pixSetRGBPixel(picture.pix, x, y, 0, 0, 0);
            }
        }
    }
    let picture = Picture::from(pixConvertRGBToGray(picture.pix, 0.0, 0.0, 0.0));
    let picture = Picture::from(pixThresholdToBinary(picture.pix, 140));

    Ok(picture)
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

    use leptonica_sys::pixWritePng;

    use super::*;
    use crate::data::slugify;

    fn ocr(ocr_reader: &OcrReader, path: &str) {
        let start = Instant::now();
        let data = fs::read(path).unwrap();
        let pic = unsafe { preprocess(Picture::from_mem(data)).unwrap() };
        let elapsed = start.elapsed();
        println!("{path}: {:?} took {elapsed:?}", ocr_reader.get_ocr(&pic).map(slugify));
    }

    fn preprocess_fn<P: AsRef<Path>>(path: P) {
        let path = path.as_ref();
        let filename = format!("prep-{}", path.file_name().unwrap().to_string_lossy());
        let mut dest_path = path.parent().unwrap().to_path_buf().join(filename);
        dest_path.set_extension("png");
        let dest_path = CString::new(dest_path.to_str().unwrap()).unwrap();

        let data = fs::read(path).unwrap();
        let pic = unsafe { preprocess(Picture::from_mem(data)).unwrap() };
        unsafe { pixWritePng(dest_path.as_ptr(), pic.pix, 0.) };
    }

    #[test]
    fn test_preprocess() {
        preprocess_fn("tests/fixtures/immagine.jpg");
        preprocess_fn("tests/fixtures/immagine(1).jpg");
        preprocess_fn("tests/fixtures/immagine(2).jpg");
        preprocess_fn("tests/fixtures/immagine(3).jpg");
        preprocess_fn("tests/fixtures/immagine(4).jpg");
        preprocess_fn("tests/fixtures/immagine(5).jpg");
        preprocess_fn("tests/fixtures/immagine(6).jpg");
        preprocess_fn("tests/fixtures/immagine(7).jpg");
        preprocess_fn("tests/fixtures/immagine(8).jpg");
    }

    #[test]
    fn test_ocr() {
        let ocr_reader = OcrReader::new().unwrap();
        ocr(&ocr_reader, "tests/fixtures/immagine.jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(1).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(2).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(3).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(4).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(5).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(6).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(7).jpg");
        ocr(&ocr_reader, "tests/fixtures/immagine(8).jpg");
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
