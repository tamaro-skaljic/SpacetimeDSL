//! Covers the heuristic table selector at `internal/integration.rs:66`: one `#[dsl]`
//! attribute over two `#[table]` attributes, where the `plural_name` decides which of
//! them the generated methods are built from.
//!
//! `gadgets2` matches `gadget2`, so the second `#[table]` must win although it is not the
//! one directly below the `#[dsl]` attribute.

#[spacetimedsl::dsl(plural_name = gadgets2, method(update = true))]
#[spacetimedb::table(
    accessor = gadget1,
    index(accessor = owner, btree(columns = [owner_id])),
    public,
)]
#[spacetimedb::table(accessor = gadget2, public)]
pub struct Gadget {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub owner_id: u64,
}
