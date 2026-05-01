use tauri::State;

use crate::{controllers::timesheet_controller, AppState, TimesheetPreviewBootstrap};
use crate::timesheet::TimesheetPreview;

#[tauri::command]
pub fn generate_timesheet(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    timesheet_controller::generate_timesheet(&state, &app)
}

#[tauri::command]
pub fn preview_timesheet(
    range: String,
    format: String,
    state: State<AppState>,
) -> Result<TimesheetPreview, String> {
    timesheet_controller::preview_timesheet(range, format, &state)
}

#[tauri::command]
pub fn get_timesheet_preview_bootstrap(state: State<AppState>) -> TimesheetPreviewBootstrap {
    timesheet_controller::get_timesheet_preview_bootstrap(&state)
}

#[tauri::command]
pub async fn open_timesheet_preview_window(
    range: String,
    format: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    timesheet_controller::open_timesheet_preview_window(range, format, &app).await
}

#[tauri::command]
pub fn hide_timesheet_preview_window(app: tauri::AppHandle) -> Result<(), String> {
    timesheet_controller::hide_timesheet_preview_window(&app)
}

#[tauri::command]
pub fn generate_timesheet_export(
    range: String,
    format: String,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    timesheet_controller::generate_timesheet_export(range, format, &state, &app)
}

#[tauri::command]
pub fn reset_timesheet(state: State<AppState>, app: tauri::AppHandle) {
    timesheet_controller::reset_timesheet(&state, &app);
}
