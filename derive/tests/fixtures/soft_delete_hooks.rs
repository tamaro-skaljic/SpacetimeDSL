//! Covers both soft-delete hooks. They take the shape of the update hooks, because a soft
//! deletion writes the row rather than removing it, and they run before the marker is
//! written.

#[spacetimedsl::dsl(
    plural_name = tickets,
    method(update = true, delete = true, soft_delete = true),
    hook(before(soft_delete), after(soft_delete)),
)]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub title: String,

    deleted_at: Option<spacetimedb::Timestamp>,
}
