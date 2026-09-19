//! Environment-agnostic prompting.
//!
//! Interactive handlers need to ask the user for input, but *how* input is
//! gathered depends on the frontend: the CLI uses `inquire` (which needs the
//! terminal in cooked mode), while the TUI must gather input inside its own raw
//! mode without leaving the alternate screen.
//!
//! [`Prompter`] abstracts that. Handlers take a generic `P: Prompter` and ask
//! it for text/confirm/select/multi-select values; the concrete prompter
//! decides how to obtain them. The CLI passes [`InquirePrompter`]; the TUI
//! passes its own ratatui-based prompter. One set of handlers, two frontends,
//! no cooked/raw dance.
//!
//! The trait is intentionally *generic* (not `dyn`): `select`/`multi_select`
//! are generic over the option type so selecting returns the caller's own type
//! (a `Provider`, a `HookExecType`, ...) instead of a `String` that must be
//! mapped back. That rules out `Box<dyn Prompter>`, so the prompter travels as
//! a type parameter through the call chain.

use anyhow::Context;

/// Optional presentation hints for a text prompt.
#[derive(Default, Clone)]
pub struct TextOptions<'a> {
    /// Prefilled default value.
    pub default: Option<&'a str>,
    /// Help line shown under the prompt.
    pub help: Option<&'a str>,
    /// Require a non-empty answer.
    pub required: bool,
}

impl<'a> TextOptions<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn default_value(mut self, value: &'a str) -> Self {
        self.default = Some(value);
        self
    }
    pub fn help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Gathers user input for the interactive commands.
///
/// See the [module docs](self) for why this is generic rather than `dyn`.
pub trait Prompter {
    /// Free-form text input.
    fn text(&self, message: &str, options: TextOptions) -> anyhow::Result<String>;

    /// Yes/no confirmation with a default.
    fn confirm(&self, message: &str, default: bool) -> anyhow::Result<bool>;

    /// Pick exactly one option, returning it by value.
    fn select<T>(&self, message: &str, options: Vec<T>) -> anyhow::Result<T>
    where
        T: std::fmt::Display;

    /// Pick zero or more options. `defaults` are the indices preselected.
    fn multi_select<T>(
        &self,
        message: &str,
        options: Vec<T>,
        defaults: &[usize],
    ) -> anyhow::Result<Vec<T>>
    where
        T: std::fmt::Display;

    /// Masked password input (no echo).
    fn password(&self, message: &str) -> anyhow::Result<String>;
}

/// [`Prompter`] backed by `inquire` (terminal, cooked mode). Used by the CLI.
#[derive(Debug, Clone, Copy, Default)]
pub struct InquirePrompter;

/// Forward `Prompter` through shared references so `&P` (as threaded by the TUI
/// dispatch) satisfies the same bound as `P`.
impl<T: Prompter + ?Sized> Prompter for &T {
    fn text(&self, message: &str, options: TextOptions) -> anyhow::Result<String> {
        (**self).text(message, options)
    }
    fn confirm(&self, message: &str, default: bool) -> anyhow::Result<bool> {
        (**self).confirm(message, default)
    }
    fn select<U>(&self, message: &str, options: Vec<U>) -> anyhow::Result<U>
    where
        U: std::fmt::Display,
    {
        (**self).select(message, options)
    }
    fn multi_select<U>(
        &self,
        message: &str,
        options: Vec<U>,
        defaults: &[usize],
    ) -> anyhow::Result<Vec<U>>
    where
        U: std::fmt::Display,
    {
        (**self).multi_select(message, options, defaults)
    }
    fn password(&self, message: &str) -> anyhow::Result<String> {
        (**self).password(message)
    }
}

impl Prompter for InquirePrompter {
    fn text(&self, message: &str, options: TextOptions) -> anyhow::Result<String> {
        let mut prompt = inquire::Text::new(message);
        if let Some(default) = options.default {
            prompt = prompt.with_default(default);
        }
        if let Some(help) = options.help {
            prompt = prompt.with_help_message(help);
        }
        if options.required {
            prompt =
                prompt.with_validator(inquire::validator::MinLengthValidator::new(1));
        }
        prompt.prompt().context("failed to read text input")
    }

    fn confirm(&self, message: &str, default: bool) -> anyhow::Result<bool> {
        inquire::Confirm::new(message)
            .with_default(default)
            .prompt()
            .context("failed to read confirmation")
    }

    fn select<T>(&self, message: &str, options: Vec<T>) -> anyhow::Result<T>
    where
        T: std::fmt::Display,
    {
        inquire::Select::new(message, options)
            .with_vim_mode(true)
            .with_page_size(10)
            .prompt()
            .context("failed to read selection")
    }

    fn multi_select<T>(
        &self,
        message: &str,
        options: Vec<T>,
        defaults: &[usize],
    ) -> anyhow::Result<Vec<T>>
    where
        T: std::fmt::Display,
    {
        inquire::MultiSelect::new(message, options)
            .with_vim_mode(true)
            .with_default(defaults)
            .prompt()
            .context("failed to read multi-selection")
    }

    fn password(&self, message: &str) -> anyhow::Result<String> {
        inquire::Password::new(message)
            .without_confirmation()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .with_validator(inquire::validator::MinLengthValidator::new(1))
            .prompt()
            .context("failed to read password")
    }
}
