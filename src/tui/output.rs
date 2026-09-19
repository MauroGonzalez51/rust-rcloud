//! Buffer-backed [`OutputSink`](crate::cli::output::OutputSink) for the TUI.
//!
//! Accumulates handler output into titled sections (one per executed action)
//! so the side panel can show *what* each block of output came from. The buffer
//! is shared (`Rc<RefCell<..>>`) between the sink handed to handlers and the
//! widget that renders it.

use crate::cli::output::{OutputLine, OutputSink};
use std::cell::RefCell;
use std::rc::Rc;

/// A titled block of output lines produced by one action.
#[derive(Debug, Clone, Default)]
pub struct OutputSection {
    /// Action label, e.g. "Add Remote".
    pub title: String,
    pub lines: Vec<OutputLine>,
}

/// Shared, section-structured output log.
#[derive(Debug, Clone, Default)]
pub struct OutputBuffer {
    pub sections: Vec<OutputSection>,
}

impl OutputBuffer {
    /// Opens a new section titled `title`; subsequent lines append to it.
    pub fn open_section(&mut self, title: impl Into<String>) {
        self.sections.push(OutputSection {
            title: title.into(),
            lines: Vec::new(),
        });
    }

    /// Total line count across all sections (used for scroll bounds).
    pub fn total_lines(&self) -> usize {
        self.sections
            .iter()
            .map(|s| s.lines.len() + 1) // +1 for the title row
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }
}

/// An [`OutputSink`] that appends to a shared [`OutputBuffer`].
///
/// Lines land in the most recently opened section. The dispatcher opens a
/// section (with the action name) before running a handler, so handlers stay
/// unaware of the framing and just emit lines.
#[derive(Clone)]
pub struct BufferSink {
    buffer: Rc<RefCell<OutputBuffer>>,
}

impl BufferSink {
    pub fn new(buffer: Rc<RefCell<OutputBuffer>>) -> Self {
        Self { buffer }
    }
}

impl OutputSink for BufferSink {
    fn line(&self, line: OutputLine) {
        let mut buffer = self.buffer.borrow_mut();

        // Defensive: if no section was opened, start an untitled one so no
        // output is silently dropped.
        if buffer.sections.is_empty() {
            buffer.open_section("");
        }

        let idx = buffer.sections.len() - 1;
        buffer.sections[idx].lines.push(line);
    }
}
