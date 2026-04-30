use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeDelta};
use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet};
use serde::Serialize;

use crate::{log, log_debug, log_warn};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const PREVIEW_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimesheetRange {
    Today,
    Week,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimesheetFormat {
    Full,
    Recent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimesheetOptions {
    pub range: TimesheetRange,
    pub format: TimesheetFormat,
}

impl TimesheetOptions {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn full(range: TimesheetRange) -> Self {
        Self {
            range,
            format: TimesheetFormat::Full,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn recent() -> Self {
        Self {
            range: TimesheetRange::Today,
            format: TimesheetFormat::Recent,
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
struct BucketedData<const N: usize> {
    projects: BTreeMap<String, [f64; N]>,
    comments: BTreeMap<String, BTreeMap<String, [f64; N]>>,
}

type WeekData = BucketedData<7>;
type RecentData = BucketedData<2>;

#[derive(Clone, Serialize)]
pub struct TimesheetPreview {
    pub title: String,
    pub generated_at: String,
    pub generated_at_epoch_ms: i64,
    pub sheets: Vec<TimesheetPreviewSheet>,
}

#[derive(Clone, Serialize)]
pub struct TimesheetPreviewSheet {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<TimesheetPreviewRow>,
}

#[derive(Clone, Serialize)]
pub struct TimesheetPreviewRow {
    pub label: String,
    pub values: Vec<f64>,
    pub total: f64,
    pub is_comment: bool,
    pub is_total: bool,
}

struct ParsedLog {
    entries: Vec<Entry>,
    today_date: NaiveDate,
    yesterday_date: NaiveDate,
    current_week: chrono::IsoWeek,
}

fn parse_log(data_dir: &Path, recent_only: bool) -> Result<ParsedLog, String> {
    let log_path = data_dir.join("log.dat");
    let content =
        std::fs::read_to_string(&log_path).map_err(|e| format!("Failed to read log: {}", e))?;

    let today = Local::now().naive_local();
    let today_date = today.date();
    let yesterday_date = today_date - TimeDelta::days(1);
    let current_week = today_date.iso_week();

    let mut entries: Vec<Entry> = Vec::new();
    let mut last_entry_before_recent: Option<Entry> = None;

    for line in content.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let Ok(timestamp) = NaiveDateTime::parse_from_str(parts[0], DATE_FORMAT) else {
            continue;
        };

        let entry = Entry {
            timestamp,
            project: parts[1].to_string(),
            comment: if parts.len() > 2 {
                parts[2..].join(" ")
            } else {
                String::new()
            },
        };

        if recent_only {
            if entry.timestamp.date() < yesterday_date {
                last_entry_before_recent = Some(entry);
                continue;
            }
            if entries.is_empty() {
                if let Some(previous) = last_entry_before_recent.take() {
                    entries.push(previous);
                }
            }
        }

        entries.push(entry);
    }

    if entries.is_empty() {
        return Err("Your timesheet is empty.".to_string());
    }

    entries.sort_by_key(|entry| entry.timestamp);

    Ok(ParsedLog {
        entries,
        today_date,
        yesterday_date,
        current_week,
    })
}

fn include_date(date: NaiveDate, options: TimesheetOptions, parsed: &ParsedLog) -> bool {
    match options.range {
        TimesheetRange::Today => date == parsed.today_date,
        TimesheetRange::Week => {
            let week = date.iso_week();
            week.year() == parsed.current_week.year() && week.week() == parsed.current_week.week()
        }
        TimesheetRange::All => true,
    }
}

fn accumulate_hours<const N: usize>(
    output: &mut BucketedData<N>,
    project: &str,
    comment: &str,
    day_index: usize,
    hours: f64,
) {
    let project_hours = output
        .projects
        .entry(project.to_string())
        .or_insert([0.0; N]);
    project_hours[day_index] += hours;

    if comment.is_empty() {
        return;
    }

    let comment_hours = output
        .comments
        .entry(project.to_string())
        .or_default()
        .entry(comment.to_string())
        .or_insert([0.0; N]);
    comment_hours[day_index] += hours;
}

fn build_rows<const N: usize>(data: &BucketedData<N>) -> Vec<TimesheetPreviewRow> {
    let mut rows = Vec::new();
    let mut day_totals = [0.0f64; N];

    for (project, hours) in &data.projects {
        if !hours.iter().any(|&hour| hour > 0.0) {
            continue;
        }

        rows.push(TimesheetPreviewRow {
            label: project.clone(),
            values: hours.to_vec(),
            total: hours.iter().sum(),
            is_comment: false,
            is_total: false,
        });

        for (index, hour) in hours.iter().enumerate() {
            day_totals[index] += hour;
        }

        if let Some(comments) = data.comments.get(project) {
            for (comment, comment_hours) in comments {
                if !comment_hours.iter().any(|&hour| hour > 0.0) {
                    continue;
                }

                rows.push(TimesheetPreviewRow {
                    label: format!("  - {}", comment),
                    values: comment_hours.to_vec(),
                    total: comment_hours.iter().sum(),
                    is_comment: true,
                    is_total: false,
                });
            }
        }
    }

    rows.push(TimesheetPreviewRow {
        label: "Total".to_string(),
        values: day_totals.to_vec(),
        total: day_totals.iter().sum(),
        is_comment: false,
        is_total: true,
    });

    rows
}

fn build_preview(parsed: ParsedLog, options: TimesheetOptions) -> Result<TimesheetPreview, String> {
    type WeekKey = (i32, u32);

    let mut weekly_output: BTreeMap<WeekKey, WeekData> = BTreeMap::new();
    let mut recent_output = RecentData::default();

    for index in 0..parsed.entries.len().saturating_sub(1) {
        let entry = &parsed.entries[index];
        let next = &parsed.entries[index + 1];

        if entry.project.is_empty() {
            continue;
        }

        let hours = (next.timestamp - entry.timestamp).num_seconds() as f64 / 3600.0;
        if hours <= 0.0 {
            continue;
        }

        let date = entry.timestamp.date();
        if !include_date(date, options, &parsed) {
            continue;
        }

        if options.format == TimesheetFormat::Recent {
            if date == parsed.yesterday_date || date == parsed.today_date {
                let day_index = if date == parsed.yesterday_date { 0 } else { 1 };
                accumulate_hours(
                    &mut recent_output,
                    &entry.project,
                    &entry.comment,
                    day_index,
                    hours,
                );
            }
            continue;
        }

        let iso_week = date.iso_week();
        let week_key = (iso_week.year(), iso_week.week());
        let weekday_index = date.weekday().num_days_from_monday() as usize;
        let week_data = weekly_output.entry(week_key).or_default();
        accumulate_hours(
            week_data,
            &entry.project,
            &entry.comment,
            weekday_index,
            hours,
        );
    }

    let generated_at = Local::now();
    let generated_label = generated_at.format(PREVIEW_TIMESTAMP_FORMAT).to_string();
    let generated_epoch_ms = generated_at.timestamp_millis();

    if options.format == TimesheetFormat::Recent {
        if !recent_output
            .projects
            .values()
            .any(|hours| hours.iter().any(|&hour| hour > 0.0))
        {
            return Err("No hours were found for today or yesterday.".to_string());
        }

        return Ok(TimesheetPreview {
            title: "Yesterday + today".to_string(),
            generated_at: generated_label,
            generated_at_epoch_ms: generated_epoch_ms,
            sheets: vec![TimesheetPreviewSheet {
                name: "Yesterday + today".to_string(),
                columns: vec![
                    parsed.yesterday_date.format("%a").to_string(),
                    parsed.today_date.format("%a").to_string(),
                ],
                rows: build_rows(&recent_output),
            }],
        });
    }

    if weekly_output.is_empty() {
        return Err("No hours were found for the selected range.".to_string());
    }

    let mut sheets = Vec::new();
    for ((year, week), week_data) in weekly_output {
        sheets.push(TimesheetPreviewSheet {
            name: format!("{}-{}", year, week),
            columns: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                .iter()
                .map(|value| value.to_string())
                .collect(),
            rows: build_rows(&week_data),
        });
    }

    Ok(TimesheetPreview {
        title: "Full timesheet".to_string(),
        generated_at: generated_label,
        generated_at_epoch_ms: generated_epoch_ms,
        sheets,
    })
}

fn write_hours_cell(
    sheet: &mut Worksheet,
    row_index: u32,
    column_index: u16,
    value: f64,
    row: &TimesheetPreviewRow,
    right_fmt: &Format,
    decimal_right_fmt: &Format,
) -> Result<(), String> {
    if row.is_comment {
        if value > 0.0 {
            sheet
                .write_number_with_format(row_index, column_index, value, decimal_right_fmt)
                .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    if row.is_total || value > 0.0 {
        sheet
            .write_number_with_format(row_index, column_index, value, decimal_right_fmt)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    sheet
        .write_string_with_format(row_index, column_index, "-", right_fmt)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn write_sheet(
    workbook: &mut Workbook,
    sheet: &TimesheetPreviewSheet,
    title: &str,
    first_column_width: f64,
) -> Result<(), String> {
    let right_fmt = Format::new().set_align(FormatAlign::Right);
    let decimal_right_fmt = Format::new()
        .set_num_format("0.0")
        .set_align(FormatAlign::Right);
    let comment_fmt = Format::new().set_align(FormatAlign::Left);

    let worksheet = workbook.add_worksheet();
    worksheet.set_name(&sheet.name).map_err(|e| e.to_string())?;
    worksheet
        .set_column_width(0, first_column_width)
        .map_err(|e| e.to_string())?;

    worksheet.write_string(0, 0, title).map_err(|e| e.to_string())?;
    for (index, header) in sheet.columns.iter().enumerate() {
        worksheet
            .write_string(0, (index + 1) as u16, header)
            .map_err(|e| e.to_string())?;
    }
    worksheet
        .write_string(0, (sheet.columns.len() + 1) as u16, "Total")
        .map_err(|e| e.to_string())?;

    for (index, row) in sheet.rows.iter().enumerate() {
        let row_index = (index + 1) as u32;
        if row.is_comment {
            worksheet
                .write_string_with_format(row_index, 0, &row.label, &comment_fmt)
                .map_err(|e| e.to_string())?;
        } else {
            worksheet
                .write_string(row_index, 0, &row.label)
                .map_err(|e| e.to_string())?;
        }

        for (column_offset, value) in row.values.iter().enumerate() {
            write_hours_cell(
                worksheet,
                row_index,
                (column_offset + 1) as u16,
                *value,
                row,
                &right_fmt,
                &decimal_right_fmt,
            )?;
        }

        worksheet
            .write_number_with_format(
                row_index,
                (row.values.len() + 1) as u16,
                row.total,
                &decimal_right_fmt,
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn generate(data_dir: &Path, options: TimesheetOptions) -> Result<PathBuf, String> {
    log!("generate timesheet started");
    let parsed = parse_log(data_dir, options.format == TimesheetFormat::Recent)?;
    let all_projects: BTreeSet<String> = parsed
        .entries
        .iter()
        .filter(|entry| !entry.project.is_empty())
        .map(|entry| entry.project.clone())
        .collect();
    log!(
        "timesheet parsed entries={} projects={}",
        parsed.entries.len(),
        all_projects.len()
    );

    let preview = build_preview(parsed, options)?;
    let mut workbook = Workbook::new();

    for sheet in &preview.sheets {
        log_debug!("writing worksheet {}", sheet.name);
        let title = if options.format == TimesheetFormat::Recent {
            "Yesterday + Today"
        } else {
            sheet.name.as_str()
        };
        let first_column_width = if options.format == TimesheetFormat::Recent {
            16.0
        } else {
            42.0
        };
        write_sheet(&mut workbook, sheet, title, first_column_width)?;
    }

    let output_filename = match options.format {
        TimesheetFormat::Recent => "timesheet-yesterday-today.xlsx",
        TimesheetFormat::Full => match options.range {
            TimesheetRange::Today => "timesheet-today.xlsx",
            TimesheetRange::Week => "timesheet-week.xlsx",
            TimesheetRange::All => "timesheet.xlsx",
        },
    };
    let output_path = data_dir.join(output_filename);
    if let Err(error) = workbook.save(output_path.to_str().unwrap()) {
        let fallback_name = match options.format {
            TimesheetFormat::Recent => format!(
                "timesheet-yesterday-today-{}.xlsx",
                Local::now().format("%Y%m%d-%H%M%S")
            ),
            TimesheetFormat::Full => match options.range {
                TimesheetRange::Today => format!(
                    "timesheet-today-{}.xlsx",
                    Local::now().format("%Y%m%d-%H%M%S")
                ),
                TimesheetRange::Week => format!(
                    "timesheet-week-{}.xlsx",
                    Local::now().format("%Y%m%d-%H%M%S")
                ),
                TimesheetRange::All => format!(
                    "timesheet-{}.xlsx",
                    Local::now().format("%Y%m%d-%H%M%S")
                ),
            },
        };
        let fallback_path = data_dir.join(&fallback_name);
        workbook
            .save(fallback_path.to_str().unwrap())
            .map_err(|fallback_error| {
                format!(
                    "Failed to save {} (it may be open in Excel). Also failed to save fallback file {}: {}. Original error: {}",
                    output_filename, fallback_name, fallback_error, error
                )
            })?;
        log_warn!(
            "timesheet primary save failed {}; saved fallback {}",
            output_path.display(),
            fallback_path.display()
        );
        return Ok(fallback_path);
    }

    log!("timesheet saved {}", output_path.display());
    Ok(output_path)
}

pub fn preview(data_dir: &Path, options: TimesheetOptions) -> Result<TimesheetPreview, String> {
    let parsed = parse_log(data_dir, options.format == TimesheetFormat::Recent)?;
    build_preview(parsed, options)
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
    fn recent_generation_creates_separate_file() {
        let dir = temp_dir("timesheet-recent");
        let today = Local::now().naive_local().date();
        let yesterday = today - TimeDelta::days(1);
        let log = format!(
            "{yesterday} 09:00:00\tAlpha\tPrep\n{yesterday} 11:00:00\t\n{today} 10:00:00\tBeta\tBuild\n{today} 13:00:00\t\n"
        );
        fs::write(dir.join("log.dat"), log).unwrap();

        let path = generate(&dir, TimesheetOptions::recent()).unwrap();

        assert!(path.ends_with("timesheet-yesterday-today.xlsx"));
        assert!(path.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preview_recent_includes_comment_rows_totals_and_generation_time() {
        let dir = temp_dir("timesheet-preview-recent");
        let today = Local::now().naive_local().date();
        let log = format!("{today} 10:00:00\tBeta\tBuild\n{today} 13:00:00\t\n");
        fs::write(dir.join("log.dat"), log).unwrap();

        let preview = preview(&dir, TimesheetOptions::recent()).unwrap();

        assert_eq!(preview.title, "Yesterday + today");
        assert!(!preview.generated_at.is_empty());
        assert!(preview.generated_at_epoch_ms > 0);
        assert_eq!(preview.sheets.len(), 1);
        assert_eq!(preview.sheets[0].rows[0].label, "Beta");
        assert_eq!(preview.sheets[0].rows[0].total, 3.0);
        assert_eq!(preview.sheets[0].rows[1].label, "  - Build");
        assert!(
            preview.sheets[0]
                .rows
                .iter()
                .find(|row| row.label == "  - Build")
                .unwrap()
                .is_comment
        );
        assert_eq!(preview.sheets[0].rows.last().unwrap().label, "Total");
        assert!(preview.sheets[0].rows.last().unwrap().is_total);
        assert_eq!(preview.sheets[0].rows.last().unwrap().total, 3.0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preview_full_groups_entries_by_week() {
        let dir = temp_dir("timesheet-preview-full");
        fs::write(
            dir.join("log.dat"),
            "2026-04-20 09:00:00\tAlpha\tKickoff\n2026-04-20 11:00:00\tBeta\n2026-04-28 09:00:00\tBeta\tBuild\n2026-04-28 12:00:00\t\n",
        )
        .unwrap();

        let preview = preview(&dir, TimesheetOptions::full(TimesheetRange::All)).unwrap();

        assert_eq!(preview.title, "Full timesheet");
        assert_eq!(preview.sheets.len(), 2);
        assert_eq!(preview.sheets[0].name, "2026-17");
        assert_eq!(preview.sheets[1].name, "2026-18");
        assert_eq!(preview.sheets[0].rows[0].label, "Alpha");
        assert_eq!(preview.sheets[0].rows[1].label, "  - Kickoff");
        assert_eq!(preview.sheets[0].rows.last().unwrap().label, "Total");
        assert_eq!(preview.sheets[1].rows[0].label, "Beta");
        assert_eq!(preview.sheets[1].rows.last().unwrap().total, 3.0);
        let _ = fs::remove_dir_all(dir);
    }
}
