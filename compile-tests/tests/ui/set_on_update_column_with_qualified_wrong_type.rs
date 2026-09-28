//! A qualified `Option` is still an `Option`, so `std::option::Option<u64>` is rejected
//! for the `set_on_update` role the same way `Option<u64>` is.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(update = true))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[set_on_update]
        modified_at: std::option::Option<u64>,
    }
}

fn main() {}
