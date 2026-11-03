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

/// Windows: resolve o ID do app (caminho, "{GUID}\caminho" de pasta conhecida ou app da Loja) para um local no disco.
#[cfg(windows)]
pub fn locate(id: &str) -> Option<PathBuf> {
    use std::os::windows::process::CommandExt;
    let known: &[(&str, &str)] = &[
        ("{6D809377-6AF0-444B-8957-A3773F02200E}", "ProgramFiles"),
        ("{7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}", "ProgramFiles(x86)"),
        ("{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}", "SystemRoot\\System32"),
        ("{F38BF404-1D43-42F2-9305-67DE0B28FC23}", "SystemRoot"),
    ];
    for (guid, var) in known {
        if let Some(rest) = id.strip_prefix(guid) {
            let mut it = var.splitn(2, '\\');
            let base = std::env::var(it.next()?).ok()?;
            let mut p = PathBuf::from(base);
            if let Some(sub) = it.next() {
                p.push(sub);
            }
            p.push(rest.trim_start_matches('\\'));
            return p.exists().then_some(p);
        }
    }
    let direct = PathBuf::from(id);
    if direct.is_absolute() && direct.exists() {
        return Some(direct);
    }
    // app da Loja: "FamiliaDoPacote!App" -> pasta de instalação do pacote
    let family = id.split('!').next()?;
    if family.is_empty() || !family.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-') {
        return None;
    }
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &format!("(Get-AppxPackage -PackageFamilyName '{family}' | Select-Object -First 1).InstallLocation")])
        .creation_flags(0x0800_0000)
        .output()
        .ok()?;
    let p = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    (p.as_os_str().len() > 0 && p.exists()).then_some(p)
}

/// Linux: o ID é o .desktop; tenta o executável do `Exec=` e cai para o próprio .desktop.
#[cfg(not(windows))]
pub fn locate(id: &str) -> Option<PathBuf> {
    let desktop = PathBuf::from(id);
    let exec = std::fs::read_to_string(&desktop).ok().and_then(|t| {
        t.lines().find_map(|l| l.strip_prefix("Exec=").map(|v| v.split_whitespace().next().unwrap_or("").to_string()))
    });
    if let Some(cmd) = exec.filter(|c| !c.is_empty()) {
        let direct = PathBuf::from(&cmd);
        if direct.is_absolute() && direct.exists() {
            return Some(direct);
        }
        if let Some(found) = std::env::var_os("PATH").and_then(|paths| std::env::split_paths(&paths).map(|d| d.join(&cmd)).find(|p| p.exists())) {
            return Some(found);
        }
    }
    desktop.exists().then_some(desktop)
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
    // quem digita "winbox.exe" quer o programa: a extensão não faz parte do nome do aplicativo
    let q = q.strip_suffix(".exe").unwrap_or(&q).to_string();
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
