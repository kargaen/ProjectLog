mod logger;
mod projects;
mod timesheet;
mod tray;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{Manager, State};

pub struct AppState {
    pub active_project: Mutex<String>,
    pub active_comment: Mutex<String>,
    pub projects: Mutex<Vec<String>>,
    pub adhoc_projects: Mutex<Vec<String>>,
    pub data_dir: PathBuf,
    pub reminder_active: Arc<AtomicBool>,
}

#[tauri::command]
fn submit_input(mode: String, value: String, state: State<AppState>, app: tauri::AppHandle) {
    match mode.as_str() {
        "add_project" => {
            if !value.is_empty() {
                let mut projs = state.projects.lock().unwrap();
                if !projs.contains(&value) {
                    projs.push(value.clone());
                    projects::save(&state.data_dir, &projs);
                }
            }
        }
        "quick_project" => {
            if !value.is_empty() {
                state.reminder_active.store(false, Ordering::Relaxed);
                logger::log_new_entry(&state.data_dir, &value, "");
                *state.active_project.lock().unwrap() = value.clone();
                *state.active_comment.lock().unwrap() = String::new();

                // Add to ad-hoc list if not already known
                let in_permanent = state.projects.lock().unwrap().contains(&value);
                if !in_permanent {
                    let mut adhoc = state.adhoc_projects.lock().unwrap();
                    if !adhoc.contains(&value) {
                        adhoc.push(value);
                    }
                }
            }
        }
        "set_comment" => {
            let active = state.active_project.lock().unwrap().clone();
            let mut comment = state.active_comment.lock().unwrap();

            if !active.is_empty() {
                if comment.is_empty() && !value.is_empty() {
                    // No existing comment — append to last line
                    logger::append_comment_to_last(&state.data_dir, &value);
                } else if !comment.is_empty() && value != *comment {
                    // Changing comment — new log entry
                    logger::log_new_entry(&state.data_dir, &active, &value);
                }
                *comment = value;
            }
        }
        _ => {}
    }

    // Hide window and rebuild tray menu
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    tray::rebuild_menu(&app);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let project_list = projects::load(&data_dir);
            let reminder_active = Arc::new(AtomicBool::new(true));
            let reminder_clone = reminder_active.clone();

            let state = AppState {
                active_project: Mutex::new(String::new()),
                active_comment: Mutex::new(String::new()),
                projects: Mutex::new(project_list),
                adhoc_projects: Mutex::new(Vec::new()),
                data_dir: data_dir.clone(),
                reminder_active,
            };
            app.manage(state);

            // Log start (no active project)
            logger::log_new_entry(&data_dir, "", "");

            // Setup system tray
            tray::setup(app)?;

            // One-shot reminder: 5 minutes after launch, nudge if no project selected
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(300));
                if reminder_clone.load(Ordering::Relaxed) {
                    use tauri_plugin_notification::NotificationExt;
                    let _ = handle
                        .notification()
                        .builder()
                        .title("Activate project...")
                        .body("Remember to activate a project if you are working.")
                        .show();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![submit_input])
        .on_window_event(|window, event| {
            // Intercept close — just hide the input dialog window
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error building application")
        .run(|app, event| {
            match &event {
                tauri::RunEvent::ExitRequested { code, api, .. } => {
                    if code.is_none() {
                        // Last window closed — keep tray app alive
                        api.prevent_exit();
                    }
                    // If code is Some, it's an explicit app.exit() — let it through
                }
                tauri::RunEvent::Exit => {
                    // Log empty entry on any exit (menu Exit, OS shutdown, etc.)
                    let state = app.state::<AppState>();
                    logger::log_new_entry(&state.data_dir, "", "");
                }
                _ => {}
            }
        });
}
