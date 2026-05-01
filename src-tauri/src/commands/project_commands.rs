use tauri::State;

use crate::{controllers::project_controller, AppState};

#[tauri::command]
pub fn submit_input(mode: String, value: String, state: State<AppState>, app: tauri::AppHandle) {
    project_controller::submit_input(mode, value, &state, &app);
}

#[tauri::command]
pub fn get_state(state: State<AppState>) -> crate::ProjectLogState {
    crate::log_debug!("get_state");
    project_controller::get_state(&state)
}

#[tauri::command]
pub fn select_project(project: String, state: State<AppState>, app: tauri::AppHandle) {
    project_controller::select_project(project, &state, &app);
}

#[tauri::command]
pub fn add_project(value: String, state: State<AppState>, app: tauri::AppHandle) {
    project_controller::add_project(value, &state, &app);
}

#[tauri::command]
pub fn quick_project(value: String, state: State<AppState>, app: tauri::AppHandle) {
    project_controller::quick_project(value, &state, &app);
}

#[tauri::command]
pub fn set_comment(value: String, state: State<AppState>, app: tauri::AppHandle) {
    project_controller::set_comment(value, &state, &app);
}

#[tauri::command]
pub fn remove_project(project: String, state: State<AppState>, app: tauri::AppHandle) {
    project_controller::remove_project(project, &state, &app);
}

#[tauri::command]
pub fn reset_projects(state: State<AppState>, app: tauri::AppHandle) {
    crate::log_warn!("reset_projects");
    let mut projs = state.projects.lock().unwrap();
    projs.clear();
    crate::projects::save(&state.data_dir, &projs);
    drop(projs);
    crate::tray::rebuild_menu(&app);
    crate::emit_state_changed(&app);
}
