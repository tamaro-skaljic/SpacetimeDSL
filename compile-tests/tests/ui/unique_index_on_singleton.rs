//! A singleton holds exactly one row, so every one of its columns is trivially unique
//! and a declared unique index would generate a second way to fetch that same row.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(
        singleton,
        method(update = true),
        unique_index(name = world_name_and_seed),
    )]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        pub world_name: String,

        pub seed: u64,
    }
}

fn main() {}
