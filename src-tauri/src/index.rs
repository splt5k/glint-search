use crate::fuzzy;
use rayon::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Serialize, Clone, Debug, Default)]
pub struct Hit {
    pub id: String,
    pub kind: String, // app | file | folder | setting
    pub title: String,
    pub subtitle: String,
    pub path: String,
    pub score: i32,
    pub mtime: u32,
}

#[derive(Clone, Copy)]
struct Ent {
    dir: u32,
    noff: u32,
    nlen: u16,
    loff: u32,
    llen: u16,
    is_dir: bool,
    mtime: u32,
}

/// Índice de arquivos em memória: nomes em dois "arenas" (original e minúsculo) para busca rápida.
#[derive(Default)]
pub struct FileIndex {
    dirs: Vec<String>,
    personal: Vec<bool>,
    ents: Vec<Ent>,
    names: String,
    lower: String,
}

const MAX_ENTRIES: usize = 2_000_000;
const MAX_CANDIDATES: usize = 6000;

const NOISE_DIRS: &[&str] = &[
    "node_modules", "AppData", "__pycache__", "$Recycle.Bin", "target", "venv", "site-packages",
];

const SYSTEM_ROOT_DIRS: &[&str] = &[
    "Windows", "Program Files", "Program Files (x86)", "ProgramData", "System Volume Information",
    "Recovery", "$WinREAgent", "PerfLogs", "Config.Msi", "Documents and Settings", "lost+found",
];

struct RawEnt {
    dir: u32,
    name: String,
    is_dir: bool,
    mtime: u32,
}

#[derive(Default)]
struct Part {
    dirs: Vec<String>,
    ents: Vec<RawEnt>,
}

#[cfg(windows)]
fn is_reparse(md: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    md.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn is_reparse(md: &std::fs::Metadata) -> bool {
    md.file_type().is_symlink()
}

#[cfg(windows)]
fn is_system(md: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    md.file_attributes() & 0x4 != 0
}
#[cfg(not(windows))]
fn is_system(_: &std::fs::Metadata) -> bool {
    false
}

fn scan(root: &Path, recursive: bool, budget: usize, exclude: &[PathBuf]) -> Part {
    let mut part = Part::default();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if part.ents.len() >= budget {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        let di = part.dirs.len() as u32;
        part.dirs.push(dir.to_string_lossy().into_owned());

        for e in rd.flatten() {
            let Ok(md) = e.metadata() else { continue };
            if is_system(&md) {
                continue;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            let is_dir = md.is_dir();

            if is_dir && (name.starts_with('.') || NOISE_DIRS.iter().any(|n| n.eq_ignore_ascii_case(&name))) {
                continue;
            }

            let mtime = md
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as u32)
                .unwrap_or(0);

            if is_dir && recursive && !is_reparse(&md) {
                let p = e.path();
                if exclude.iter().any(|x| p.starts_with(x)) {
                    continue;
                }
                stack.push(p);
            }
            part.ents.push(RawEnt { dir: di, name, is_dir, mtime });
        }
    }
    part
}

impl FileIndex {
    /// Varre as raízes em paralelo (cada subpasta de primeiro nível é um trabalho).
    pub fn build(roots: &[PathBuf], is_drive: bool, personal_roots: &[PathBuf], exclude: &[PathBuf]) -> FileIndex {
        let mut jobs: Vec<(PathBuf, bool)> = Vec::new();
        for r in roots {
            jobs.push((r.clone(), false)); // arquivos soltos na raiz
            if let Ok(rd) = std::fs::read_dir(r) {
                for e in rd.flatten() {
                    let Ok(md) = e.metadata() else { continue };
                    if !md.is_dir() || is_reparse(&md) || is_system(&md) {
                        continue;
                    }
                    let name = e.file_name().to_string_lossy().into_owned();
                    if name.starts_with('.') || NOISE_DIRS.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                        continue;
                    }
                    if is_drive && SYSTEM_ROOT_DIRS.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                        continue;
                    }
                    if exclude.iter().any(|x| e.path().starts_with(x)) {
                        continue;
                    }
                    jobs.push((e.path(), true));
                }
            }
        }

        let budget = MAX_ENTRIES / jobs.len().max(1) + 50_000;
        let parts: Vec<Part> = jobs.par_iter().map(|(p, rec)| scan(p, *rec, budget, exclude)).collect();

        let mut idx = FileIndex::default();
        for part in parts {
            if idx.ents.len() >= MAX_ENTRIES {
                break;
            }
            let base = idx.dirs.len() as u32;
            for d in &part.dirs {
                let p = Path::new(d);
                idx.personal.push(personal_roots.iter().any(|r| p.starts_with(r)));
            }
            idx.dirs.extend(part.dirs);
            for e in part.ents {
                let lower = e.name.to_lowercase();
                let noff = idx.names.len() as u32;
                let loff = idx.lower.len() as u32;
                idx.names.push_str(&e.name);
                idx.lower.push_str(&lower);
                idx.ents.push(Ent {
                    dir: base + e.dir,
                    noff,
                    nlen: e.name.len().min(u16::MAX as usize) as u16,
                    loff,
                    llen: lower.len().min(u16::MAX as usize) as u16,
                    is_dir: e.is_dir,
                    mtime: e.mtime,
                });
            }
        }
        idx.ents.shrink_to_fit();
        idx.dirs.shrink_to_fit();
        idx
    }

    pub fn len(&self) -> usize {
        self.ents.len()
    }

    fn name(&self, e: &Ent) -> &str {
        &self.names[e.noff as usize..e.noff as usize + e.nlen as usize]
    }
    fn lname(&self, e: &Ent) -> &str {
        &self.lower[e.loff as usize..e.loff as usize + e.llen as usize]
    }

    fn hit(&self, e: &Ent, score: i32) -> Hit {
        let dir = &self.dirs[e.dir as usize];
        let name = self.name(e);
        let path = Path::new(dir).join(name).to_string_lossy().into_owned();
        Hit {
            id: format!("{}:{}", if e.is_dir { "folder" } else { "file" }, path),
            kind: if e.is_dir { "folder".into() } else { "file".into() },
            title: name.to_string(),
            subtitle: dir.clone(),
            path,
            score,
            mtime: e.mtime,
        }
    }

    pub fn search(&self, query: &str, max_files: usize, max_dirs: usize) -> Vec<Hit> {
        let q = query.trim().to_lowercase();
        if q.chars().count() < 2 {
            return vec![];
        }
        // "maio .docx" = o nome precisa conter TODAS as palavras (em qualquer ordem); ".docx" também vale como extensão.
        let toks: Vec<&str> = q.split_whitespace().collect();
        let main = toks.iter().filter(|t| !t.starts_with('.')).max_by_key(|t| t.chars().count()).copied().unwrap_or(q.as_str());
        let ext = toks.iter().find(|t| t.starts_with('.') && t.len() > 1).copied();

        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);

        let mut cands: Vec<(i32, usize)> = self
            .ents
            .par_chunks(65_536)
            .enumerate()
            .flat_map_iter(|(ci, chunk)| {
                let mut out = Vec::new();
                for (i, e) in chunk.iter().enumerate() {
                    let ln = self.lname(e);
                    if toks.iter().all(|t| ln.contains(t)) {
                        let mut s = fuzzy::score_lower(main, ln).max(100);
                        if ext.map(|x| ln.ends_with(x)).unwrap_or(false) {
                            s += 30;
                        }
                        if self.personal[e.dir as usize] {
                            s += 40;
                        }
                        if e.mtime > 0 && now.saturating_sub(e.mtime) < 30 * 86_400 {
                            s += 25;
                        }
                        out.push((s, ci * 65_536 + i));
                        if out.len() >= MAX_CANDIDATES {
                            break;
                        }
                    }
                }
                out
            })
            .collect();

        cands.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| self.ents[a.1].nlen.cmp(&self.ents[b.1].nlen)));

        let (mut files, mut dirs) = (0, 0);
        let mut out = Vec::new();
        for (s, i) in cands {
            let e = &self.ents[i];
            if e.is_dir {
                if dirs >= max_dirs {
                    continue;
                }
                dirs += 1;
            } else {
                if files >= max_files {
                    continue;
                }
                files += 1;
            }
            out.push(self.hit(e, s));
            if files >= max_files && dirs >= max_dirs {
                break;
            }
        }
        out
    }

    /// Itens mais recentes das pastas pessoais. `kind`: all | images | docs | pdf | media | folders
    pub fn recent(&self, kind: &str, limit: usize) -> Vec<Hit> {
        let mut items: Vec<&Ent> = self
            .ents
            .iter()
            .filter(|e| self.personal[e.dir as usize] && e.mtime > 0)
            .filter(|e| matches_kind(kind, self.lname(e), e.is_dir))
            .collect();

        if items.len() > limit {
            items.select_nth_unstable_by(limit, |a, b| b.mtime.cmp(&a.mtime));
            items.truncate(limit);
        }
        items.sort_by(|a, b| b.mtime.cmp(&a.mtime));
        items.into_iter().map(|e| self.hit(e, 0)).collect()
    }
}

fn ext_of(lname: &str) -> &str {
    lname.rsplit_once('.').map(|(_, e)| e).unwrap_or("")
}

fn matches_kind(kind: &str, lname: &str, is_dir: bool) -> bool {
    if kind == "folders" {
        return is_dir;
    }
    if is_dir {
        return kind == "all";
    }
    let ext = ext_of(lname);
    if matches!(ext, "lnk" | "ini" | "tmp" | "log" | "dat" | "db" | "url" | "map" | "lock" | "sqlite") {
        return false;
    }
    match kind {
        "images" => matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "heic" | "avif" | "ico" | "psd"),
        "docs" => matches!(ext, "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" | "odt" | "ods" | "csv" | "rtf" | "pdf"),
        "pdf" => ext == "pdf",
        "media" => matches!(ext, "mp4" | "mkv" | "mov" | "avi" | "webm" | "mp3" | "wav" | "flac" | "m4a" | "ogg"),
        _ => true,
    }
}

pub fn personal_roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    for d in [
        dirs::desktop_dir(),
        dirs::document_dir(),
        dirs::download_dir(),
        dirs::picture_dir(),
        dirs::video_dir(),
        dirs::audio_dir(),
    ]
    .into_iter()
    .flatten()
    {
        if d.exists() && !v.contains(&d) {
            v.push(d);
        }
    }
    v
}

#[cfg(windows)]
pub fn drive_roots() -> Vec<PathBuf> {
    ('A'..='Z')
        .map(|c| PathBuf::from(format!("{}:\\", c)))
        .filter(|p| p.exists())
        .filter(|p| {
            // somente discos fixos
            use std::os::windows::ffi::OsStrExt;
            let w: Vec<u16> = p.as_os_str().encode_wide().chain(Some(0)).collect();
            unsafe { windows::Win32::Storage::FileSystem::GetDriveTypeW(windows::core::PCWSTR(w.as_ptr())) == 3 }
        })
        .collect()
}

/// Linux: todos os discos reais montados (ext4, btrfs, xfs, ntfs, exfat...), sem os diretórios do sistema.
#[cfg(target_os = "linux")]
pub fn drive_roots() -> Vec<PathBuf> {
    const REAL_FS: &[&str] = &[
        "ext2", "ext3", "ext4", "btrfs", "xfs", "f2fs", "jfs", "reiserfs", "zfs", "ntfs", "ntfs3", "fuseblk", "vfat", "exfat",
    ];
    // Do disco do sistema só entram as pastas de dados (o resto de "/" é ruído: usr, var, etc...).
    const ROOT_DATA_DIRS: &[&str] = &["/home", "/root", "/mnt", "/media", "/srv", "/opt"];

    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let mut f = line.split_whitespace();
            let (_dev, mp, fs) = (f.next(), f.next(), f.next());
            let (Some(mp), Some(fs)) = (mp, fs) else { continue };
            if !REAL_FS.contains(&fs) {
                continue;
            }
            let mp = mp.replace("\\040", " "); // espaços aparecem escapados em /proc/mounts
            if mp == "/" {
                roots.extend(ROOT_DATA_DIRS.iter().map(PathBuf::from).filter(|p| p.is_dir()));
            } else if !["/boot", "/snap", "/var", "/usr", "/etc", "/tmp"].iter().any(|x| mp.starts_with(x)) {
                roots.push(PathBuf::from(mp));
            }
        }
    }
    if let Some(h) = dirs::home_dir() {
        roots.push(h);
    }
    // sem duplicatas nem pastas dentro de outra raiz já incluída
    roots.sort();
    roots.dedup();
    let all = roots.clone();
    roots.retain(|r| !all.iter().any(|o| o != r && r.starts_with(o)));
    roots
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn drive_roots() -> Vec<PathBuf> {
    dirs::home_dir().into_iter().collect()
}
