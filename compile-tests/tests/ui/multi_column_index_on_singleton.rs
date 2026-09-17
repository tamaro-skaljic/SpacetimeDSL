//! A singleton holds exactly one row, so an index over several of its columns is as
//! pointless as an index over one of them.
//!
//! `unique_index_on_singleton.rs` pins the column-attribute spellings, which
//! `SpacetimeDBColumn::map` rejects. This pins the table-attribute spelling, which
//! `SpacetimeDBTable::map` rejects first, before any column is looked at -- so the
//! generator never reaches a singleton carrying a multi-column index.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(
        singleton,
        method(update = true),
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
