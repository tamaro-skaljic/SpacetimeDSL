//! Covers the lookup methods a `#[foreign_key]` column adds to its wrapper type, in the
//! shapes the other fixtures with foreign keys do not reach:
//!
//! - `Category` references its own table, so its method takes the long name although only
//!   one column references the table: `category_id.get_categories()` would read like a
//!   lookup of the category itself.
//! - `Profile` carries its foreign key on the primary key, a one-to-one relationship.
//! - `AccountNote` names its wrapper by a qualified path, so the `impl` block is emitted on
//!   the path while the doc comment names the last segment.
//! - `Membership` is expanded once per `#[dsl]` attribute, and each pass adds the method for
//!   its own table.
//!
//! The fixture is only expanded, never compiled, so `crate::accounts` does not have to exist.

#[spacetimedsl::dsl(plural_name = accounts, method(update = true))]
#[spacetimedb::table(accessor = account, public)]
pub struct Account {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = profile)]
    #[referenced_by(path = self, table = account_note)]
    #[referenced_by(path = self, table = active_membership)]
    #[referenced_by(path = self, table = expired_membership)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = categories, method(update = true))]
#[spacetimedb::table(accessor = category, public)]
pub struct Category {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = category)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CategoryId)]
    #[foreign_key(path = self, table = category, column = id, on_delete = Error)]
    pub parent_category_id: u64,
}

#[spacetimedsl::dsl(plural_name = profiles, method(update = true))]
#[spacetimedb::table(accessor = profile, public)]
pub struct Profile {
    #[primary_key]
    #[use_wrapper(AccountId)]
    #[foreign_key(path = self, table = account, column = id, on_delete = Error)]
    account_id: u64,

    pub display_name: String,
}

#[spacetimedsl::dsl(plural_name = account_notes, method(update = true))]
#[spacetimedb::table(accessor = account_note, public)]
pub struct AccountNote {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::accounts::AccountId)]
    #[foreign_key(path = crate::accounts, table = account, column = id, on_delete = Error)]
    pub account_id: u64,
}

#[spacetimedsl::dsl(plural_name = active_memberships, method(update = true))]
#[spacetimedb::table(accessor = active_membership, public)]
#[spacetimedsl::dsl(plural_name = expired_memberships, method(update = true))]
#[spacetimedb::table(accessor = expired_membership, public)]
pub struct Membership {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[use_wrapper(AccountId)]
    #[foreign_key(path = self, table = account, column = id, on_delete = Error)]
    pub account_id: u64,
}
