use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Theme {
    pub dark: bool,
    pub accent: String,
}

/// Lê o tema (claro/escuro) e a cor de destaque do sistema.
#[cfg(windows)]
pub fn current() -> Theme {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let dark = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
        .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|v| v == 0)
        .unwrap_or(true);

    // AccentColor está em ABGR (0xAABBGGRR)
    let accent = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\DWM")
        .and_then(|k| k.get_value::<u32, _>("AccentColor"))
        .map(|v| format!("#{:02x}{:02x}{:02x}", v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff))
        .unwrap_or_else(|_| "#0a84ff".into());

    // Só para testes visuais: força o tema sem mexer nas configurações do sistema.
    Theme { dark: apply_override(dark), accent }
}

/// Preferência do usuário; a variável GLINT_THEME (só para testes visuais) tem prioridade.
fn apply_override(system_dark: bool) -> bool {
    let pref = std::env::var("GLINT_THEME").unwrap_or_else(|_| crate::config::THEME.read().unwrap().clone());
    match pref.as_str() {
        "light" => false,
        "dark" => true,
        _ => system_dark,
    }
}

#[cfg(not(windows))]
pub fn current() -> Theme {
    let dark = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("dark"))
        .unwrap_or(true);
    Theme { dark: apply_override(dark), accent: "#0a84ff".into() }
}
