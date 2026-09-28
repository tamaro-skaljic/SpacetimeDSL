//! A singleton is its one table: a second `#[table]` attribute would declare a second
//! singleton on the same struct, so a singleton must have exactly one, even with a
//! `table = <accessor>` selector.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = configuration1, public)]
    #[spacetimedb::table(accessor = configuration2, public)]
    pub struct Configuration {
        pub world_name: String,
    }
}

fn main() {}
