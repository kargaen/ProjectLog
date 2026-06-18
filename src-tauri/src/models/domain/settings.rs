use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn default_quickpanel_opacity() -> f64 {
    1.0
}

fn default_project_sort_mode() -> String {
    "manual".to_string()
}

fn default_quickpanel_mode() -> String {
    "normal".to_string()
}

fn default_timesheet_rounding_enabled() -> bool {
    false
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiSettings {
    pub always_on_top: bool,
    pub open_on_start: bool,
    pub quickpanel_x: Option<f64>,
    pub quickpanel_y: Option<f64>,
    pub quickpanel_width: Option<f64>,
    pub quickpanel_height: Option<f64>,
    #[serde(default = "default_quickpanel_opacity")]
    pub quickpanel_opacity: f64,
    #[serde(default = "default_project_sort_mode")]
    pub project_sort_mode: String,
    #[serde(default = "default_quickpanel_mode")]
    pub quickpanel_mode: String,
    #[serde(default)]
    pub project_manual_order: Vec<String>,
    #[serde(default)]
    pub project_recent_usage: HashMap<String, u64>,
    #[serde(default = "default_timesheet_rounding_enabled")]
    pub timesheet_rounding_enabled: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            always_on_top: false,
            open_on_start: false,
            quickpanel_x: None,
            quickpanel_y: None,
            quickpanel_width: None,
            quickpanel_height: None,
            quickpanel_opacity: default_quickpanel_opacity(),
            project_sort_mode: "manual".to_string(),
            quickpanel_mode: default_quickpanel_mode(),
            project_manual_order: Vec::new(),
            project_recent_usage: HashMap::new(),
            timesheet_rounding_enabled: default_timesheet_rounding_enabled(),
        }
    }
}
