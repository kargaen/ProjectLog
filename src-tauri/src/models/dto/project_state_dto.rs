use serde::Serialize;

use crate::models::domain::settings::UiSettings;

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
