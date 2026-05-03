use std::path::PathBuf;
use std::sync::atomic::Ordering;

use tauri::{App, Manager};

use crate::{diagnostics, log, log_warn, logger, projects, settings, tray, AppState};

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

fn sync_autostart(app: &mut App, open_on_start: bool) {
    use tauri_plugin_autostart::ManagerExt;

    let autostart_result = if open_on_start {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    };

    if let Err(err) = autostart_result {
        log_warn!(
            "failed to sync autostart open_on_start={}: {}",
            open_on_start,
            err
        );
    } else {
        log!("autostart synced open_on_start={}", open_on_start);
    }
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
    sync_autostart(app, open_on_start);

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

    Ok(())
}
