use anyhow::{anyhow, Result};
use leptonica_sys::pixReadMem;
use screenshots::Screen;
use windows::w;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetClientRect, GetWindowRect};

use crate::Picture;

pub fn capture() -> Result<Picture> {
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

unsafe fn get_witcher_rect() -> (i32, i32, u32, u32) {
    let title = w!("The Witcher 3");
    let hwnd = FindWindowW(None, title);
    let mut rect = RECT::default();

    GetClientRect(hwnd, &mut rect);

    let (left, top) = (rect.left, rect.top);

    GetWindowRect(hwnd, &mut rect);

    let (left, top) = (rect.left + left, rect.top + top);
    let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);

    (left, top, width as u32, height as u32)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::capture;
    use crate::preprocess;

    #[test]
    fn test_screenshot() {
        let screenshot = capture().unwrap();
        fs::write("foo.png", screenshot.to_vec()).unwrap();
        let thresh = unsafe { preprocess(screenshot) }.unwrap();
        fs::write("bar.png", thresh.to_vec()).unwrap();
    }
}
