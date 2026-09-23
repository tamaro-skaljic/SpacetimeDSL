//! A singleton holds exactly one row, so every one of its columns is trivially unique
//! and a `#[unique]` column would generate a second way to fetch that same row.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        #[unique]
        pub world_name: String,
    }
}

fn main() {}
