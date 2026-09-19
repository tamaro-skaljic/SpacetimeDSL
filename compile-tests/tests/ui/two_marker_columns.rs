//! A soft deletion writes one column. Two columns claiming the role would leave it
//! undecided which one records that the row was retired, and a reader consulting the
//! other would see a row which is still live.

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

        deleted: bool,

        deleted_at: Option<spacetimedb::Timestamp>,
    }
}

fn main() {}
