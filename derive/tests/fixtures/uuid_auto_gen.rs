//! Covers `#[auto_gen(v4)]` and `#[auto_gen(v7)]`: the create method does not ask for these
//! `Uuid` columns, it generates them through the `v4` or `v7` constructor of their wrapper.
//!
//! `Account` has an auto-generated primary key which `SessionToken` references through a
//! foreign key, `SessionToken` has an auto-generated unique column next to an `auto_inc`
//! primary key, and `Installation` shows that a `#[dsl(singleton)]` table generates it too.

use spacetimedb::Uuid;

#[spacetimedsl::dsl(plural_name = accounts, method(update = true))]
#[spacetimedb::table(accessor = account, public)]
pub struct Account {
    #[primary_key]
    #[create_wrapper]
    #[auto_gen(v7)]
    #[referenced_by(path = self, table = session_token)]
    id: Uuid,

    pub name: String,
}

#[spacetimedsl::dsl(plural_name = session_tokens, method(update = true))]
#[spacetimedb::table(accessor = session_token, public)]
pub struct SessionToken {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[create_wrapper]
    #[auto_gen(v4)]
    token: spacetimedb::Uuid,

    #[index(btree)]
    #[use_wrapper(AccountId)]
    #[foreign_key(path = self, table = account, column = id, on_delete = Delete)]
    pub account_id: Uuid,
}

#[spacetimedsl::dsl(singleton, method(update = true))]
#[spacetimedb::table(accessor = installation, public)]
pub struct Installation {
    #[create_wrapper]
    #[auto_gen(v4)]
    installation_id: Uuid,

    pub name: String,
}
