//! With several `#[table]` attributes on one struct, `#[dsl]` cannot tell which of them it
//! belongs to, so it has to be named with `table = <accessor>`.

::spacetimedsl::spacetimedsl!();

pub mod gadget {
    #[spacetimedsl::dsl(plural_name = gadgets, method(update = false))]
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
