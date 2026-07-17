use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager};

use crate::{emit_state_changed, logger, projects, tray, AppState, ProjectLogState};

fn clean_input(value: &str) -> String {
    value
        .replace(['\t', '\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn next_recent_usage_timestamp(settings: &crate::settings::UiSettings) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
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
    crate::settings::save(&state.data_dir, &settings);
}

pub(crate) fn add_project_value(state: &AppState, value: &str) {
    let value = clean_input(value);
    if value.is_empty() {
        return;
    }

    let mut projs = state.projects.lock().unwrap();
    if !projs.contains(&value) {
        projs.push(value.clone());
        projects::save(&state.data_dir, &projs);
    }
    drop(projs);

    let mut settings = state.settings.lock().unwrap();
    if !settings.project_recent_usage.contains_key(&value) {
        let timestamp = next_recent_usage_timestamp(&settings);
        settings.project_recent_usage.insert(value, timestamp);
        crate::settings::save(&state.data_dir, &settings);
    }
}

pub(crate) fn quick_project_value(state: &AppState, value: &str) {
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
            adhoc.push(value.clone());
        }
    }
}

pub(crate) fn set_comment_value(state: &AppState, value: &str) {
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

pub fn get_state(state: &AppState) -> ProjectLogState {
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

pub fn submit_input(mode: String, value: String, state: &AppState, app: &AppHandle) {
    match mode.as_str() {
        "add_project" => add_project_value(state, &value),
        "quick_project" => quick_project_value(state, &value),
        "set_comment" => set_comment_value(state, &value),
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
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn select_project(project: String, state: &AppState, app: &AppHandle) {
    let project = clean_input(&project);
    if project.is_empty() {
        crate::log_warn!("select_project ignored empty project");
        return;
    }

    crate::log!("select_project project={}", project);
    state.reminder_active.store(false, Ordering::Relaxed);
    let mut active = state.active_project.lock().unwrap();
    let mut comment = state.active_comment.lock().unwrap();

    if *active == project {
        logger::log_new_entry(&state.data_dir, "", "");
        *active = String::new();
    } else {
        logger::log_new_entry(&state.data_dir, &project, "");
        *active = project;
        remember_project_use(state, &*active);
    }
    *comment = String::new();

    drop(active);
    drop(comment);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn add_project(value: String, state: &AppState, app: &AppHandle) {
    crate::log!("add_project");
    add_project_value(state, &value);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn quick_project(value: String, state: &AppState, app: &AppHandle) {
    crate::log!("quick_project");
    quick_project_value(state, &value);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn set_comment(value: String, state: &AppState, app: &AppHandle) {
    crate::log!("set_comment len={}", value.len());
    set_comment_value(state, &value);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn remove_project(project: String, state: &AppState, app: &AppHandle) {
    let project = clean_input(&project);
    crate::log!("remove_project project={}", project);
    let mut projs = state.projects.lock().unwrap();
    projs.retain(|p| p != &project);
    projects::save(&state.data_dir, &projs);
    drop(projs);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}

pub fn reset_projects(state: &AppState, app: &AppHandle) {
    crate::log_warn!("reset_projects");
    let mut projs = state.projects.lock().unwrap();
    projs.clear();
    projects::save(&state.data_dir, &projs);
    drop(projs);
    tray::rebuild_menu(app);
    emit_state_changed(app);
}
