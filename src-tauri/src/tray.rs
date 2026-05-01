use std::sync::atomic::Ordering;
use image::GenericImageView;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::controllers::{project_controller, timesheet_controller};
use crate::{log, log_debug, log_warn};
use crate::{logger, projects, AppState};

fn sorted_project_lists(
    projects_list: &[String],
    adhoc_list: &[String],
    settings: &crate::settings::UiSettings,
) -> (Vec<String>, Vec<String>) {
    let sort_project_names = |items: &[String]| -> Vec<String> {
        let mut items = items.to_vec();
        match settings.project_sort_mode.as_str() {
            "alphabetical" => items.sort_by_key(|project| project.to_lowercase()),
            "recent" => items.sort_by(|a, b| {
                settings
                    .project_recent_usage
                    .get(b)
                    .copied()
                    .unwrap_or(0)
                    .cmp(&settings.project_recent_usage.get(a).copied().unwrap_or(0))
                    .then_with(|| a.cmp(b))
            }),
            _ => {
                let mut ordered = Vec::new();
                for name in &settings.project_manual_order {
                    if items.iter().any(|project| project == name) {
                        ordered.push(name.clone());
                    }
                }
                for item in items {
                    if !ordered.contains(&item) {
                        ordered.push(item);
                    }
                }
                return ordered;
            }
        }
        items
    };

    (
        sort_project_names(projects_list),
        sort_project_names(adhoc_list),
    )
}

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    log!("setup tray");
    let menu = build_menu(app.handle())?;

    let img = image::load_from_memory(include_bytes!("../icons/icon.png"))
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
    let update_available = state.update_available.load(Ordering::Relaxed);
    let settings = state.settings.lock().unwrap().clone();
    let (sorted_projects, sorted_adhoc) =
        sorted_project_lists(&projects_list, &adhoc_list, &settings);

    let menu = Menu::new(app)?;

    if update_available {
        menu.append(&MenuItem::with_id(
            app,
            "update_available",
            "Update available...",
            true,
            None::<&str>,
        )?)?;
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    menu.append(&MenuItem::with_id(
        app,
        "open_quickpanel",
        "Open ProjectLog QuickPanel",
        true,
        None::<&str>,
    )?)?;

    let panel_mode_sub = Submenu::with_id(app, "panel_mode", "QuickPanel mode", true)?;
    panel_mode_sub.append(&CheckMenuItem::with_id(
        app,
        "qp_mode::normal",
        "Normal",
        true,
        settings.quickpanel_mode == "normal",
        None::<&str>,
    )?)?;
    panel_mode_sub.append(&CheckMenuItem::with_id(
        app,
        "qp_mode::compact",
        "Compact",
        true,
        settings.quickpanel_mode == "compact",
        None::<&str>,
    )?)?;
    menu.append(&panel_mode_sub)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Project checkboxes (permanent + ad-hoc)
    for (idx, project) in sorted_projects.iter().enumerate() {
        let checked = *active == *project;
        let item = CheckMenuItem::with_id(
            app,
            &format!("select::project::{}", idx),
            project,
            true,
            checked,
            None::<&str>,
        )?;
        menu.append(&item)?;
    }
    for (idx, project) in sorted_adhoc.iter().enumerate() {
        let checked = *active == *project;
        let item = CheckMenuItem::with_id(
            app,
            &format!("select::adhoc::{}", idx),
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
    for (idx, project) in sorted_projects.iter().enumerate() {
        remove_sub.append(&MenuItem::with_id(
            app,
            &format!("remove::{}", idx),
            project,
            true,
            None::<&str>,
        )?)?;
    }
    menu.append(&remove_sub)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Timesheet and log actions
    let timesheet_sub = Submenu::with_id(app, "timesheet_menu", "Generate timesheet", true)?;
    timesheet_sub.append(&MenuItem::with_id(
        app,
        "generate_sheet::all",
        "Full timesheet",
        true,
        None::<&str>,
    )?)?;
    timesheet_sub.append(&MenuItem::with_id(
        app,
        "generate_sheet::recent",
        "Yesterday + today",
        true,
        None::<&str>,
    )?)?;
    menu.append(&timesheet_sub)?;
    menu.append(&MenuItem::with_id(
        app,
        "open_log",
        "Open log file",
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

    menu.append(&MenuItem::with_id(
        app,
        "open_diagnostic_log",
        "Open diagnostic log",
        true,
        None::<&str>,
    )?)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let feedback_sub = Submenu::with_id(app, "feedback_menu", "Feedback", true)?;
    feedback_sub.append(&MenuItem::with_id(
        app,
        "feedback_email",
        "Send feedback...",
        true,
        None::<&str>,
    )?)?;
    feedback_sub.append(&MenuItem::with_id(
        app,
        "feedback_github",
        "GitHub issues...",
        true,
        None::<&str>,
    )?)?;
    menu.append(&feedback_sub)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    menu.append(&MenuItem::with_id(
        app,
        "about",
        "About",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?)?;

    Ok(menu)
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
    if mode != "normal" && mode != "compact" {
        return;
    }

    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().unwrap();
    if settings.quickpanel_mode == mode {
        return;
    }

    settings.quickpanel_mode = mode.to_string();
    crate::settings::save(&state.data_dir, &settings);
    drop(settings);
    rebuild_menu(app);
    emit_state_changed(app);
}

fn handle_select_by_id(app: &AppHandle, id: &str) {
    let Some((kind, idx)) = id.split_once("::") else {
        return;
    };
    let Ok(idx) = idx.parse::<usize>() else {
        return;
    };

    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let project = match kind {
        "project" => {
            let projects = state.projects.lock().unwrap().clone();
            let adhoc = state.adhoc_projects.lock().unwrap().clone();
            let (sorted_projects, _) = sorted_project_lists(&projects, &adhoc, &settings);
            sorted_projects.get(idx).cloned()
        }
        "adhoc" => {
            let projects = state.projects.lock().unwrap().clone();
            let adhoc = state.adhoc_projects.lock().unwrap().clone();
            let (_, sorted_adhoc) = sorted_project_lists(&projects, &adhoc, &settings);
            sorted_adhoc.get(idx).cloned()
        }
        _ => None,
    };

    if let Some(project) = project {
        handle_select(app, &project);
    }
}

fn handle_remove_by_id(app: &AppHandle, id: &str) {
    let Ok(idx) = id.parse::<usize>() else {
        return;
    };
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let projects = state.projects.lock().unwrap().clone();
    let adhoc = state.adhoc_projects.lock().unwrap().clone();
    let (sorted_projects, _) = sorted_project_lists(&projects, &adhoc, &settings);
    let project = sorted_projects.get(idx).cloned();

    if let Some(project) = project {
        handle_remove(app, &project);
    }
}

fn emit_state_changed(app: &AppHandle) {
    let _ = app.emit("state-changed", ());
}

fn show_quickpanel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn remember_project_use(app: &AppHandle, project: &str) {
    if project.is_empty() {
        return;
    }

    let state = app.state::<AppState>();
    let mut settings = state.settings.lock().unwrap();
    let timestamp = project_controller::next_recent_usage_timestamp(&settings);
    settings
        .project_recent_usage
        .insert(project.to_string(), timestamp);
    crate::settings::save(&state.data_dir, &settings);
}

fn handle_update_available(app: &AppHandle) {
    show_quickpanel(app);
    let _ = app.emit("show-update-prompt", ());
}

fn handle_select(app: &AppHandle, name: &str) {
    log!("tray select project={}", name);
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
        remember_project_use(app, name);
    }
    *comment = String::new();

    drop(active);
    drop(comment);
    rebuild_menu(app);
    emit_state_changed(app);
}

fn handle_remove(app: &AppHandle, name: &str) {
    log_warn!("tray remove project={}", name);
    let state = app.state::<AppState>();
    let mut projs = state.projects.lock().unwrap();
    projs.retain(|p| p != name);
    projects::save(&state.data_dir, &projs);
    drop(projs);
    rebuild_menu(app);
    emit_state_changed(app);
}

fn show_input(app: &AppHandle, mode: &str, title: &str) {
    let state = app.state::<AppState>();
    let current_value = match mode {
        "set_comment" => state.active_comment.lock().unwrap().clone(),
        _ => String::new(),
    };

    if let Some(window) = app.get_webview_window("main") {
        let was_visible = window.is_visible().unwrap_or(false);
        let _ = window.emit(
            "show-input",
            serde_json::json!({
                "mode": mode,
                "title": title,
                "value": current_value,
                "closeOnSubmit": !was_visible,
            }),
        );
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn handle_generate(app: &AppHandle, mode: &str) {
    log!("tray open timesheet preview mode={}", mode);
    let (range, format) = match mode {
        "all" => ("all", "full"),
        "recent" => ("today", "recent"),
        _ => return,
    };
    let _ = timesheet_controller::open_timesheet_preview_window(
        range.to_string(),
        format.to_string(),
        app,
    );
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
        logger::reset_log(&state.data_dir);
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
        let mut projs = state.projects.lock().unwrap();
        projs.clear();
        projects::save(&state.data_dir, &projs);
        drop(projs);
        rebuild_menu(app);
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
    let log_path = state.data_dir.join("log.dat");
    if !log_path.exists() {
        let _ = std::fs::File::create(&log_path);
    }
    use tauri_plugin_opener::OpenerExt;
    let _ = app
        .opener()
        .open_path(log_path.to_str().unwrap(), None::<&str>);
}

fn handle_open_diagnostic_log(app: &AppHandle) {
    log!("tray open diagnostic log");
    let state = app.state::<AppState>();
    let log_path = state.data_dir.join("ProjectLog-debug.log");
    if !log_path.exists() {
        let _ = std::fs::File::create(&log_path);
    }
    use tauri_plugin_opener::OpenerExt;
    let _ = app
        .opener()
        .open_path(log_path.to_str().unwrap(), None::<&str>);
}

fn handle_feedback_email(app: &AppHandle) {
    use tauri_plugin_opener::OpenerExt;
    let _ = app.opener().open_url(
        "mailto:karga@karga.dk?subject=ProjectLog%20feedback&body=Hello%2C%0A%0AI%20have%20ProjectLog%20feedback%3A%0A",
        None::<&str>,
    );
}

fn handle_feedback_github(app: &AppHandle) {
    use tauri_plugin_opener::OpenerExt;
    let _ = app
        .opener()
        .open_url("https://github.com/kargaen/ProjectLog/issues", None::<&str>);
}

fn handle_about(app: &AppHandle) {
    show_quickpanel(app);
    let _ = app.emit("show-about", ());
}

fn handle_exit(app: &AppHandle) {
    app.exit(0);
}
