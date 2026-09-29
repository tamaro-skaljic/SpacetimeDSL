//! The reference-integrity checks of `create_<table>` and `update_<table>_by_<key>` report
//! the row they could not use by column name and value, the way every other generated error
//! does.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = integrity_depots, method(update = false, delete = true))]
#[spacetimedb::table(accessor = integrity_depot)]
pub struct IntegrityDepot {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(IntegrityDepotId)]
    #[referenced_by(path = crate::reference_integrity_message_test, table = integrity_parcel)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = integrity_parcels, method(update = true, delete = true))]
#[spacetimedb::table(accessor = integrity_parcel)]
pub struct IntegrityParcel {
    #[primary_key]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(IntegrityDepotId)]
    #[foreign_key(
        path = crate::reference_integrity_message_test,
        table = integrity_depot,
        column = id,
        on_delete = Delete
    )]
    pub depot_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    create_names_the_column_which_references_no_row(dsl)?;
    update_of_a_missing_row_names_its_primary_key(dsl)?;

    Ok(())
}

/// A parcel whose `depot_id` references no depot is rejected, and the error names the column
/// and the value it held.
fn create_names_the_column_which_references_no_row<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let result = dsl.create_integrity_parcel(CreateIntegrityParcel {
        id: 1,
        depot_id: IntegrityDepotId::new(u64::MAX),
    });

    let error = match result {
        Ok(parcel) => {
            return Err(format!(
                "Creating a parcel whose depot_id references no depot should fail! Got:\n{parcel:?}"
            ));
        }
        Err(error) => error,
    };

    let expected = format!(
        "Reference Integrity Violation Error while trying to create a row in the `integrity_parcel` table because of `{{ depot_id : {} }}`!",
        u64::MAX
    );
    if error.to_string() != expected {
        return Err(format!(
            "The reference-integrity error of create_integrity_parcel should name the column and its value!\n\nExpected:\n{expected}\n\nActual:\n{error}"
        ));
    }

    Ok(())
}

/// Updating a parcel which no longer exists fails with the primary key value the lookup used,
/// not with the value of its foreign key.
fn update_of_a_missing_row_names_its_primary_key<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let depot = dsl.create_integrity_depot()?;
    let parcel = dsl.create_integrity_parcel(CreateIntegrityParcel {
        id: 1_000_000,
        depot_id: depot.get_id(),
    })?;
    dsl.delete_integrity_parcel_by_id(parcel.get_id())?;

    let error = match dsl.update_integrity_parcel_by_id(parcel) {
        Ok(parcel) => {
            return Err(format!(
                "Updating a parcel which no longer exists should fail! Got:\n{parcel:?}"
            ));
        }
        Err(error) => error,
    };

    let expected = "Not Found Error while trying to find a row in the `integrity_parcel` table with `{ id : 1000000 }`!";
    if error.to_string() != expected {
        return Err(format!(
            "Updating a parcel which no longer exists should name the primary key value it looked up!\n\nExpected:\n{expected}\n\nActual:\n{error}"
        ));
    }

    Ok(())
}
