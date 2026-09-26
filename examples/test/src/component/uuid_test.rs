//! Covers `#[auto_gen(v4)]` and `#[auto_gen(v7)]` on every index shape a `Uuid` column
//! can take part in, and on a `#[dsl(singleton)]` table.

use crate::spacetimedsl::prelude::*;
use spacetimedb::Uuid;

#[spacetimedsl::dsl(
    plural_name = uuid_primary_key_records,
    method(update = true),
)]
#[spacetimedb::table(
    accessor = uuid_primary_key_record,
    public,
)]
pub struct UUIDPrimaryKeyRecord {
    #[primary_key]
    #[create_wrapper]
    #[auto_gen(v7)]
    #[referenced_by(path = crate::component::uuid_reference_test, table = uuid_reference)]
    id: Uuid,

    pub name: String,
}

#[spacetimedsl::dsl(
    plural_name = uuid_unique_records,
    method(update = false),
)]
#[spacetimedb::table(
    accessor = uuid_unique_record,
    public,
)]
pub struct UUIDUniqueRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[create_wrapper]
    #[auto_gen(v4)]
    token: spacetimedb::Uuid,
}

#[spacetimedsl::dsl(
    plural_name = uuid_index_records,
    method(update = false),
)]
#[spacetimedb::table(
    accessor = uuid_index_record,
    public,
)]
pub struct UUIDIndexRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[create_wrapper]
    #[auto_gen(v4)]
    token: Uuid,
}

#[spacetimedsl::dsl(
    plural_name = uuid_multi_column_index_records,
    method(update = false),
)]
#[spacetimedb::table(
    accessor = uuid_multi_column_index_record,
    index(accessor = token_and_group, btree(columns = [token, group])),
    public,
)]
pub struct UUIDMultiColumnIndexRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[create_wrapper]
    #[auto_gen(v4)]
    token: Uuid,

    group: u32,
}

#[spacetimedsl::dsl(
    plural_name = uuid_unique_multi_column_index_records,
    method(update = false),
    unique_index(name = token_and_group),
)]
#[spacetimedb::table(
    accessor = uuid_unique_multi_column_index_record,
    index(accessor = token_and_group, btree(columns = [token, group])),
    public,
)]
pub struct UUIDUniqueMultiColumnIndexRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[create_wrapper]
    #[auto_gen(v7)]
    token: Uuid,

    group: u32,
}

#[spacetimedsl::dsl(singleton, method(update = false))]
#[spacetimedb::table(
    accessor = uuid_singleton_record,
    public,
)]
pub struct UUIDSingletonRecord {
    #[create_wrapper]
    #[auto_gen(v4)]
    token: Uuid,
}

fn expect_uuid_version(
    uuid: spacetimedb::Uuid,
    expected_version: spacetimedb::sats::uuid::Version,
    column: &str,
) -> Result<(), String> {
    if uuid.get_version() != Some(expected_version) {
        return Err(format!(
            "`{column}` should be a UUID {expected_version:?}, found {uuid}."
        ));
    }
    Ok(())
}

/// Each `#[auto_gen]` table gets two rows, which must receive different UUIDs of the
/// configured version, and must be found again by the stored UUID.
pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    use spacetimedb::sats::uuid::Version;

    let first_record = dsl.create_uuid_primary_key_record(CreateUuidPrimaryKeyRecord {
        name: "first".to_string(),
    })?;
    let second_record = dsl.create_uuid_primary_key_record(CreateUuidPrimaryKeyRecord {
        name: "second".to_string(),
    })?;
    expect_uuid_version(first_record.get_id().value(), Version::V7, "id")?;
    if first_record.get_id().value() >= second_record.get_id().value() {
        return Err("A UUID v7 primary key should sort in creation order.".to_string());
    }
    if dsl
        .get_uuid_primary_key_record_by_id(second_record.get_id())?
        .ne(&second_record)
    {
        return Err("The row should be found by its UUID v7 primary key.".to_string());
    }

    let first_unique = dsl.create_uuid_unique_record()?;
    let second_unique = dsl.create_uuid_unique_record()?;
    expect_uuid_version(first_unique.get_token().value(), Version::V4, "token")?;
    if first_unique.get_token().eq(&second_unique.get_token()) {
        return Err("Two rows should get different UUIDs v4.".to_string());
    }
    if dsl
        .get_uuid_unique_record_by_token(second_unique.get_token())?
        .ne(&second_unique)
    {
        return Err("The row should be found by its unique UUID.".to_string());
    }

    let first_indexed = dsl.create_uuid_index_record()?;
    let second_indexed = dsl.create_uuid_index_record()?;
    expect_uuid_version(first_indexed.get_token().value(), Version::V4, "token")?;
    if first_indexed.get_token().eq(&second_indexed.get_token()) {
        return Err("Two rows should get different indexed UUIDs.".to_string());
    }
    if dsl
        .get_uuid_index_records_by_token(first_indexed.get_token())
        .collect_vec()
        .ne(&vec![first_indexed])
    {
        return Err("The row should be found by its indexed UUID.".to_string());
    }

    let first_multi_column_indexed =
        dsl.create_uuid_multi_column_index_record(CreateUuidMultiColumnIndexRecord { group: 1 })?;
    let second_multi_column_indexed =
        dsl.create_uuid_multi_column_index_record(CreateUuidMultiColumnIndexRecord { group: 1 })?;
    expect_uuid_version(
        first_multi_column_indexed.get_token().value(),
        Version::V4,
        "token",
    )?;
    if first_multi_column_indexed
        .get_token()
        .eq(&second_multi_column_indexed.get_token())
    {
        return Err("Two rows should get different multi-column indexed UUIDs.".to_string());
    }
    if dsl
        .get_uuid_multi_column_index_records_by_token_and_group(
            first_multi_column_indexed.get_token(),
            first_multi_column_indexed.get_group(),
        )
        .collect_vec()
        .ne(&vec![first_multi_column_indexed])
    {
        return Err("The row should be found by its multi-column indexed UUID.".to_string());
    }

    let first_unique_multi_column_indexed =
        dsl.create_uuid_unique_multi_column_index_record(CreateUuidUniqueMultiColumnIndexRecord {
            group: 1,
        })?;
    let second_unique_multi_column_indexed =
        dsl.create_uuid_unique_multi_column_index_record(CreateUuidUniqueMultiColumnIndexRecord {
            group: 1,
        })?;
    expect_uuid_version(
        first_unique_multi_column_indexed.get_token().value(),
        Version::V7,
        "token",
    )?;
    if first_unique_multi_column_indexed.get_token().value()
        >= second_unique_multi_column_indexed.get_token().value()
    {
        return Err("UUIDs v7 should sort in creation order.".to_string());
    }
    if dsl
        .get_uuid_unique_multi_column_index_record_by_token_and_group(
            second_unique_multi_column_indexed.get_token(),
            second_unique_multi_column_indexed.get_group(),
        )?
        .ne(&second_unique_multi_column_indexed)
    {
        return Err("The row should be found by its unique multi-column indexed UUID.".to_string());
    }

    let singleton = dsl.create_uuid_singleton_record()?;
    expect_uuid_version(singleton.get_token().value(), Version::V4, "token")?;
    if dsl.get_uuid_singleton_record()?.ne(&singleton) {
        return Err("The singleton row should keep its generated UUID.".to_string());
    }

    Ok(())
}
