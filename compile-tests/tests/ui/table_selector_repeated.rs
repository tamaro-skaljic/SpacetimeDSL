//! A `#[dsl]` belongs to exactly one `#[table]`, so `table = <accessor>` may appear once.

::spacetimedsl::spacetimedsl!();

pub mod gadget {
    #[spacetimedsl::dsl(
        plural_name = gadgets,
        table = gadget1,
        table = gadget2,
        method(update = false)
    )]
    #[spacetimedb::table(accessor = gadget1, public)]
    #[spacetimedb::table(accessor = gadget2, public)]
    pub struct Gadget {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
