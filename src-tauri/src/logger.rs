use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use chrono::Local;

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Write a new log entry on its own line: `datetime\tproject\tcomment\n`
pub fn log_new_entry(data_dir: &Path, project: &str, comment: &str) {
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
/// Seeks back past the trailing newline and appends `\tcomment\n`.
pub fn append_comment_to_last(data_dir: &Path, comment: &str) {
    let path = data_dir.join("log.dat");
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("failed to open log file");

    let end = f.seek(SeekFrom::End(0)).expect("failed to seek");
    if end == 0 {
        return;
    }

    // Overwrite the trailing newline with \tcomment\n
    f.seek(SeekFrom::End(-1)).expect("failed to seek");
    write!(f, "\t{}\n", comment).expect("failed to write comment");
}

/// Reset the log file (empty it).
pub fn reset_log(data_dir: &Path) {
    let path = data_dir.join("log.dat");
    let _ = fs::write(path, "");
}
