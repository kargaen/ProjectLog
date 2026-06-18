use std::path::Path;

use crate::models::repository_traits::log_repository::LogRepository;
use crate::repositories::file_log_repository::FileLogRepository;

pub fn log_new_entry(data_dir: &Path, project: &str, comment: &str) {
    FileLogRepository::new(data_dir).append_entry(project, comment);
}

pub fn append_comment_to_last(data_dir: &Path, comment: &str) {
    FileLogRepository::new(data_dir).append_comment_to_last(comment);
}

pub fn reset_log(data_dir: &Path) {
    FileLogRepository::new(data_dir).reset();
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
    fn append_comment_preserves_last_character_without_trailing_newline() {
        let dir = temp_dir("append-no-newline");
        let path = dir.join("log.dat");
        fs::write(&path, "2026-04-25 08:00:00\tProject").unwrap();

        append_comment_to_last(&dir, "Checked drawing");

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "2026-04-25 08:00:00\tProject\tChecked drawing\n");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn append_comment_replaces_trailing_newline() {
        let dir = temp_dir("append-newline");
        let path = dir.join("log.dat");
        fs::write(&path, "2026-04-25 08:00:00\tProject\n").unwrap();

        append_comment_to_last(&dir, "Meeting");

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "2026-04-25 08:00:00\tProject\tMeeting\n");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn log_new_entry_writes_project_and_comment() {
        let dir = temp_dir("log-entry-comment");
        let path = dir.join("log.dat");

        log_new_entry(&dir, "Alpha", "Planning");

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("\tAlpha\tPlanning\n"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn log_new_entry_without_comment_writes_two_columns() {
        let dir = temp_dir("log-entry-no-comment");
        let path = dir.join("log.dat");

        log_new_entry(&dir, "Alpha", "");

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("\tAlpha\n"));
        assert!(!content.contains("\tAlpha\t\n"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reset_log_clears_existing_content() {
        let dir = temp_dir("reset-log");
        let path = dir.join("log.dat");
        fs::write(&path, "2026-04-25 08:00:00\tProject\tComment\n").unwrap();

        reset_log(&dir);

        let content = fs::read_to_string(path).unwrap();
        assert!(content.is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}
