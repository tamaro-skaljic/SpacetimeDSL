//! A hook on a table whose accessor contains digits and underscores.
//!
//! `#[spacetimedsl::hook]` derives the trait to implement from the hook function's name,
//! and the generator declares that trait from the table. The hook only compiles when both
//! arrive at the same name, which is least obvious where the PascalCase conversion meets
//! digits and underscores.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(
    plural_name = hook_tables_2_b,
    method(update = false),
    hook(before(insert))
)]
#[spacetimedb::table(accessor = hook_table_2_b)]
pub struct HookTable2B {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    value: u32,
}

#[spacetimedsl::hook]
fn before_hook_table_2_b_insert(
    _dsl: &DSL<'_, T>,
    mut create_request: CreateHookTable2B,
) -> Result<CreateHookTable2B, SpacetimeDSLError> {
    create_request.value += 1;

    Ok(create_request)
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let row = dsl
        .create_hook_table_2_b(CreateHookTable2B { value: 7 })
        .map_err(|error| format!("Creating a row should succeed: {error}"))?;

    if *row.get_value() != 8 {
        return Err(format!(
            "The before_insert hook of `hook_table_2_b` should have run and incremented the value to 8, found {}!",
            row.get_value()
        ));
    }

    Ok(())
}
