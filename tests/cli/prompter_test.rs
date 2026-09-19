//! Tests for the generic `Prompter` abstraction.
//!
//! The key property of the generic (vs. `dyn`) design is that `select` returns
//! the caller's own type, not a `String` to be re-parsed. These exercise that
//! through the `ScriptedPrompter` double.

use crate::support::ScriptedPrompter;
use rcloud::cli::prompter::{Prompter, TextOptions};
use rcloud::HookExecType;

#[derive(Debug, Clone, PartialEq)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

impl std::fmt::Display for Fruit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fruit::Apple => write!(f, "Apple"),
            Fruit::Banana => write!(f, "Banana"),
            Fruit::Cherry => write!(f, "Cherry"),
        }
    }
}

#[test]
fn select_returns_the_typed_value_not_a_string() -> anyhow::Result<()> {
    let prompter = ScriptedPrompter::new().with_select_index(1);
    let chosen: Fruit = prompter.select("Pick:", vec![Fruit::Apple, Fruit::Banana, Fruit::Cherry])?;
    // Comes back as Fruit, no string round-trip.
    assert_eq!(chosen, Fruit::Banana);
    Ok(())
}

#[test]
fn select_works_over_domain_enums() -> anyhow::Result<()> {
    let prompter = ScriptedPrompter::new().with_select_index(1);
    let dir: HookExecType =
        prompter.select("Direction:", vec![HookExecType::Push, HookExecType::Pull])?;
    assert_eq!(dir, HookExecType::Pull);
    Ok(())
}

#[test]
fn select_out_of_range_errors() {
    let prompter = ScriptedPrompter::new().with_select_index(9);
    let result = prompter.select("Pick:", vec![Fruit::Apple]);
    assert!(result.is_err());
}

#[test]
fn multi_select_returns_the_checked_typed_values() -> anyhow::Result<()> {
    let mut prompter = ScriptedPrompter::new();
    prompter.multi_indices = vec![0, 2];
    let chosen: Vec<Fruit> = prompter.multi_select(
        "Pick many:",
        vec![Fruit::Apple, Fruit::Banana, Fruit::Cherry],
        &[],
    )?;
    assert_eq!(chosen, vec![Fruit::Apple, Fruit::Cherry]);
    Ok(())
}

#[test]
fn text_and_confirm_are_scripted_in_order() -> anyhow::Result<()> {
    let prompter = ScriptedPrompter::new()
        .with_texts(["first", "second"])
        .with_confirms([true, false]);

    assert_eq!(prompter.text("a", TextOptions::new())?, "first");
    assert_eq!(prompter.text("b", TextOptions::new())?, "second");
    assert!(prompter.confirm("c", false)?);
    assert!(!prompter.confirm("d", true)?);
    Ok(())
}

#[test]
fn text_exhausted_errors() {
    let prompter = ScriptedPrompter::new();
    assert!(prompter.text("x", TextOptions::new()).is_err());
}
