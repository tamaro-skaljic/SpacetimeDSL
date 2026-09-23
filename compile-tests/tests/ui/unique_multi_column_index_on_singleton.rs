//! A singleton holds exactly one row, so a unique multi-column index would be a second
//! way to fetch that same row. It takes two declarations - the table-level index and the
//! `unique_index` naming it - and removing only `unique_index` would leave the index that
//! `multi_column_index_on_singleton.rs` pins, so the message names both.

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
}

fn main() {}
