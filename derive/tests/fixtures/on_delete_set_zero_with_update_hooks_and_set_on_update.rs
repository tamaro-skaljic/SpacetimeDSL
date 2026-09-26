//! Covers `on_delete = SetZero` on a referencing table with `before_update` and
//! `after_update` hooks and a `#[set_on_update]` column. Clearing the column is an update of
//! the row, so the cascade runs the update hooks around the write and sets the timestamp
//! after the before hook, as `update_<table>_by_<key>` does. It carries both, because the
//! `mut` rebinding between the before hook and the timestamp only appears when both are
//! present.
//!
//! Identical to `on_delete_set_zero` apart from the hooks and the timestamp column, so
//! diffing their snapshots shows exactly what those two add.

#[spacetimedsl::dsl(plural_name = authors, method(update = true))]
#[spacetimedb::table(accessor = author, public)]
pub struct Author {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(AuthorId)]
    #[referenced_by(path = self, table = book)]
    id: u64,

    pub name: String,
}

#[spacetimedsl::dsl(
    plural_name = books,
    method(update = true),
    hook(before(update), after(update)),
)]
#[spacetimedb::table(accessor = book, public)]
pub struct Book {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(AuthorId)]
    #[foreign_key(path = self, table = author, column = id, on_delete = SetZero)]
    pub author_id: u64,

    #[set_on_update]
    modified_at: Option<spacetimedb::Timestamp>,
}
