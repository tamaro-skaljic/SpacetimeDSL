//! The name a column claims the marker role with also fixes its type: `deleted` is the
//! flag shape, so it is a `bool`. Any other type would leave the generated code without
//! a value to write.

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

        deleted: u8,
    }
}

fn main() {}
