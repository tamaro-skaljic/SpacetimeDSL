//! Regression guard for [#138](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/138)
//! (closed): a table with delete hooks whose `#[foreign_key]` sits on a `#[unique]`
//! column used to generate Rust that does not compile, because the cascade path emitted
//! the hook calls into a scope that did not have the deleted row.
//!
//! The shape matters here, not any single method: delete hooks plus a cascading foreign
//! key plus a unique - not primary key - index on the referencing column.

#[spacetimedsl::dsl(plural_name = parent_records, method(update = false, delete = true))]
#[spacetimedb::table(accessor = parent_record, public)]
pub struct ParentRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ParentRecordId)]
    #[referenced_by(path = self, table = child_marker)]
    id: u64,
}

#[spacetimedsl::dsl(
    plural_name = child_markers,
    method(update = true, delete = true),
    hook(
        before(insert, update, delete),
        after(insert, update, delete),
    ),
)]
#[spacetimedb::table(accessor = child_marker, public)]
pub struct ChildMarker {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[use_wrapper(ParentRecordId)]
    #[foreign_key(path = self, table = parent_record, column = id, on_delete = Delete)]
    pub parent_id: u64,
}
