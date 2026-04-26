use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use chrono::Local;

use crate::{log, log_warn};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Write a new log entry on its own line: `datetime\tproject\tcomment\n`
pub fn log_new_entry(data_dir: &Path, project: &str, comment: &str) {
    log!(
        "time entry project='{}' comment_len={}",
        project,
        comment.len()
    );
    let path = data_dir.join("log.dat");
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("failed to open log file");

    let now = Local::now().format(DATE_FORMAT);
    if comment.is_empty() {
        writeln!(f, "{}\t{}", now, project).expect("failed to write log");
    } else {
        writeln!(f, "{}\t{}\t{}", now, project, comment).expect("failed to write log");
    }
}

/// Append a comment to the last line (which currently has no comment).
pub fn append_comment_to_last(data_dir: &Path, comment: &str) {
    log!("append_comment_to_last len={}", comment.len());
    let path = data_dir.join("log.dat");
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("failed to open log file");

    let end = f.seek(SeekFrom::End(0)).expect("failed to seek");
    if end == 0 {
        log_warn!("append_comment_to_last skipped empty log");
        return;
    }

    let mut last = [0u8; 1];
    f.seek(SeekFrom::End(-1)).expect("failed to seek");
    f.read_exact(&mut last).expect("failed to read log");
    if last[0] == b'\n' {
        f.seek(SeekFrom::End(-1)).expect("failed to seek");
    } else {
        f.seek(SeekFrom::End(0)).expect("failed to seek");
    }
    write!(f, "\t{}\n", comment).expect("failed to write comment");
}

/// Reset the log file (empty it).
pub fn reset_log(data_dir: &Path) {
    log_warn!("reset_log");
    let path = data_dir.join("log.dat");
    let _ = fs::write(path, "");
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
