#![feature(iter_intersperse)]

use std::ffi::CStr;
use std::ops::Range;
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

pub struct Crop {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

// pub const CROP_RANGE: (f32, f32) = (0.6, 0.25);
pub const CROP_RANGE: Crop = Crop { left: 0.01, right: 0.5, top: 0.45, bottom: 0.35 };
pub const HSV_RANGE: (Range<u8>, Range<u8>, Range<u8>) = (0..50, 50..120, 200..255);
pub const STRSIM_THRESHOLD: f64 = 0.7;

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
                TessOcrEngineMode_OEM_LSTM_ONLY,
                null_mut(),
                0,
                null_mut(),
                null_mut(),
                0,
                1,
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

        println!("{text_str}");
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
    use crate::data::parse_action;
    use crate::picture::preprocess;

    fn run_ocr(ocr_reader: &OcrReader, path: &str) {
        let start = Instant::now();
        let data = fs::read(path).unwrap();

        let start_crop = Instant::now();
        let pic = Picture::from_mem(data).into_cropped().unwrap();
        let elapsed_crop = start_crop.elapsed();

        let start_preprocess = Instant::now();
        let pic = unsafe { preprocess(pic).unwrap() };
        let elapsed_preprocess = start_preprocess.elapsed();

        let start_ocr = Instant::now();
        let res = ocr_reader.get_ocr(&pic);
        let elapsed_ocr = start_ocr.elapsed();

        let start_parse = Instant::now();
        let tok = res.as_ref().ok().and_then(parse_action);
        let elapsed_parse = start_parse.elapsed();

        let elapsed = start.elapsed();
        println!("{path}\n{res:?}\n{tok:?}\nTook:");
        println!("  All         {elapsed:?}");
        println!("  Crop        {elapsed_crop:?}");
        println!("  Preprocess  {elapsed_preprocess:?}");
        println!("  Ocr         {elapsed_ocr:?}");
        println!("  Parse       {elapsed_parse:?}");
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
