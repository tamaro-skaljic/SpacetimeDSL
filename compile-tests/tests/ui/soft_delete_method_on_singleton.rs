//! A singleton holds one row which the DSL reaches through an injected primary key.
//! Retiring that row would leave the table holding a row no method can reach, so a
//! singleton is never soft-deletable.

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
