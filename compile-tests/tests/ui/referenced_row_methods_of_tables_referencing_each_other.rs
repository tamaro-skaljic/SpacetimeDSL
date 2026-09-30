//! UNSUPPORTED COMBINATION - this file pins a limit of the methods SpacetimeDSL adds to wrapper
//! types, not a rule it enforces: no single `#[dsl]` sees both tables, so the macro cannot
//! report it itself.
//!
//! `player_account` and `player_character` reference each other through unique foreign keys.
//! `player_account.player_character_id` adds `get_player_account` to `PlayerCharacterId` for
//! the account which references a character, and `player_character.player_account_id` adds
//! `get_player_account` to `PlayerCharacterId` for the account a character references; the
//! same happens to `get_player_character` on `PlayerAccountId`. rustc rejects both as
//! duplicate definitions. Adding `referenced_row_method = false` to both foreign keys keeps
//! the methods for the referencing rows and leaves a program that compiles.

::spacetimedsl::spacetimedsl!();

pub mod account {
    #[spacetimedsl::dsl(plural_name = player_accounts, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player_account, public)]
    pub struct PlayerAccount {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerAccountId)]
        #[referenced_by(path = crate::character, table = player_character)]
        id: u64,

        #[unique]
        #[use_wrapper(crate::character::PlayerCharacterId)]
        #[foreign_key(path = crate::character, table = player_character, column = id)]
        player_character_id: u64,
    }
}

pub mod character {
    #[spacetimedsl::dsl(plural_name = player_characters, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player_character, public)]
    pub struct PlayerCharacter {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerCharacterId)]
        #[referenced_by(path = crate::account, table = player_account)]
        id: u64,

        #[unique]
        #[use_wrapper(crate::account::PlayerAccountId)]
        #[foreign_key(path = crate::account, table = player_account, column = id)]
        player_account_id: u64,
    }
}

fn main() {}
