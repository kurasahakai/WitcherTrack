use std::{
    ffi::{CStr, CString},
    fs,
    path::Path,
};

use anyhow::{anyhow, Result};
use leptonica_sys::pixRead;
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

    /// Load an image from a
    pub fn get_ocr(&self, image_path: &Path) -> Result<String> {
        let image_path = CString::new(
            image_path
                .to_str()
                .ok_or_else(|| anyhow!("Could not convert path to string: {image_path:?}"))?,
        )?;
        let image = unsafe { pixRead(image_path.as_ptr()) };

        unsafe { TessBaseAPISetImage2(self.handle, image) };

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
