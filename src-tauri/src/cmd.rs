//! Comandos digitados na busca: ssh, rdp, ping, pasta de rede e navegador com pesquisa.
//! Tudo é validado antes e executado SEM passar por shell: os argumentos vão separados.

use serde::Deserialize;
#[cfg(not(windows))]
use std::path::PathBuf;
#[cfg(windows)]
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
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub admin: bool,
}

/// Comandos do "Executar" do Windows (Win+R) que a busca reconhece: (nome digitado, descrição, programa, parâmetros).
/// Só o que está nesta lista roda; o texto digitado nunca vai direto para o sistema.
/// `.cpl` e `.msc` passam pelo control/mmc para que "executar como administrador" também funcione.
#[cfg(windows)]
const RUN: &[(&str, &str, &str, &str)] = &[
    ("cmd", "Prompt de Comando", "cmd.exe", ""),
    ("powershell", "Windows PowerShell", "powershell.exe", ""),
    ("pwsh", "PowerShell 7", "pwsh.exe", ""),
    ("wt", "Terminal do Windows", "wt.exe", ""),
    ("mstsc", "Conexão de Área de Trabalho Remota", "mstsc.exe", ""),
    ("ncpa.cpl", "Conexões de Rede", "control.exe", "ncpa.cpl"),
    ("appwiz.cpl", "Programas e Recursos", "control.exe", "appwiz.cpl"),
    ("sysdm.cpl", "Propriedades do Sistema", "control.exe", "sysdm.cpl"),
    ("inetcpl.cpl", "Opções da Internet", "control.exe", "inetcpl.cpl"),
    ("firewall.cpl", "Firewall do Windows Defender", "control.exe", "firewall.cpl"),
    ("mmsys.cpl", "Som", "control.exe", "mmsys.cpl"),
    ("main.cpl", "Propriedades do Mouse", "control.exe", "main.cpl"),
    ("desk.cpl", "Configurações de Vídeo", "control.exe", "desk.cpl"),
    ("powercfg.cpl", "Opções de Energia", "control.exe", "powercfg.cpl"),
    ("timedate.cpl", "Data e Hora", "control.exe", "timedate.cpl"),
    ("intl.cpl", "Região", "control.exe", "intl.cpl"),
    ("control", "Painel de Controle", "control.exe", ""),
    ("control printers", "Dispositivos e Impressoras", "control.exe", "printers"),
    ("devmgmt.msc", "Gerenciador de Dispositivos", "mmc.exe", "devmgmt.msc"),
    ("diskmgmt.msc", "Gerenciamento de Disco", "mmc.exe", "diskmgmt.msc"),
    ("services.msc", "Serviços", "mmc.exe", "services.msc"),
    ("compmgmt.msc", "Gerenciamento do Computador", "mmc.exe", "compmgmt.msc"),
    ("eventvwr.msc", "Visualizador de Eventos", "mmc.exe", "eventvwr.msc"),
    ("taskschd.msc", "Agendador de Tarefas", "mmc.exe", "taskschd.msc"),
    ("lusrmgr.msc", "Usuários e Grupos Locais", "mmc.exe", "lusrmgr.msc"),
    ("gpedit.msc", "Editor de Política de Grupo Local", "mmc.exe", "gpedit.msc"),
    ("secpol.msc", "Política de Segurança Local", "mmc.exe", "secpol.msc"),
    ("wf.msc", "Firewall com Segurança Avançada", "mmc.exe", "wf.msc"),
    ("certmgr.msc", "Certificados do Usuário", "mmc.exe", "certmgr.msc"),
    ("printmanagement.msc", "Gerenciamento de Impressão", "mmc.exe", "printmanagement.msc"),
    ("regedit", "Editor do Registro", "regedit.exe", ""),
    ("msconfig", "Configuração do Sistema", "msconfig.exe", ""),
    ("taskmgr", "Gerenciador de Tarefas", "taskmgr.exe", ""),
    ("resmon", "Monitor de Recursos", "resmon.exe", ""),
    ("perfmon", "Monitor de Desempenho", "perfmon.exe", ""),
    ("msinfo32", "Informações do Sistema", "msinfo32.exe", ""),
    ("winver", "Sobre o Windows", "winver.exe", ""),
    ("dxdiag", "Ferramenta de Diagnóstico do DirectX", "dxdiag.exe", ""),
    ("cleanmgr", "Limpeza de Disco", "cleanmgr.exe", ""),
    ("dfrgui", "Otimizar Unidades", "dfrgui.exe", ""),
    ("explorer", "Explorador de Arquivos", "explorer.exe", ""),
    ("notepad", "Bloco de Notas", "notepad.exe", ""),
    ("calc", "Calculadora", "calc.exe", ""),
    ("mspaint", "Paint", "mspaint.exe", ""),
    ("snippingtool", "Ferramenta de Captura", "snippingtool.exe", ""),
    ("charmap", "Mapa de Caracteres", "charmap.exe", ""),
    ("osk", "Teclado Virtual", "osk.exe", ""),
    ("%temp%", "Pasta de arquivos temporários", "explorer.exe", "%TEMP%"),
    ("shell:startup", "Pasta Inicializar", "explorer.exe", "shell:startup"),
];

/// Lista para a interface: (nome digitado, descrição). Vazia fora do Windows.
pub fn run_list() -> Vec<(&'static str, &'static str)> {
    #[cfg(windows)]
    {
        RUN.iter().map(|r| (r.0, r.1)).collect()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// ShellExecute: resolve o programa pelo "App Paths" do Windows e não passa por cmd. `runas` pede elevação (UAC).
#[cfg(windows)]
fn shell_exec(verb: &str, file: &str, params: &str) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let w = |s: &str| -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() };
    let (op, f, p) = (w(verb), w(file), w(params));
    let par = if params.is_empty() { PCWSTR::null() } else { PCWSTR(p.as_ptr()) };
    let r = unsafe { ShellExecuteW(None, PCWSTR(op.as_ptr()), PCWSTR(f.as_ptr()), par, PCWSTR::null(), SW_SHOWNORMAL) };
    r.0 as isize > 32
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
                return crate::util::command(term).args(*flags).arg(prog).args(args).spawn().map(|_| ()).map_err(|e| e.to_string());
            }
        }
        Err("Nenhum terminal encontrado (instale gnome-terminal, konsole, xterm...).".into())
    }
}

#[cfg(windows)]
fn open_in_browser(browser: &str, url: &str) -> Result<(), String> {
    let exe = match browser {
        "brave" => "brave.exe",
        "chrome" => "chrome.exe",
        "firefox" => "firefox.exe",
        "edge" | "msedge" => "msedge.exe",
        "opera" => "opera.exe",
        "vivaldi" => "vivaldi.exe",
        _ => return Err("Navegador desconhecido.".into()),
    };
    if shell_exec("open", exe, &format!("\"{url}\"")) {
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
            return crate::util::command(b).arg(url).spawn().map(|_| ()).map_err(|e| e.to_string());
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
                crate::util::open(&a.path)
            }
            #[cfg(not(windows))]
            {
                // \\host\share\pasta -> smb://host/share/pasta
                let url = format!("smb://{}", a.path.trim_start_matches('\\').replace('\\', "/"));
                crate::util::open(url)
            }
        }
        "browser" => {
            if !valid_url(&a.url) {
                return Err("Endereço inválido.".into());
            }
            open_in_browser(&a.browser, &a.url)
        }
        "run" => {
            #[cfg(windows)]
            {
                let Some(&(_, title, file, params)) = RUN.iter().find(|r| r.0 == a.name) else {
                    return Err("Comando desconhecido.".into());
                };
                let params = params.replace("%TEMP%", &std::env::temp_dir().to_string_lossy());
                if shell_exec(if a.admin { "runas" } else { "open" }, file, &params) {
                    Ok(())
                } else {
                    Err(format!("Não consegui abrir: {title}."))
                }
            }
            #[cfg(not(windows))]
            {
                Err("Esse comando só existe no Windows.".into())
            }
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
