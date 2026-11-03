//! Iniciar com o sistema: registro (Windows) ou ~/.config/autostart (Linux).

#[cfg(windows)]
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub fn is_enabled() -> bool {
    #[cfg(windows)]
    {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(RUN_KEY)
            .and_then(|k| k.get_value::<String, _>("Glint"))
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        desktop_path().map(|p| p.exists()).unwrap_or(false)
    }
}

pub fn set(enabled: bool) -> Result<(), String> {
    // AppImage: o executável em uso fica numa pasta temporária que some a cada reinício; vale o caminho do próprio arquivo .AppImage.
    let exe = match std::env::var_os("APPIMAGE").map(std::path::PathBuf::from).filter(|p| p.exists()) {
        Some(p) => p,
        None => std::env::current_exe().map_err(|e| e.to_string())?,
    };
    #[cfg(windows)]
    {
        use winreg::{enums::*, RegKey};
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
            .map_err(|e| e.to_string())?;
        if enabled {
            key.set_value("Glint", &format!("\"{}\"", exe.display())).map_err(|e| e.to_string())
        } else {
            let _ = key.delete_value("Glint");
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        let path = desktop_path().ok_or("sem diretório de configuração")?;
        if enabled {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            let body = format!(
                "[Desktop Entry]\nType=Application\nName=Glint\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\n",
                exe.display()
            );
            std::fs::write(path, body).map_err(|e| e.to_string())
        } else {
            let _ = std::fs::remove_file(path);
            Ok(())
        }
    }
}

#[cfg(not(windows))]
fn desktop_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("autostart").join("glint.desktop"))
}
