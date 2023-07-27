use anyhow::{anyhow, Result};
use leptonica_sys::pixReadMem;
use screenshots::Screen;
use windows::w;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetClientRect, GetWindowRect};

use crate::Picture;

/// Capture the entire Witcher's window.
pub fn capture() -> Result<Picture> {
    let (left, top, width, height) = unsafe { get_witcher_rect() };

    let screen = Screen::all()?.into_iter().next().ok_or_else(|| anyhow!("No screen found"))?;
    let image = screen.capture_area(left, top, width, height)?.to_png(None)?;

    Ok(Picture::from(unsafe { pixReadMem(image.as_ptr(), image.len()) }))
}

/// Return rectangle of the Witcher's window
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
