pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then(|| url_decode(v))
    })
}

/// Texto de depuração: só aparece quando GLINT_LOG está definida.
#[macro_export]
macro_rules! dlog {
    ($($a:tt)*) => {
        if std::env::var_os("GLINT_LOG").is_some() {
            $crate::elog!($($a)*);
        }
    };
}

/// Escreve no stderr sem nunca derrubar o app (eprintln! entra em pânico se o terminal já foi fechado).
#[macro_export]
macro_rules! elog {
    ($($a:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), $($a)*);
    }};
}

/// Prepara um programa externo para rodar. No AppImage, o Glint roda com variáveis que apontam para as bibliotecas
/// do próprio pacote (LD_LIBRARY_PATH, PYTHONHOME, GIO_MODULE_DIR...); herdadas, elas quebram o que abrimos
/// (terminal, monitor do sistema, `gio`/`xdg-open`, apps). Aqui elas são removidas só para o processo filho.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    #[allow(unused_mut)]
    let mut c = std::process::Command::new(program);
    #[cfg(target_os = "linux")]
    clean_appimage_env(&mut c);
    c
}

#[cfg(target_os = "linux")]
fn clean_appimage_env(c: &mut std::process::Command) {
    let Some(appdir) = std::env::var("APPDIR").ok().filter(|d| !d.is_empty()) else { return };
    let inside = |p: &str| p.starts_with(&appdir) || p.contains("/.mount_");
    // O Glint roda dentro da montagem do AppImage; o terminal e os apps abertos começam na pasta pessoal.
    if let Some(home) = dirs::home_dir() {
        c.current_dir(home);
    }
    for (k, v) in std::env::vars_os() {
        let Some(key) = k.to_str() else { continue };
        if matches!(key, "APPDIR" | "APPIMAGE" | "ARGV0" | "OWD" | "GTK_THEME" | "PYTHONHOME" | "PYTHONDONTWRITEBYTECODE") {
            c.env_remove(&k);
            continue;
        }
        let val = v.to_string_lossy();
        if !val.split(':').any(inside) {
            continue;
        }
        let kept: Vec<&str> = val.split(':').filter(|p| !p.is_empty() && !inside(p)).collect();
        if kept.is_empty() {
            c.env_remove(&k);
        } else {
            c.env(&k, kept.join(":"));
        }
    }
}

/// Abre arquivo, pasta ou link no programa padrão. No Linux usa o `xdg-open` com o ambiente limpo (ver `command`).
pub fn open(target: impl AsRef<std::ffi::OsStr>) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if std::env::var_os("APPDIR").is_some() {
        let mut child = command("xdg-open")
            .arg(target.as_ref())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        return Ok(());
    }
    opener::open(target).map_err(|e| e.to_string())
}
