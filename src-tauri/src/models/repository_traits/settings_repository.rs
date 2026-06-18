use crate::models::domain::settings::UiSettings;

pub trait SettingsRepository {
    /// Load settings from disk, returning defaults on any read or parse failure.
    fn load(&self) -> UiSettings;

    /// Persist settings to disk, normalizing values before writing.
    fn save(&self, settings: &UiSettings);
}
