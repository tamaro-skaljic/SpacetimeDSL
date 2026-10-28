//! SpacetimeDSL supports only path types as column types, so a column whose type is
//! an array gets a diagnostic.
//!
//! The errors after the first are SpacetimeDB's own: a rejected `#[dsl]` emits its struct,
//! and SpacetimeDB refuses the column's type as well.
//!
//! Maintenance note: these errors quote source lines of SpacetimeDB's `spacetimedb-sats`
//! crate, so this file churns on SpacetimeDB upgrades, and regenerating it with
//! `TRYBUILD=overwrite` is part of the upgrade routine.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(update = true))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub value: [u8; 4],
    }
}

fn main() {}
