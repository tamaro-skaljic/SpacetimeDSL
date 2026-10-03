//! A `singleton(with_default)` table has no create method: `upsert_settings` writes its row,
//! and `DefaultSingleton::get_default` supplies the whole default row.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so the
//! `impl DefaultSingleton` names a `Settings` which does not exist.

::spacetimedsl::spacetimedsl!();

pub mod settings {
    use crate::spacetimedsl::prelude::*;

    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(accessor = settings, public)]
    pub struct Settings {
        #[creation_default(8)]
        pub maximum_player_count: u32,
    }

    impl DefaultSingleton for Settings {
        fn get_default(
            _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
        ) -> Result<Settings, SpacetimeDSLError> {
            Ok(Settings {
                id: 0,
                maximum_player_count: 8,
            })
        }
    }
}

fn main() {}
