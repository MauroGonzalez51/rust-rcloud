// Integration tests, organized to mirror `src/`.
//
// - `hooks`, `config`, `utils`: mirror the matching `src/` modules, one test
//   file per source file, exercising it in isolation through the public API.
// - `scenarios`: end-to-end tests that cross several modules (full sync
//   pipelines, regressions) and do not map to a single source file.
//
// Tests that need access to private items live inline in `src/` as
// `#[cfg(test)] mod tests` instead.

pub mod config;
pub mod hooks;
pub mod scenarios;
pub mod sync;
pub mod utils;
