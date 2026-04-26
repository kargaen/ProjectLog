use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, NaiveDateTime, TimeDelta};
use rust_xlsxwriter::{Format, FormatAlign, Workbook};

use crate::{log, log_debug, log_warn};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Clone)]
struct Entry {
    timestamp: NaiveDateTime,
    project: String,
    comment: String,
}

#[derive(Default)]
struct WeekData {
    projects: BTreeMap<String, [f64; 7]>,
    comments: BTreeMap<String, BTreeMap<String, [f64; 7]>>,
}

pub fn generate(data_dir: &Path) -> Result<PathBuf, String> {
    log!("generate timesheet started");
    let log_path = data_dir.join("log.dat");
    let content =
        std::fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {}", e))?;

    let mut entries: Vec<Entry> = Vec::new();
    let mut all_projects: BTreeSet<String> = BTreeSet::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            if let Ok(timestamp) = NaiveDateTime::parse_from_str(parts[0], DATE_FORMAT) {
                let project = parts[1].to_string();
                let comment = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    String::new()
                };

                if !project.is_empty() {
                    all_projects.insert(project.clone());
                }
                entries.push(Entry {
                    timestamp,
                    project,
                    comment,
                });
            }
        }
    }

    if entries.is_empty() {
        log_warn!("timesheet generation found no entries");
        return Err("Your timesheet is empty.".to_string());
    }
    log!(
        "timesheet parsed entries={} projects={}",
        entries.len(),
        all_projects.len()
    );

    type WeekKey = (i32, u32);
    let mut output: BTreeMap<WeekKey, WeekData> = BTreeMap::new();
    let first_ts = entries.first().unwrap().timestamp;
    let today = Local::now().naive_local();

    let mut current_date = first_ts.date();
    let today_iso = today.date().iso_week();
    loop {
        let iso = current_date.iso_week();
        let wk: WeekKey = (iso.year(), iso.week());
        let week_data = output.entry(wk).or_default();
        for project in &all_projects {
            week_data
                .projects
                .entry(project.clone())
                .or_insert([0.0; 7]);
        }

        if iso.year() == today_iso.year() && iso.week() == today_iso.week() {
            break;
        }
        current_date += TimeDelta::days(7);
        if current_date.year() > today.date().year() + 1 {
            break;
        }
    }

    for i in 0..entries.len().saturating_sub(1) {
        let entry = &entries[i];
        let next = &entries[i + 1];

        if entry.project.is_empty() {
            continue;
        }

        let hours = (next.timestamp - entry.timestamp).num_seconds() as f64 / 3600.0;
        if hours <= 0.0 {
            continue;
        }

        let iso = entry.timestamp.date().iso_week();
        let wk: WeekKey = (iso.year(), iso.week());
        let weekday = entry.timestamp.date().weekday().num_days_from_monday() as usize;
        let week_data = output.entry(wk).or_default();

        let project_hours = week_data
            .projects
            .entry(entry.project.clone())
            .or_insert([0.0; 7]);
        project_hours[weekday] += hours;

        if !entry.comment.is_empty() {
            let comment_hours = week_data
                .comments
                .entry(entry.project.clone())
                .or_default()
                .entry(entry.comment.clone())
                .or_insert([0.0; 7]);
            comment_hours[weekday] += hours;
        }
    }

    let mut workbook = Workbook::new();
    let right_fmt = Format::new().set_align(FormatAlign::Right);
    let decimal_right_fmt = Format::new()
        .set_num_format("0.0")
        .set_align(FormatAlign::Right);
    let comment_fmt = Format::new().set_align(FormatAlign::Left);

    let day_headers = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun", "Total"];

    for ((year, week), week_data) in &output {
        log_debug!("writing worksheet {}-{}", year, week);
        let sheet_name = format!("{}-{}", year, week);
        let ws = workbook.add_worksheet();
        ws.set_name(&sheet_name).map_err(|e| e.to_string())?;
        ws.set_column_width(0, 42).map_err(|e| e.to_string())?;

        ws.write_string(0, 0, &sheet_name)
            .map_err(|e| e.to_string())?;
        for (i, hdr) in day_headers.iter().enumerate() {
            ws.write_string(0, (i + 1) as u16, *hdr)
                .map_err(|e| e.to_string())?;
        }

        let mut row: u32 = 0;
        let mut day_totals = [0.0f64; 7];

        for (project, hours) in &week_data.projects {
            if !hours.iter().any(|&h| h > 0.0) {
                continue;
            }

            row += 1;
            ws.write_string(row, 0, project)
                .map_err(|e| e.to_string())?;

            for (i, &h) in hours.iter().enumerate() {
                day_totals[i] += h;
                if h == 0.0 {
                    ws.write_string_with_format(row, (i + 1) as u16, "-", &right_fmt)
                        .map_err(|e| e.to_string())?;
                } else {
                    ws.write_number_with_format(row, (i + 1) as u16, h, &decimal_right_fmt)
                        .map_err(|e| e.to_string())?;
                }
            }

            let total: f64 = hours.iter().sum();
            ws.write_number_with_format(row, 8, total, &decimal_right_fmt)
                .map_err(|e| e.to_string())?;

            if let Some(comments) = week_data.comments.get(project) {
                for (comment, comment_hours) in comments {
                    if !comment_hours.iter().any(|&h| h > 0.0) {
                        continue;
                    }

                    row += 1;
                    ws.write_string_with_format(row, 0, &format!("  - {}", comment), &comment_fmt)
                        .map_err(|e| e.to_string())?;

                    for (i, &h) in comment_hours.iter().enumerate() {
                        if h > 0.0 {
                            ws.write_number_with_format(row, (i + 1) as u16, h, &decimal_right_fmt)
                                .map_err(|e| e.to_string())?;
                        }
                    }

                    let total: f64 = comment_hours.iter().sum();
                    ws.write_number_with_format(row, 8, total, &decimal_right_fmt)
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        row += 1;
        ws.write_string(row, 0, "Total")
            .map_err(|e| e.to_string())?;
        for (i, &t) in day_totals.iter().enumerate() {
            ws.write_number_with_format(row, (i + 1) as u16, t, &decimal_right_fmt)
                .map_err(|e| e.to_string())?;
        }
        let grand_total: f64 = day_totals.iter().sum();
        ws.write_number_with_format(row, 8, grand_total, &decimal_right_fmt)
            .map_err(|e| e.to_string())?;
    }

    let output_path = data_dir.join("timesheet.xlsx");
    workbook
        .save(output_path.to_str().unwrap())
        .map_err(|e| e.to_string())?;

    log!("timesheet saved {}", output_path.display());
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("projectlog-{name}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn generates_for_long_stretches_past_midnight() {
        let dir = temp_dir("timesheet-long");
        fs::write(
            dir.join("log.dat"),
            "2026-04-25 19:22:00\tNight project\tLate work\n2026-04-26 02:53:00\t\n",
        )
        .unwrap();

        let path = generate(&dir).unwrap();

        assert!(path.exists());
        assert!(fs::metadata(path).unwrap().len() > 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn empty_log_returns_friendly_error() {
        let dir = temp_dir("timesheet-empty");
        fs::write(dir.join("log.dat"), "").unwrap();

        let err = generate(&dir).unwrap_err();

        assert_eq!(err, "Your timesheet is empty.");
        let _ = fs::remove_dir_all(dir);
    }
}
