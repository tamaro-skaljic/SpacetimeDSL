//! Covers `#[dsl(singleton(with_default))]`: the missing `create_<table>` method and its
//! missing `Create<Table>` struct, the `upsert_<table>` which replaces `update_<table>`,
//! the default branch of `get_<table>`, and the timestamp columns whose two write paths
//! differ.
//!
//! All six hooks are here as well, because a table with a default is the one shape whose
//! `before_insert` hook takes the whole row rather than a create request.

use spacetimedb::Timestamp;

#[spacetimedsl::dsl(
    singleton(with_default),
    method(update = true, delete = true),
    hook(
        before(insert, update, delete),
        after(insert, update, delete),
    ),
)]
#[spacetimedb::table(accessor = world_settings, public)]
pub struct WorldSettings {
    pub maximum_player_count: u32,

    pub world_name: String,

    #[created_at]
    created_at: Timestamp,

    #[updated_at]
    modified_at: Option<Timestamp>,
}

/// The smallest shape: a default, no hooks, no timestamps, no delete method.
#[spacetimedsl::dsl(singleton(with_default), method(update = true, delete = false))]
#[spacetimedb::table(accessor = feature_flags, public)]
pub struct FeatureFlags {
    pub tutorial_is_enabled: bool,
}
