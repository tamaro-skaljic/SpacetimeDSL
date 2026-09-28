//! References an auto-generated `Uuid` primary key from another module, where the private
//! field of its wrapper is out of reach.

use crate::component::uuid_test::{CreateUuidPrimaryKeyRecord, UUIDPrimaryKeyRecordId};
use crate::spacetimedsl::prelude::*;
use spacetimedb::Uuid;

#[spacetimedsl::dsl(
    plural_name = uuid_references,
    method(update = false),
)]
#[spacetimedb::table(
    accessor = uuid_reference,
    public,
)]
pub struct UUIDReference {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::component::uuid_test::UUIDPrimaryKeyRecordId)]
    #[foreign_key(path = crate::component::uuid_test, table = uuid_primary_key_record, column = id, on_delete = Delete)]
    record_id: Uuid,
}

/// Like `UUIDReference`, but deleting the referenced record resets the key to `Uuid::NIL`,
/// the `Uuid` that references no row.
#[spacetimedsl::dsl(
    plural_name = uuid_set_zero_references,
    method(update = true),
)]
#[spacetimedb::table(
    accessor = uuid_set_zero_reference,
    public,
)]
pub struct UUIDSetZeroReference {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::component::uuid_test::UUIDPrimaryKeyRecordId)]
    #[foreign_key(path = crate::component::uuid_test, table = uuid_primary_key_record, column = id, on_delete = SetZero)]
    pub record_id: Uuid,
}

pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let referenced_record = dsl.create_uuid_primary_key_record(CreateUuidPrimaryKeyRecord {
        name: "referenced".to_string(),
    })?;

    let reference = dsl.create_uuid_reference(CreateUuidReference {
        record_id: referenced_record.get_id(),
    })?;
    if reference.get_record_id().ne(&referenced_record.get_id()) {
        return Err("The foreign key getter should return the referenced UUID.".to_string());
    }
    dsl.delete_uuid_primary_key_record_by_id(referenced_record.get_id())?;
    if dsl.get_uuid_reference_by_id(reference.get_id()).is_ok() {
        return Err("Deleting the UUID row should delete the referencing row.".to_string());
    }

    set_zero_resets_the_key_to_nil(dsl)?;
    nil_references_no_row(dsl)?;

    Ok(())
}

/// Deleting the referenced record resets the key of a `SetZero` reference to `Uuid::NIL`.
fn set_zero_resets_the_key_to_nil(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let referenced_record = dsl.create_uuid_primary_key_record(CreateUuidPrimaryKeyRecord {
        name: "referenced by a SetZero reference".to_string(),
    })?;
    let reference = dsl.create_uuid_set_zero_reference(CreateUuidSetZeroReference {
        record_id: referenced_record.get_id(),
    })?;

    dsl.delete_uuid_primary_key_record_by_id(referenced_record.get_id())
        .map_err(|error| format!("Deleting the referenced record should succeed: {error}"))?;

    let reference = dsl.get_uuid_set_zero_reference_by_id(reference.get_id())?;
    if reference.get_record_id().value() != Uuid::NIL {
        return Err(format!(
            "Deleting the referenced record should reset the key to Uuid::NIL, found {}!",
            reference.get_record_id()
        ));
    }

    Ok(())
}

/// A key of `Uuid::NIL` references no row, like `0` for an unsigned integer, so create and
/// update accept it without a referenced row, while any other key still needs one.
fn nil_references_no_row(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let nil = UUIDPrimaryKeyRecordId::new(Uuid::NIL);
    let mut reference = dsl
        .create_uuid_set_zero_reference(CreateUuidSetZeroReference {
            record_id: nil.clone(),
        })
        .map_err(|error| {
            format!("Creating a reference with a Uuid::NIL key should succeed: {error}")
        })?;

    reference.set_record_id(nil);
    dsl.update_uuid_set_zero_reference_by_id(reference)
        .map_err(|error| {
            format!("Updating a reference to a Uuid::NIL key should succeed: {error}")
        })?;

    let dangling = UUIDPrimaryKeyRecordId::new(Uuid::MAX);
    if dsl
        .create_uuid_set_zero_reference(CreateUuidSetZeroReference {
            record_id: dangling,
        })
        .is_ok()
    {
        return Err(
            "Creating a reference whose non-NIL key references no row should fail!".to_string(),
        );
    }

    Ok(())
}
