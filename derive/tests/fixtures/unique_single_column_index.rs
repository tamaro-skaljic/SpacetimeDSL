//! Covers `#[unique]` on a single column: the `SpacetimeDSLColumnMethods::ForUniqueIndex`
//! branch, which drives the single-row `get_one_option` and `delete_one`.
//!
//! `method(update = true)` is on, and the snapshots pin where the update method actually
//! lands: on the primary key only. A `#[unique]` column which is not the primary key gets
//! no `update_..._by_...` method - unlike the unique *multi*-column index in
//! `unique_multi_column_index`, which does get one. (FIXME)

#[spacetimedsl::dsl(plural_name = accounts, method(update = true))]
#[spacetimedb::table(accessor = account, public)]
pub struct Account {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    pub external_id: u64,
}
