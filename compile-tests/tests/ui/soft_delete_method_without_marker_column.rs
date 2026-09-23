//! A soft deletion retires a row by writing one column. Without a column claiming that
//! role there is nothing to write, so the table could declare the method but never
//! perform it.

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
    }
}

fn main() {}
