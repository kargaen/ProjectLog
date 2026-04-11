use std::sync::atomic::Ordering;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::{logger, projects, timesheet, AppState};

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app.handle())?;

    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.ico"))
        .expect("failed to load tray icon");

    let _tray = TrayIconBuilder::new("main")
        .icon(icon)
        .menu(&menu)
        .menu_on_left_click(false)
        .tooltip("ProjectLog")
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
}

pub fn rebuild_menu(app: &AppHandle) {
    if let Ok(menu) = build_menu(app) {
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn build_menu(app: &AppHandle) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let projects_list = state.projects.lock().unwrap();
    let adhoc_list = state.adhoc_projects.lock().unwrap();
    let active = state.active_project.lock().unwrap();
    let comment = state.active_comment.lock().unwrap();

    let menu = Menu::new(app)?;

    // Project checkboxes (permanent + ad-hoc)
    for project in projects_list.iter().chain(adhoc_list.iter()) {
        let checked = *active == *project;
        let item = CheckMenuItem::with_id(
            app,
            &format!("select::{}", project),
            project,
            true,
            checked,
            None::<&str>,
        )?;
        menu.append(&item)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Comment item (disabled if no active project)
    let has_project = !active.is_empty();
    let comment_label = if comment.is_empty() {
        "Set comment...".to_string()
    } else {
        format!("Comment: {}", *comment)
    };
    menu.append(&MenuItem::with_id(
        app,
        "set_comment",
        &comment_label,
        has_project,
        None::<&str>,
    )?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Project management
    menu.append(&MenuItem::with_id(
        app,
        "add_project",
        "Add project...",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "quick_project",
        "Quick project...",
        true,
        None::<&str>,
    )?)?;

    let remove_sub = Submenu::with_id(app, "remove_menu", "Remove project", true)?;
    for project in projects_list.iter() {
        remove_sub.append(&MenuItem::with_id(
            app,
            &format!("remove::{}", project),
            project,
            true,
            None::<&str>,
        )?)?;
    }
    menu.append(&remove_sub)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Timesheet actions
    menu.append(&MenuItem::with_id(
        app,
        "generate_sheet",
        "Generate timesheet",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "reset_sheet",
        "Reset timesheet",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "reset_projects",
        "Reset projects",
        true,
        None::<&str>,
    )?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    menu.append(&MenuItem::with_id(
        app,
        "open_log",
        "Open log file",
        true,
        None::<&str>,
    )?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    menu.append(&MenuItem::with_id(
        app,
        "about",
        "About",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "exit",
        "Exit",
        true,
        None::<&str>,
    )?)?;

    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();

    if let Some(name) = id.strip_prefix("select::") {
        handle_select(app, name);
    } else if let Some(name) = id.strip_prefix("remove::") {
        handle_remove(app, name);
    } else {
        match id {
            "set_comment" => show_input(app, "set_comment", "Set comment:"),
            "add_project" => show_input(app, "add_project", "Add project:"),
            "quick_project" => show_input(app, "quick_project", "Quick project:"),
            "generate_sheet" => handle_generate(app),
            "reset_sheet" => handle_reset_sheet(app),
            "reset_projects" => handle_reset_projects(app),
            "open_log" => handle_open_log(app),
            "about" => handle_about(app),
            "exit" => handle_exit(app),
            _ => {}
        }
    }
}

fn handle_select(app: &AppHandle, name: &str) {
    let state = app.state::<AppState>();
    state.reminder_active.store(false, Ordering::Relaxed);

    let mut active = state.active_project.lock().unwrap();
    let mut comment = state.active_comment.lock().unwrap();

    if *active == name {
        // Deselect: log empty project (clock out)
        logger::log_new_entry(&state.data_dir, "", "");
        *active = String::new();
    } else {
        // Select new project
        logger::log_new_entry(&state.data_dir, name, "");
        *active = name.to_string();
    }
    *comment = String::new();

    drop(active);
    drop(comment);
    rebuild_menu(app);
}

fn handle_remove(app: &AppHandle, name: &str) {
    let state = app.state::<AppState>();
    let mut projs = state.projects.lock().unwrap();
    projs.retain(|p| p != name);
    projects::save(&state.data_dir, &projs);
    drop(projs);
    rebuild_menu(app);
}

fn show_input(app: &AppHandle, mode: &str, title: &str) {
    let state = app.state::<AppState>();
    let current_value = match mode {
        "set_comment" => state.active_comment.lock().unwrap().clone(),
        _ => String::new(),
    };

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit(
            "show-input",
            serde_json::json!({
                "mode": mode,
                "title": title,
                "value": current_value,
            }),
        );
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn handle_generate(app: &AppHandle) {
    let state = app.state::<AppState>();

    // Log current state for an accurate timesheet
    let active = state.active_project.lock().unwrap().clone();
    let comment = state.active_comment.lock().unwrap().clone();
    logger::log_new_entry(&state.data_dir, &active, &comment);

    match timesheet::generate(&state.data_dir) {
        Ok(path) => {
            use tauri_plugin_opener::OpenerExt;
            let _ = app.opener().open_path(path.to_str().unwrap(), None::<&str>);
        }
        Err(msg) => {
            use tauri_plugin_dialog::DialogExt;
            app.dialog()
                .message(&msg)
                .title("Timesheet")
                .blocking_show();
        }
    }
}

fn handle_reset_sheet(app: &AppHandle) {
    use tauri_plugin_dialog::DialogExt;
    let confirmed = app
        .dialog()
        .message("Are you sure you want to reset the timesheet?")
        .title("Reset timesheet")
        .ok_button_label("Yes")
        .cancel_button_label("No")
        .blocking_show();
    if confirmed {
        let state = app.state::<AppState>();
        logger::reset_log(&state.data_dir);
    }
}

fn handle_reset_projects(app: &AppHandle) {
    use tauri_plugin_dialog::DialogExt;
    let confirmed = app
        .dialog()
        .message("Are you sure you want to reset all projects?")
        .title("Reset projects")
        .ok_button_label("Yes")
        .cancel_button_label("No")
        .blocking_show();
    if confirmed {
        let state = app.state::<AppState>();
        let mut projs = state.projects.lock().unwrap();
        projs.clear();
        projects::save(&state.data_dir, &projs);
        drop(projs);
        rebuild_menu(app);
    }
}

fn handle_open_log(app: &AppHandle) {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .message("You may modify the log file if you made a mistake, but be cautious.")
        .title("Open log file")
        .blocking_show();

    let state = app.state::<AppState>();
    let log_path = state.data_dir.join("log.dat");
    if !log_path.exists() {
        let _ = std::fs::File::create(&log_path);
    }
    use tauri_plugin_opener::OpenerExt;
    let _ = app.opener().open_path(log_path.to_str().unwrap(), None::<&str>);
}

fn handle_about(app: &AppHandle) {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .message("ProjectLog v2.0\nDeveloped by Karsten Garborg.\nBuilt with Tauri + Svelte.")
        .title("About ProjectLog")
        .blocking_show();
}

fn handle_exit(app: &AppHandle) {
    app.exit(0);
}
