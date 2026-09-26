use crate::spacetimedsl::prelude::*;
use spacetimedb::Timestamp;

#[spacetimedsl::dsl(
    plural_name = timestamp_records,
    method(update = true),
)]
#[spacetimedb::table(
    accessor = timestamp_record,
    public,
)]
pub struct TimestampRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[set_on_create]
    started_at: Timestamp,

    #[set_on_update]
    finished_at: Option<Timestamp>,

    pub value: u32,
}

pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let ctx = dsl.ctx();

    let timestamp_record = dsl.create_timestamp_record(CreateTimestampRecord { value: 1 })?;
    if timestamp_record.get_started_at().ne(&ctx.timestamp)
        || timestamp_record.get_finished_at().is_some()
    {
        return Err(
            "The helper timestamp attributes should initialize create timestamps correctly."
                .to_string(),
        );
    }

    let mut updated_timestamp_record = timestamp_record;
    updated_timestamp_record.set_value(2);
    let updated_timestamp_record = dsl.update_timestamp_record_by_id(updated_timestamp_record)?;
    if updated_timestamp_record
        .get_finished_at()
        .ne(&Some(ctx.timestamp))
    {
        return Err(
            "The #[set_on_update] helper attribute should refresh the timestamp on update."
                .to_string(),
        );
    }

    Ok(())
}
