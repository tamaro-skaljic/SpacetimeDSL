//! SpacetimeDB's unique-constraint error does not say which constraint a row broke, so the
//! error of `create_<table>` lists every unique column with the value handed to SpacetimeDB.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = constraint_badges, method(update = false, delete = false))]
#[spacetimedb::table(accessor = constraint_badge)]
pub struct ConstraintBadge {
    #[primary_key]
    #[create_wrapper]
    id: u64,

    #[unique]
    code: String,

    note: String,
}

/// Creating a badge with the id of a stored one names the primary key and the `#[unique]`
/// column with the values handed over, and leaves out the column that cannot collide.
pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    dsl.create_constraint_badge(CreateConstraintBadge {
        id: 1,
        code: "first".to_string(),
        note: "the stored badge".to_string(),
    })?;

    let result = dsl.create_constraint_badge(CreateConstraintBadge {
        id: 1,
        code: "second".to_string(),
        note: "a badge with the id of the stored one".to_string(),
    });

    let error = match result {
        Ok(badge) => {
            return Err(format!(
                "Creating a badge with the id of a stored one should fail! Got:\n{badge:?}"
            ));
        }
        Err(error) => error,
    };

    let expected = "Unique Constraint Violation Error while trying to create a row in the `constraint_badge` table! Unfortunately SpacetimeDB doesn't provide more information, so here are the unique columns and the values handed to SpacetimeDB: `{ id : 1, code : second }`.";
    if error.to_string() != expected {
        return Err(format!(
            "A unique-constraint violation should list only the unique columns and the values handed to SpacetimeDB!\n\nExpected:\n{expected}\n\nActual:\n{error}"
        ));
    }

    Ok(())
}
