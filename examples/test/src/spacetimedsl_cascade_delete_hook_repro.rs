use crate::spacetimedsl::SpacetimeDSLError;

#[spacetimedsl::dsl(plural_name = parent_records, method(update = false, delete = true))]
#[spacetimedb::table(accessor = parent_record)]
pub struct ParentRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ParentRecordId)]
    #[referenced_by(
        path = crate::spacetimedsl_cascade_delete_hook_repro,
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
        path = crate::spacetimedsl_cascade_delete_hook_repro,
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
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    Ok(())
}

#[spacetimedsl::hook]
fn after_child_marker_delete(
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    _row: &ChildMarker,
) -> Result<(), SpacetimeDSLError> {
    Ok(())
}
