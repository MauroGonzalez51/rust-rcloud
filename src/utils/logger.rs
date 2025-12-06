use console::Style;
use std::sync::RwLock;

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

    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.get_level()
    }

    pub fn error(&self, msg: impl std::fmt::Display) {
        if self.should_log(LogLevel::Error) {
            eprintln!("{}", self.error.apply_to(format!("[ ERROR ] {msg}")))
        }
    }

    pub fn warn(&self, msg: impl std::fmt::Display) {
        if self.should_log(LogLevel::Warn) {
            eprintln!("{}", self.warn.apply_to(format!("[ WARN ] {msg}")))
        }
    }

    pub fn info(&self, msg: impl std::fmt::Display) {
        if self.should_log(LogLevel::Info) {
            println!("{}", self.info.apply_to(format!("[ INFO ] {msg}")))
        }
    }

    pub fn success(&self, msg: impl std::fmt::Display) {
        if self.should_log(LogLevel::Success) {
            println!("{}", self.success.apply_to(format!("[ SUCCESS ] {msg}")))
        }
    }

    pub fn debug(&self, msg: impl std::fmt::Display) {
        if self.should_log(LogLevel::Debug) {
            println!("{}", self.debug.apply_to(format!("[ DEBUG ] {msg}")))
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
