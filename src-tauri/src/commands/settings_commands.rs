use tauri::State;

use crate::{controllers::settings_controller, AppState};

#[tauri::command]
pub fn save_ui_settings(
    always_on_top: bool,
    open_on_start: bool,
    quickpanel_opacity: f64,
    project_sort_mode: String,
    quickpanel_mode: String,
    project_manual_order: Vec<String>,
    project_recent_usage: std::collections::HashMap<String, u64>,
    timesheet_rounding_enabled: bool,
    state: State<AppState>,
    app: tauri::AppHandle,
) {
    settings_controller::save_ui_settings(
        always_on_top,
        open_on_start,
        quickpanel_opacity,
        project_sort_mode,
        quickpanel_mode,
        project_manual_order,
        project_recent_usage,
        timesheet_rounding_enabled,
        &state,
        &app,
    );
}

#[tauri::command]
pub fn set_timesheet_rounding_enabled(
    enabled: bool,
    state: State<AppState>,
    app: tauri::AppHandle,
) {
    settings_controller::set_timesheet_rounding_enabled(enabled, &state, &app);
}

#[tauri::command]
pub fn save_quickpanel_bounds(x: f64, y: f64, width: f64, height: f64, state: State<AppState>) {
    settings_controller::save_quickpanel_bounds(x, y, width, height, &state);
}
