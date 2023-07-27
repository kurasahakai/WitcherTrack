#![feature(iter_intersperse)]

use std::ffi::CStr;
use std::ptr::null_mut;

use anyhow::Result;
use picture::Picture;
use tesseract_sys::*;

pub mod data;
pub mod db;
pub mod picture;
pub mod screenshot;

// Tesseract trained data.
const TRAINED_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/eng.traineddata"));

/// RAII wrapper around Tesseract API
pub struct OcrReader {
    handle: *mut TessBaseAPI,
}

unsafe impl Send for OcrReader {}
unsafe impl Sync for OcrReader {}

impl OcrReader {
    /// Construct a new instance.
    pub fn new() -> Result<Self> {
        let handle = unsafe { TessBaseAPICreate() };
        unsafe {
            TessBaseAPIInit5(
                handle,
                TRAINED_DATA.as_ptr() as *const i8,
                TRAINED_DATA.len() as i32,
                b"eng\0".as_ptr() as *const i8,
                TessOcrEngineMode_OEM_DEFAULT,
                null_mut(),
                0,
                null_mut(),
                null_mut(),
                0,
                0,
            )
        };

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
    use std::fs;
    use std::time::Instant;

    use super::*;
    use crate::data::slugify;
    use crate::picture::preprocess;

    fn run_ocr(ocr_reader: &OcrReader, path: &str) {
        let start = Instant::now();
        let data = fs::read(path).unwrap();
        let pic = unsafe { preprocess(Picture::from_mem(data)).unwrap() };
        let elapsed = start.elapsed();
        println!("{path}: {:?} took {elapsed:?}", ocr_reader.get_ocr(&pic).map(slugify));
    }

    #[test]
    fn test_ocr() {
        let ocr_reader = OcrReader::new().unwrap();
        run_ocr(&ocr_reader, "tests/fixtures/immagine.jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(1).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(2).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(3).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(4).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(5).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(6).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(7).jpg");
        run_ocr(&ocr_reader, "tests/fixtures/immagine(8).jpg");
    }
}
