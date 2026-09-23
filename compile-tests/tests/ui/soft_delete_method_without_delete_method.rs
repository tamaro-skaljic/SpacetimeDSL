//! `method(soft_delete)` decides whether a table retires rows instead of removing them,
//! which only means something next to a statement about whether it removes them at all.
//! Leaving `method(delete)` to its default would hide that decision, so mentioning
//! `soft_delete` makes `delete` mandatory.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,

        deleted: bool,
    }
}

fn main() {}
