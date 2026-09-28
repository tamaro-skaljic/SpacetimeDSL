//! Covers `Uuid` foreign keys on a `singleton(with_default)` table. `upsert_<table>` runs one
//! set of reference-integrity checks per write path, and both have to treat `Uuid::NIL` as
//! referencing no row, the way they treat `0` for an unsigned integer.
//!
//! `ActiveBracket` mirrors `ActiveTournament` of `singleton_with_foreign_key` with `Uuid`
//! keys: a public key checked on both paths and a private one checked on insert only.

use spacetimedb::Uuid;

#[spacetimedsl::dsl(plural_name = brackets, method(update = true))]
#[spacetimedb::table(accessor = bracket, public)]
pub struct Bracket {
    #[primary_key]
    #[create_wrapper(BracketId)]
    #[auto_gen(v7)]
    #[referenced_by(path = self, table = active_bracket)]
    id: Uuid,

    pub name: String,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = active_bracket, public)]
pub struct ActiveBracket {
    #[use_wrapper(BracketId)]
    #[foreign_key(path = self, table = bracket, column = id, on_delete = Delete)]
    pub bracket_id: Uuid,

    #[use_wrapper(BracketId)]
    #[foreign_key(path = self, table = bracket, column = id, on_delete = Error)]
    seed_bracket_id: Uuid,
}
