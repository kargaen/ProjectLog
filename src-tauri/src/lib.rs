pub mod diagnostics;
mod logger;
mod projects;
mod settings;
mod timesheet;
mod tray;
use tauri_plugin_autostart::ManagerExt;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use settings::UiSettings;
use tauri::{Emitter, LogicalSize, Manager, State, WebviewWindowBuilder};
use timesheet::{TimesheetFormat, TimesheetOptions, TimesheetPreview, TimesheetRange};

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

fn clean_input(value: &str) -> String {
    value
        .replace(['\t', '\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn emit_state_changed(app: &tauri::AppHandle) {
    let _ = app.emit("state-changed", ());
}

fn parse_timesheet_options(range: &str, format: &str) -> Result<TimesheetOptions, String> {
    let range = match range {
        "today" => TimesheetRange::Today,
        "week" => TimesheetRange::Week,
        "all" => TimesheetRange::All,
        _ => return Err("Unknown timesheet range.".to_string()),
    };
    let format = match format {
        "full" => TimesheetFormat::Full,
        "recent" => TimesheetFormat::Recent,
        _ => return Err("Unknown timesheet format.".to_string()),
    };

    if format == TimesheetFormat::Recent && range != TimesheetRange::Today {
        return Err("Yesterday + today export only supports the two-day overview.".to_string());
    }

    Ok(TimesheetOptions { range, format })
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

pub(crate) fn next_recent_usage_timestamp(settings: &UiSettings) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros() as u64)
        .unwrap_or(0);

    let highest_seen = settings
        .project_recent_usage
        .values()
        .copied()
        .max()
        .unwrap_or(0);

    now.max(highest_seen.saturating_add(1))
}

fn remember_project_use(state: &AppState, project: &str) {
    if project.is_empty() {
        return;
    }

    let mut settings = state.settings.lock().unwrap();
    let timestamp = next_recent_usage_timestamp(&settings);
    settings
        .project_recent_usage
        .insert(project.to_string(), timestamp);
    settings::save(&state.data_dir, &settings);
}

fn add_project_value(state: &AppState, value: &str) {
    let value = clean_input(value);
    if value.is_empty() {
        return;
    }

    let mut projs = state.projects.lock().unwrap();
    if !projs.contains(&value) {
        projs.push(value);
        projects::save(&state.data_dir, &projs);
    }
}

fn quick_project_value(state: &AppState, value: &str) {
    let value = clean_input(value);
    if value.is_empty() {
        return;
    }

    state.reminder_active.store(false, Ordering::Relaxed);
    logger::log_new_entry(&state.data_dir, &value, "");
    *state.active_project.lock().unwrap() = value.clone();
    *state.active_comment.lock().unwrap() = String::new();
    remember_project_use(state, &value);

    let in_permanent = state.projects.lock().unwrap().contains(&value);
    if !in_permanent {
        let mut adhoc = state.adhoc_projects.lock().unwrap();
        if !adhoc.contains(&value) {
            adhoc.push(value);
        }
    }
}

fn set_comment_value(state: &AppState, value: &str) {
    let value = clean_input(value);
    let active = state.active_project.lock().unwrap().clone();
    let mut comment = state.active_comment.lock().unwrap();

    if active.is_empty() {
        return;
    }

    if comment.is_empty() && !value.is_empty() {
        logger::append_comment_to_last(&state.data_dir, &value);
    } else if !comment.is_empty() && value != *comment {
        logger::log_new_entry(&state.data_dir, &active, &value);
    }
    *comment = value;
}

#[tauri::command]
fn submit_input(mode: String, value: String, state: State<AppState>, app: tauri::AppHandle) {
    log!("submit_input mode={}", mode);
    match mode.as_str() {
        "add_project" => add_project_value(&state, &value),
        "quick_project" => quick_project_value(&state, &value),
        "set_comment" => set_comment_value(&state, &value),
        _ => {}
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit(
            "input-submitted",
            serde_json::json!({
                "mode": mode,
            }),
        );
    }
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn get_state(state: State<AppState>) -> ProjectLogState {
    log_debug!("get_state");
    ProjectLogState {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        active_project: state.active_project.lock().unwrap().clone(),
        active_comment: state.active_comment.lock().unwrap().clone(),
        projects: state.projects.lock().unwrap().clone(),
        adhoc_projects: state.adhoc_projects.lock().unwrap().clone(),
        update_available: state.update_available.load(Ordering::Relaxed),
        settings: state.settings.lock().unwrap().clone(),
    }
}

#[tauri::command]
fn select_project(project: String, state: State<AppState>, app: tauri::AppHandle) {
    let project = clean_input(&project);
    if project.is_empty() {
        log_warn!("select_project ignored empty project");
        return;
    }

    log!("select_project project={}", project);
    state.reminder_active.store(false, Ordering::Relaxed);
    let mut active = state.active_project.lock().unwrap();
    let mut comment = state.active_comment.lock().unwrap();

    if *active == project {
        logger::log_new_entry(&state.data_dir, "", "");
        *active = String::new();
    } else {
        logger::log_new_entry(&state.data_dir, &project, "");
        *active = project;
        remember_project_use(&state, &*active);
    }
    *comment = String::new();

    drop(active);
    drop(comment);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn add_project(value: String, state: State<AppState>, app: tauri::AppHandle) {
    log!("add_project");
    add_project_value(&state, &value);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn quick_project(value: String, state: State<AppState>, app: tauri::AppHandle) {
    log!("quick_project");
    quick_project_value(&state, &value);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn set_comment(value: String, state: State<AppState>, app: tauri::AppHandle) {
    log!("set_comment len={}", value.len());
    set_comment_value(&state, &value);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn remove_project(project: String, state: State<AppState>, app: tauri::AppHandle) {
    let project = clean_input(&project);
    log!("remove_project project={}", project);
    let mut projs = state.projects.lock().unwrap();
    projs.retain(|p| p != &project);
    projects::save(&state.data_dir, &projs);
    drop(projs);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn generate_timesheet(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    generate_timesheet_export("all".to_string(), "full".to_string(), state, app)
}

#[tauri::command]
fn preview_timesheet(
    range: String,
    format: String,
    state: State<AppState>,
) -> Result<TimesheetPreview, String> {
    log!("preview_timesheet range={} format={}", range, format);
    let active = state.active_project.lock().unwrap().clone();
    let comment = state.active_comment.lock().unwrap().clone();
    logger::log_new_entry(&state.data_dir, &active, &comment);
    let options = parse_timesheet_options(&range, &format)?;
    timesheet::preview(&state.data_dir, options)
}

#[tauri::command]
fn get_timesheet_preview_bootstrap(state: State<AppState>) -> TimesheetPreviewBootstrap {
    TimesheetPreviewBootstrap {
        request: state.timesheet_preview_request.lock().unwrap().clone(),
        rounding_enabled: state.settings.lock().unwrap().timesheet_rounding_enabled,
    }
}

#[tauri::command]
async fn open_timesheet_preview_window(
    range: String,
    format: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    log!("open_timesheet_preview_window range={} format={}", range, format);
    let options = parse_timesheet_options(&range, &format)?;
    let title = match options.format {
        TimesheetFormat::Recent => "ProjectLog Timesheet: Yesterday + Today",
        TimesheetFormat::Full => "ProjectLog Timesheet: Full",
    };
    let (width, height) = match options.format {
        TimesheetFormat::Recent => (860.0, 600.0),
        TimesheetFormat::Full => (1120.0, 760.0),
    };

    *app.state::<AppState>().timesheet_preview_request.lock().unwrap() = Some(TimesheetPreviewRequest {
        range: range.clone(),
        format: format.clone(),
    });

    if let Some(window) = app.get_webview_window("timesheet-preview") {
        let _ = window.set_title(title);
        let _ = window.set_size(LogicalSize::new(width, height));
        let _ = window.emit(
            "show-timesheet-preview",
            serde_json::json!({
                "range": range,
                "format": format,
            }),
        );
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "timesheet-preview")
        .ok_or_else(|| "Missing timesheet-preview window config.".to_string())?;

    let window = WebviewWindowBuilder::from_config(&app, config)
        .map_err(|e| e.to_string())?
        .build()
        .map_err(|e| e.to_string())?;

    let _ = window.set_title(title);
    let _ = window.set_size(LogicalSize::new(width, height));
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit(
        "show-timesheet-preview",
        serde_json::json!({
            "range": range,
            "format": format,
        }),
    );

    Ok(())
}

#[tauri::command]
fn generate_timesheet_export(
    range: String,
    format: String,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    log!(
        "generate_timesheet_export range={} format={}",
        range,
        format
    );
    let active = state.active_project.lock().unwrap().clone();
    let comment = state.active_comment.lock().unwrap().clone();
    logger::log_new_entry(&state.data_dir, &active, &comment);
    let options = parse_timesheet_options(&range, &format)?;

    match timesheet::generate(&state.data_dir, options) {
        Ok(path) => {
            use tauri_plugin_opener::OpenerExt;
            app.opener()
                .open_path(path.to_string_lossy().as_ref(), None::<&str>)
                .map_err(|e| e.to_string())
        }
        Err(msg) => {
            log_error!("generate_timesheet failed: {}", msg);
            Err(msg)
        }
    }
}

#[tauri::command]
fn reset_timesheet(state: State<AppState>, app: tauri::AppHandle) {
    log_warn!("reset_timesheet");
    logger::reset_log(&state.data_dir);
    emit_state_changed(&app);
}

#[tauri::command]
fn reset_projects(state: State<AppState>, app: tauri::AppHandle) {
    log_warn!("reset_projects");
    let mut projs = state.projects.lock().unwrap();
    projs.clear();
    projects::save(&state.data_dir, &projs);
    drop(projs);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn open_log_file(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
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

#[tauri::command]
fn open_diagnostic_log(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
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

#[tauri::command]
fn open_feedback(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_feedback mailto");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(
            "mailto:karga@karga.dk?subject=ProjectLog%20feedback&body=Hello%2C%0A%0AI%20have%20ProjectLog%20feedback%3A%0A",
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_github_issues(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_github_issues");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog/issues", None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_portfolio(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_portfolio");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://kargaen.github.io/", None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_project_homepage(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_project_homepage");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("https://github.com/kargaen/ProjectLog", None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_release_notes(app: tauri::AppHandle) -> Result<(), String> {
    log!("open_release_notes");
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(
            "https://github.com/kargaen/ProjectLog/releases",
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_update_available(available: bool, state: State<AppState>, app: tauri::AppHandle) {
    log!("set_update_available available={}", available);
    state.update_available.store(available, Ordering::Relaxed);
    tray::rebuild_menu(&app);
    emit_state_changed(&app);
}

#[tauri::command]
fn save_ui_settings(
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
    let normalized_opacity = quickpanel_opacity.clamp(0.35, 1.0);
    let normalized_sort_mode = match project_sort_mode.as_str() {
        "alphabetical" | "recent" | "manual" => project_sort_mode,
        _ => "manual".to_string(),
    };
    let normalized_quickpanel_mode = match quickpanel_mode.as_str() {
        "compact" | "normal" => quickpanel_mode,
        _ => "normal".to_string(),
    };
    log!(
        "save_ui_settings always_on_top={} open_on_start={} quickpanel_opacity={:.2} project_sort_mode={} quickpanel_mode={}",
        always_on_top,
        open_on_start,
        normalized_opacity,
        normalized_sort_mode,
        normalized_quickpanel_mode
    );
    let mut settings = state.settings.lock().unwrap();
    let tray_needs_rebuild = settings.project_sort_mode != normalized_sort_mode
        || settings.quickpanel_mode != normalized_quickpanel_mode
        || settings.project_manual_order != project_manual_order
        || settings.project_recent_usage != project_recent_usage;
    let changed = settings.always_on_top != always_on_top
        || settings.open_on_start != open_on_start
        || (settings.quickpanel_opacity - normalized_opacity).abs() > f64::EPSILON
        || settings.timesheet_rounding_enabled != timesheet_rounding_enabled
        || tray_needs_rebuild;

    if !changed {
        log_debug!("save_ui_settings skipped unchanged settings");
        return;
    }

    settings.always_on_top = always_on_top;
    settings.open_on_start = open_on_start;
    settings.quickpanel_opacity = normalized_opacity;
    settings.project_sort_mode = normalized_sort_mode;
    settings.quickpanel_mode = normalized_quickpanel_mode;
    settings.project_manual_order = project_manual_order;
    settings.project_recent_usage = project_recent_usage;
    settings.timesheet_rounding_enabled = timesheet_rounding_enabled;
    settings::save(&state.data_dir, &settings);
    if open_on_start {
        let _ = app.autolaunch().enable();
    } else {
        let _ = app.autolaunch().disable();
    }
    drop(settings);
    if tray_needs_rebuild {
        tray::rebuild_menu(&app);
    }
}

#[tauri::command]
fn set_timesheet_rounding_enabled(
    enabled: bool,
    state: State<AppState>,
    app: tauri::AppHandle,
) {
    let mut settings = state.settings.lock().unwrap();
    if settings.timesheet_rounding_enabled == enabled {
        return;
    }

    settings.timesheet_rounding_enabled = enabled;
    settings::save(&state.data_dir, &settings);
    drop(settings);
    emit_state_changed(&app);
}

#[tauri::command]
fn save_quickpanel_bounds(x: f64, y: f64, width: f64, height: f64, state: State<AppState>) {
    let mut settings = state.settings.lock().unwrap();
    settings.quickpanel_x = Some(x);
    settings.quickpanel_y = Some(y);
    settings.quickpanel_width = Some(width);
    settings.quickpanel_height = Some(height);
    settings::save(&state.data_dir, &settings);
}

#[tauri::command]
fn log_from_frontend(level: String, module: String, message: String, data: Option<String>) {
    diagnostics::frontend(&level, &module, &message, data);
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
            submit_input,
            get_state,
            select_project,
            add_project,
            quick_project,
            set_comment,
            remove_project,
            generate_timesheet,
            preview_timesheet,
            get_timesheet_preview_bootstrap,
            open_timesheet_preview_window,
            generate_timesheet_export,
            reset_timesheet,
            reset_projects,
            open_log_file,
            open_diagnostic_log,
            open_feedback,
            open_github_issues,
            open_portfolio,
            open_project_homepage,
            open_release_notes,
            set_update_available,
            save_ui_settings,
            set_timesheet_rounding_enabled,
            save_quickpanel_bounds,
            log_from_frontend
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
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
    use std::collections::HashMap;

    #[test]
    fn clean_input_collapses_whitespace_and_newlines() {
        let actual = clean_input("  Alpha\tBeta\r\nGamma  ");

        assert_eq!(actual, "Alpha Beta Gamma");
    }

    #[test]
    fn clean_input_returns_empty_for_only_whitespace() {
        let actual = clean_input(" \t\r\n  ");

        assert_eq!(actual, "");
    }

    #[test]
    fn parse_timesheet_options_accepts_full_all() {
        let actual = parse_timesheet_options("all", "full").unwrap();

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
        let actual = parse_timesheet_options("today", "recent").unwrap();

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
        let err = parse_timesheet_options("month", "full").unwrap_err();

        assert_eq!(err, "Unknown timesheet range.");
    }

    #[test]
    fn parse_timesheet_options_rejects_unknown_format() {
        let err = parse_timesheet_options("all", "compact").unwrap_err();

        assert_eq!(err, "Unknown timesheet format.");
    }

    #[test]
    fn parse_timesheet_options_rejects_recent_for_non_today_ranges() {
        let err = parse_timesheet_options("week", "recent").unwrap_err();

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

        let actual = next_recent_usage_timestamp(&settings);

        assert!(actual > 250);
    }

    #[test]
    fn next_recent_usage_timestamp_advances_when_clock_value_is_not_newer() {
        let mut settings = UiSettings::default();
        settings.project_recent_usage = HashMap::from([("Alpha".to_string(), u64::MAX - 1)]);

        let actual = next_recent_usage_timestamp(&settings);

        assert_eq!(actual, u64::MAX);
    }
}
