use std::path::{Path, PathBuf};

use chrono::{Datelike, Local};
use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet};

use crate::{log, log_debug, log_warn};
use crate::models::domain::timesheet::{TimesheetFormat, TimesheetOptions, TimesheetRange};
use crate::models::dto::timesheet_dto::TimesheetPreviewRow;
use crate::services::timesheet_service;

const ZERO_EPSILON: f64 = 0.000001;

fn is_zero_hours(value: f64) -> bool {
    value.abs() < ZERO_EPSILON
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
    if is_zero_hours(value) {
        sheet
            .write_string_with_format(row_index, column_index, "-", right_fmt)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    if row.is_comment || row.is_total || value > 0.0 {
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
    sheet: &crate::models::dto::timesheet_dto::TimesheetPreviewSheet,
    title: &str,
    first_column_width: f64,
    active: bool,
) -> Result<(), String> {
    let right_fmt = Format::new().set_align(FormatAlign::Right);
    let decimal_right_fmt = Format::new()
        .set_num_format("0.0")
        .set_align(FormatAlign::Right);
    let comment_fmt = Format::new().set_align(FormatAlign::Left);

    let worksheet = workbook.add_worksheet();
    worksheet.set_name(&sheet.name).map_err(|e| e.to_string())?;
    if active {
        worksheet.set_active(true);
    }
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

        write_hours_cell(
            worksheet,
            row_index,
            (row.values.len() + 1) as u16,
            row.total,
            row,
            &right_fmt,
            &decimal_right_fmt,
        )?;
    }

    Ok(())
}

pub fn generate(data_dir: &Path, options: TimesheetOptions) -> Result<PathBuf, String> {
    log!("generate timesheet started");
    let preview = timesheet_service::preview(data_dir, options)?;
    let mut workbook = Workbook::new();
    let current_week = Local::now().iso_week();
    let current_week_name = format!("{}-{}", current_week.year(), current_week.week());
    let active_sheet_index = preview
        .sheets
        .iter()
        .position(|sheet| sheet.name == current_week_name)
        .unwrap_or_else(|| preview.sheets.len().saturating_sub(1));

    for (index, sheet) in preview.sheets.iter().enumerate() {
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
        write_sheet(
            &mut workbook,
            sheet,
            title,
            first_column_width,
            index == active_sheet_index,
        )?;
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
