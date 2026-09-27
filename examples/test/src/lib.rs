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

#[spacetimedb::reducer]
fn tester(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    let test_groups: &[(&str, TestGroup)] = &[
        ("entity", entity::run_tests),
        ("timestamp_helper", timestamp_helper_test::run_tests),
        ("component_identifier", component::identifier::run_tests),
        ("component_position", component::position::run_tests),
        ("component_test", component::test::run_tests),
        ("component_uuid", component::uuid_test::run_tests),
        (
            "component_uuid_reference",
            component::uuid_reference_test::run_tests,
        ),
        ("component_hooks", component::hook_test::run_tests),
        ("singleton", singleton_test::run_tests),
        (
            "singleton_with_default",
            singleton_with_default_test::run_tests,
        ),
        (
            "singleton_with_foreign_key",
            singleton_with_foreign_key_test::run_tests,
        ),
        ("hash_index", hash_index_test::run_tests),
        ("cascade_hook_error", cascade_hook_error_test::run_tests),
        (
            "primary_key_foreign_key_cascade",
            primary_key_foreign_key_cascade_test::run_tests,
        ),
        ("soft_deletion", soft_deletion::run_tests),
        (
            "update_and_soft_delete_hooks",
            update_and_soft_delete_hook_test::run_tests,
        ),
        (
            "self_referencing_cascade",
            self_referencing_cascade_test::run_tests,
        ),
    ];

    let failures: Vec<String> = test_groups
        .iter()
        .filter_map(|(group_name, run_tests)| {
            run_tests(&dsl)
                .err()
                .map(|error| format!("{group_name}: {error}"))
        })
        .collect();

    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }

    info!("Test executed successfully!");
    Ok(())
}
