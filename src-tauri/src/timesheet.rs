pub use crate::models::domain::timesheet::{TimesheetFormat, TimesheetOptions, TimesheetRange};
pub use crate::models::dto::timesheet_dto::{
    TimesheetPreview, TimesheetPreviewRow, TimesheetPreviewSheet,
};
pub use crate::services::export_service::generate;
pub use crate::services::timesheet_service::preview;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::{Datelike, Local, TimeDelta};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("projectlog-{name}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_log(dir: &std::path::Path, lines: &[String]) {
        fs::write(dir.join("log.dat"), lines.join("\n") + "\n").unwrap();
    }

    fn build_relative_log_fixture() -> Vec<String> {
        let today = Local::now().naive_local().date();
        let yesterday = today - TimeDelta::days(1);
        let monday_this_week = today - TimeDelta::days(today.weekday().num_days_from_monday() as i64);
        let previous_week_monday = monday_this_week - TimeDelta::days(7);

        vec![
            format!("{} 09:00:00\tAlpha\tLegacy planning", previous_week_monday.format("%Y-%m-%d")),
            format!("{} 11:00:00\tBeta", previous_week_monday.format("%Y-%m-%d")),
            format!("{} 09:00:00\tBeta\tBuild", monday_this_week.format("%Y-%m-%d")),
            format!("{} 12:00:00\tGamma", monday_this_week.format("%Y-%m-%d")),
            format!("{} 08:30:00\tGamma\tReview", yesterday.format("%Y-%m-%d")),
            format!("{} 10:00:00\tAlpha\tSupport", yesterday.format("%Y-%m-%d")),
            format!("{} 09:15:00\tAlpha\tDelivery", today.format("%Y-%m-%d")),
            format!("{} 12:15:00\t", today.format("%Y-%m-%d")),
        ]
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
    fn preview_recent_uses_relative_fixture_for_yesterday_and_today() {
        let dir = temp_dir("timesheet-preview-recent-relative");
        write_log(&dir, &build_relative_log_fixture());

        let result = preview(&dir, TimesheetOptions::recent()).unwrap();

        assert_eq!(result.title, "Yesterday + today");
        assert_eq!(result.sheets.len(), 1);
        assert_eq!(result.sheets[0].columns.len(), 2);
        assert!(result.sheets[0].rows.iter().any(|row| row.label == "Alpha"));
        assert!(result.sheets[0].rows.iter().any(|row| row.label == "Total"));
        assert!(result.sheets[0].rows.iter().any(|row| row.total >= 3.0));
        assert_eq!(result.sheets[0].rows.last().unwrap().label, "Total");
        assert_eq!(result.sheets[0].rows.last().unwrap().values.len(), 2);
        assert!(result.generated_at_epoch_ms > 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preview_recent_includes_comment_rows_totals_and_generation_time() {
        let dir = temp_dir("timesheet-preview-recent");
        let today = Local::now().naive_local().date();
        let log = format!("{today} 10:00:00\tBeta\tBuild\n{today} 13:00:00\t\n");
        fs::write(dir.join("log.dat"), log).unwrap();

        let result = preview(&dir, TimesheetOptions::recent()).unwrap();

        assert_eq!(result.title, "Yesterday + today");
        assert!(!result.generated_at.is_empty());
        assert!(result.generated_at_epoch_ms > 0);
        assert_eq!(result.sheets.len(), 1);
        assert_eq!(result.sheets[0].rows[0].label, "Beta");
        assert_eq!(result.sheets[0].rows[0].total, 3.0);
        assert_eq!(result.sheets[0].rows[1].label, "  - Build");
        assert!(
            result.sheets[0]
                .rows
                .iter()
                .find(|row| row.label == "  - Build")
                .unwrap()
                .is_comment
        );
        assert_eq!(result.sheets[0].rows.last().unwrap().label, "Total");
        assert!(result.sheets[0].rows.last().unwrap().is_total);
        assert_eq!(result.sheets[0].rows.last().unwrap().total, 3.0);
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

        let result = preview(&dir, TimesheetOptions::full(TimesheetRange::All)).unwrap();

        assert_eq!(result.title, "Full timesheet");
        assert_eq!(result.sheets.len(), 2);
        assert_eq!(result.sheets[0].name, "2026-17");
        assert_eq!(result.sheets[1].name, "2026-18");
        assert_eq!(result.sheets[0].rows[0].label, "Alpha");
        assert_eq!(result.sheets[0].rows[1].label, "  - Kickoff");
        assert_eq!(result.sheets[0].rows.last().unwrap().label, "Total");
        assert_eq!(result.sheets[1].rows[0].label, "Beta");
        assert_eq!(result.sheets[1].rows.last().unwrap().total, 3.0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn preview_full_relative_fixture_spans_multiple_weeks_and_comments() {
        let dir = temp_dir("timesheet-preview-full-relative");
        write_log(&dir, &build_relative_log_fixture());

        let result = preview(&dir, TimesheetOptions::full(TimesheetRange::All)).unwrap();

        assert_eq!(result.title, "Full timesheet");
        assert!(result.sheets.len() >= 2);
        assert!(result.sheets.iter().all(|sheet| sheet.name.contains('-')));
        assert!(result
            .sheets
            .iter()
            .flat_map(|sheet| sheet.rows.iter())
            .any(|row| row.label == "  - Legacy planning"));
        assert!(result
            .sheets
            .iter()
            .flat_map(|sheet| sheet.rows.iter())
            .any(|row| row.label == "  - Build"));
        assert!(result
            .sheets
            .iter()
            .all(|sheet| sheet.rows.last().map(|row| row.label.as_str()) == Some("Total")));

        let _ = fs::remove_dir_all(dir);
    }

    // Deferred: inject a clock into preview/generate parsing so these tests can freeze "now"
    // instead of building relative fixtures from Local::now(). The current relative approach is
    // practical and valuable, but a clock abstraction would make week and recent boundaries fully deterministic.
}
