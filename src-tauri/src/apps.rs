use crate::fuzzy;
use crate::index::Hit;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub id: String, // Windows: AppUserModelID | Linux: caminho do .desktop
    #[serde(default)]
    pub exec: String,
}

fn cache_path() -> PathBuf {
    crate::store::data_dir().join("apps.json")
}

pub fn load_cached() -> Vec<AppEntry> {
    std::fs::read_to_string(cache_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_cache(apps: &[AppEntry]) {
    let _ = std::fs::create_dir_all(crate::store::data_dir());
    if let Ok(s) = serde_json::to_string(apps) {
        let _ = std::fs::write(cache_path(), s);
    }
}

fn skip(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("uninstall") || n.contains("desinstal") || n.contains("readme") || n.ends_with(" help") || n.ends_with(" ajuda")
}

#[cfg(windows)]
pub fn scan() -> Vec<AppEntry> {
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-StartApps | Sort-Object Name | ConvertTo-Json -Compress",
        ])
        .creation_flags(0x0800_0000)
        .output();

    let Ok(out) = out else { return vec![] };
    let text = String::from_utf8_lossy(&out.stdout);
    let Ok(v) = serde_json::from_str::<serde_json::Value>(text.trim()) else { return vec![] };

    let arr = match v {
        serde_json::Value::Array(a) => a,
        o @ serde_json::Value::Object(_) => vec![o],
        _ => vec![],
    };

    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();
    for item in arr {
        let name = item["Name"].as_str().unwrap_or("").trim().to_string();
        let id = item["AppID"].as_str().unwrap_or("").trim().to_string();
        if name.is_empty() || id.is_empty() || skip(&name) {
            continue;
        }
        if seen.insert(name.to_lowercase()) {
            apps.push(AppEntry { name, id, exec: String::new() });
        }
    }
    apps
}

#[cfg(not(windows))]
pub fn scan() -> Vec<AppEntry> {
    let mut dirs_list = vec![PathBuf::from("/usr/share/applications"), PathBuf::from("/usr/local/share/applications")];
    if let Some(h) = dirs::data_dir() {
        dirs_list.push(h.join("applications"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();
    for d in dirs_list {
        let Ok(rd) = std::fs::read_dir(d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("desktop") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let (mut name, mut exec, mut hidden) = (String::new(), String::new(), false);
            let mut in_entry = false;
            for line in text.lines() {
                if line.starts_with('[') {
                    in_entry = line == "[Desktop Entry]";
                } else if in_entry {
                    if let Some(v) = line.strip_prefix("Name=") {
                        if name.is_empty() {
                            name = v.to_string();
                        }
                    } else if let Some(v) = line.strip_prefix("Exec=") {
                        exec = v.split_whitespace().filter(|w| !w.starts_with('%')).collect::<Vec<_>>().join(" ");
                    } else if line == "NoDisplay=true" || line == "Hidden=true" {
                        hidden = true;
                    }
                }
            }
            if !hidden && !name.is_empty() && !exec.is_empty() && !skip(&name) && seen.insert(name.to_lowercase()) {
                apps.push(AppEntry { name, id: p.to_string_lossy().into_owned(), exec });
            }
        }
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

pub fn launch(app_id: &str, exec: &str) {
    #[cfg(windows)]
    {
        let _ = exec;
        let _ = std::process::Command::new("explorer.exe").arg(format!("shell:AppsFolder\\{}", app_id)).spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = app_id;
        let _ = std::process::Command::new("sh").arg("-c").arg(format!("nohup {} >/dev/null 2>&1 &", exec)).spawn();
    }
}

pub fn to_hit(a: &AppEntry, score: i32) -> Hit {
    Hit {
        id: format!("app:{}", a.id),
        kind: "app".into(),
        title: a.name.clone(),
        subtitle: "Aplicativo".into(),
        path: a.id.clone(),
        score,
        mtime: 0,
    }
}

pub fn search(apps: &[AppEntry], query: &str) -> Vec<Hit> {
    let q = query.trim().to_lowercase();
    let mut v: Vec<Hit> = apps
        .iter()
        .filter_map(|a| {
            let s = fuzzy::score_lower(&q, &a.name.to_lowercase());
            (s > 0).then(|| to_hit(a, s))
        })
        .collect();
    v.sort_by(|a, b| b.score.cmp(&a.score));
    v
}
