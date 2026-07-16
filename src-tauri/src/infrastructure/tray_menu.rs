use std::sync::atomic::Ordering;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Wry};

use crate::AppState;

fn project_id(project: &str) -> String {
    let mut encoded = String::new();
    for byte in project.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn project_from_id(id: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let raw = id.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' {
            let hex = std::str::from_utf8(raw.get(i + 1..i + 3)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            bytes.push(raw[i]);
            i += 1;
        }
    }
    String::from_utf8(bytes).ok()
}

fn grouped_level_one(
    projects: Vec<String>,
    settings: &crate::settings::UiSettings,
) -> Vec<(Option<String>, Vec<String>)> {
    if !settings.group_projects_enabled {
        return projects
            .into_iter()
            .map(|project| (None, vec![project]))
            .collect();
    }

    let mut entries: Vec<(Option<String>, Vec<String>)> = Vec::new();
    for project in projects {
        if let Some(group) = settings.project_groups.get(&project) {
            if let Some((_, members)) = entries
                .iter_mut()
                .find(|(name, _)| name.as_deref() == Some(group.as_str()))
            {
                members.push(project);
            } else {
                entries.push((Some(group.clone()), vec![project]));
            }
        } else {
            entries.push((None, vec![project]));
        }
    }

    if settings.project_sort_mode == "alphabetical" {
        entries.sort_by(|(a, ap), (b, bp)| {
            let an = a.as_ref().unwrap_or(&ap[0]);
            let bn = b.as_ref().unwrap_or(&bp[0]);
            an.to_lowercase().cmp(&bn.to_lowercase())
        });
        for (_, members) in &mut entries {
            members.sort_by_key(|name| name.to_lowercase());
        }
    } else if settings.project_sort_mode == "recent" {
        let recency = |members: &Vec<String>| -> u64 {
            members
                .iter()
                .filter_map(|project| settings.project_recent_usage.get(project).copied())
                .max()
                .unwrap_or(0)
        };
        entries.sort_by(|(a, ap), (b, bp)| {
            recency(bp).cmp(&recency(ap)).then_with(|| {
                let an = a.as_ref().unwrap_or(&ap[0]);
                let bn = b.as_ref().unwrap_or(&bp[0]);
                an.cmp(bn)
            })
        });
        for (_, members) in &mut entries {
            members.sort_by(|a, b| {
                settings
                    .project_recent_usage
                    .get(b)
                    .copied()
                    .unwrap_or(0)
                    .cmp(&settings.project_recent_usage.get(a).copied().unwrap_or(0))
                    .then_with(|| a.cmp(b))
            });
        }
    }

    entries
}

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

    let project_entries = grouped_level_one(
        sorted_projects
            .iter()
            .chain(sorted_adhoc.iter())
            .cloned()
            .collect(),
        &settings,
    );
    for (group_name, projects) in project_entries {
        if let Some(group_name) = group_name {
            let group_sub = Submenu::with_id(
                app,
                &format!("group::{}", project_id(&group_name)),
                &group_name,
                true,
            )?;
            for project in projects {
                group_sub.append(&CheckMenuItem::with_id(
                    app,
                    &format!("select::{}", project_id(&project)),
                    &project,
                    true,
                    *active == project,
                    None::<&str>,
                )?)?;
            }
            menu.append(&group_sub)?;
        } else {
            let project = &projects[0];
            menu.append(&CheckMenuItem::with_id(
                app,
                &format!("select::{}", project_id(project)),
                project,
                true,
                *active == *project,
                None::<&str>,
            )?)?;
        }
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

    menu.append(&MenuItem::with_id(
        app,
        "feedback_github",
        "Report an issue...",
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
    menu.append(&MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?)?;

    Ok(menu)
}

pub fn project_from_select_id(_app: &AppHandle, id: &str) -> Option<String> {
    project_from_id(id)
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

#[cfg(test)]
mod tests {
    use super::{grouped_level_one, project_from_id, project_id};
    use crate::settings::UiSettings;
    use std::collections::HashMap;

    #[test]
    fn project_ids_roundtrip_names_instead_of_positions() {
        let name = "Client Alpha / Q3::Kickoff";
        let encoded = project_id(name);

        assert_ne!(encoded, name);
        assert_eq!(project_from_id(&encoded), Some(name.to_string()));
    }

    #[test]
    fn grouped_level_one_builds_recent_ordered_group_submenus() {
        let settings = UiSettings {
            group_projects_enabled: true,
            project_sort_mode: "recent".to_string(),
            project_groups: HashMap::from([
                ("Bravo".to_string(), "Work".to_string()),
                ("Delta".to_string(), "Work".to_string()),
                ("Charlie".to_string(), "Personal".to_string()),
            ]),
            project_recent_usage: HashMap::from([
                ("Alpha".to_string(), 40),
                ("Bravo".to_string(), 10),
                ("Charlie".to_string(), 30),
                ("Delta".to_string(), 50),
            ]),
            ..UiSettings::default()
        };

        let entries = grouped_level_one(
            vec![
                "Alpha".to_string(),
                "Bravo".to_string(),
                "Charlie".to_string(),
                "Delta".to_string(),
            ],
            &settings,
        );

        assert_eq!(
            entries,
            vec![
                (
                    Some("Work".to_string()),
                    vec!["Delta".to_string(), "Bravo".to_string()]
                ),
                (None, vec!["Alpha".to_string()]),
                (Some("Personal".to_string()), vec!["Charlie".to_string()]),
            ]
        );
    }
}
