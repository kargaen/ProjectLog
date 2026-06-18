use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::models::repository_traits::log_repository::LogRepository;
use crate::{log, log_warn};

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub struct FileLogRepository {
    data_dir: PathBuf,
}

impl FileLogRepository {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }
}

impl LogRepository for FileLogRepository {
    fn append_entry(&self, project: &str, comment: &str) {
        log!(
            "time entry project='{}' comment_len={}",
            project,
            comment.len()
        );
        let path = self.data_dir.join("log.dat");
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

    fn append_comment_to_last(&self, comment: &str) {
        log!("append_comment_to_last len={}", comment.len());
        let path = self.data_dir.join("log.dat");
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

    fn reset(&self) {
        log_warn!("reset_log");
        let path = self.data_dir.join("log.dat");
        let _ = fs::write(path, "");
    }
}
