//! A singleton gets `#[primary_key] id: u8` injected, so a hand-written `id` field would
//! silently collide with it.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        id: u8,

        pub world_name: String,
    }
}

fn main() {}
