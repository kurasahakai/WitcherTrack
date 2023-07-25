use core::slice;
use std::ptr::null_mut;

use anyhow::{anyhow, Result};
use leptonica_sys::{
    lept_free, pixConvertRGBToGray, pixDestroy, pixReadMem, pixThresholdToBinary, pixWriteMemPng,
    Pix,
};
use screenshots::Screen;
use windows::{
    w,
    Win32::{
        Foundation::RECT,
        UI::WindowsAndMessaging::{FindWindowW, GetClientRect, GetWindowRect},
    },
};

// RAII picture
pub struct Picture {
    pix: *mut Pix,
}

impl Picture {
    fn is_null(&self) -> bool {
        self.pix.is_null()
    }

    fn to_vec(&self) -> Vec<u8> {
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

pub fn capture_screenshot() -> Result<Picture> {
    let (left, top, wnd_width, wnd_height) = unsafe { get_witcher_rect() };

    // Calculate the crop height as 50% of the window height.
    let height = (wnd_height as f32 * 0.5) as u32;
    // Calculate the top so that the cropped part is in the middle.
    let top = top + (((wnd_height - height) as f32) * 0.5) as i32;
    // Calculate the crop width as half of the window width.
    let width = (wnd_width as f32 * 0.5) as u32;

    let screen = Screen::all()?.into_iter().next().ok_or_else(|| anyhow!("No screen found"))?;
    let image = screen.capture_area(left, top, width, height)?.to_png(None)?;

    Ok(Picture::from(unsafe { pixReadMem(image.as_ptr(), image.len()) }))
}

pub fn gray_and_threshold(picture: Picture) -> Result<Picture> {
    let grayscale = Picture::from(unsafe { pixConvertRGBToGray(picture.pix, 0.0, 0.0, 0.0) });
    if grayscale.is_null() {
        return Err(anyhow!("Could not convert picture to grayscale"));
    }

    let threshold = Picture::from(unsafe { pixThresholdToBinary(grayscale.pix, 140) });
    if threshold.is_null() {
        return Err(anyhow!("Could not threshold picture"));
    }

    Ok(threshold)
}

unsafe fn get_witcher_rect() -> (i32, i32, u32, u32) {
    let title = w!("The Witcher 3");
    let hwnd = FindWindowW(None, title);
    let mut rect = RECT::default();

    GetClientRect(hwnd, &mut rect);
    println!("{rect:?}");

    let (left, top) = (rect.left, rect.top);

    GetWindowRect(hwnd, &mut rect);
    println!("{rect:?}");

    let (left, top) = (rect.left + left, rect.top + top);
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);

    (left, top, width as u32, height as u32)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{capture_screenshot, gray_and_threshold};

    #[test]
    fn test_screenshot() {
        let screenshot = capture_screenshot().unwrap();
        fs::write("foo.png", screenshot.to_vec()).unwrap();
        let thresh = gray_and_threshold(screenshot).unwrap();
        fs::write("bar.png", thresh.to_vec()).unwrap();
    }
}
