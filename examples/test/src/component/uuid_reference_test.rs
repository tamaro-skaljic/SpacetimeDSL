//! References an auto-generated `Uuid` primary key from another module, where the private
//! field of its wrapper is out of reach.

use crate::component::uuid_test::CreateUuidPrimaryKeyRecord;
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

pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
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

    Ok(())
}
