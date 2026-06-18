use std::fs;
use std::path::{Path, PathBuf};

use crate::models::domain::settings::UiSettings;
use crate::models::repository_traits::settings_repository::SettingsRepository;
use crate::{log, log_debug, log_warn};

pub struct FileSettingsRepository {
    data_dir: PathBuf,
}

impl FileSettingsRepository {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }
}

impl SettingsRepository for FileSettingsRepository {
    fn load(&self) -> UiSettings {
        let path = self.data_dir.join("settings.json");
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

    fn save(&self, settings: &UiSettings) {
        let path = self.data_dir.join("settings.json");
        let temp_path = self.data_dir.join("settings.json.tmp");
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
}

fn normalize_opacity(value: f64) -> f64 {
    value.clamp(0.35, 1.0)
}

fn normalize_sort_mode(value: &str) -> String {
    match value {
        "alphabetical" | "recent" | "manual" => value.to_string(),
        _ => "manual".to_string(),
    }
}

fn normalize_quickpanel_mode(value: &str) -> String {
    match value {
        "normal" | "compact" => value.to_string(),
        _ => "normal".to_string(),
    }
}
