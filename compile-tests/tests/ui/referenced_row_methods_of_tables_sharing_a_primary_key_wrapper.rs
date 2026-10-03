//! UNSUPPORTED COMBINATION - this file pins a limit of the methods SpacetimeDSL adds to wrapper
//! types, not a rule it enforces: no single `#[dsl]` sees both tables, so the macro cannot
//! report it itself.
//!
//! `circle` and `food` both use `EntityId` for their primary key, and both reference `player`
//! through `player_id`. Each adds `get_player` to `EntityId` for the player its row
//! references, so rustc rejects the second as a duplicate definition. Adding
//! `referenced_row_method = false` to one of the two foreign keys to `player` leaves a program
//! that compiles.

::spacetimedsl::spacetimedsl!();

pub mod entity {
    #[spacetimedsl::dsl(plural_name = entities, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = entity, public)]
    pub struct Entity {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(EntityId)]
        #[referenced_by(path = crate::circle, table = circle)]
        #[referenced_by(path = crate::food, table = food)]
        id: u64,
    }
}

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerId)]
        #[referenced_by(path = crate::circle, table = circle)]
        #[referenced_by(path = crate::food, table = food)]
        id: u64,
    }
}

pub mod circle {
    #[spacetimedsl::dsl(plural_name = circles, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = circle, public)]
    pub struct Circle {
        #[primary_key]
        #[use_wrapper(crate::entity::EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = id)]
        entity_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        player_id: u64,
    }
}

pub mod food {
    #[spacetimedsl::dsl(plural_name = foods, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = food, public)]
    pub struct Food {
        #[primary_key]
        #[use_wrapper(crate::entity::EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = id)]
        entity_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        player_id: u64,
    }
}

fn main() {}
