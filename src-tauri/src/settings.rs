use std::path::Path;

pub use crate::models::domain::settings::UiSettings;
use crate::models::repository_traits::settings_repository::SettingsRepository;
use crate::repositories::file_settings_repository::FileSettingsRepository;

pub fn load(data_dir: &Path) -> UiSettings {
    FileSettingsRepository::new(data_dir).load()
}

pub fn save(data_dir: &Path, settings: &UiSettings) {
    FileSettingsRepository::new(data_dir).save(settings);
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

    #[test]
    fn load_clamps_invalid_opacity_and_normalizes_invalid_modes() {
        let dir = temp_dir();
        fs::write(
            dir.join("settings.json"),
            r#"{
  "always_on_top": false,
  "open_on_start": true,
  "quickpanel_opacity": 0.1,
  "project_sort_mode": "sideways",
  "quickpanel_mode": "floating",
  "timesheet_rounding_enabled": true
}
"#,
        )
        .unwrap();

        let settings = load(&dir);
        assert_eq!(settings.quickpanel_opacity, 0.35);
        assert_eq!(settings.project_sort_mode, "manual");
        assert_eq!(settings.quickpanel_mode, "normal");
        assert!(settings.timesheet_rounding_enabled);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn save_persists_normalized_values() {
        let dir = temp_dir();
        let settings = UiSettings {
            quickpanel_opacity: 5.0,
            project_sort_mode: "zigzag".to_string(),
            quickpanel_mode: "mini".to_string(),
            ..UiSettings::default()
        };

        save(&dir, &settings);
        let actual = load(&dir);

        assert_eq!(actual.quickpanel_opacity, 1.0);
        assert_eq!(actual.project_sort_mode, "manual");
        assert_eq!(actual.quickpanel_mode, "normal");
        let _ = fs::remove_dir_all(dir);
    }
}
