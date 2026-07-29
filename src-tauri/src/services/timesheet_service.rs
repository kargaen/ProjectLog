use std::collections::BTreeMap;
use std::path::Path;

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, TimeDelta};

use crate::models::domain::timesheet::{TimesheetFormat, TimesheetOptions, TimesheetRange};
use crate::models::dto::timesheet_dto::{
    TimesheetPreview, TimesheetPreviewRow, TimesheetPreviewSheet,
};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const PREVIEW_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M";

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

        if !include_date(date, options, &parsed) {
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

fn round_half_preserving_sum(values: &[f64]) -> Vec<f64> {
    let step = 0.5_f64;
    let floors: Vec<f64> = values.iter().map(|&v| (v / step).floor() * step).collect();
    let rounded_total = (values.iter().sum::<f64>() / step).round() * step;
    let current_total: f64 = floors.iter().sum();
    let mut increments = ((rounded_total - current_total) / step).round() as i64;

    let mut ranked: Vec<(usize, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| (i, v - floors[i]))
        .collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });

    let mut result = floors;
    for &(idx, _) in &ranked {
        if increments <= 0 {
            break;
        }
        result[idx] += step;
        increments -= 1;
    }

    result
}

fn apply_rounding_to_preview(mut preview: TimesheetPreview) -> TimesheetPreview {
    for sheet in &mut preview.sheets {
        let project_indices: Vec<usize> = sheet
            .rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| (!row.is_total && !row.is_comment).then_some(index))
            .collect();
        let value_count = sheet
            .rows
            .iter()
            .find(|r| r.is_total)
            .map_or(0, |r| r.values.len());

        // Allocate half-hour increments within each day so the visible entries retain the
        // rounded daily aggregate instead of accumulating independent per-project losses.
        for value_index in 0..value_count {
            let values: Vec<f64> = project_indices
                .iter()
                .map(|&row_index| sheet.rows[row_index].values[value_index])
                .collect();
            let rounded = round_half_preserving_sum(&values);
            for (&row_index, value) in project_indices.iter().zip(rounded) {
                sheet.rows[row_index].values[value_index] = value;
            }
        }

        for row in sheet.rows.iter_mut().filter(|row| row.is_comment) {
            row.values = round_half_preserving_sum(&row.values);
            row.total = row.values.iter().sum();
        }

        for &row_index in &project_indices {
            sheet.rows[row_index].total = sheet.rows[row_index].values.iter().sum();
        }

        let mut column_totals = vec![0.0_f64; value_count];
        for row in sheet.rows.iter() {
            if row.is_total || row.is_comment {
                continue;
            }
            for (i, &v) in row.values.iter().enumerate() {
                column_totals[i] += v;
            }
        }
        if let Some(total_row) = sheet.rows.iter_mut().find(|r| r.is_total) {
            total_row.total = column_totals.iter().sum();
            total_row.values = column_totals;
        }
    }
    preview
}

pub fn preview(data_dir: &Path, options: TimesheetOptions) -> Result<TimesheetPreview, String> {
    let parsed = parse_log(data_dir, options.format == TimesheetFormat::Recent)?;
    let preview = build_preview(parsed, options)?;
    if options.rounding_enabled {
        Ok(apply_rounding_to_preview(preview))
    } else {
        Ok(preview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preview_with_project_values(values: Vec<Vec<f64>>) -> TimesheetPreview {
        let value_count = values.first().map_or(0, Vec::len);
        let mut rows: Vec<TimesheetPreviewRow> = values
            .into_iter()
            .enumerate()
            .map(|(index, values)| TimesheetPreviewRow {
                label: format!("Project {index}"),
                total: values.iter().sum(),
                values,
                is_comment: false,
                is_total: false,
            })
            .collect();
        rows.push(TimesheetPreviewRow {
            label: "Total".to_string(),
            values: vec![0.0; value_count],
            total: 0.0,
            is_comment: false,
            is_total: true,
        });

        TimesheetPreview {
            title: "Test".to_string(),
            generated_at: String::new(),
            generated_at_epoch_ms: 0,
            sheets: vec![TimesheetPreviewSheet {
                name: "Test".to_string(),
                columns: (0..value_count).map(|index| index.to_string()).collect(),
                rows,
            }],
        }
    }

    /// Authority: the Timesheet Domain requires per-project daily totals and deliberate
    /// anti-inflation rounding, so displayed daily entries must add up to the rounded daily total.
    #[test]
    fn rounding_preserves_the_rounded_total_for_each_day() {
        let preview = preview_with_project_values(vec![
            vec![3.52, 0.08],
            vec![0.47, 0.0],
            vec![0.10, 0.0],
            vec![0.77, 0.0],
            vec![0.0, 0.71],
            vec![3.20, 0.0],
            vec![0.0, 6.69],
            vec![0.0, 0.52],
        ]);

        let rounded = apply_rounding_to_preview(preview);
        let total = rounded.sheets[0].rows.last().unwrap();

        assert_eq!(total.values, vec![8.0, 8.0]);
        assert_eq!(total.total, 16.0);
    }

    /// Authority: the Timesheet Domain's anti-inflation rule rounds the aggregate once instead
    /// of allowing repeated per-entry rounding losses to lower the reported daily total.
    #[test]
    fn rounding_distributes_half_hours_without_losing_the_daily_sum() {
        let preview = preview_with_project_values(vec![
            vec![7.24],
            vec![7.23],
            vec![7.22],
            vec![7.21],
            vec![7.20],
        ]);

        let rounded = apply_rounding_to_preview(preview);
        let rows = &rounded.sheets[0].rows;
        let values: Vec<f64> = rows[..5].iter().map(|row| row.values[0]).collect();

        assert_eq!(values, vec![7.5, 7.5, 7.0, 7.0, 7.0]);
        assert_eq!(rows.last().unwrap().total, 36.0);
    }
}
