use std::{fs, mem::size_of, path::Path, ptr};

use tokio_util::sync::CancellationToken;
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::SIZE,
        Graphics::Gdi::{
            DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ,
        },
        System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
        UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_THUMBNAILONLY},
    },
};

use super::ensure_not_cancelled;

const LONG_EDGE: i32 = 1024;

pub(super) fn save(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let source = source.to_path_buf();
    let target = target.to_path_buf();
    let cancellation = cancellation.clone();
    std::thread::Builder::new()
        .name("windows-shell-thumbnail".into())
        .spawn(move || save_on_sta(&source, &target, &cancellation))
        .map_err(|error| format!("Windows Shell thumbnail worker could not start: {error}"))?
        .join()
        .map_err(|_| "Windows Shell thumbnail worker panicked".to_owned())?
}

fn save_on_sta(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    ensure_not_cancelled(cancellation)?;
    let _apartment = ComApartment::enter()?;
    let path = HSTRING::from(source.as_os_str());
    let factory: IShellItemImageFactory = unsafe { SHCreateItemFromParsingName(&path, None) }
        .map_err(|error| format!("Windows Shell could not open this document: {error}"))?;
    let bitmap = unsafe {
        factory.GetImage(
            SIZE {
                cx: LONG_EDGE,
                cy: LONG_EDGE,
            },
            SIIGBF_THUMBNAILONLY,
        )
    }
    .map_err(|error| format!("No Windows thumbnail handler accepted this document: {error}"))?;
    let bitmap = OwnedBitmap(bitmap);
    ensure_not_cancelled(cancellation)?;
    let image = bitmap.rgba()?;
    let partial = target.with_extension("partial");
    let published = image
        .save_with_format(&partial, image::ImageFormat::Png)
        .map_err(|error| format!("Windows thumbnail could not be encoded as PNG: {error}"))
        .and_then(|()| ensure_not_cancelled(cancellation))
        .and_then(|()| fs::rename(&partial, target).map_err(|error| error.to_string()));
    if published.is_err() {
        let _ = fs::remove_file(partial);
    }
    published
}

struct ComApartment;

impl ComApartment {
    fn enter() -> Result<Self, String> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|error| format!("Windows Shell STA initialization failed: {error}"))?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct OwnedBitmap(HBITMAP);

impl OwnedBitmap {
    fn rgba(&self) -> Result<image::RgbaImage, String> {
        let mut bitmap = BITMAP::default();
        let read = unsafe {
            GetObjectW(
                HGDIOBJ::from(self.0),
                size_of::<BITMAP>() as i32,
                Some(ptr::from_mut(&mut bitmap).cast()),
            )
        };
        if read != size_of::<BITMAP>() as i32 || bitmap.bmWidth <= 0 || bitmap.bmHeight == 0 {
            return Err("Windows Shell returned an invalid thumbnail bitmap".into());
        }
        let width = bitmap.bmWidth as u32;
        let height = bitmap.bmHeight.unsigned_abs();
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        let mut info = BITMAPINFO::default();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let dc = unsafe { GetDC(None) };
        if dc.is_invalid() {
            return Err("Windows could not acquire a display context for the thumbnail".into());
        }
        let lines = unsafe {
            GetDIBits(
                dc,
                self.0,
                0,
                height,
                Some(pixels.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            )
        };
        let _ = unsafe { ReleaseDC(None, dc) };
        if lines != height as i32 {
            return Err("Windows could not read the Shell thumbnail bitmap".into());
        }
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        image::RgbaImage::from_raw(width, height, pixels)
            .ok_or_else(|| "Windows thumbnail dimensions did not match its bitmap".to_owned())
    }
}

impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        let _ = unsafe { DeleteObject(HGDIOBJ::from(self.0)) };
    }
}
