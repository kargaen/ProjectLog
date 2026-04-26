use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::Local;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init(data_dir: &Path) {
    let _ = fs::create_dir_all(data_dir);
    let path = data_dir.join("ProjectLog-debug.log");
    let _ = LOG_PATH.set(path);
    write(
        "info",
        "diagnostics",
        "logging initialized",
        file!(),
        line!(),
    );
}

pub fn write(level: &str, module: &str, message: &str, file: &str, line: u32) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!("{timestamp}\t{level}\t{module}\t{file}:{line}\t{message}\n");

    match level {
        "error" => eprint!("{entry}"),
        _ => print!("{entry}"),
    }

    if let Some(path) = LOG_PATH.get() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

pub fn frontend(level: &str, module: &str, message: &str, data: Option<String>) {
    let message = match data {
        Some(data) if !data.is_empty() => format!("{message} | {data}"),
        _ => message.to_string(),
    };
    write(level, module, &message, "frontend", 0);
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::diagnostics::write("debug", module_path!(), &format!($($arg)*), file!(), line!())
    };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::diagnostics::write("info", module_path!(), &format!($($arg)*), file!(), line!())
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::diagnostics::write("warn", module_path!(), &format!($($arg)*), file!(), line!())
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::diagnostics::write("error", module_path!(), &format!($($arg)*), file!(), line!())
    };
}
