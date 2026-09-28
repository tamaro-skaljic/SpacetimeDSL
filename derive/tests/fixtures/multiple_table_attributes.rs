//! Covers the explicit table selector: one `#[dsl]` attribute over two `#[table]`
//! attributes, where `table = gadget2` names the one the generated methods are built from.
//!
//! The second `#[table]` must win although it is not the one directly below the `#[dsl]`
//! attribute.

#[spacetimedsl::dsl(plural_name = gadgets2, table = gadget2, method(update = true))]
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
