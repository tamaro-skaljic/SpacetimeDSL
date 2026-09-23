//! `#[set_on_soft_delete]` claims the marker role for a column whose name does not, and
//! names no shape of its own. The column's type picks the shape, so a type which fits
//! neither leaves the attribute with nothing to write.

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

        #[set_on_soft_delete]
        retired: u8,
    }
}

fn main() {}
