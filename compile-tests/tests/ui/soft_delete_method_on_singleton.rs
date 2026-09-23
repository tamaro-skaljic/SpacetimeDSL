//! A singleton is never soft-deletable: it holds one row which the DSL reaches through
//! an injected primary key, so retiring that row would leave a row no method can reach.
//!
//! The table also lacks a marker column, but asking for one would lead the fix astray:
//! once `soft_delete = true` is gone, no marker column is needed.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(
        singleton,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        pub world_name: String,
    }
}

fn main() {}
