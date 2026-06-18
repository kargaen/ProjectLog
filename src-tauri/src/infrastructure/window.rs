use tauri::{Manager, WindowEvent};

use crate::{logger, AppState};

pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == "main" || window.label() == "timesheet-preview" {
            api.prevent_close();
            let _ = window.hide();
        }
    }
}

pub fn handle_run_event(app: &tauri::AppHandle, event: &tauri::RunEvent) {
    match event {
        tauri::RunEvent::ExitRequested { code, api, .. } => {
            if code.is_none() {
                api.prevent_exit();
            }
        }
        tauri::RunEvent::Exit => {
            let state = app.state::<AppState>();
            logger::log_new_entry(&state.data_dir, "", "");
        }
        _ => {}
    }
}
