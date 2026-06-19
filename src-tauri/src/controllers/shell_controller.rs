use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, LogicalSize, Manager, WebviewWindowBuilder};

use crate::{
    emit_state_changed, log, tray, AppState, TimesheetPreviewRequest,
};

fn open_path(path: &std::path::Path, app: &AppHandle) -> Result<(), String> {
    if !path.exists() {
        std::fs::File::create(path).map_err(|e| e.to_string())?;
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_log_file(state: &AppState, app: &AppHandle) -> Result<(), String> {
    log!("open_log_file");
    let log_path = state.data_dir.join("log.dat");
    open_path(&log_path, app)
}

pub fn open_diagnostic_log(state: &AppState, app: &AppHandle) -> Result<(), String> {
    log!("open_diagnostic_log");
    let log_path = state.data_dir.join("ProjectLog-debug.log");
    open_path(&log_path, app)
}

pub fn open_feedback(app: &AppHandle) -> Result<(), String> {
    log!("open_feedback mailto");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(
            "mailto:karga@karga.dk?subject=ProjectLog%20feedback&body=Hello%2C%0A%0AI%20have%20ProjectLog%20feedback%3A%0A",
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

pub fn open_github_issues(app: &AppHandle) -> Result<(), String> {
    log!("open_github_issues");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog/issues", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_portfolio(app: &AppHandle) -> Result<(), String> {
    log!("open_portfolio");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://kargaen.github.io/", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_project_homepage(app: &AppHandle) -> Result<(), String> {
    log!("open_project_homepage");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn open_release_notes(app: &AppHandle) -> Result<(), String> {
    log!("open_release_notes");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog/releases", None::<&str>)
        .map_err(|e| e.to_string())
}

pub fn show_quickpanel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn show_update_prompt(app: &AppHandle) {
    show_quickpanel(app);
    let _ = app.emit("show-update-prompt", ());
}

pub fn show_about(app: &AppHandle) {
    show_quickpanel(app);
    let _ = app.emit("show-about", ());
}

pub fn show_input_prompt(state: &AppState, app: &AppHandle, mode: &str, title: &str) {
    let current_value = match mode {
        "set_comment" => state.active_comment.lock().unwrap().clone(),
        _ => String::new(),
    };

    if let Some(window) = app.get_webview_window("main") {
        let was_visible = window.is_visible().unwrap_or(false);
        let _ = window.emit(
            "show-input",
            serde_json::json!({
                "mode": mode,
                "title": title,
                "value": current_value,
                "closeOnSubmit": !was_visible,
            }),
        );
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn show_timesheet_preview_window(
    app: &AppHandle,
    request: TimesheetPreviewRequest,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("timesheet-preview") {
        let _ = window.set_title(title);
        let _ = window.set_size(LogicalSize::new(width, height));
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("show-timesheet-preview", request);
        return Ok(());
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "timesheet-preview")
        .ok_or_else(|| "Missing timesheet-preview window config.".to_string())?;

    let window = WebviewWindowBuilder::from_config(app, config)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let _ = window.set_title(title);
    let _ = window.set_size(LogicalSize::new(width, height));
    let _ = window.show();
    let _ = window.set_focus();

    Ok(())
}

pub fn hide_timesheet_preview_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("timesheet-preview") {
        window.hide().map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn set_update_available(available: bool, state: &AppState, app: &AppHandle) {
    log!("set_update_available available={}", available);
    state.update_available.store(available, Ordering::Relaxed);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn log_from_frontend(level: String, module: String, message: String, data: Option<String>) {
    crate::diagnostics::frontend(&level, &module, &message, data);
}
