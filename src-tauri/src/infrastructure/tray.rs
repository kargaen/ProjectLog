use image::GenericImageView;
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use crate::controllers::{
    project_controller, settings_controller, shell_controller, timesheet_controller,
};
use crate::{log, log_debug, log_warn};
use crate::AppState;
use super::tray_menu;

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    log!("setup tray");
    let menu = tray_menu::build_menu(app.handle())?;

    let img = image::load_from_memory(include_bytes!("../../icons/icon.png"))
        .expect("failed to decode icon");
    let rgba = img.to_rgba8();
    let (w, h) = img.dimensions();
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ProjectLog")
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
}

pub fn rebuild_menu(app: &AppHandle) {
    log_debug!("rebuild tray menu");
    if let Ok(menu) = tray_menu::build_menu(app) {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    log!("tray menu event id={}", id);

    if let Some(id) = id.strip_prefix("select::") {
        handle_select_by_id(app, id);
    } else if let Some(id) = id.strip_prefix("remove::") {
        handle_remove_by_id(app, id);
    } else if let Some(id) = id.strip_prefix("generate_sheet::") {
        handle_generate(app, id);
    } else if let Some(id) = id.strip_prefix("qp_mode::") {
        handle_quickpanel_mode(app, id);
    } else {
        match id {
            "set_comment" => show_input(app, "set_comment", "Set comment:"),
            "add_project" => show_input(app, "add_project", "Add project:"),
            "quick_project" => show_input(app, "quick_project", "Quick project:"),
            "open_quickpanel" => show_quickpanel(app),
            "update_available" => handle_update_available(app),
            "reset_sheet" => handle_reset_sheet(app),
            "reset_projects" => handle_reset_projects(app),
            "open_log" => handle_open_log(app),
            "open_diagnostic_log" => handle_open_diagnostic_log(app),
            "feedback_email" => handle_feedback_email(app),
            "feedback_github" => handle_feedback_github(app),
            "about" => handle_about(app),
            "exit" => handle_exit(app),
            _ => {}
        }
    }
}

fn handle_quickpanel_mode(app: &AppHandle, mode: &str) {
    let state = app.state::<AppState>();
    settings_controller::set_quickpanel_mode(mode, &state, app);
}

fn handle_select_by_id(app: &AppHandle, id: &str) {
    if let Some(project) = tray_menu::project_from_select_id(app, id) {
        handle_select(app, &project);
    }
}

fn handle_remove_by_id(app: &AppHandle, id: &str) {
    if let Some(project) = tray_menu::project_from_remove_id(app, id) {
        handle_remove(app, &project);
    }
}

fn show_quickpanel(app: &AppHandle) {
    shell_controller::show_quickpanel(app);
}

fn handle_update_available(app: &AppHandle) {
    shell_controller::show_update_prompt(app);
}

fn handle_select(app: &AppHandle, name: &str) {
    log!("tray select project={}", name);
    let state = app.state::<AppState>();
    project_controller::select_project(name.to_string(), &state, app);
}

fn handle_remove(app: &AppHandle, name: &str) {
    log_warn!("tray remove project={}", name);
    let state = app.state::<AppState>();
    project_controller::remove_project(name.to_string(), &state, app);
}

fn show_input(app: &AppHandle, mode: &str, title: &str) {
    let state = app.state::<AppState>();
    shell_controller::show_input_prompt(&state, app, mode, title);
}

fn handle_generate(app: &AppHandle, mode: &str) {
    log!("tray open timesheet preview mode={}", mode);
    let (range, format) = match mode {
        "all" => ("all", "full"),
        "recent" => ("today", "recent"),
        _ => return,
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = timesheet_controller::open_timesheet_preview_window(
            range.to_string(),
            format.to_string(),
            &app,
        )
        .await;
    });
}

fn handle_reset_sheet(app: &AppHandle) {
    use tauri_plugin_dialog::DialogExt;
    let confirmed = app
        .dialog()
        .message("Are you sure you want to reset the timesheet?")
        .title("Reset timesheet")
        .blocking_show();
    if confirmed {
        log_warn!("tray reset timesheet confirmed");
        let state = app.state::<AppState>();
        timesheet_controller::reset_timesheet(&state, app);
    }
}

fn handle_reset_projects(app: &AppHandle) {
    use tauri_plugin_dialog::DialogExt;
    let confirmed = app
        .dialog()
        .message("Are you sure you want to reset all projects?")
        .title("Reset projects")
        .blocking_show();
    if confirmed {
        log_warn!("tray reset projects confirmed");
        let state = app.state::<AppState>();
        project_controller::reset_projects(&state, app);
    }
}

fn handle_open_log(app: &AppHandle) {
    log!("tray open log");
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .message("You may modify the log file if you made a mistake, but be cautious.")
        .title("Open log file")
        .blocking_show();

    let state = app.state::<AppState>();
    let _ = shell_controller::open_log_file(&state, app);
}

fn handle_open_diagnostic_log(app: &AppHandle) {
    log!("tray open diagnostic log");
    let state = app.state::<AppState>();
    let _ = shell_controller::open_diagnostic_log(&state, app);
}

fn handle_feedback_email(app: &AppHandle) {
    let _ = shell_controller::open_feedback(app);
}

fn handle_feedback_github(app: &AppHandle) {
    let _ = shell_controller::open_github_issues(app);
}

fn handle_about(app: &AppHandle) {
    shell_controller::show_about(app);
}

fn handle_exit(app: &AppHandle) {
    app.exit(0);
}
