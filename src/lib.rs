//! # rcloud
//!
//! A CLI wrapper around [rclone](https://rclone.org/) that adds a hook
//! pipeline, tag-based batch operations, and hash-based skip detection on top
//! of cloud storage sync.
//!
//! ## Modules
//! - [`cli`]: argument parsing, command handlers, and the shared
//!   `CommandContext`.
//! - [`config`]: persisted state — `AppConfig`, `Registry`, and the hook
//!   config types.
//! - [`hooks`]: the `Hook` trait and built-in hooks (Zip, Encryption, Backup).
//!   See `docs/HOOKS.md` to add a new one.
//! - [`tui`]: the interactive terminal UI.
//! - [`utils`]: hashing, directories, path expansion, logging.
//!
//! ## Sync model
//! A `PathConfig` maps a local path to a remote path and carries push/pull
//! hook pipelines. On push, hooks run in order before upload; on pull, in
//! reverse order after download. See `docs/ARCHITECTURE.md` for the full flow.

pub mod cli;
pub mod config;
pub mod hooks;
pub mod tui;
pub mod utils;

pub use config::prelude::*;
pub use hooks::prelude::*;
