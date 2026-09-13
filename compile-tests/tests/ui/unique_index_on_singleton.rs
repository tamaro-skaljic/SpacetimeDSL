//! A singleton holds exactly one row, so every one of its columns is trivially unique
//! and a declared unique index would generate a second way to fetch that same row.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(
        singleton,
        method(update = true),
        unique_index(name = world_name_and_seed),
    )]
    #[spacetimedb::table(
        accessor = configuration,
        index(accessor = world_name_and_seed, btree(columns = [world_name, seed])),
        public,
    )]
    pub struct Configuration {
        pub world_name: String,

        pub seed: u64,
    }

    #[spacetimedsl::dsl(
        singleton,
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = another_configuration,
        public,
    )]
    pub struct AnotherConfiguration {
        #[unique]
        pub world_name: String,

        pub seed: u64,
    }

    #[spacetimedsl::dsl(
        singleton,
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = yet_another_configuration,
        public,
    )]
    pub struct YetAnotherConfiguration {
        pub world_name: String,

        #[index(btree)]
        pub seed: u64,
    }
}

fn main() {}
