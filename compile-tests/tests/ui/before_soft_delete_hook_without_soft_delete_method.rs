//! A `before_soft_delete` hook runs when a row is retired. On a table which never retires
//! a row, the trait would be emitted and never used, and the developer would wait for a
//! call that cannot come.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, delete = true),
        hook(before(soft_delete)),
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
