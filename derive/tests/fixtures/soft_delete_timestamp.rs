//! Covers a marker claimed by `#[set_on_soft_delete]` rather than by its name, with the
//! `Option<Timestamp>` shape, on a table with a unique and a non-unique index.

#[spacetimedsl::dsl(plural_name = tickets, method(update = true, delete = true, soft_delete = true))]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[create_wrapper]
    pub reference: String,

    #[index(btree)]
    pub priority: u8,

    #[set_on_soft_delete]
    retired_at: Option<spacetimedb::Timestamp>,
}
