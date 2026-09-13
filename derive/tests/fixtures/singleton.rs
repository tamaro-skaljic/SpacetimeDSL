//! Covers `#[dsl(singleton)]`: the injected `#[primary_key] id: u8`, the suppressed
//! `get_all` / `get_count` methods and the singular method names that take no primary key
//! argument.

#[spacetimedsl::dsl(singleton, method(update = true, delete = true))]
#[spacetimedb::table(accessor = configuration, public)]
pub struct Configuration {
    pub maximum_player_count: u32,

    pub world_name: String,
}
