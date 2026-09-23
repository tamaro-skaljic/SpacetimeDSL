//! A singleton is never soft-deletable: it holds one row which the DSL reaches through
//! an injected primary key, so retiring that row would leave a row no method can reach.
//!
//! With both the flag and a marker column present, removing only the flag would leave a
//! column claiming a role nothing writes, so the message names both.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(
        singleton,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        pub world_name: String,

        deleted: bool,
    }
}

fn main() {}
