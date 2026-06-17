use tauri::App;

use crate::{log, log_warn};

pub fn sync_autostart(app: &mut App, open_on_start: bool) {
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
