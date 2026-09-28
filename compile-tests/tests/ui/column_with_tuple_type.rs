//! SpacetimeDSL supports only path types as column types, so a column whose type is
//! a tuple gets a diagnostic.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(update = true))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub value: (u8, u8),
    }
}

fn main() {}
