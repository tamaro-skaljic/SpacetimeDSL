use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = parent_records, method(update = false, delete = true))]
#[spacetimedb::table(accessor = parent_record)]
pub struct ParentRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ParentRecordId)]
    #[referenced_by(
        path = crate::primary_key_foreign_key_cascade_test,
        table = child_marker
    )]
    id: u64,
}

#[spacetimedsl::dsl(
    plural_name = child_markers,
    method(update = true, delete = true),
    hook(before(insert, update, delete), after(insert, update, delete))
)]
#[spacetimedb::table(accessor = child_marker)]
pub struct ChildMarker {
    #[primary_key]
    #[use_wrapper(ParentRecordId)]
    #[foreign_key(
        path = crate::primary_key_foreign_key_cascade_test,
        table = parent_record,
        column = id,
        on_delete = Delete
    )]
    parent_id: u64,
}

/// One row per hook call on `child_marker`, naming the hook, in the order the hooks ran.
#[spacetimedsl::dsl(
    plural_name = primary_key_foreign_key_cascade_hook_calls,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = primary_key_foreign_key_cascade_hook_call)]
pub struct PrimaryKeyForeignKeyCascadeHookCall {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    hook_name: String,
}

#[spacetimedsl::hook]
fn before_child_marker_insert(
    dsl: &DSL<'_, T>,
    row: CreateChildMarker,
) -> Result<CreateChildMarker, SpacetimeDSLError> {
    log_hook_call(dsl, "before_child_marker_insert")?;

    Ok(row)
}

#[spacetimedsl::hook]
fn after_child_marker_insert(
    dsl: &DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    log_hook_call(dsl, "after_child_marker_insert")?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_child_marker_update(
    dsl: &DSL<'_, T>,
    _old: &ChildMarker,
    new: ChildMarker,
) -> Result<ChildMarker, SpacetimeDSLError> {
    log_hook_call(dsl, "before_child_marker_update")?;

    Ok(new)
}

#[spacetimedsl::hook]
fn after_child_marker_update(
    dsl: &DSL<'_, T>,
    _old: &ChildMarker,
    _new: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    log_hook_call(dsl, "after_child_marker_update")?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_child_marker_delete(
    dsl: &DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    log_hook_call(dsl, "before_child_marker_delete")?;

    Ok(())
}

#[spacetimedsl::hook]
fn after_child_marker_delete(
    dsl: &DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    log_hook_call(dsl, "after_child_marker_delete")?;

    Ok(())
}

fn log_hook_call<T: WriteContext>(
    dsl: &DSL<'_, T>,
    hook_name: &str,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_primary_key_foreign_key_cascade_hook_call(
        CreatePrimaryKeyForeignKeyCascadeHookCall {
            hook_name: hook_name.to_string(),
        },
    )?;

    Ok(())
}

/// The names of the hooks called so far, in the order they ran.
fn hook_calls<T: WriteContext>(dsl: &DSL<'_, T>) -> Vec<String> {
    let mut hook_calls: Vec<PrimaryKeyForeignKeyCascadeHookCall> = dsl
        .get_all_primary_key_foreign_key_cascade_hook_calls()
        .collect();
    hook_calls.sort_by_key(|hook_call| hook_call.get_id().value());

    hook_calls
        .into_iter()
        .map(|hook_call| hook_call.get_hook_name().to_string())
        .collect()
}

/// A `ChildMarker` whose primary key is the foreign key to its `ParentRecord` is deleted
/// together with it through `on_delete = Delete`, and its delete hooks run around that.
pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let parent_record = dsl.create_parent_record()?;
    let child_marker = dsl.create_child_marker(CreateChildMarker {
        parent_id: parent_record.get_id(),
    })?;
    let logged_before = hook_calls(dsl).len();

    dsl.delete_parent_record_by_id(&parent_record)?;

    if dsl
        .get_child_marker_by_parent_id(child_marker.get_parent_id())
        .is_ok()
    {
        return Err(
            "Deleting a ParentRecord should delete the ChildMarker whose primary key references it, through on_delete = Delete!"
                .to_string(),
        );
    }

    let hook_calls = hook_calls(dsl).split_off(logged_before);
    let expected_hook_calls = vec![
        "before_child_marker_delete".to_string(),
        "after_child_marker_delete".to_string(),
    ];
    if hook_calls.ne(&expected_hook_calls) {
        return Err(format!(
            "Deleting a ParentRecord should run the delete hooks of the ChildMarker it deletes, and no others!\n\nExpected:\n{expected_hook_calls:?}\n\nActual:\n{hook_calls:?}"
        ));
    }

    Ok(())
}
