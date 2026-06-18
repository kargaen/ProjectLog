use tauri::{AppHandle, Manager};

use crate::{
    controllers::shell_controller, logger, timesheet, AppState, TimesheetPreviewBootstrap,
    TimesheetPreviewRequest,
};
use timesheet::{TimesheetFormat, TimesheetOptions, TimesheetPreview, TimesheetRange};

pub fn parse_timesheet_options(range: &str, format: &str) -> Result<TimesheetOptions, String> {
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

    Ok(TimesheetOptions { range, format, rounding_enabled: false })
}

pub fn generate_timesheet(state: &AppState, app: &AppHandle) -> Result<(), String> {
    generate_timesheet_export("all".to_string(), "full".to_string(), state, app)
}

pub fn preview_timesheet(
    range: String,
    format: String,
    state: &AppState,
) -> Result<TimesheetPreview, String> {
    crate::log!("preview_timesheet range={} format={}", range, format);
    let active = state.active_project.lock().unwrap().clone();
    let comment = state.active_comment.lock().unwrap().clone();
    logger::log_new_entry(&state.data_dir, &active, &comment);
    let options = parse_timesheet_options(&range, &format)?;
    timesheet::preview(&state.data_dir, options)
}

pub fn get_timesheet_preview_bootstrap(state: &AppState) -> TimesheetPreviewBootstrap {
    TimesheetPreviewBootstrap {
        request: state.timesheet_preview_request.lock().unwrap().clone(),
        rounding_enabled: state.settings.lock().unwrap().timesheet_rounding_enabled,
    }
}

pub async fn open_timesheet_preview_window(
    range: String,
    format: String,
    app: &AppHandle,
) -> Result<(), String> {
    crate::log!("open_timesheet_preview_window range={} format={}", range, format);
    let options = parse_timesheet_options(&range, &format)?;
    let title = match options.format {
        TimesheetFormat::Recent => "ProjectLog Timesheet: Yesterday + Today",
        TimesheetFormat::Full => "ProjectLog Timesheet: Full",
    };
    let (width, height) = match options.format {
        TimesheetFormat::Recent => (860.0, 600.0),
        TimesheetFormat::Full => (1120.0, 760.0),
    };

    let request = TimesheetPreviewRequest {
        range: range.clone(),
        format: format.clone(),
    };

    *app.state::<AppState>().timesheet_preview_request.lock().unwrap() = Some(request.clone());

    shell_controller::show_timesheet_preview_window(app, request, title, width, height)
}

pub fn hide_timesheet_preview_window(app: &AppHandle) -> Result<(), String> {
    shell_controller::hide_timesheet_preview_window(app)
}

pub fn generate_timesheet_export(
    range: String,
    format: String,
    state: &AppState,
    app: &AppHandle,
) -> Result<(), String> {
    crate::log!(
        "generate_timesheet_export range={} format={}",
        range,
        format
    );
    let active = state.active_project.lock().unwrap().clone();
    let comment = state.active_comment.lock().unwrap().clone();
    logger::log_new_entry(&state.data_dir, &active, &comment);
    let mut options = parse_timesheet_options(&range, &format)?;
    options.rounding_enabled = state.settings.lock().unwrap().timesheet_rounding_enabled;

    match timesheet::generate(&state.data_dir, options) {
        Ok(path) => {
            use tauri_plugin_opener::OpenerExt;
            app.opener()
                .open_path(path.to_string_lossy().as_ref(), None::<&str>)
                .map_err(|e| e.to_string())
        }
        Err(msg) => {
            crate::log_error!("generate_timesheet failed: {}", msg);
            Err(msg)
        }
    }
}

pub fn reset_timesheet(state: &AppState, app: &AppHandle) {
    crate::log_warn!("reset_timesheet");
    logger::reset_log(&state.data_dir);
    crate::emit_state_changed(app);
}
