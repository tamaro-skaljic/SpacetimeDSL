::spacetimedsl::spacetimedsl!();

use crate::spacetimedsl::{DSL, dsl};
use log::info;
use spacetimedb::ReducerContext;

pub mod cascade_hook_error_test;
pub mod component;
pub mod entity;
pub mod hash_index_test;
pub mod primary_key_foreign_key_cascade_test;
pub mod self_referencing_cascade_test;
pub mod singleton_test;
pub mod singleton_with_default_test;
pub mod singleton_with_foreign_key_test;
pub mod soft_deletion;
pub mod timestamp_helper_test;
pub mod update_and_soft_delete_hook_test;

type TestGroup = fn(&DSL<'_, ReducerContext>) -> Result<(), String>;

/// Every group of runtime tests, in the order `tester` runs them. The groups share one
/// database, so a count a group asserts has to be relative to a count taken before its act.
const TEST_GROUPS: &[(&str, TestGroup)] = &[
    ("entity", entity::run_tests),
    ("timestamp_helper_test", timestamp_helper_test::run_tests),
    ("component::identifier", component::identifier::run_tests),
    ("component::position", component::position::run_tests),
    ("component::test", component::test::run_tests),
    ("component::uuid_test", component::uuid_test::run_tests),
    (
        "component::uuid_reference_test",
        component::uuid_reference_test::run_tests,
    ),
    ("component::hook_test", component::hook_test::run_tests),
    ("singleton_test", singleton_test::run_tests),
    (
        "singleton_with_default_test",
        singleton_with_default_test::run_tests,
    ),
    (
        "singleton_with_foreign_key_test",
        singleton_with_foreign_key_test::run_tests,
    ),
    ("hash_index_test", hash_index_test::run_tests),
    (
        "cascade_hook_error_test",
        cascade_hook_error_test::run_tests,
    ),
    ("soft_deletion", soft_deletion::run_tests),
    (
        "update_and_soft_delete_hook_test",
        update_and_soft_delete_hook_test::run_tests,
    ),
    (
        "self_referencing_cascade_test",
        self_referencing_cascade_test::run_tests,
    ),
];

/// Runs every test group, also after one of them failed, and fails with the messages of all
/// groups that failed. A group that panics still aborts the reducer, and with it every group
/// after it.
#[spacetimedb::reducer]
fn tester(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    let failures: Vec<String> = TEST_GROUPS
        .iter()
        .filter_map(|(group_name, run_tests)| {
            info!("Running the {group_name} tests...");
            run_tests(&dsl)
                .err()
                .map(|message| format!("{group_name}: {message}"))
        })
        .collect();

    if !failures.is_empty() {
        return Err(format!(
            "{} test groups failed:\n{}",
            failures.len(),
            failures.join("\n")
        ));
    }

    info!("Test executed successfully!");
    Ok(())
}
