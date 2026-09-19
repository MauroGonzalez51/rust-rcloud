// Public-API hook tests.
//
// Only hooks with headless, public behavior appear here. Encryption and
// backup are tested elsewhere:
// - private logic (encrypt/decrypt, compress, replica parsing/rotation) lives
//   inline in the matching `src/hooks/**` files as `#[cfg(test)] mod tests`,
//   since Rust integration tests cannot reach private items;
// - cross-module flows (zip push/pull round-trip) live in `tests/scenarios/`.
// Interactive builders and rclone-backed paths are not covered.

pub mod hook_builder_test;
pub mod zip_test;
