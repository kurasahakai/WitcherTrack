use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::null_mut;
use std::{fs, slice};

use anyhow::{anyhow, Result};
use leptonica_sys::{
    lept_free, pixConvertRGBToGray, pixDestroy, pixThresholdToBinary, pixWriteMemPng, Pix,
};
use tesseract_sys::{
    TessBaseAPI, TessBaseAPICreate, TessBaseAPIDelete, TessBaseAPIGetUTF8Text, TessBaseAPIInit3,
    TessBaseAPISetImage2, TessDeleteText,
};

mod data;
mod screenshot;

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

// Process picture to obtain something that's easy to extract OCR from.
pub fn gray_and_threshold(picture: Picture) -> Result<Picture> {
    // Convert to grayscale.
    let grayscale = Picture::from(unsafe { pixConvertRGBToGray(picture.pix, 0.0, 0.0, 0.0) });
    if grayscale.is_null() {
        return Err(anyhow!("Could not convert picture to grayscale"));
    }

    // Threshold the picture.
    let threshold = Picture::from(unsafe { pixThresholdToBinary(grayscale.pix, 140) });
    if threshold.is_null() {
        return Err(anyhow!("Could not threshold picture"));
    }

    //

    Ok(threshold)
}

// RAII picture
pub struct Picture {
    pix: *mut Pix,
}

impl Picture {
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
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_ocr() {
        download_trained_data().unwrap();
        let ocr_reader = OcrReader::new().unwrap();
        loop {
            let screenshot =
                screenshot::capture_screenshot().and_then(screenshot::gray_and_threshold).unwrap();

            let ocr = ocr_reader.get_ocr(&screenshot);
            println!("---\n{ocr:?}\n\n");

            thread::sleep(Duration::from_millis(1000));
        }
    }
}
