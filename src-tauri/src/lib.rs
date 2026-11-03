mod actions;
mod autostart;
mod apps;
mod clip;
mod config;
mod fuzzy;
mod index;
mod power;
mod settings;
mod shell_img;
mod store;
mod theme;
mod util;
mod window;

use apps::AppEntry;
use clip::{ClipItem, ClipStore};
use config::Config;
use index::{FileIndex, Hit};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use store::Store;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalSize, Manager, State, WindowEvent,
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub index: RwLock<Arc<FileIndex>>,
    pub apps: RwLock<Vec<AppEntry>>,
    pub clip: Arc<ClipStore>,
    pub usage: Store<HashMap<String, u32>>,
    pub quick: Store<HashMap<String, Value>>,
    pub backdrop: Mutex<Option<Arc<Vec<u8>>>>,
    pub backdrop_seq: AtomicU64,
    pub shown_at: Mutex<Instant>,
    pub icons: Mutex<HashMap<String, Arc<Vec<u8>>>>,
    pub indexing: AtomicBool,
    pub config: Store<Config>,
    pub hotkey_ok: AtomicBool,
    /// Diálogo nativo aberto: a janela não pode se esconder ao perder o foco.
    pub modal: AtomicBool,
}

type St<'a> = State<'a, Arc<AppState>>;

// ---------------------------------------------------------------- indexação

fn reindex(state: Arc<AppState>) {
    if state.indexing.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        power::background_thread();
        let cfg = state.config.data.lock().unwrap().clone();

        // aplicativos
        let apps = apps::scan();
        if !apps.is_empty() {
            apps::save_cache(&apps);
            *state.apps.write().unwrap() = apps;
        }

        // Varredura com poucas threads de baixa prioridade: o PC continua fluido durante a indexação.
        let workers = (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2) / 2).clamp(1, 4);
        let Ok(pool) = rayon::ThreadPoolBuilder::new().num_threads(workers).start_handler(|_| power::background_thread()).build() else {
            state.indexing.store(false, Ordering::SeqCst);
            return;
        };

        // fase 1: pastas pessoais + extras; fase 2: todos os discos (opcional, pausada na bateria)
        let mut personal = index::personal_roots();
        personal.extend(cfg.include_paths());
        let exclude = cfg.exclude_paths();
        let t = Instant::now();
        let phase1 = pool.install(|| FileIndex::build(&personal, false, &personal, &exclude));
        *state.index.write().unwrap() = Arc::new(phase1);

        if cfg.all_drives && !(cfg.pause_on_battery && power::on_battery()) {
            let mut roots = index::drive_roots();
            roots.extend(cfg.include_paths());
            let phase2 = pool.install(|| FileIndex::build(&roots, true, &personal, &exclude));
            eprintln!("índice: {} itens em {:?}", phase2.len(), t.elapsed());
            *state.index.write().unwrap() = Arc::new(phase2);
        }

        state.indexing.store(false, Ordering::SeqCst);
    });
}

// ---------------------------------------------------------------- comandos

#[tauri::command]
async fn search(state: St<'_>, query: String, filter: String) -> Result<Vec<Hit>, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut out: Vec<Hit> = Vec::new();
        let idx = state.index.read().unwrap().clone();
        let (max_apps, max_files, max_dirs, max_settings) = match filter.as_str() {
            "apps" => (80, 0, 0, 0),
            "files" => (0, 40, 15, 0),
            "actions" => (0, 0, 0, 20),
            _ => (8, 6, 4, 4),
        };

        if max_apps > 0 {
            let apps = state.apps.read().unwrap();
            out.extend(apps::search(&apps, &query).into_iter().take(max_apps).map(|mut h| {
                h.score += 40;
                h
            }));
        }
        if max_files + max_dirs > 0 {
            out.extend(idx.search(&query, max_files, max_dirs));
        }
        if max_settings > 0 {
            out.extend(settings::search(&query, true).into_iter().take(max_settings));
        }
        out
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_apps(state: St<'_>) -> Result<Vec<Hit>, String> {
    let usage = state.usage.data.lock().unwrap().clone();
    let apps = state.apps.read().unwrap();
    let mut v: Vec<Hit> = apps
        .iter()
        .map(|a| {
            let mut h = apps::to_hit(a, 0);
            h.score = *usage.get(&h.id).unwrap_or(&0) as i32;
            h
        })
        .collect();
    v.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase())));
    Ok(v)
}

#[tauri::command]
async fn recent(state: St<'_>, kind: String, limit: usize) -> Result<Vec<Hit>, String> {
    let idx = state.index.read().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || idx.recent(&kind, limit))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_item(state: St<'_>, id: String, kind: String, path: String) -> Result<(), String> {
    {
        let mut u = state.usage.data.lock().unwrap();
        *u.entry(id).or_insert(0) += 1;
    }
    state.usage.save();

    match kind.as_str() {
        "app" => {
            let exec = state
                .apps
                .read()
                .unwrap()
                .iter()
                .find(|a| a.id == path)
                .map(|a| a.exec.clone())
                .unwrap_or_default();
            apps::launch(&path, &exec);
            Ok(())
        }
        _ => opener::open(&path).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
async fn reveal(path: String) -> Result<(), String> {
    opener::reveal(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn file_info(path: String) -> Result<Value, String> {
    let md = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    let mtime = md
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(json!({ "size": md.len(), "mtime": mtime, "dir": md.is_dir() }))
}

#[tauri::command]
async fn clip_list(state: St<'_>) -> Result<Vec<ClipItem>, String> {
    Ok(state.clip.list())
}
#[tauri::command]
async fn clip_copy(state: St<'_>, id: u64) -> Result<(), String> {
    state.clip.copy(id);
    Ok(())
}
#[tauri::command]
async fn clip_delete(state: St<'_>, id: u64) -> Result<(), String> {
    state.clip.delete(id);
    Ok(())
}
#[tauri::command]
async fn clip_clear(state: St<'_>) -> Result<(), String> {
    state.clip.clear();
    Ok(())
}
#[tauri::command]
async fn copy_text(state: St<'_>, text: String) -> Result<(), String> {
    state.clip.set_text(&text);
    Ok(())
}

#[tauri::command]
async fn run_action(app: AppHandle, state: St<'_>, id: String, args: Value) -> Result<String, String> {
    {
        let mut u = state.usage.data.lock().unwrap();
        *u.entry(format!("action:{}", id)).or_insert(0) += 1;
    }
    state.usage.save();
    actions::run(&app, &id, &args)
}

#[tauri::command]
async fn usage_get(state: St<'_>) -> Result<HashMap<String, u32>, String> {
    Ok(state.usage.data.lock().unwrap().clone())
}

#[tauri::command]
async fn quick_get(state: St<'_>) -> Result<HashMap<String, Value>, String> {
    Ok(state.quick.data.lock().unwrap().clone())
}
#[tauri::command]
async fn quick_set(state: St<'_>, keyword: String, item: Value) -> Result<(), String> {
    let kw = keyword.trim().to_lowercase();
    if kw.is_empty() {
        return Ok(());
    }
    let id = item["id"].clone();
    {
        let mut q = state.quick.data.lock().unwrap();
        q.retain(|_, v| v["id"] != id); // um item só tem um atalho
        q.insert(kw, item);
    }
    state.quick.save();
    Ok(())
}
#[tauri::command]
async fn quick_remove(state: St<'_>, keyword: String) -> Result<(), String> {
    state.quick.data.lock().unwrap().remove(&keyword.to_lowercase());
    state.quick.save();
    Ok(())
}

#[tauri::command]
async fn hide_window(app: AppHandle) -> Result<(), String> {
    window::hide(&app);
    Ok(())
}

#[tauri::command]
async fn resize_window(app: AppHandle, height: f64) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        let h = height.clamp(window::WIN_H, window::WIN_MAX_H);
        let _ = w.set_size(LogicalSize::new(window::WIN_W, h));
    }
    Ok(())
}

#[tauri::command]
async fn reindex_now(state: St<'_>) -> Result<(), String> {
    reindex(state.inner().clone());
    Ok(())
}

// ---------------------------------------------------------------- configurações

/// Registra `new` no lugar de `old`; se falhar (atalho em uso), restaura o anterior.
fn apply_hotkey(app: &AppHandle, new: &str, old: &str) -> Result<(), String> {
    let ns: Shortcut = new.parse().map_err(|e| format!("Atalho inválido: {e}"))?;
    let gs = app.global_shortcut();
    let os = old.parse::<Shortcut>().ok();
    if let Some(o) = os {
        let _ = gs.unregister(o);
    }
    if let Err(e) = gs.register(ns) {
        if let Some(o) = os {
            let _ = gs.register(o);
        }
        eprintln!("atalho {new}: {e}");
        return Err(format!("{new} já está em uso por outro programa. Escolha outro atalho."));
    }
    Ok(())
}

fn config_json(state: &AppState) -> Value {
    json!({
        "config": &*state.config.data.lock().unwrap(),
        "hotkeyOk": state.hotkey_ok.load(Ordering::SeqCst),
        "wayland": power::wayland(),
        "windows": cfg!(windows),
    })
}

#[tauri::command]
fn config_get(state: St<'_>) -> Value {
    config_json(&state)
}

#[tauri::command]
async fn config_set(app: AppHandle, state: St<'_>, config: Config) -> Result<Value, String> {
    let state = state.inner().clone();
    let new = config.sanitized()?;
    let old = state.config.data.lock().unwrap().clone();

    if new.hotkey != old.hotkey || !state.hotkey_ok.load(Ordering::SeqCst) {
        apply_hotkey(&app, &new.hotkey, &old.hotkey)?;
        state.hotkey_ok.store(true, Ordering::SeqCst);
    }
    if new.autostart != autostart::is_enabled() {
        autostart::set(new.autostart)?;
    }
    *config::THEME.write().unwrap() = new.theme.clone();
    let reindex_needed = new.include != old.include || new.exclude != old.exclude || new.all_drives != old.all_drives;
    *state.config.data.lock().unwrap() = new;
    state.config.save();
    if reindex_needed {
        reindex(state.clone());
    }
    Ok(config_json(&state))
}

/// Abre um diálogo nativo sem deixar a janela se esconder por perda de foco.
async fn with_dialog<T: Send + 'static>(
    app: &AppHandle,
    state: &AppState,
    f: impl FnOnce(AppHandle) -> T + Send + 'static,
) -> Result<T, String> {
    state.modal.store(true, Ordering::SeqCst);
    let handle = app.clone();
    let out = tauri::async_runtime::spawn_blocking(move || f(handle)).await.map_err(|e| e.to_string());
    state.modal.store(false, Ordering::SeqCst);
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_focus();
    }
    out
}

#[tauri::command]
async fn config_export(app: AppHandle, state: St<'_>) -> Result<String, String> {
    let json = serde_json::to_string_pretty(&*state.config.data.lock().unwrap()).map_err(|e| e.to_string())?;
    let picked = with_dialog(&app, &state, |a| {
        a.dialog().file().set_file_name("glint-config.json").add_filter("JSON", &["json"]).blocking_save_file()
    })
    .await?;
    let Some(path) = picked else { return Ok(String::new()) };
    let path = path.into_path().map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path.display().to_string())
}

/// Lê e valida o arquivo, mas só devolve: quem aplica é o botão Salvar da tela.
#[tauri::command]
async fn config_import(app: AppHandle, state: St<'_>) -> Result<Option<Config>, String> {
    let picked = with_dialog(&app, &state, |a| a.dialog().file().add_filter("JSON", &["json"]).blocking_pick_file()).await?;
    let Some(path) = picked else { return Ok(None) };
    let path = path.into_path().map_err(|e| e.to_string())?;
    if std::fs::metadata(&path).map(|m| m.len() > 1 << 20).unwrap_or(true) {
        return Err("Arquivo inválido ou grande demais.".into());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let cfg: Config = serde_json::from_str(&text).map_err(|e| format!("Não é uma configuração do Glint: {e}"))?;
    cfg.sanitized().map(Some)
}

// ---------------------------------------------------------------- ícones (ico://)

fn handle_ico(app: &AppHandle, query: &str) -> tauri::http::Response<Vec<u8>> {
    let state = app.state::<Arc<AppState>>();
    let ok = |bytes: Vec<u8>| {
        tauri::http::Response::builder()
            .header("Content-Type", if bytes.starts_with(&[0xFF, 0xD8]) { "image/jpeg" } else { "image/png" })
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "max-age=600")
            .body(bytes)
            .unwrap()
    };
    let none = || {
        tauri::http::Response::builder()
            .status(404)
            .header("Cache-Control", "no-store")
            .body(Vec::new())
            .unwrap()
    };

    // imagens do histórico da área de transferência (?c=id[&th=1])
    if let Some(id) = util::query_param(query, "c").and_then(|v| v.parse::<u64>().ok()) {
        let thumb = util::query_param(query, "th").is_some();
        return match state.clip.blob(id, thumb) {
            Some(b) => ok((*b).clone()),
            None => none(),
        };
    }

    // fundo capturado para o vidro fosco
    if util::query_param(query, "bd").is_some() {
        return match state.backdrop.lock().unwrap().clone() {
            Some(b) => ok((*b).clone()),
            None => none(),
        };
    }

    let size: i32 = util::query_param(query, "s").and_then(|s| s.parse().ok()).unwrap_or(64).clamp(16, 512);
    let thumb = util::query_param(query, "t").as_deref() == Some("1");

    let name = if let Some(a) = util::query_param(query, "a") {
        format!("shell:AppsFolder\\{}", a)
    } else if let Some(p) = util::query_param(query, "p") {
        p
    } else {
        return none();
    };

    let key = format!("{}|{}|{}", name, size, thumb);
    if let Some(b) = state.icons.lock().unwrap().get(&key) {
        return ok((**b).clone());
    }

    // O shell às vezes falha de forma transitória sob carga: tenta de novo antes de desistir.
    let mut img = None;
    for attempt in 0..3 {
        img = shell_img::shell_image(&name, size, thumb);
        if img.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25 * (attempt + 1)));
    }

    match img {
        Some(png) => {
            let mut cache = state.icons.lock().unwrap();
            if cache.len() > 900 {
                cache.clear();
            }
            cache.insert(key, Arc::new(png.clone()));
            ok(png)
        }
        None => none(),
    }
}

// ---------------------------------------------------------------- app

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState {
        index: RwLock::new(Arc::new(FileIndex::default())),
        apps: RwLock::new(apps::load_cached()),
        clip: Arc::new(ClipStore::default()),
        usage: Store::open("usage.json"),
        quick: Store::open("quick_keys.json"),
        backdrop: Mutex::new(None),
        backdrop_seq: AtomicU64::new(0),
        shown_at: Mutex::new(Instant::now()),
        icons: Mutex::new(HashMap::new()),
        indexing: AtomicBool::new(false),
        config: Store::open("config.json"),
        hotkey_ok: AtomicBool::new(false),
        modal: AtomicBool::new(false),
    });
    *config::THEME.write().unwrap() = state.config.data.lock().unwrap().theme.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| window::show(app)))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _sc, event| {
                    // só registramos um atalho: o configurado
                    if event.state() == ShortcutState::Pressed {
                        window::toggle(app);
                    }
                })
                .build(),
        )
        .register_asynchronous_uri_scheme_protocol("ico", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let query = request.uri().query().unwrap_or("").to_string();
            rayon::spawn(move || responder.respond(handle_ico(&app, &query)));
        })
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            search, list_apps, recent, open_item, reveal, file_info, clip_list, clip_copy, clip_delete,
            clip_clear, copy_text, run_action, usage_get, quick_get, quick_set, quick_remove, hide_window,
            resize_window, reindex_now, config_get, config_set, config_export, config_import
        ])
        .on_window_event(|win, event| {
            if let WindowEvent::Focused(false) = event {
                let st = win.app_handle().state::<Arc<AppState>>();
                let since = st.shown_at.lock().unwrap().elapsed();
                let busy = st.modal.load(Ordering::SeqCst) || std::env::var_os("GLINT_KEEP").is_some();
                if since.as_millis() > 350 && !busy {
                    window::hide(win.app_handle());
                }
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();

            let hotkey = state.config.data.lock().unwrap().hotkey.clone();
            match apply_hotkey(&handle, &hotkey, "") {
                Ok(()) => state.hotkey_ok.store(true, Ordering::SeqCst),
                Err(e) => eprintln!("atalho: {e}"),
            }

            clip::start_watcher(state.clip.clone());
            reindex(state.clone());
            // reindexação periódica (a própria reindex é educada: baixa prioridade e pausa na bateria)
            let st_tick = state.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(30 * 60));
                reindex(st_tick.clone());
            });

            // Na primeira execução liga o início com o sistema (configurável depois).
            let marker = store::data_dir().join("first_run_done");
            if !marker.exists() {
                let _ = std::fs::create_dir_all(store::data_dir());
                let _ = std::fs::write(&marker, "1");
                if let Err(e) = autostart::set(state.config.data.lock().unwrap().autostart) {
                    eprintln!("autostart: {e}");
                }
            }

            // Bandeja do sistema
            let open_i = MenuItem::with_id(app, "open", "Abrir Glint", true, None::<&str>)?;
            let prefs_i = MenuItem::with_id(app, "prefs", "Configurações…", true, None::<&str>)?;
            let reindex_i = MenuItem::with_id(app, "reindex", "Reindexar arquivos", true, None::<&str>)?;
            let auto_i = CheckMenuItem::with_id(
                app,
                "autostart",
                "Iniciar com o sistema",
                true,
                autostart::is_enabled(),
                None::<&str>,
            )?;
            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&open_i, &prefs_i, &reindex_i, &auto_i, &PredefinedMenuItem::separator(app)?, &quit_i],
            )?;

            let st_menu = state.clone();
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Glint")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, ev| match ev.id().as_ref() {
                    "open" => window::show(app),
                    "prefs" => {
                        window::show(app);
                        let _ = app.emit("open-settings", ());
                    }
                    "reindex" => reindex(st_menu.clone()),
                    "autostart" => {
                        let _ = autostart::set(!autostart::is_enabled());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        window::toggle(tray.app_handle());
                    }
                })
                .build(app)?;

            if std::env::args().any(|a| a == "--show") {
                window::show(&handle);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Glint");
}
