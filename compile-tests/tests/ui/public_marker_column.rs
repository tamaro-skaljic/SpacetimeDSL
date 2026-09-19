//! `soft_delete_<table>_by_<index>` is the only writer of the marker column, so the
//! column gets a getter but no setter. A public column would hand callers a second way
//! to retire a row, one which runs no hook and cascades to nothing.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,

        pub deleted: bool,
    }
}

fn main() {}
