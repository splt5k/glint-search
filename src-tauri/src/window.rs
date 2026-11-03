use crate::{theme, AppState};
use serde_json::json;
use std::sync::{atomic::Ordering, Arc};
use std::time::Instant;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition};

/// Janela transparente: o painel de vidro é desenhado pela UI dentro dela.
pub const WIN_W: f64 = 860.0;
pub const WIN_H: f64 = 150.0;
pub const WIN_MAX_H: f64 = 840.0;

pub fn toggle(app: &AppHandle) {
    let visible = app
        .get_webview_window("main")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    if visible {
        hide(app);
    } else {
        show(app);
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
        let _ = app.emit("hidden", ());
    }
}

pub fn show(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else { return };
    let state = app.state::<Arc<AppState>>();
    *state.shown_at.lock().unwrap() = Instant::now();

    let cursor = w.cursor_position().unwrap_or(PhysicalPosition::new(0.0, 0.0));
    let monitor = w
        .monitor_from_point(cursor.x, cursor.y)
        .ok()
        .flatten()
        .or_else(|| w.primary_monitor().ok().flatten());

    let mut bd_ok = false;
    if let Some(m) = monitor {
        let sf = m.scale_factor();
        let mp = m.position();
        let ms = m.size();
        let win_w_px = (WIN_W * sf) as i32;
        let x = mp.x + (ms.width as i32 - win_w_px) / 2;
        let y = mp.y + (ms.height as f64 * 0.16) as i32;

        // Captura do fundo (com a janela ainda oculta) para o efeito de vidro fosco.
        if let Some(jpeg) = capture_backdrop(x, y, win_w_px, (WIN_MAX_H * sf) as i32) {
            if let Some(p) = std::env::var_os("GLINT_DUMP_BD") {
                let _ = std::fs::write(p, &jpeg);
            }
            *state.backdrop.lock().unwrap() = Some(Arc::new(jpeg));
            bd_ok = true;
        }

        let _ = w.set_size(LogicalSize::new(WIN_W, WIN_H));
        let _ = w.set_position(PhysicalPosition::new(x, y));
    }

    let seq = state.backdrop_seq.fetch_add(1, Ordering::SeqCst) + 1;
    let t = theme::current();
    let r1 = w.show();
    let r2 = w.set_focus();
    crate::dlog!("show: {:?} focus: {:?} visible: {:?} pos: {:?} size: {:?}", r1, r2, w.is_visible(), w.outer_position(), w.outer_size());
    let _ = app.emit(
        "shown",
        json!({ "dark": t.dark, "accent": t.accent, "bd": if bd_ok { seq } else { 0 }, "w": WIN_W, "h": WIN_MAX_H }),
    );

    // Linux (X11): o gerenciador de janelas costuma recusar o foco do teclado a uma janela que aparece por atalho global.
    // Insistimos algumas vezes logo depois de abrir e avisamos a interface para recolocar o cursor no campo.
    #[cfg(target_os = "linux")]
    {
        let app = app.clone();
        std::thread::spawn(move || {
            for ms in [40u64, 100, 200, 400] {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                if let Some(w) = app.get_webview_window("main") {
                    if !w.is_visible().unwrap_or(false) {
                        return;
                    }
                    let _ = w.set_focus();
                }
                let _ = app.emit("refocus", ());
            }
        });
    }
}

/// Linux: sem captura do fundo por enquanto (a UI cai no vidro com véu sólido).
#[cfg(not(windows))]
fn capture_backdrop(_x: i32, _y: i32, _w: i32, _h: i32) -> Option<Vec<u8>> {
    None
}

#[cfg(windows)]
fn capture_backdrop(x: i32, y: i32, w: i32, h: i32) -> Option<Vec<u8>> {
    let mon = xcap::Monitor::from_point(x, y).ok()?;
    let rx = (x - mon.x().ok()?).max(0) as u32;
    let ry = (y - mon.y().ok()?).max(0) as u32;
    let mw = mon.width().ok()?;
    let mh = mon.height().ok()?;
    let cw = (w as u32).min(mw.saturating_sub(rx));
    let ch = (h as u32).min(mh.saturating_sub(ry));
    if cw == 0 || ch == 0 {
        return None;
    }

    let img = mon.capture_region(rx, ry, cw, ch).ok()?;
    // Reduz (será borrado de qualquer forma) e comprime.
    let small = image::imageops::thumbnail(&img, (cw / 3).max(1), (ch / 3).max(1));
    let rgb = image::DynamicImage::ImageRgba8(small).to_rgb8();
    let mut out = Vec::new();
    let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 78);
    rgb.write_with_encoder(enc).ok()?;
    Some(out)
}

