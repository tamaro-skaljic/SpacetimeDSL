//! `upsert_<table>` is the only method which writes the row of a table with a default, so a
//! table which disables the update method could never hold a row at all.
//!
//! The `DefaultSingleton` implementation is here for the fix: once `update` is enabled,
//! `get_world_settings` falls back to it. Until then a rejected `#[dsl]` emits nothing
//! else, so the implementation misses the `WorldSettings` struct, which is where the
//! errors after the first come from.

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

    impl crate::spacetimedsl::DefaultSingleton for WorldSettings {
        fn get_default(
            _dsl: &crate::spacetimedsl::ReadOnlyDSL<'_, impl crate::spacetimedsl::ReadContext>,
        ) -> Result<WorldSettings, crate::spacetimedsl::SpacetimeDSLError> {
            Ok(WorldSettings {
                id: 0,
                world_name: "Default World".to_string(),
            })
        }
    }
}

fn main() {}
