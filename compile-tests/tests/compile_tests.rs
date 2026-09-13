//! Pins the diagnostics SpacetimeDSL emits for table definitions it rejects, the way the
//! snapshots in `spacetimedsl_derive` pin the code it accepts.
//!
//! Every file in `tests/ui` must fail to compile, and its error output must match the
//! `.stderr` file beside it. Each file is standalone: it invokes
//! `::spacetimedsl::spacetimedsl!()`, defines the table which triggers the diagnostic and
//! ends in an empty `main`.
//!
//! Run with `TRYBUILD=overwrite cargo test -p spacetimedsl_compile_tests` to regenerate
//! the `.stderr` files after a deliberate change to a diagnostic, then read every diff
//! before committing it.
//!
//! The `.stderr` files are only reproducible against the compiler in `rust-toolchain.toml`
//! - that is why it pins an exact version rather than a channel.

#[test]
fn invalid_table_definitions_are_rejected() {
    let test_cases = trybuild::TestCases::new();

    test_cases.compile_fail("tests/ui/*.rs");
}
