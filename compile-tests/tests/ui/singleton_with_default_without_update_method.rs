//! `upsert_<table>` is the only method which writes the row of a table with a default, so a
//! table which disables the update method could never hold a row at all.

::spacetimedsl::spacetimedsl!();

pub mod world_settings {
    #[spacetimedsl::dsl(
        singleton(with_default),
        method(update = false),
    )]
    #[spacetimedb::table(
        accessor = world_settings,
        public,
    )]
    pub struct WorldSettings {
        world_name: String,
    }
}

fn main() {}
