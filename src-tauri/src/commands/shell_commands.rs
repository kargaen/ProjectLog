use tauri::State;

use crate::{controllers::shell_controller, AppState};

#[tauri::command]
pub fn open_log_file(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    shell_controller::open_log_file(&state, &app)
}

#[tauri::command]
pub fn open_diagnostic_log(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    shell_controller::open_diagnostic_log(&state, &app)
}

#[tauri::command]
pub fn open_github_issues(app: tauri::AppHandle) -> Result<(), String> {
    shell_controller::open_github_issues(&app)
}

#[tauri::command]
pub fn open_portfolio(app: tauri::AppHandle) -> Result<(), String> {
    shell_controller::open_portfolio(&app)
}

#[tauri::command]
pub fn open_project_homepage(app: tauri::AppHandle) -> Result<(), String> {
    shell_controller::open_project_homepage(&app)
}

#[tauri::command]
pub fn open_release_notes(app: tauri::AppHandle) -> Result<(), String> {
    shell_controller::open_release_notes(&app)
}

#[tauri::command]
pub fn set_update_available(available: bool, state: State<AppState>, app: tauri::AppHandle) {
    shell_controller::set_update_available(available, &state, &app)
}

#[tauri::command]
pub fn log_from_frontend(level: String, module: String, message: String, data: Option<String>) {
    shell_controller::log_from_frontend(level, module, message, data)
}
