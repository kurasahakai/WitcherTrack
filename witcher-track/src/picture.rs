//! Functions for loading and preprocessing pictures.

use std::ops::{Deref, Range};
use std::ptr::null_mut;
use std::slice;

use anyhow::{anyhow, Result};
use leptonica_sys::*;

use crate::HSV_RANGE;

/// RAII picture.
pub struct Picture {
    pix: *mut Pix,
}

impl Picture {
    /// Read image from memory.
    pub fn from_mem(mem: Vec<u8>) -> Self {
        Picture::from(unsafe { pixReadMem(mem.as_ptr(), mem.len()) })
    }

    /// Return pointer to leptonica Pix.
    pub fn pix(&self) -> *mut Pix {
        self.pix
    }

    /// Check if pointer is null.
    pub fn is_null(&self) -> bool {
        self.pix.is_null()
    }

    /// Convert to bytes vector.
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

impl Deref for Picture {
    type Target = *mut Pix;

    fn deref(&self) -> &Self::Target {
        &self.pix
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

/// Process picture to obtain something that's easy to extract OCR from.
///
/// # Safety
///
/// haha
pub unsafe fn preprocess(picture: Picture) -> Result<Picture> {
    const HRANGE: Range<u8> = HSV_RANGE.0;
    const SRANGE: Range<u8> = HSV_RANGE.1;
    const VRANGE: Range<u8> = HSV_RANGE.2;

    // Discard pixels outside of a narrow HSV range.
    let hsv_pic = Picture::from(pixConvertRGBToHSV(null_mut(), *picture));
    let bin_pic = Picture::from(pixCreate(pixGetWidth(*picture), pixGetHeight(*picture), 1));

    // Loop through all pixels.
    for y in 0..pixGetHeight(*picture) {
        for x in 0..pixGetWidth(*picture) {
            // Extract HSV point at (x, y).
            let (mut h, mut s, mut v) = (0, 0, 0);
            pixGetRGBPixel(*hsv_pic, x, y, &mut h, &mut s, &mut v);
            let (h, s, v) = (h as u8, s as u8, v as u8);

            let is_in_range = HRANGE.contains(&h) && SRANGE.contains(&s) && VRANGE.contains(&v);
            if !is_in_range {
                pixSetPixel(*bin_pic, x, y, 1);
            }
        }
    }

    Ok(bin_pic)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::CROP_RANGE;

    fn preprocess_and_save<P: AsRef<Path>>(path: P) {
        let path = path.as_ref();
        let filename = format!("prep-{}", path.file_name().unwrap().to_string_lossy());
        let mut dest_path = path.parent().unwrap().to_path_buf().join(filename);
        dest_path.set_extension("png");
        let dest_path = CString::new(dest_path.to_str().unwrap()).unwrap();

        let data = fs::read(path).unwrap();
        let cropped = Picture::from_mem(data).into_cropped(CROP_RANGE.0, CROP_RANGE.1).unwrap();
        let pic = unsafe { preprocess(cropped).unwrap() };
        unsafe { pixWritePng(dest_path.as_ptr(), pic.pix, 0.) };
    }

    #[test]
    fn test_preprocess() {
        preprocess_and_save("tests/fixtures/immagine.jpg");
        preprocess_and_save("tests/fixtures/immagine(1).jpg");
        preprocess_and_save("tests/fixtures/immagine(2).jpg");
        preprocess_and_save("tests/fixtures/immagine(3).jpg");
        preprocess_and_save("tests/fixtures/immagine(4).jpg");
        preprocess_and_save("tests/fixtures/immagine(5).jpg");
        preprocess_and_save("tests/fixtures/immagine(6).jpg");
        preprocess_and_save("tests/fixtures/immagine(7).jpg");
        preprocess_and_save("tests/fixtures/immagine(8).jpg");
    }
}
