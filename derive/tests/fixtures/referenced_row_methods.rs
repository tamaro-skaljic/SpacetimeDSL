//! Covers the methods a table adds to the wrapper type of its primary key, which look up the
//! row a foreign key column of the row with that key references:
//!
//! - `Server` and `Alliance` reference one table through one column each, so their methods
//!   take the name of the referenced table: `server_id.get_season(&dsl)`,
//!   `alliance_id.get_server(&dsl)`.
//! - `Tournament` references `Season` through two columns, so each method takes the name of
//!   its column without the key suffix: `get_opening_season`, `get_closing_season`.
//! - `Stage` references its own table through a unique column, which adds
//!   `get_stage_by_next_stage_id` to `StageId` for the referencing row; its method for the
//!   referenced row is `get_next_stage`.
//! - `Circle` has a foreign key on its primary key, which adds no such method, and one on
//!   `player_id`, which adds `get_player` to `EntityId`, the wrapper its key uses.
//! - `Referee` is a singleton, whose injected primary key has no wrapper type: it adds none.
//! - `Banner` switches its method off with `referenced_row_method = false`, which leaves only
//!   `ServerId::get_banners`.
//!
//! A struct with several `#[dsl]` attributes adds none either; the `Membership` snapshots of
//! the fixture `wrapper_methods` pin that.

#[spacetimedsl::dsl(plural_name = seasons, method(update = false, delete = false))]
#[spacetimedb::table(accessor = season, public)]
pub struct Season {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = server)]
    #[referenced_by(path = self, table = tournament)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = servers, method(update = false, delete = false))]
#[spacetimedb::table(accessor = server, public)]
pub struct Server {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = alliance)]
    #[referenced_by(path = self, table = banner)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(SeasonId)]
    #[foreign_key(path = self, table = season, column = id)]
    season_id: u64,
}

#[spacetimedsl::dsl(plural_name = alliances, method(update = false, delete = false))]
#[spacetimedb::table(accessor = alliance, public)]
pub struct Alliance {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ServerId)]
    #[foreign_key(path = self, table = server, column = id)]
    server_id: u64,
}

#[spacetimedsl::dsl(plural_name = banners, method(update = false, delete = false))]
#[spacetimedb::table(accessor = banner, public)]
pub struct Banner {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ServerId)]
    #[foreign_key(path = self, table = server, column = id, referenced_row_method = false)]
    server_id: u64,
}

#[spacetimedsl::dsl(plural_name = tournaments, method(update = false, delete = false))]
#[spacetimedb::table(accessor = tournament, public)]
pub struct Tournament {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(SeasonId)]
    #[foreign_key(path = self, table = season, column = id)]
    opening_season_id: u64,

    #[index(btree)]
    #[use_wrapper(SeasonId)]
    #[foreign_key(path = self, table = season, column = id)]
    closing_season_id: u64,
}

#[spacetimedsl::dsl(plural_name = stages, method(update = false, delete = false))]
#[spacetimedb::table(accessor = stage, public)]
pub struct Stage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = stage)]
    id: u64,

    #[unique]
    #[use_wrapper(StageId)]
    #[foreign_key(path = self, table = stage, column = id)]
    next_stage_id: u64,
}

#[spacetimedsl::dsl(plural_name = entities, method(update = false, delete = false))]
#[spacetimedb::table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = circle)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = players, method(update = false, delete = false))]
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = circle)]
    #[referenced_by(path = self, table = referee)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = circles, method(update = true, delete = false))]
#[spacetimedb::table(accessor = circle, public)]
pub struct Circle {
    #[primary_key]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id)]
    entity_id: u64,

    #[index(btree)]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id)]
    pub player_id: u64,
}

#[spacetimedsl::dsl(singleton, method(update = true))]
#[spacetimedb::table(accessor = referee, public)]
pub struct Referee {
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id)]
    pub player_id: u64,
}
