mod commands {
    pub mod project_commands;
    pub mod shell_commands;
    pub mod settings_commands;
    pub mod timesheet_commands;
}
mod controllers {
    pub mod project_controller;
    pub mod shell_controller;
    pub mod settings_controller;
    pub mod timesheet_controller;
}
pub mod diagnostics;
mod logger;
mod projects;
mod settings;
mod timesheet;
mod tray;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use settings::UiSettings;
use tauri::{Emitter, Manager};

pub struct AppState {
    pub active_project: Mutex<String>,
    pub active_comment: Mutex<String>,
    pub projects: Mutex<Vec<String>>,
    pub adhoc_projects: Mutex<Vec<String>>,
    pub data_dir: PathBuf,
    pub reminder_active: Arc<AtomicBool>,
    pub update_available: AtomicBool,
    pub settings: Mutex<UiSettings>,
    timesheet_preview_request: Mutex<Option<TimesheetPreviewRequest>>,
}

#[derive(Serialize)]
struct ProjectLogState {
    app_version: String,
    active_project: String,
    active_comment: String,
    projects: Vec<String>,
    adhoc_projects: Vec<String>,
    update_available: bool,
    settings: UiSettings,
}

#[derive(Clone, Serialize)]
struct TimesheetPreviewRequest {
    range: String,
    format: String,
}

#[derive(Serialize)]
struct TimesheetPreviewBootstrap {
    request: Option<TimesheetPreviewRequest>,
    rounding_enabled: bool,
}

pub(crate) fn emit_state_changed(app: &tauri::AppHandle) {
    let _ = app.emit("state-changed", ());
}

fn migrate_legacy_file(data_dir: &PathBuf, filename: &str) {
    let target = data_dir.join(filename);
    if target.exists() {
        return;
    }

    let mut candidates = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join(filename));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join(filename));
        }
    }

    for source in candidates {
        if source.exists() && source != target {
            let _ = std::fs::copy(source, &target);
            break;
        }
    }
}

fn migrate_legacy_files(data_dir: &PathBuf) {
    migrate_legacy_file(data_dir, "projects.dat");
    migrate_legacy_file(data_dir, "log.dat");
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            diagnostics::init(&data_dir);
            log!("setup data_dir={}", data_dir.display());
            migrate_legacy_files(&data_dir);

            let project_list = projects::load(&data_dir);
            let ui_settings = settings::load(&data_dir);
            let reminder_active = Arc::new(AtomicBool::new(true));
            let reminder_clone = reminder_active.clone();

            let state = AppState {
                active_project: Mutex::new(String::new()),
                active_comment: Mutex::new(String::new()),
                projects: Mutex::new(project_list),
                adhoc_projects: Mutex::new(Vec::new()),
                data_dir: data_dir.clone(),
                reminder_active,
                update_available: AtomicBool::new(false),
                settings: Mutex::new(ui_settings),
                timesheet_preview_request: Mutex::new(None),
            };
            app.manage(state);

            logger::log_new_entry(&data_dir, "", "");
            log!("startup entry written");
            tray::setup(app)?;
            log!("tray initialized");

            use tauri_plugin_autostart::ManagerExt;
            let _ = app.autolaunch().enable();

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
        .invoke_handler(tauri::generate_handler![
            commands::project_commands::submit_input,
            commands::project_commands::get_state,
            commands::project_commands::select_project,
            commands::project_commands::add_project,
            commands::project_commands::quick_project,
            commands::project_commands::set_comment,
            commands::project_commands::remove_project,
            commands::timesheet_commands::generate_timesheet,
            commands::timesheet_commands::preview_timesheet,
            commands::timesheet_commands::get_timesheet_preview_bootstrap,
            commands::timesheet_commands::open_timesheet_preview_window,
            commands::timesheet_commands::hide_timesheet_preview_window,
            commands::timesheet_commands::generate_timesheet_export,
            commands::timesheet_commands::reset_timesheet,
            commands::project_commands::reset_projects,
            commands::shell_commands::open_log_file,
            commands::shell_commands::open_diagnostic_log,
            commands::shell_commands::open_feedback,
            commands::shell_commands::open_github_issues,
            commands::shell_commands::open_portfolio,
            commands::shell_commands::open_project_homepage,
            commands::shell_commands::open_release_notes,
            commands::shell_commands::set_update_available,
            commands::settings_commands::save_ui_settings,
            commands::settings_commands::set_timesheet_rounding_enabled,
            commands::settings_commands::save_quickpanel_bounds,
            commands::shell_commands::log_from_frontend
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                } else if window.label() == "timesheet-preview" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error building application")
        .run(|app, event| match &event {
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
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timesheet::{TimesheetFormat, TimesheetOptions, TimesheetRange};
    use std::collections::HashMap;

    #[test]
    fn clean_input_collapses_whitespace_and_newlines() {
        let actual = controllers::project_controller::add_project_value;
        let cleaned = "  Alpha\tBeta\r\nGamma  "
            .replace(['\t', '\r', '\n'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let _ = actual;
        assert_eq!(cleaned, "Alpha Beta Gamma");
    }

    #[test]
    fn clean_input_returns_empty_for_only_whitespace() {
        let actual = " \t\r\n  "
            .replace(['\t', '\r', '\n'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert_eq!(actual, "");
    }

    #[test]
    fn parse_timesheet_options_accepts_full_all() {
        let actual = controllers::timesheet_controller::parse_timesheet_options("all", "full").unwrap();

        assert_eq!(
            actual,
            TimesheetOptions {
                range: TimesheetRange::All,
                format: TimesheetFormat::Full,
            }
        );
    }

    #[test]
    fn parse_timesheet_options_accepts_recent_today() {
        let actual = controllers::timesheet_controller::parse_timesheet_options("today", "recent").unwrap();

        assert_eq!(
            actual,
            TimesheetOptions {
                range: TimesheetRange::Today,
                format: TimesheetFormat::Recent,
            }
        );
    }

    #[test]
    fn parse_timesheet_options_rejects_unknown_range() {
        let err = controllers::timesheet_controller::parse_timesheet_options("month", "full").unwrap_err();

        assert_eq!(err, "Unknown timesheet range.");
    }

    #[test]
    fn parse_timesheet_options_rejects_unknown_format() {
        let err = controllers::timesheet_controller::parse_timesheet_options("all", "compact").unwrap_err();

        assert_eq!(err, "Unknown timesheet format.");
    }

    #[test]
    fn parse_timesheet_options_rejects_recent_for_non_today_ranges() {
        let err = controllers::timesheet_controller::parse_timesheet_options("week", "recent").unwrap_err();

        assert_eq!(
            err,
            "Yesterday + today export only supports the two-day overview."
        );
    }

    #[test]
    fn next_recent_usage_timestamp_is_strictly_greater_than_existing_max() {
        let mut settings = UiSettings::default();
        settings.project_recent_usage = HashMap::from([
            ("Alpha".to_string(), 100),
            ("Beta".to_string(), 250),
        ]);

        let actual = controllers::project_controller::next_recent_usage_timestamp(&settings);

        assert!(actual > 250);
    }

    #[test]
    fn next_recent_usage_timestamp_advances_when_clock_value_is_not_newer() {
        let mut settings = UiSettings::default();
        settings.project_recent_usage = HashMap::from([("Alpha".to_string(), u64::MAX - 1)]);

        let actual = controllers::project_controller::next_recent_usage_timestamp(&settings);

        assert_eq!(actual, u64::MAX);
    }
}
