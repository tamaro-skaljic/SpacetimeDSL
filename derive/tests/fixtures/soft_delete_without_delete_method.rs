//! Covers a table whose rows can only be retired, never removed: `method(delete = false)`
//! beside `method(soft_delete = true)`. It earns `soft_delete_*` and no `delete_*`.

#[spacetimedsl::dsl(plural_name = tickets, method(update = true, delete = false, soft_delete = true))]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub title: String,

    deleted: bool,
}
