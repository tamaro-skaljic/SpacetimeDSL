//! SpacetimeDSL supports only path types as column types, so a column whose type is
//! a reference gets a diagnostic.
//!
//! The error after the first is SpacetimeDB's own: a rejected `#[dsl]` emits its struct, and
//! SpacetimeDB refuses the borrowed column as well.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(update = true))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub value: &'static str,
    }
}

fn main() {}
