//! Environment-agnostic output for the interactive commands.
//!
//! Handlers produce two kinds of output: results the user asked for (a listing,
//! remote ls output) and progress/status feedback ("remote added", warnings).
//! Where that output *goes* depends on the frontend: the CLI prints to stdout,
//! while the TUI (in raw mode, on the alternate screen) must capture it into a
//! panel instead — a raw `println!` there would corrupt the frame.
//!
//! [`OutputSink`] abstracts that. Handlers take a generic `S: OutputSink` and
//! emit [`OutputLine`]s; the concrete sink decides where they land. The CLI
//! passes [`StdoutSink`]; the TUI passes a buffer-backed sink rendered in a
//! side panel.
//!
//! This is *presentation* output. Internal debug tracing still goes through
//! `log_debug!` to the log file and is not routed here.

use console::Style;

/// Severity/kind of an output line, driving its styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLevel {
    /// Neutral result text (listings, ls entries).
    Plain,
    /// Informational progress.
    Info,
    /// Successful completion.
    Success,
    /// Non-fatal warning.
    Warn,
    /// Error feedback.
    Error,
}

/// A single line of user-facing output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputLine {
    pub level: OutputLevel,
    pub text: String,
}

impl OutputLine {
    pub fn new(level: OutputLevel, text: impl Into<String>) -> Self {
        Self {
            level,
            text: text.into(),
        }
    }
}

/// Receives user-facing output from the interactive commands.
///
/// Generic (not `dyn`) to stay consistent with
/// [`Prompter`](crate::cli::prompter::Prompter); handlers take it by type
/// parameter.
pub trait OutputSink {
    /// Emits one line of output.
    fn line(&self, line: OutputLine);

    // --- Convenience helpers ------------------------------------------------

    fn plain(&self, text: impl Into<String>) {
        self.line(OutputLine::new(OutputLevel::Plain, text));
    }
    fn info(&self, text: impl Into<String>) {
        self.line(OutputLine::new(OutputLevel::Info, text));
    }
    fn success(&self, text: impl Into<String>) {
        self.line(OutputLine::new(OutputLevel::Success, text));
    }
    fn warn(&self, text: impl Into<String>) {
        self.line(OutputLine::new(OutputLevel::Warn, text));
    }
    fn error(&self, text: impl Into<String>) {
        self.line(OutputLine::new(OutputLevel::Error, text));
    }
}

/// [`OutputSink`] that prints straight to stdout/stderr with styling. Used by
/// the CLI.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdoutSink;

impl OutputSink for StdoutSink {
    fn line(&self, line: OutputLine) {
        let styled = match line.level {
            OutputLevel::Plain => Style::new(),
            OutputLevel::Info => Style::new().cyan(),
            OutputLevel::Success => Style::new().green().bold(),
            OutputLevel::Warn => Style::new().yellow().bold(),
            OutputLevel::Error => Style::new().red().bold(),
        };

        match line.level {
            OutputLevel::Warn | OutputLevel::Error => {
                eprintln!("{}", styled.apply_to(&line.text))
            }
            _ => println!("{}", styled.apply_to(&line.text)),
        }
    }
}

/// Forward `OutputSink` through shared references so `&S` satisfies the bound.
impl<T: OutputSink + ?Sized> OutputSink for &T {
    fn line(&self, line: OutputLine) {
        (**self).line(line);
    }
}
