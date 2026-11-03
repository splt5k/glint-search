//! Preferências do usuário (config.json): atalho, início com o sistema, tema e pastas indexadas.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    pub hotkey: String,
    pub autostart: bool,
    /// "system" | "light" | "dark"
    pub theme: String,
    /// Pastas extras para indexar (além das pessoais).
    pub include: Vec<String>,
    /// Pastas que nunca entram no índice.
    pub exclude: Vec<String>,
    /// Indexar todos os discos fixos (desligado: só as pastas pessoais e as extras).
    pub all_drives: bool,
    pub pause_on_battery: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkey: "Ctrl+Space".into(),
            autostart: true,
            theme: "system".into(),
            include: vec![],
            exclude: vec![],
            all_drives: true,
            pause_on_battery: true,
        }
    }
}

impl Config {
    /// Corrige valores inválidos (também usado ao importar um arquivo de terceiros).
    pub fn sanitized(mut self) -> Result<Self, String> {
        self.hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>().map_err(|e| format!("Atalho inválido: {e}"))?;
        if !matches!(self.theme.as_str(), "system" | "light" | "dark") {
            self.theme = "system".into();
        }
        let clean = |v: Vec<String>| -> Vec<String> {
            let mut out: Vec<String> = Vec::new();
            for s in v {
                let s = s.trim().to_string();
                if !s.is_empty() && !out.contains(&s) {
                    out.push(s);
                }
            }
            out
        };
        self.include = clean(self.include);
        self.exclude = clean(self.exclude);
        Ok(self)
    }

    pub fn include_paths(&self) -> Vec<PathBuf> {
        self.include.iter().map(PathBuf::from).filter(|p| p.is_dir()).collect()
    }
    pub fn exclude_paths(&self) -> Vec<PathBuf> {
        self.exclude.iter().map(PathBuf::from).collect()
    }
}

/// Tema escolhido nas preferências; lido por theme::current().
pub static THEME: RwLock<String> = RwLock::new(String::new());
