use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, NaiveDateTime, TimeDelta};
use rust_xlsxwriter::{Format, FormatAlign, Workbook};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn generate(data_dir: &Path) -> Result<PathBuf, String> {
    let log_path = data_dir.join("log.dat");
    let content =
        std::fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {}", e))?;

    // Parse entries
    let mut entries: Vec<(NaiveDateTime, String)> = Vec::new();
    let mut all_projects: BTreeSet<String> = BTreeSet::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            if let Ok(ts) = NaiveDateTime::parse_from_str(parts[0], DATE_FORMAT) {
                let project = parts[1].to_string();
                if !project.is_empty() {
                    all_projects.insert(project.clone());
                }
                entries.push((ts, project));
            }
        }
    }

    if entries.is_empty() {
        return Err("Your timesheet is empty.".to_string());
    }

    let first_ts = entries.first().unwrap().0;
    let today = Local::now().naive_local();

    // Build week structure: (year, week) -> project -> [7 days of hours]
    type WeekKey = (i32, u32);
    let mut output: BTreeMap<WeekKey, BTreeMap<String, [f64; 7]>> = BTreeMap::new();

    // Initialize weeks from first entry through current week
    let mut current_date = first_ts.date();
    let today_iso = today.date().iso_week();
    loop {
        let iso = current_date.iso_week();
        let wk: WeekKey = (iso.year(), iso.week());
        let week_data = output.entry(wk).or_default();
        for p in &all_projects {
            week_data.entry(p.clone()).or_insert([0.0; 7]);
        }
        if iso.year() == today_iso.year() && iso.week() == today_iso.week() {
            break;
        }
        current_date += TimeDelta::days(7);
        // Safety: don't loop forever
        if current_date.year() > today.date().year() + 1 {
            break;
        }
    }

    // Calculate hours between consecutive entries
    for i in 0..entries.len().saturating_sub(1) {
        let (ts, ref project) = entries[i];
        let (next_ts, _) = entries[i + 1];

        if !project.is_empty() {
            let hours = (next_ts - ts).num_seconds() as f64 / 3600.0;
            // Sanity: skip gaps larger than 24 hours
            if hours > 0.0 && hours < 24.0 {
                let iso = ts.date().iso_week();
                let wk: WeekKey = (iso.year(), iso.week());
                let weekday = ts.date().weekday().num_days_from_monday() as usize;

                if let Some(week_data) = output.get_mut(&wk) {
                    let project_hours = week_data.entry(project.clone()).or_insert([0.0; 7]);
                    project_hours[weekday] += hours;
                }
            }
        }
    }

    // Generate Excel workbook
    let mut workbook = Workbook::new();
    let right_fmt = Format::new().set_align(FormatAlign::Right);
    let decimal_right_fmt = Format::new()
        .set_num_format("0.0")
        .set_align(FormatAlign::Right);

    let day_headers = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun", "Total"];

    for ((year, week), projects) in &output {
        let sheet_name = format!("{}-{}", year, week);
        let ws = workbook.add_worksheet();
        ws.set_name(&sheet_name).map_err(|e| e.to_string())?;
        ws.set_column_width(0, 30).map_err(|e| e.to_string())?;

        // Header row
        ws.write_string(0, 0, &sheet_name)
            .map_err(|e| e.to_string())?;
        for (i, hdr) in day_headers.iter().enumerate() {
            ws.write_string(0, (i + 1) as u16, hdr)
                .map_err(|e| e.to_string())?;
        }

        let mut row: u32 = 0;
        let mut day_totals = [0.0f64; 7];

        for (project, hours) in projects {
            if hours.iter().any(|&h| h > 0.0) {
                row += 1;
                ws.write_string(row, 0, project)
                    .map_err(|e| e.to_string())?;

                for (i, &h) in hours.iter().enumerate() {
                    day_totals[i] += h;
                    if h == 0.0 {
                        ws.write_string_with_format(row, (i + 1) as u16, "-", &right_fmt)
                            .map_err(|e| e.to_string())?;
                    } else {
                        ws.write_number_with_format(
                            row,
                            (i + 1) as u16,
                            h,
                            &decimal_right_fmt,
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }

                let total: f64 = hours.iter().sum();
                ws.write_number_with_format(row, 8, total, &decimal_right_fmt)
                    .map_err(|e| e.to_string())?;
            }
        }

        // Totals row
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

    Ok(output_path)
}
