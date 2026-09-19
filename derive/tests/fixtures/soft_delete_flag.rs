//! Covers a soft-deletable table whose marker is a `bool` claimed by its name.
//!
//! The marker is private, so it earns a getter and no setter, and the create method fills
//! it in rather than asking for it.

#[spacetimedsl::dsl(plural_name = tickets, method(update = true, delete = true, soft_delete = true))]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub title: String,

    #[index(btree)]
    pub priority: u8,

    deleted: bool,
}
