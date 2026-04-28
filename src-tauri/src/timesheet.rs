use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeDelta, Weekday};
use rust_xlsxwriter::{Format, FormatAlign, Workbook};

use crate::{log, log_debug, log_warn};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimesheetRange {
    Today,
    Week,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimesheetFormat {
    Full,
    Lite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimesheetOptions {
    pub range: TimesheetRange,
    pub format: TimesheetFormat,
}

impl TimesheetOptions {
    pub const fn full(range: TimesheetRange) -> Self {
        Self {
            range,
            format: TimesheetFormat::Full,
        }
    }

    pub const fn lite() -> Self {
        Self {
            range: TimesheetRange::Today,
            format: TimesheetFormat::Lite,
        }
    }
}

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

#[derive(Default)]
struct DayData {
    projects: BTreeMap<String, f64>,
}

pub fn generate(data_dir: &Path, options: TimesheetOptions) -> Result<PathBuf, String> {
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

    entries.sort_by_key(|entry| entry.timestamp);
    log!(
        "timesheet parsed entries={} projects={}",
        entries.len(),
        all_projects.len()
    );

    let today = Local::now().naive_local();
    let today_date = today.date();
    let yesterday_date = today_date - TimeDelta::days(1);
    let current_week = today_date.iso_week();

    let in_range = |date: NaiveDate| match options.range {
        TimesheetRange::Today => date == today_date,
        TimesheetRange::Week => {
            let week = date.iso_week();
            week.year() == current_week.year() && week.week() == current_week.week()
        }
        TimesheetRange::All => true,
    };

    type WeekKey = (i32, u32);
    let mut weekly_output: BTreeMap<WeekKey, WeekData> = BTreeMap::new();
    let mut lite_output: BTreeMap<NaiveDate, DayData> = BTreeMap::new();

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

        let date = entry.timestamp.date();
        if !in_range(date) {
            continue;
        }

        if options.format == TimesheetFormat::Lite {
            if date == today_date || date == yesterday_date {
                let day_data = lite_output.entry(date).or_default();
                *day_data.projects.entry(entry.project.clone()).or_insert(0.0) += hours;
            }
        } else {
            let iso = date.iso_week();
            let wk: WeekKey = (iso.year(), iso.week());
            let weekday = date.weekday().num_days_from_monday() as usize;
            let week_data = weekly_output.entry(wk).or_default();

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
    }

    if options.format == TimesheetFormat::Lite && lite_output.is_empty() {
        return Err("No hours were found for today or yesterday.".to_string());
    }
    if options.format == TimesheetFormat::Full && weekly_output.is_empty() {
        return Err("No hours were found for the selected range.".to_string());
    }

    let mut workbook = Workbook::new();
    let right_fmt = Format::new().set_align(FormatAlign::Right);
    let decimal_right_fmt = Format::new()
        .set_num_format("0.0")
        .set_align(FormatAlign::Right);
    let comment_fmt = Format::new().set_align(FormatAlign::Left);
    let day_headers = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun", "Total"];

    if options.format == TimesheetFormat::Lite {
        let ws = workbook.add_worksheet();
        ws.set_name("Lite").map_err(|e| e.to_string())?;
        ws.set_column_width(0, 16).map_err(|e| e.to_string())?;
        ws.set_column_width(1, 42).map_err(|e| e.to_string())?;

        ws.write_string(0, 0, "Date").map_err(|e| e.to_string())?;
        ws.write_string(0, 1, "Project").map_err(|e| e.to_string())?;
        ws.write_string(0, 2, "Hours").map_err(|e| e.to_string())?;

        let mut row = 1u32;
        let ordered_dates = [yesterday_date, today_date];
        for date in ordered_dates {
            let Some(day_data) = lite_output.get(&date) else {
                continue;
            };

            let label = match date.weekday() {
                Weekday::Mon => "Mon",
                Weekday::Tue => "Tue",
                Weekday::Wed => "Wed",
                Weekday::Thu => "Thu",
                Weekday::Fri => "Fri",
                Weekday::Sat => "Sat",
                Weekday::Sun => "Sun",
            };

            for (project, hours) in &day_data.projects {
                ws.write_string(row, 0, &format!("{date} ({label})"))
                    .map_err(|e| e.to_string())?;
                ws.write_string(row, 1, project).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 2, *hours, &decimal_right_fmt)
                    .map_err(|e| e.to_string())?;
                row += 1;
            }

            let total: f64 = day_data.projects.values().sum();
            ws.write_string(row, 1, "Total").map_err(|e| e.to_string())?;
            ws.write_number_with_format(row, 2, total, &decimal_right_fmt)
                .map_err(|e| e.to_string())?;
            row += 2;
        }
    } else {
        for ((year, week), week_data) in &weekly_output {
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
    }

    let output_filename = match options.format {
        TimesheetFormat::Lite => "timesheet-lite.xlsx",
        TimesheetFormat::Full => match options.range {
            TimesheetRange::Today => "timesheet-today.xlsx",
            TimesheetRange::Week => "timesheet-week.xlsx",
            TimesheetRange::All => "timesheet.xlsx",
        },
    };
    let output_path = data_dir.join(output_filename);
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

        let path = generate(&dir, TimesheetOptions::full(TimesheetRange::All)).unwrap();

        assert!(path.exists());
        assert!(fs::metadata(path).unwrap().len() > 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn empty_log_returns_friendly_error() {
        let dir = temp_dir("timesheet-empty");
        fs::write(dir.join("log.dat"), "").unwrap();

        let err = generate(&dir, TimesheetOptions::full(TimesheetRange::All)).unwrap_err();

        assert_eq!(err, "Your timesheet is empty.");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lite_generation_creates_separate_file() {
        let dir = temp_dir("timesheet-lite");
        let today = Local::now().naive_local().date();
        let yesterday = today - TimeDelta::days(1);
        let log = format!(
            "{yesterday} 09:00:00\tAlpha\tPrep\n{yesterday} 11:00:00\t\n{today} 10:00:00\tBeta\tBuild\n{today} 13:00:00\t\n"
        );
        fs::write(dir.join("log.dat"), log).unwrap();

        let path = generate(&dir, TimesheetOptions::lite()).unwrap();

        assert!(path.ends_with("timesheet-lite.xlsx"));
        assert!(path.exists());
        let _ = fs::remove_dir_all(dir);
    }
}
