//! `#[foreign_key(..., referenced_row_method = false)]` keeps a table from adding the method
//! which looks up the referenced row to the wrapper type of its primary key.
//!
//! An account and a character reference each other through unique foreign keys, so the method
//! each table adds for the referenced row would take the name of the method the other table
//! adds for the referencing row, on the same wrapper type. With both switched off the module
//! compiles, and the methods for the referencing rows keep working.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = opt_out_accounts, method(update = true, delete = false))]
#[spacetimedb::table(accessor = opt_out_account)]
pub struct OptOutAccount {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_opt_out_test, table = opt_out_character)]
    id: u64,

    #[unique]
    #[use_wrapper(OptOutCharacterId)]
    #[foreign_key(
        path = crate::referenced_row_method_opt_out_test,
        table = opt_out_character,
        column = id,
        referenced_row_method = false
    )]
    pub character_id: u64,
}

#[spacetimedsl::dsl(plural_name = opt_out_characters, method(update = true, delete = false))]
#[spacetimedb::table(accessor = opt_out_character)]
pub struct OptOutCharacter {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_opt_out_test, table = opt_out_account)]
    id: u64,

    #[unique]
    #[use_wrapper(OptOutAccountId)]
    #[foreign_key(
        path = crate::referenced_row_method_opt_out_test,
        table = opt_out_account,
        column = id,
        referenced_row_method = false
    )]
    pub account_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut account = dsl.create_opt_out_account(CreateOptOutAccount {
        character_id: OptOutCharacterId::new(0),
    })?;
    let character = dsl.create_opt_out_character(CreateOptOutCharacter {
        account_id: account.get_id(),
    })?;
    account.set_character_id(&character);
    let account = dsl.update_opt_out_account_by_id(account)?;

    if character.get_id().get_opt_out_account(dsl)?.get_id() != account.get_id() {
        return Err(
            "OptOutCharacterId::get_opt_out_account should find the account whose character_id references the character!"
                .to_string(),
        );
    }

    if account.get_id().get_opt_out_character(dsl)?.get_id() != character.get_id() {
        return Err(
            "OptOutAccountId::get_opt_out_character should find the character whose account_id references the account!"
                .to_string(),
        );
    }

    Ok(())
}
