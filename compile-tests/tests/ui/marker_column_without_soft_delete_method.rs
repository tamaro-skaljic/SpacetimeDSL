//! A column named `deleted` claims the soft-delete marker role. On a table which never
//! soft-deletes, nothing would ever write it, so the name would promise something the
//! generated code does not do.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, delete = true),
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
