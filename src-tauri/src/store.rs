use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(std::env::temp_dir).join("Glint")
}

/// Pequeno armazenamento JSON persistente (uso, atalhos rápidos, preferências).
pub struct Store<T> {
    path: PathBuf,
    pub data: Mutex<T>,
}

impl<T: Serialize + DeserializeOwned + Default> Store<T> {
    pub fn open(file: &str) -> Self {
        let path = data_dir().join(file);
        let data = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Store { path, data: Mutex::new(data) }
    }

    pub fn save(&self) {
        let json = match serde_json::to_string_pretty(&*self.data.lock().unwrap()) {
            Ok(j) => j,
            Err(_) => return,
        };
        let path = self.path.clone();
        std::thread::spawn(move || {
            let _ = std::fs::create_dir_all(path.parent().unwrap());
            let _ = std::fs::write(path, json);
        });
    }
}
