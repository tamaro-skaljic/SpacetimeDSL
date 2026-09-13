//! Covers the base shape every other fixture varies: a single `#[primary_key]`
//! `#[auto_inc]` column, no indices, no hooks and no foreign keys.
//!
//! Drives `create`, `get_all`, `get_count` and the primary key's
//! `get_one_option` / `delete_one`.

#[spacetimedsl::dsl(plural_name = things, method(update = false))]
#[spacetimedb::table(accessor = thing, public)]
pub struct Thing {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
}
