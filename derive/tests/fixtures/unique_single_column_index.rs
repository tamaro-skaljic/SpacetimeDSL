//! Covers `#[unique]` on a single column: the `SpacetimeDSLColumnMethods::ForUniqueIndex`
//! branch, which drives the single-row `get_one_option` and `delete_one`.
//!
//! `method(update = true)` is on, and the snapshots pin where the update method lands:
//! on the primary key only. A `#[unique]` column which is not the primary key gets a
//! getter and a deleter but no `update_..._by_...`, because SpacetimeDB's `update` lives
//! on the primary key index and no other index implements `PrimaryKey`.

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
