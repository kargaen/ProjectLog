use std::sync::atomic::Ordering;

use tauri::{Emitter, Manager, State};

use crate::{emit_state_changed, log, tray, AppState};

pub fn open_log_file(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    log!("open_log_file");
    let log_path = state.data_dir.join("log.dat");
    if !log_path.exists() {
        std::fs::File::create(&log_path).map_err(|e| e.to_string())?;
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(log_path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_diagnostic_log(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    log!("open_diagnostic_log");
    let log_path = state.data_dir.join("ProjectLog-debug.log");
    if !log_path.exists() {
        std::fs::File::create(&log_path).map_err(|e| e.to_string())?;
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(log_path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_feedback(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_feedback mailto");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(
            "mailto:karga@karga.dk?subject=ProjectLog%20feedback&body=Hello%2C%0A%0AI%20have%20ProjectLog%20feedback%3A%0A",
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

pub fn open_github_issues(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_github_issues");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog/issues", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_portfolio(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_portfolio");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://kargaen.github.io/", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_project_homepage(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_project_homepage");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_release_notes(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_release_notes");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog/releases", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn set_update_available(available: bool, state: State<AppState>, app: tauri::AppHandle) {
    log!("set_update_available available={}", available);
    state.update_available.store(available, Ordering::Relaxed);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

pub fn log_from_frontend(level: String, module: String, message: String, data: Option<String>) {
    crate::diagnostics::frontend(&level, &module, &message, data);
}
