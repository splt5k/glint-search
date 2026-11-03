use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MAX_ITEMS: usize = 200;
const MAX_LEN: usize = 50_000;
/// Imagens ficam em memória (PNG): limites para não pesar no PC.
const MAX_IMAGES: usize = 15;
const MAX_IMAGE_BYTES: usize = 120 * 1024 * 1024;
const THUMB: u32 = 160;

#[derive(Serialize, Clone)]
pub struct ClipItem {
    pub id: u64,
    /// "text" | "image" | "files"
    pub kind: &'static str,
    /// texto copiado; para imagens, a descrição ("Imagem 1920×1080"); para arquivos, os nomes
    pub text: String,
    pub ts: i64,
    pub w: u32,
    pub h: u32,
    pub files: Vec<String>,
    #[serde(skip)]
    sig: u64,
}

struct Blob {
    png: Arc<Vec<u8>>,
    thumb: Arc<Vec<u8>>,
}

/// Histórico da área de transferência (texto, imagens e arquivos). Fica só na memória: nada vai para o disco.
#[derive(Default)]
pub struct ClipStore {
    items: Mutex<Vec<ClipItem>>,
    next_id: Mutex<u64>,
    /// assinatura do que nós mesmos colocamos na área de transferência (não vira item novo)
    ignore: Mutex<Option<u64>>,
    blobs: Mutex<HashMap<u64, Blob>>,
}

fn sig_of(tag: &str, data: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    tag.hash(&mut h);
    data.hash(&mut h);
    h.finish()
}

impl ClipStore {
    pub fn list(&self) -> Vec<ClipItem> {
        self.items.lock().unwrap().clone()
    }

    fn insert(&self, mut item: ClipItem, blob: Option<Blob>) {
        if self.ignore.lock().unwrap().take() == Some(item.sig) {
            return;
        }
        let mut items = self.items.lock().unwrap();
        let mut blobs = self.blobs.lock().unwrap();
        items.retain(|c| {
            let same = c.sig == item.sig;
            if same {
                blobs.remove(&c.id);
            }
            !same
        });
        let mut id = self.next_id.lock().unwrap();
        *id += 1;
        item.id = *id;
        if let Some(b) = blob {
            blobs.insert(item.id, b);
        }
        items.insert(0, item);
        items.truncate(MAX_ITEMS);
        // descarta imagens antigas além dos limites
        let mut total = 0usize;
        let mut count = 0usize;
        let ids: Vec<u64> = items.iter().filter(|c| c.kind == "image").map(|c| c.id).collect();
        for iid in ids {
            let size = blobs.get(&iid).map(|b| b.png.len() + b.thumb.len()).unwrap_or(0);
            count += 1;
            total += size;
            if count > MAX_IMAGES || total > MAX_IMAGE_BYTES {
                blobs.remove(&iid);
                items.retain(|c| c.id != iid);
            }
        }
        blobs.retain(|k, _| items.iter().any(|c| c.id == *k));
    }

    pub fn push(&self, text: String) {
        if text.trim().is_empty() || text.len() > MAX_LEN {
            return;
        }
        let sig = sig_of("t", text.as_bytes());
        self.insert(ClipItem { id: 0, kind: "text", text, ts: now(), w: 0, h: 0, files: vec![], sig }, None);
    }

    pub fn push_files(&self, files: Vec<String>) {
        if files.is_empty() {
            return;
        }
        let names: Vec<String> = files
            .iter()
            .map(|f| std::path::Path::new(f).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| f.clone()))
            .collect();
        let sig = sig_of("f", files.join("\n").as_bytes());
        self.insert(ClipItem { id: 0, kind: "files", text: names.join(", "), ts: now(), w: 0, h: 0, files, sig }, None);
    }

    pub fn push_image(&self, w: u32, h: u32, rgba: &[u8]) {
        let sig = sig_of("i", rgba);
        if self.items.lock().unwrap().first().map(|c| c.sig) == Some(sig) {
            return; // mesma imagem ainda na área de transferência: evita recodificar
        }
        let Some(img) = image::RgbaImage::from_raw(w, h, rgba.to_vec()) else { return };
        let mut png = Vec::new();
        if image::DynamicImage::ImageRgba8(img.clone())
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .is_err()
        {
            return;
        }
        let small = image::imageops::thumbnail(&img, THUMB.min(w), THUMB.min(h));
        let mut thumb = Vec::new();
        let rgb = image::DynamicImage::ImageRgba8(small).to_rgb8();
        let _ = rgb.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut thumb, 80));
        let blob = Blob { png: Arc::new(png), thumb: Arc::new(thumb) };
        self.insert(
            ClipItem { id: 0, kind: "image", text: format!("Imagem {w}×{h}"), ts: now(), w, h, files: vec![], sig },
            Some(blob),
        );
    }

    /// Bytes da imagem (PNG) ou da miniatura (JPEG).
    pub fn blob(&self, id: u64, thumb: bool) -> Option<Arc<Vec<u8>>> {
        self.blobs.lock().unwrap().get(&id).map(|b| if thumb { b.thumb.clone() } else { b.png.clone() })
    }

    pub fn delete(&self, id: u64) {
        self.items.lock().unwrap().retain(|c| c.id != id);
        self.blobs.lock().unwrap().remove(&id);
    }

    pub fn clear(&self) {
        self.items.lock().unwrap().clear();
        self.blobs.lock().unwrap().clear();
    }

    /// Coloca o item de volta na área de transferência, no formato original.
    pub fn copy(&self, id: u64) {
        let Some(item) = self.items.lock().unwrap().iter().find(|c| c.id == id).cloned() else { return };
        *self.ignore.lock().unwrap() = Some(item.sig);
        match item.kind {
            "image" => {
                if let Some(png) = self.blob(id, false) {
                    if let Ok(img) = image::load_from_memory(&png) {
                        let rgba = img.to_rgba8();
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_image(arboard::ImageData {
                                width: rgba.width() as usize,
                                height: rgba.height() as usize,
                                bytes: std::borrow::Cow::Owned(rgba.into_raw()),
                            });
                        }
                    }
                }
            }
            "files" => {
                if !set_files(&item.files) {
                    // sem suporte nativo (ex.: Linux): devolve os caminhos como texto
                    *self.ignore.lock().unwrap() = None;
                    self.set_text(&item.files.join("\n"));
                }
            }
            _ => self.set_text(&item.text),
        }
    }

    pub fn set_text(&self, text: &str) {
        *self.ignore.lock().unwrap() = Some(sig_of("t", text.as_bytes()));
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text.to_string());
        }
    }
}

fn now() -> i64 {
    chrono::Local::now().timestamp()
}

/// Windows: coloca arquivos de verdade (CF_HDROP) para colar no Explorer.
#[cfg(windows)]
fn set_files(paths: &[String]) -> bool {
    use windows::Win32::Foundation::{HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT};
    use windows::Win32::UI::Shell::DROPFILES;

    let mut wide: Vec<u16> = Vec::new();
    for p in paths {
        wide.extend(p.encode_utf16());
        wide.push(0);
    }
    wide.push(0);
    let head = std::mem::size_of::<DROPFILES>();
    unsafe {
        let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, head + wide.len() * 2) else { return false };
        let ptr = GlobalLock(hmem) as *mut u8;
        if ptr.is_null() {
            return false;
        }
        let df = ptr as *mut DROPFILES;
        (*df).pFiles = head as u32;
        (*df).fWide = true.into();
        std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, ptr.add(head), wide.len() * 2);
        let _ = GlobalUnlock(hmem);

        if OpenClipboard(None).is_err() {
            return false;
        }
        let _ = EmptyClipboard();
        let ok = SetClipboardData(15, Some(HANDLE(hmem.0))).is_ok(); // CF_HDROP
        let _ = CloseClipboard();
        let _ = HGLOBAL::default();
        ok
    }
}
#[cfg(not(windows))]
fn set_files(_paths: &[String]) -> bool {
    false
}

/// Windows: número que muda a cada alteração da área de transferência (evita ler/decodificar à toa).
#[cfg(windows)]
fn change_count() -> Option<u32> {
    Some(unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() })
}
#[cfg(not(windows))]
fn change_count() -> Option<u32> {
    None
}

pub fn start_watcher(store: Arc<ClipStore>) {
    std::thread::spawn(move || {
        crate::power::background_thread();
        let mut last_seq = None::<u32>;
        let mut last_text = String::new();
        loop {
            std::thread::sleep(Duration::from_millis(450));
            let seq = change_count();
            if seq.is_some() && seq == last_seq {
                continue;
            }
            last_seq = seq;
            let Ok(mut cb) = arboard::Clipboard::new() else { continue };

            // prioridade: arquivos > texto > imagem (Excel, por exemplo, põe texto e imagem juntos)
            if let Ok(files) = cb.get().file_list() {
                if !files.is_empty() {
                    store.push_files(files.iter().map(|p| p.to_string_lossy().into_owned()).collect());
                    continue;
                }
            }
            if let Ok(t) = cb.get_text() {
                if !t.trim().is_empty() {
                    if t != last_text || seq.is_some() {
                        last_text = t.clone();
                        store.push(t);
                    }
                    continue;
                }
            }
            if let Ok(img) = cb.get_image() {
                store.push_image(img.width as u32, img.height as u32, &img.bytes);
            }
        }
    });
}
