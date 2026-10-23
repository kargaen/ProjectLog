use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

use tauri::{App, Manager};

use crate::controllers::shell_controller;
use crate::infrastructure::autostart;
use crate::{diagnostics, log, logger, projects, settings, tray, AppState};

fn migrate_legacy_file(data_dir: &PathBuf, filename: &str) {
    let target = data_dir.join(filename);
    if target.exists() {
        return;
    }

    let mut candidates = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join(filename));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join(filename));
        }
    }

    for source in candidates {
        if source.exists() && source != target {
            let _ = std::fs::copy(source, &target);
            break;
        }
    }
}

fn migrate_legacy_files(data_dir: &PathBuf) {
    migrate_legacy_file(data_dir, "projects.dat");
    migrate_legacy_file(data_dir, "log.dat");
}

pub fn initialize(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    diagnostics::init(&data_dir);
    log!("setup data_dir={}", data_dir.display());
    migrate_legacy_files(&data_dir);

    let project_list = projects::load(&data_dir);
    let ui_settings = settings::load(&data_dir);
    let open_on_start = ui_settings.open_on_start;

    let state = AppState::new(data_dir.clone(), project_list, ui_settings);
    let reminder_clone = state.reminder_active.clone();
    app.manage(state);

    logger::log_new_entry(&data_dir, "", "");
    tray::setup(app)?;
    log!("tray initialized");
    autostart::sync_autostart(app, open_on_start);

    let handle = app.handle().clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(300));
        if reminder_clone.load(Ordering::Relaxed) {
            use tauri_plugin_notification::NotificationExt;
            let _ = handle
                .notification()
                .builder()
                .title("Activate project...")
                .body("Remember to activate a project if you are working.")
                .show();
        }
    });

    let handle = app.handle().clone();
    std::thread::spawn(move || {
        let mut interval = 0;
        // Wall clock, so a pop-up is due right after the PC wakes from a long sleep.
        let mut last_popup = SystemTime::now();
        loop {
            std::thread::sleep(Duration::from_secs(60));
            let minutes = handle
                .state::<AppState>()
                .settings
                .lock()
                .unwrap()
                .reminder_interval_minutes;
            if minutes != interval {
                interval = minutes;
                last_popup = SystemTime::now();
            } else if reminder_due(minutes, last_popup.elapsed().unwrap_or_default()) {
                log!("pop-up reminder interval={}m", minutes);
                shell_controller::show_quickpanel(&handle);
                last_popup = SystemTime::now();
            }
        }
    });

    Ok(())
}

fn reminder_due(minutes: u32, elapsed: Duration) -> bool {
    minutes > 0 && elapsed >= Duration::from_secs(u64::from(minutes) * 60)
}

#[cfg(test)]
mod tests {
    use super::reminder_due;
    use std::time::Duration;

    #[test]
    fn reminder_is_never_due_when_off() {
        assert!(!reminder_due(0, Duration::from_secs(24 * 3600)));
    }

    #[test]
    fn reminder_is_due_once_the_interval_has_passed() {
        assert!(!reminder_due(30, Duration::from_secs(30 * 60 - 1)));
        assert!(reminder_due(30, Duration::from_secs(30 * 60)));
        assert!(reminder_due(120, Duration::from_secs(5 * 3600)));
    }
}
