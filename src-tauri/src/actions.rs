use crate::util::url_encode;
use serde_json::Value;
use std::process::Command;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

fn arg_str(args: &Value, key: &str) -> String {
    args.get(key).and_then(|v| v.as_str()).unwrap_or("").trim().to_string()
}

fn silent(program: &str, args: &[&str]) {
    let mut c = Command::new(program);
    c.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000);
    }
    let _ = c.spawn();
}

fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if "<>:\"/\\|?*\n\r\t".contains(c) { '-' } else { c })
        .collect();
    let s = s.trim().trim_matches('.').to_string();
    if s.is_empty() { "Nota".into() } else { s.chars().take(80).collect() }
}

pub fn run(app: &AppHandle, id: &str, args: &Value) -> Result<String, String> {
    match id {
        "lock" => {
            #[cfg(windows)]
            silent("rundll32.exe", &["user32.dll,LockWorkStation"]);
            #[cfg(not(windows))]
            silent("loginctl", &["lock-session"]);
            Ok("Tela bloqueada".into())
        }
        "sleep" => {
            #[cfg(windows)]
            silent("rundll32.exe", &["powrprof.dll,SetSuspendState", "0,1,0"]);
            #[cfg(not(windows))]
            silent("systemctl", &["suspend"]);
            Ok("Suspendendo".into())
        }
        "shutdown" => {
            #[cfg(windows)]
            silent("shutdown", &["/s", "/t", "0"]);
            #[cfg(not(windows))]
            silent("systemctl", &["poweroff"]);
            Ok("Desligando".into())
        }
        "restart" => {
            #[cfg(windows)]
            silent("shutdown", &["/r", "/t", "0"]);
            #[cfg(not(windows))]
            silent("systemctl", &["reboot"]);
            Ok("Reiniciando".into())
        }
        "empty_trash" => {
            #[cfg(windows)]
            silent("powershell", &["-NoProfile", "-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"]);
            Ok("Lixeira esvaziada".into())
        }
        "taskmgr" => {
            #[cfg(windows)]
            silent("taskmgr.exe", &[]);
            #[cfg(not(windows))]
            silent("gnome-system-monitor", &[]);
            Ok(String::new())
        }
        "terminal" => {
            #[cfg(windows)]
            {
                if Command::new("wt.exe").spawn().is_err() {
                    silent("powershell.exe", &[]);
                }
            }
            #[cfg(not(windows))]
            silent("x-terminal-emulator", &[]);
            Ok(String::new())
        }
        "timer" => {
            let secs = args.get("seconds").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if secs <= 0.0 {
                return Err("Duração inválida".into());
            }
            let label = arg_str(args, "label");
            let app = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs_f64(secs));
                let body = if label.is_empty() { "Tempo esgotado".to_string() } else { label };
                let _ = app.notification().builder().title("Timer").body(body).show();
            });
            Ok("Timer iniciado".into())
        }
        "note" => {
            let title = arg_str(args, "title");
            let text = arg_str(args, "text");
            let dir = dirs::document_dir().unwrap_or_else(std::env::temp_dir).join("Notas");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let name = if title.is_empty() {
                chrono::Local::now().format("Nota %Y-%m-%d %H-%M").to_string()
            } else {
                sanitize(&title)
            };
            let path = dir.join(format!("{}.md", name));
            let content = format!("# {}\n\n{}\n", if title.is_empty() { &name } else { &title }, text);
            std::fs::write(&path, content).map_err(|e| e.to_string())?;
            opener::open(&path).map_err(|e| e.to_string())?;
            Ok("Nota criada".into())
        }
        "email" => {
            let to = arg_str(args, "to");
            let subject = arg_str(args, "subject");
            let body = arg_str(args, "body");
            let url = format!("mailto:{}?subject={}&body={}", url_encode(&to), url_encode(&subject), url_encode(&body));
            opener::open(url).map_err(|e| e.to_string())?;
            Ok(String::new())
        }
        "message" => {
            let text = arg_str(args, "text");
            opener::open(format!("https://wa.me/?text={}", url_encode(&text))).map_err(|e| e.to_string())?;
            Ok(String::new())
        }
        "event" => {
            let title = arg_str(args, "title");
            let start = arg_str(args, "start"); // YYYYMMDDTHHMMSS
            let end = arg_str(args, "end");
            if start.len() != 15 || end.len() != 15 {
                return Err("Data inválida".into());
            }
            let ics = format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Glint//PT\r\nBEGIN:VEVENT\r\nUID:{}@glint\r\nDTSTAMP:{}\r\nDTSTART:{}\r\nDTEND:{}\r\nSUMMARY:{}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                chrono::Local::now().timestamp_nanos_opt().unwrap_or(0),
                chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
                start,
                end,
                title.replace(['\r', '\n'], " ")
            );
            let path = std::env::temp_dir().join(format!("{}.ics", sanitize(&title)));
            std::fs::write(&path, ics).map_err(|e| e.to_string())?;
            opener::open(&path).map_err(|e| e.to_string())?;
            Ok("Evento criado".into())
        }
        "url" => {
            let url = arg_str(args, "url");
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Err("URL inválida".into());
            }
            opener::open(url).map_err(|e| e.to_string())?;
            Ok(String::new())
        }
        other => Err(format!("Ação desconhecida: {}", other)),
    }
}
