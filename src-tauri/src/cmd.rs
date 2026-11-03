//! Comandos digitados na busca: ssh, rdp, ping, pasta de rede e navegador com pesquisa.
//! Tudo é validado antes e executado SEM passar por shell: os argumentos vão separados.

use serde::Deserialize;
#[cfg(not(windows))]
use std::path::PathBuf;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct CmdArgs {
    pub kind: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u32,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub browser: String,
    #[serde(default)]
    pub path: String,
}

fn valid_name(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.chars().next().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false)
        && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn valid_url(u: &str) -> bool {
    (u.starts_with("http://") || u.starts_with("https://"))
        && u.len() <= 2000
        && !u.chars().any(|c| c.is_control() || c.is_whitespace() || c == '"' || c == '\\')
}

#[cfg(not(windows))]
fn which(exe: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| std::env::split_paths(&paths).map(|d| d.join(exe)).find(|p| p.is_file()))
}

/// Abre `prog args...` num terminal novo.
fn in_terminal(prog: &str, args: &[String]) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        Command::new("cmd")
            .arg("/K")
            .arg(prog)
            .args(args)
            .creation_flags(0x0000_0010) // CREATE_NEW_CONSOLE
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        const TERMS: &[(&str, &[&str])] = &[
            ("x-terminal-emulator", &["-e"]),
            ("gnome-terminal", &["--"]),
            ("konsole", &["-e"]),
            ("xfce4-terminal", &["-x"]),
            ("alacritty", &["-e"]),
            ("kitty", &[]),
            ("xterm", &["-e"]),
        ];
        for (term, flags) in TERMS {
            if which(term).is_some() {
                return Command::new(term).args(*flags).arg(prog).args(args).spawn().map(|_| ()).map_err(|e| e.to_string());
            }
        }
        Err("Nenhum terminal encontrado (instale gnome-terminal, konsole, xterm...).".into())
    }
}

#[cfg(windows)]
fn open_in_browser(browser: &str, url: &str) -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let exe = match browser {
        "brave" => "brave.exe",
        "chrome" => "chrome.exe",
        "firefox" => "firefox.exe",
        "edge" | "msedge" => "msedge.exe",
        "opera" => "opera.exe",
        "vivaldi" => "vivaldi.exe",
        _ => return Err("Navegador desconhecido.".into()),
    };
    let w = |s: &str| -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() };
    let (op, file, par) = (w("open"), w(exe), w(&format!("\"{url}\"")));
    // ShellExecute resolve o programa pelo "App Paths" do Windows e não passa por cmd.
    let r = unsafe { ShellExecuteW(None, PCWSTR(op.as_ptr()), PCWSTR(file.as_ptr()), PCWSTR(par.as_ptr()), PCWSTR::null(), SW_SHOWNORMAL) };
    if r.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("Não consegui abrir o {browser}. Ele está instalado?"))
    }
}

#[cfg(not(windows))]
fn open_in_browser(browser: &str, url: &str) -> Result<(), String> {
    let bins: &[&str] = match browser {
        "brave" => &["brave-browser", "brave", "brave-browser-stable"],
        "chrome" => &["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"],
        "firefox" => &["firefox"],
        "edge" | "msedge" => &["microsoft-edge", "microsoft-edge-stable"],
        "opera" => &["opera"],
        "vivaldi" => &["vivaldi", "vivaldi-stable"],
        _ => return Err("Navegador desconhecido.".into()),
    };
    for b in bins {
        if which(b).is_some() {
            return Command::new(b).arg(url).spawn().map(|_| ()).map_err(|e| e.to_string());
        }
    }
    Err(format!("Não encontrei o {browser} instalado."))
}

pub fn run(a: &CmdArgs) -> Result<(), String> {
    let port_ok = a.port == 0 || (1..=65535).contains(&a.port);
    match a.kind.as_str() {
        "ssh" => {
            if !valid_name(&a.host, 253) || !port_ok || (!a.user.is_empty() && !valid_name(&a.user, 64)) {
                return Err("Endereço SSH inválido.".into());
            }
            let target = if a.user.is_empty() { a.host.clone() } else { format!("{}@{}", a.user, a.host) };
            let mut args = Vec::new();
            if a.port != 0 {
                args.extend(["-p".to_string(), a.port.to_string()]);
            }
            args.push(target);
            in_terminal("ssh", &args)
        }
        "ping" => {
            if !valid_name(&a.host, 253) {
                return Err("Endereço inválido.".into());
            }
            in_terminal("ping", &[a.host.clone()])
        }
        "rdp" => {
            #[cfg(windows)]
            {
                if !valid_name(&a.host, 253) || !port_ok {
                    return Err("Endereço inválido.".into());
                }
                let target = if a.port != 0 { format!("{}:{}", a.host, a.port) } else { a.host.clone() };
                Command::new("mstsc").arg(format!("/v:{target}")).spawn().map(|_| ()).map_err(|e| e.to_string())
            }
            #[cfg(not(windows))]
            {
                Err("Área de Trabalho Remota só está disponível no Windows.".into())
            }
        }
        "unc" => {
            if !a.path.starts_with("\\\\") || a.path.len() > 500 || a.path.chars().any(|c| c.is_control() || c == '"') {
                return Err("Caminho de rede inválido.".into());
            }
            #[cfg(windows)]
            {
                opener::open(&a.path).map_err(|e| e.to_string())
            }
            #[cfg(not(windows))]
            {
                // \\host\share\pasta -> smb://host/share/pasta
                let url = format!("smb://{}", a.path.trim_start_matches('\\').replace('\\', "/"));
                opener::open(url).map_err(|e| e.to_string())
            }
        }
        "browser" => {
            if !valid_url(&a.url) {
                return Err("Endereço inválido.".into());
            }
            open_in_browser(&a.browser, &a.url)
        }
        _ => Err("Comando desconhecido.".into()),
    }
}

/// Apelidos de `Host` do ~/.ssh/config (sem curingas), para completar "ssh <apelido>".
pub fn ssh_hosts() -> Vec<String> {
    let Some(home) = dirs::home_dir() else { return vec![] };
    let Ok(text) = std::fs::read_to_string(home.join(".ssh").join("config")) else { return vec![] };
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        if it.next().map(|k| k.eq_ignore_ascii_case("host")).unwrap_or(false) {
            for h in it {
                if valid_name(h, 100) && !out.iter().any(|x| x == h) {
                    out.push(h.to_string());
                }
            }
        }
    }
    out.truncate(60);
    out
}
