use console::Style;

pub struct Logger {
    error: Style,
    warn: Style,
    info: Style,
    success: Style,
    debug: Style,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            error: Style::new().red().bold(),
            warn: Style::new().yellow().bold(),
            info: Style::new().cyan(),
            success: Style::new().green().bold(),
            debug: Style::new().dim(),
        }
    }
}

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn error(&self, msg: impl std::fmt::Display) {
        eprintln!("{}", self.error.apply_to(format!("[ ERROR ] {msg}")))
    }

    pub fn warn(&self, msg: impl std::fmt::Display) {
        eprintln!("{}", self.warn.apply_to(format!("[ WARN ] {msg}")))
    }

    pub fn info(&self, msg: impl std::fmt::Display) {
        println!("{}", self.info.apply_to(format!("[ INFO ] {msg}")))
    }

    pub fn success(&self, msg: impl std::fmt::Display) {
        println!("{}", self.success.apply_to(format!("[ SUCCESS ] {msg}")))
    }

    #[allow(dead_code)]
    pub fn debug(&self, msg: impl std::fmt::Display) {
        if cfg!(debug_assertions) {
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

lazy_static::lazy_static! {
    pub static ref LOG: Logger = Logger::new();
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::utils::logger::LOG.error(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::utils::logger::LOG.warn(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::utils::logger::LOG.info(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::utils::logger::LOG.success(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::utils::logger::LOG.debug(format!($($arg)*))
    };
}
