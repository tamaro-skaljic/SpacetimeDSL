::spacetimedsl::spacetimedsl!();

use crate::spacetimedsl::dsl;
use log::info;
use spacetimedb::ReducerContext;

pub mod cascade_hook_error_test;
pub mod component;
pub mod entity;
pub mod hash_index_test;
pub mod singleton_test;
pub mod singleton_with_default_test;
pub mod singleton_with_foreign_key_test;
pub mod soft_deletion;
pub mod spacetimedsl_cascade_delete_hook_repro;
pub mod timestamp_helper_test;
pub mod update_and_soft_delete_hook_test;

#[spacetimedb::reducer]
fn tester(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    entity::run_tests(&dsl)?;
    timestamp_helper_test::run_tests(&dsl)?;
    component::identifier::run_tests(&dsl)?;
    component::position::run_tests(&dsl)?;
    component::test::run_tests(&dsl)?;
    component::uuid_test::run_tests(&dsl)?;
    component::uuid_reference_test::run_tests(&dsl)?;
    component::hook_test::run_tests(&dsl)?;
    singleton_test::run_tests(&dsl)?;
    singleton_with_default_test::run_tests(&dsl)?;
    singleton_with_foreign_key_test::run_tests(&dsl)?;
    hash_index_test::run_tests(&dsl)?;
    cascade_hook_error_test::run_tests(&dsl)?;
    soft_deletion::run_tests(&dsl)?;
    update_and_soft_delete_hook_test::run_tests(&dsl)?;

    info!("Test executed successfully!");
    Ok(())
}
