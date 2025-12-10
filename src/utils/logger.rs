use anyhow::Context;
use console::Style;
use crossterm::terminal;
use std::{
    io::Write,
    sync::{Mutex, OnceLock, RwLock},
};

const LOG_ROTATE_BYTES: u64 = 5 * 1024 * 1024;

static LOG_FILE: OnceLock<(Mutex<std::fs::File>, std::path::PathBuf)> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Success = 2,
    Warn = 3,
    Error = 4,
}

pub struct Logger {
    error: Style,
    warn: Style,
    info: Style,
    success: Style,
    debug: Style,
    level: RwLock<LogLevel>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            error: Style::new().red().bold(),
            warn: Style::new().yellow().bold(),
            info: Style::new().cyan(),
            success: Style::new().green().bold(),
            debug: Style::new().dim(),
            level: RwLock::new(LogLevel::Info),
        }
    }
}

impl Logger {
    pub fn setup(path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let path = path.as_ref().to_path_buf();

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .context("failed to setup logger")?;

        let _ = LOG_FILE.set((Mutex::new(file), path));

        Ok(())
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_level(&self, level: LogLevel) {
        if let Ok(mut current_level) = self.level.write() {
            *current_level = level;
        }
    }

    pub fn get_level(&self) -> LogLevel {
        self.level.read().map(|l| *l).unwrap_or(LogLevel::Info)
    }

    fn rotate(&self, mutex: &std::sync::Mutex<std::fs::File>, path: &std::path::Path) {
        if let Ok(meta) = mutex.lock().unwrap().metadata()
            && meta.len() <= LOG_ROTATE_BYTES
        {
            return;
        }

        drop(mutex.lock().unwrap());

        let rotated = path.with_extension(format!(
            "{}.{}",
            path.extension().and_then(|e| e.to_str()).unwrap_or("log"),
            chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
        ));

        let _ = std::fs::rename(path, &rotated);

        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            && let Ok(mut guard) = mutex.lock()
        {
            *guard = file;
        }
    }

    fn write_file(&self, line: &str) {
        if let Some((mutex, path)) = LOG_FILE.get() {
            self.rotate(mutex, path);
            let _ = writeln!(mutex.lock().unwrap(), "{} / {}", chrono::Utc::now(), line);
        }
    }

    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.get_level()
    }

    fn should_print(&self) -> bool {
        !terminal::is_raw_mode_enabled().unwrap_or(true)
    }

    pub fn error(&self, msg: impl std::fmt::Display) {
        if !self.should_log(LogLevel::Error) {
            return;
        }
        let plain = format!("[ ERROR ] {msg}");
        self.write_file(&plain);
        if self.should_print() {
            eprintln!("{}", self.error.apply_to(&plain));
        }
    }

    pub fn warn(&self, msg: impl std::fmt::Display) {
        if !self.should_log(LogLevel::Warn) {
            return;
        }
        let plain = format!("[ WARN ] {msg}");
        self.write_file(&plain);
        if self.should_print() {
            eprintln!("{}", self.warn.apply_to(&plain));
        }
    }

    pub fn info(&self, msg: impl std::fmt::Display) {
        if !self.should_log(LogLevel::Info) {
            return;
        }
        let plain = format!("[ INFO ] {msg}");
        self.write_file(&plain);
        if self.should_print() {
            println!("{}", self.info.apply_to(&plain));
        }
    }

    pub fn success(&self, msg: impl std::fmt::Display) {
        if !self.should_log(LogLevel::Success) {
            return;
        }
        let plain = format!("[ SUCCESS ] {msg}");
        self.write_file(&plain);
        if self.should_print() {
            println!("{}", self.success.apply_to(&plain));
        }
    }

    pub fn debug(&self, msg: impl std::fmt::Display) {
        if !self.should_log(LogLevel::Debug) {
            return;
        }
        let plain = format!("[ DEBUG ] {msg}");
        self.write_file(&plain);
        if self.should_print() {
            println!("{}", self.debug.apply_to(&plain));
        }
    }

    pub fn with_context(&self, error: &anyhow::Error) {
        self.error(error);

        for (i, cause) in error.chain().skip(1).enumerate() {
            eprintln!(
                "  {} {}: {}",
                self.debug.apply_to("│"),
                self.debug.apply_to(format!("Caused by ({})", i + 1)),
                cause
            );
        }
    }
}

pub fn logger() -> &'static Logger {
    static LOG: std::sync::OnceLock<Logger> = std::sync::OnceLock::new();
    LOG.get_or_init(Logger::new)
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::utils::logger::logger().error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::utils::logger::logger().warn(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::utils::logger::logger().info(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::utils::logger::logger().success(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::utils::logger::logger().debug(format!($($arg)*))
    };
}
