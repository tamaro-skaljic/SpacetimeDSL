//! `#[dsl(singleton(with_default))]` makes `get_<table>` fall back to
//! `DefaultSingleton::get_default`, so the table's struct has to implement that trait.
//!
//! Forgetting the implementation is the most likely mistake with this feature, which is why
//! the message rustc gives for it is pinned here rather than dressed up in a diagnostic of
//! our own.

::spacetimedsl::spacetimedsl!();

pub mod world_settings {
    #[spacetimedsl::dsl(
        singleton(with_default),
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = world_settings,
        public,
    )]
    pub struct WorldSettings {
        pub world_name: String,
    }
}

fn main() {}
