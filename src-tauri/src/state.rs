use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::Emitter;

use crate::settings::UiSettings;

pub struct AppState {
    pub active_project: Mutex<String>,
    pub active_comment: Mutex<String>,
    pub projects: Mutex<Vec<String>>,
    pub adhoc_projects: Mutex<Vec<String>>,
    pub data_dir: PathBuf,
    pub reminder_active: Arc<AtomicBool>,
    pub update_available: AtomicBool,
    pub settings: Mutex<UiSettings>,
    pub timesheet_preview_request: Mutex<Option<TimesheetPreviewRequest>>,
}

impl AppState {
    pub fn new(data_dir: PathBuf, projects: Vec<String>, settings: UiSettings) -> Self {
        Self {
            active_project: Mutex::new(String::new()),
            active_comment: Mutex::new(String::new()),
            projects: Mutex::new(projects),
            adhoc_projects: Mutex::new(Vec::new()),
            data_dir,
            reminder_active: Arc::new(AtomicBool::new(true)),
            update_available: AtomicBool::new(false),
            settings: Mutex::new(settings),
            timesheet_preview_request: Mutex::new(None),
        }
    }
}

#[derive(Serialize)]
pub struct ProjectLogState {
    pub app_version: String,
    pub active_project: String,
    pub active_comment: String,
    pub projects: Vec<String>,
    pub adhoc_projects: Vec<String>,
    pub update_available: bool,
    pub settings: UiSettings,
}

#[derive(Clone, Serialize)]
pub struct TimesheetPreviewRequest {
    pub range: String,
    pub format: String,
}

#[derive(Serialize)]
pub struct TimesheetPreviewBootstrap {
    pub request: Option<TimesheetPreviewRequest>,
    pub rounding_enabled: bool,
}

pub(crate) fn emit_state_changed(app: &tauri::AppHandle) {
    let _ = app.emit("state-changed", ());
}
