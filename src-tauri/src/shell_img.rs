//! Ícones e miniaturas nativas (as mesmas do Explorer). Windows: IShellItemImageFactory.

#[cfg(windows)]
pub fn shell_image(name: &str, size: i32, thumb: bool) -> Option<Vec<u8>> {
    use std::ffi::c_void;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::SIZE;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::*;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let factory: IShellItemImageFactory = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None).ok()?;

        // Miniatura real quando existe; senão o ícone puro (o shell devolve uma "placa" com relevo atrás de pastas/arquivos genéricos).
        let sz = SIZE { cx: size, cy: size };
        let hbmp = (if thumb { factory.GetImage(sz, SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK).ok() } else { None })
            .or_else(|| factory.GetImage(sz, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK).ok())?;

        let mut bm = BITMAP::default();
        if GetObjectW(
            HGDIOBJ(hbmp.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut _ as *mut c_void),
        ) == 0
        {
            let _ = DeleteObject(HGDIOBJ(hbmp.0));
            return None;
        }

        let (w, h) = (bm.bmWidth, bm.bmHeight);
        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buf = vec![0u8; (w * h * 4) as usize];
        let hdc = GetDC(None);
        let lines = GetDIBits(hdc, hbmp, 0, h as u32, Some(buf.as_mut_ptr() as *mut c_void), &mut bi, DIB_RGB_COLORS);
        ReleaseDC(None, hdc);
        let _ = DeleteObject(HGDIOBJ(hbmp.0));
        if lines == 0 {
            return None;
        }

        // BGRA -> RGBA; se a imagem não tiver canal alfa (miniaturas), força opaco.
        let has_alpha = buf.chunks_exact(4).any(|p| p[3] != 0);
        for p in buf.chunks_exact_mut(4) {
            p.swap(0, 2);
            if !has_alpha {
                p[3] = 255;
            }
        }

        let img = image::RgbaImage::from_raw(w as u32, h as u32, buf)?;
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png).ok()?;
        Some(out)
    }
}

#[cfg(not(windows))]
pub fn shell_image(_name: &str, _size: i32, _thumb: bool) -> Option<Vec<u8>> {
    None
}
