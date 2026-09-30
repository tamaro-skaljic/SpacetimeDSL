//! `owner_id` and `owner` both reference `player`, so their methods for the referenced row
//! take the stems of their columns, which are the same: both would add `get_owner` to
//! `GameId`.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so the
//! `player` table's expansion misses the trait the `game` table would have declared for it.

::spacetimedsl::spacetimedsl!();

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerId)]
        #[referenced_by(path = crate::game, table = game)]
        id: u64,
    }
}

pub mod game {
    #[spacetimedsl::dsl(plural_name = games, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = game, public)]
    pub struct Game {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        owner_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        owner: u64,
    }
}

fn main() {}
