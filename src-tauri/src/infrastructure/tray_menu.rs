use std::sync::atomic::Ordering;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Wry};

use crate::AppState;

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

pub fn build_menu(app: &AppHandle) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
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

pub fn project_from_select_id(app: &AppHandle, id: &str) -> Option<String> {
    let (kind, idx) = id.split_once("::")?;
    let idx = idx.parse::<usize>().ok()?;

    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let projects = state.projects.lock().unwrap().clone();
    let adhoc = state.adhoc_projects.lock().unwrap().clone();
    let (sorted_projects, sorted_adhoc) = sorted_project_lists(&projects, &adhoc, &settings);

    match kind {
        "project" => sorted_projects.get(idx).cloned(),
        "adhoc" => sorted_adhoc.get(idx).cloned(),
        _ => None,
    }
}

pub fn project_from_remove_id(app: &AppHandle, id: &str) -> Option<String> {
    let idx = id.parse::<usize>().ok()?;
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let projects = state.projects.lock().unwrap().clone();
    let adhoc = state.adhoc_projects.lock().unwrap().clone();
    let (sorted_projects, _) = sorted_project_lists(&projects, &adhoc, &settings);
    sorted_projects.get(idx).cloned()
}
