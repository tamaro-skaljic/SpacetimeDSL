//! A foreign key on the primary key adds no method for the referenced row: the row it
//! references has the key's own value, so `referenced_row_method` would switch off nothing.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so the
//! `account` table's expansion misses the trait the `profile` table would have declared for it.

::spacetimedsl::spacetimedsl!();

pub mod account {
    #[spacetimedsl::dsl(plural_name = accounts, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = account, public)]
    pub struct Account {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(AccountId)]
        #[referenced_by(path = crate::profile, table = profile)]
        id: u64,
    }
}

pub mod profile {
    #[spacetimedsl::dsl(plural_name = profiles, method(update = true, delete = false))]
    #[spacetimedb::table(accessor = profile, public)]
    pub struct Profile {
        #[primary_key]
        #[use_wrapper(crate::account::AccountId)]
        #[foreign_key(path = crate::account, table = account, column = id, referenced_row_method = false)]
        account_id: u64,

        pub display_name: String,
    }
}

fn main() {}
