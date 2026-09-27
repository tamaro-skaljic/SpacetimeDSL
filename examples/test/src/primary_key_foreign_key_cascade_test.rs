use crate::spacetimedsl::{DSL, SpacetimeDSLError, WriteContext};

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

    description: String,
}

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

#[spacetimedsl::hook]
fn before_child_marker_insert(
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    row: CreateChildMarker,
) -> Result<CreateChildMarker, SpacetimeDSLError> {
    Ok(row)
}

#[spacetimedsl::hook]
fn after_child_marker_insert(
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    Ok(())
}

#[spacetimedsl::hook]
fn before_child_marker_update(
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    _old: &ChildMarker,
    new: ChildMarker,
) -> Result<ChildMarker, SpacetimeDSLError> {
    Ok(new)
}

#[spacetimedsl::hook]
fn after_child_marker_update(
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    _old: &ChildMarker,
    _new: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    Ok(())
}

#[spacetimedsl::hook]
fn before_child_marker_delete(
    dsl: &crate::spacetimedsl::DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_primary_key_foreign_key_cascade_hook_call(
        CreatePrimaryKeyForeignKeyCascadeHookCall {
            description: "before_child_marker_delete".to_string(),
        },
    )?;
    Ok(())
}

#[spacetimedsl::hook]
fn after_child_marker_delete(
    dsl: &crate::spacetimedsl::DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_primary_key_foreign_key_cascade_hook_call(
        CreatePrimaryKeyForeignKeyCascadeHookCall {
            description: "after_child_marker_delete".to_string(),
        },
    )?;
    Ok(())
}

pub fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let parent = dsl.create_parent_record()?;
    dsl.create_child_marker(CreateChildMarker {
        parent_id: parent.get_id(),
    })?;

    let logged_before = dsl
        .get_all_primary_key_foreign_key_cascade_hook_calls()
        .count();
    dsl.delete_parent_record_by_id(&parent)?;

    if dsl.get_child_marker_by_parent_id(&parent).is_ok() {
        return Err("Deleting a parent should delete its child marker".to_string());
    }

    let hook_calls: Vec<String> = dsl
        .get_all_primary_key_foreign_key_cascade_hook_calls()
        .skip(logged_before)
        .map(|call| call.get_description().to_string())
        .collect();
    let expected_hook_calls = vec![
        "before_child_marker_delete".to_string(),
        "after_child_marker_delete".to_string(),
    ];
    if hook_calls != expected_hook_calls {
        return Err(format!(
            "Deleting a parent should run the child delete hooks in order. Expected {expected_hook_calls:?}, got {hook_calls:?}"
        ));
    }

    Ok(())
}
