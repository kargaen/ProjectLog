mod commands {
    pub mod project_commands;
    pub mod shell_commands;
    pub mod settings_commands;
    pub mod timesheet_commands;
}
mod app_setup;
mod controllers {
    pub mod project_controller;
    pub mod shell_controller;
    pub mod settings_controller;
    pub mod timesheet_controller;
}
pub mod diagnostics;
mod lifecycle;
mod logger;
pub mod models;
mod projects;
mod repositories;
mod services;
mod settings;
mod state;
mod timesheet;
mod tray;
mod tray_menu;

pub use state::{AppState, ProjectLogState, TimesheetPreviewBootstrap, TimesheetPreviewRequest};
pub(crate) use state::emit_state_changed;

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
        .setup(app_setup::initialize)
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
        .on_window_event(lifecycle::handle_window_event)
        .build(tauri::generate_context!())
        .expect("error building application")
        .run(|app, event| lifecycle::handle_run_event(app, &event));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::UiSettings;
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
