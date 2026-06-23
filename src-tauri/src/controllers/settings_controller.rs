use std::collections::HashMap;

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::{emit_state_changed, settings, tray, AppState};

pub fn save_ui_settings(
    always_on_top: bool,
    open_on_start: bool,
    quickpanel_opacity: f64,
    project_sort_mode: String,
    quickpanel_mode: String,
    project_manual_order: Vec<String>,
    project_recent_usage: HashMap<String, u64>,
    timesheet_rounding_enabled: bool,
    state: &AppState,
    app: &AppHandle,
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

    crate::log!(
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
        crate::log_debug!("save_ui_settings skipped unchanged settings");
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
        tray::rebuild_menu(app);
    }
}

pub fn set_timesheet_rounding_enabled(enabled: bool, state: &AppState, app: &AppHandle) {
    let mut settings = state.settings.lock().unwrap();
    if settings.timesheet_rounding_enabled == enabled {
        return;
    }

    settings.timesheet_rounding_enabled = enabled;
    settings::save(&state.data_dir, &settings);
    drop(settings);
    emit_state_changed(app);
}

pub fn set_ui_font_scale(scale: f64, state: &AppState, app: &AppHandle) {
    let clamped = scale.clamp(0.5, 2.0);
    let mut settings = state.settings.lock().unwrap();
    if (settings.ui_font_scale - clamped).abs() < f64::EPSILON {
        return;
    }
    settings.ui_font_scale = clamped;
    settings::save(&state.data_dir, &settings);
    drop(settings);
    emit_state_changed(app);
}

pub fn save_quickpanel_bounds(x: f64, y: f64, width: f64, height: f64, state: &AppState) {
    let mut settings = state.settings.lock().unwrap();
    settings.quickpanel_x = Some(x);
    settings.quickpanel_y = Some(y);
    settings.quickpanel_width = Some(width);
    settings.quickpanel_height = Some(height);
    settings::save(&state.data_dir, &settings);
}

pub fn set_quickpanel_mode(mode: &str, state: &AppState, app: &AppHandle) {
    if mode != "normal" && mode != "compact" {
        return;
    }

    let mut settings = state.settings.lock().unwrap();
    if settings.quickpanel_mode == mode {
        return;
    }

    settings.quickpanel_mode = mode.to_string();
    settings::save(&state.data_dir, &settings);
    drop(settings);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}
