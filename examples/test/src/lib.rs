::spacetimedsl::spacetimedsl!();

use crate::spacetimedsl::{DSL, Wrapper, dsl};
use log::info;
use spacetimedb::ReducerContext;

pub mod cascade_hook_error_test;
pub mod component;
pub mod entity;
pub mod hand_written_wrapper_test;
pub mod hash_index_test;
pub mod hook_trait_name_test;
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
    ("crate_root_wrapper_test", crate_root_wrapper_test),
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
    (
        "hand_written_wrapper_test",
        hand_written_wrapper_test::run_tests,
    ),
    ("hash_index_test", hash_index_test::run_tests),
    ("hook_trait_name_test", hook_trait_name_test::run_tests),
    (
        "primary_key_foreign_key_cascade_test",
        primary_key_foreign_key_cascade_test::run_tests,
    ),
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

/// A table declared at the crate root, next to `spacetimedsl!()`, where `spacetimedsl` names
/// both the runtime crate and the module that macro generates. Generated code has to reach the
/// runtime without naming it ambiguously.
#[::spacetimedsl::dsl(plural_name = crate_root_things, method(update = false))]
#[spacetimedb::table(accessor = crate_root_thing)]
pub struct CrateRootThing {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
}

fn crate_root_wrapper_test(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let thing = dsl.create_crate_root_thing()?;

    let found_thing = dsl
        .get_crate_root_thing_by_id(CrateRootThingId::new(thing.get_id().value()))
        .map_err(|error| {
            format!("Should find a CrateRootThing by the value its wrapper holds! Got:\n{error}")
        })?;

    if found_thing.get_id().ne(&thing.get_id()) {
        return Err(
            "A CrateRootThingId built from the value of another should equal it!".to_string(),
        );
    }

    Ok(())
}
