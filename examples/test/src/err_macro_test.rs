//! `err!`, the shortcut for `Err(SpacetimeDSLError::Error(…))`, reached through the prelude
//! like every other runtime item.

use crate::spacetimedsl::prelude::*;

const MAXIMUM_LEVEL: u8 = 10;

pub(crate) fn run_tests<T: WriteContext>(_dsl: &DSL<'_, T>) -> Result<(), String> {
    let level = 12;

    expect_message(
        err!("Level {level} is above {}", MAXIMUM_LEVEL),
        "Level 12 is above 10",
    )?;

    expect_message(
        err!(
            "Level {} is above {maximum}",
            level,
            maximum = MAXIMUM_LEVEL
        ),
        "Level 12 is above 10",
    )?;

    expect_message(err!("No level given"), "No level given")?;

    let message = format!("Level {level} is too high");
    expect_message(err!(message), "Level 12 is too high")?;

    expect_message(err!(MAXIMUM_LEVEL), "10")?;

    // A literal followed by anything but its format arguments is an expression like any other.
    expect_message(err!("No level given".to_string()), "No level given")?;

    Ok(())
}

/// Checks that `result` is the error `err!` builds, with `expected_message` as its message.
fn expect_message(
    result: Result<(), SpacetimeDSLError>,
    expected_message: &str,
) -> Result<(), String> {
    match result {
        Err(SpacetimeDSLError::Error(message)) if message == expected_message => Ok(()),
        other => Err(format!(
            "err! should give Err(SpacetimeDSLError::Error({expected_message:?}))! Got: {other:?}"
        )),
    }
}
