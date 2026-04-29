use std::fs;
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{log, log_debug, log_warn};

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

fn normalize_opacity(value: f64) -> f64 {
    value.clamp(0.35, 1.0)
}

fn normalize_sort_mode(value: &str) -> String {
    match value {
        "alphabetical" | "recent" | "manual" => value.to_string(),
        _ => default_project_sort_mode(),
    }
}

fn normalize_quickpanel_mode(value: &str) -> String {
    match value {
        "normal" | "compact" => value.to_string(),
        _ => default_quickpanel_mode(),
    }
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

pub fn load(data_dir: &Path) -> UiSettings {
    let path = data_dir.join("settings.json");
    if !path.exists() {
        log!("settings.json missing; using defaults");
        return UiSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<UiSettings>(&content) {
            Ok(settings) => {
                log!("loaded settings");
                UiSettings {
                    quickpanel_opacity: normalize_opacity(settings.quickpanel_opacity),
                    project_sort_mode: normalize_sort_mode(&settings.project_sort_mode),
                    quickpanel_mode: normalize_quickpanel_mode(&settings.quickpanel_mode),
                    ..settings
                }
            }
            Err(err) => {
                log_warn!("failed to parse settings.json: {}", err);
                UiSettings::default()
            }
        },
        Err(err) => {
            log_warn!("failed to read settings.json: {}", err);
            UiSettings::default()
        }
    }
}

pub fn save(data_dir: &Path, settings: &UiSettings) {
    let path = data_dir.join("settings.json");
    let temp_path = data_dir.join("settings.json.tmp");
    let normalized = UiSettings {
        quickpanel_opacity: normalize_opacity(settings.quickpanel_opacity),
        project_sort_mode: normalize_sort_mode(&settings.project_sort_mode),
        quickpanel_mode: normalize_quickpanel_mode(&settings.quickpanel_mode),
        ..settings.clone()
    };
    match serde_json::to_string_pretty(&normalized) {
        Ok(content) => {
            if let Err(err) = fs::write(&temp_path, format!("{content}\n")) {
                log_warn!("failed to save settings.json: {}", err);
            } else if let Err(err) = fs::rename(&temp_path, &path) {
                let _ = fs::remove_file(&temp_path);
                log_warn!("failed to replace settings.json: {}", err);
            } else {
                log_debug!("saved settings");
            }
        }
        Err(err) => log_warn!("failed to serialize settings: {}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::{load, save, UiSettings};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("projectlog-settings-test-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn load_returns_defaults_when_missing() {
        let dir = temp_dir();
        let settings = load(&dir);
        assert!(!settings.always_on_top);
        assert!(!settings.open_on_start);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = temp_dir();
        let expected = UiSettings {
            always_on_top: true,
            open_on_start: true,
            quickpanel_x: Some(120.0),
            quickpanel_y: Some(90.0),
            quickpanel_width: Some(480.0),
            quickpanel_height: Some(640.0),
            quickpanel_opacity: 0.5,
            project_sort_mode: "recent".to_string(),
            quickpanel_mode: "compact".to_string(),
            project_manual_order: vec!["Alpha".to_string(), "Beta".to_string()],
            project_recent_usage: HashMap::from([("Alpha".to_string(), 123)]),
            timesheet_rounding_enabled: true,
        };

        save(&dir, &expected);
        let actual = load(&dir);

        assert!(actual.always_on_top);
        assert!(actual.open_on_start);
        assert_eq!(actual.quickpanel_x, expected.quickpanel_x);
        assert_eq!(actual.quickpanel_y, expected.quickpanel_y);
        assert_eq!(actual.quickpanel_width, expected.quickpanel_width);
        assert_eq!(actual.quickpanel_height, expected.quickpanel_height);
        assert_eq!(actual.quickpanel_opacity, expected.quickpanel_opacity);
        assert_eq!(actual.project_sort_mode, expected.project_sort_mode);
        assert_eq!(actual.quickpanel_mode, expected.quickpanel_mode);
        assert_eq!(actual.project_manual_order, expected.project_manual_order);
        assert_eq!(actual.project_recent_usage, expected.project_recent_usage);
        assert_eq!(
            actual.timesheet_rounding_enabled,
            expected.timesheet_rounding_enabled
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_normalizes_empty_sort_mode() {
        let dir = temp_dir();
        fs::write(
            dir.join("settings.json"),
            r#"{
  "always_on_top": false,
  "open_on_start": false,
  "quickpanel_opacity": 1.0,
  "project_sort_mode": "",
  "quickpanel_mode": ""
}
"#,
        )
        .unwrap();

        let settings = load(&dir);
        assert_eq!(settings.project_sort_mode, "manual");
        assert_eq!(settings.quickpanel_mode, "normal");
        assert!(!settings.timesheet_rounding_enabled);
        let _ = fs::remove_dir_all(dir);
    }
}
