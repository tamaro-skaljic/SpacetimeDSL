//! Hash indices, which no other table here uses.
//!
//! The snapshots in `spacetimedsl_derive` pin the tokens generated for `#[index(hash)]`,
//! but only a module that is actually built proves those tokens compile and run. Covers a
//! non-unique single-column hash index, a unique single-column hash index and a
//! multi-column hash index.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = sessions, method(update = true, delete = true))]
#[spacetimedb::table(
    accessor = session,
    index(accessor = region_and_shard, hash(columns = [region_id, shard_id])),
    public,
)]
pub struct Session {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(hash)]
    pub token: String,

    #[index(hash)]
    #[unique]
    pub device_id: u64,

    pub region_id: u64,

    pub shard_id: u64,
}

/// Exercises every method shape a hash index produces: `filter` through the
/// non-unique single-column index, `find` through the unique single-column one, and
/// `filter` through the multi-column one.
pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    dsl.create_session(CreateSession {
        token: "first".to_string(),
        device_id: 11,
        region_id: 1,
        shard_id: 2,
    })
    .map_err(|e| format!("Should be able to create a Session! Got:\n{e}"))?;

    dsl.create_session(CreateSession {
        token: "first".to_string(),
        device_id: 22,
        region_id: 1,
        shard_id: 2,
    })
    .map_err(|e| format!("Should be able to create a second Session! Got:\n{e}"))?;

    let count_by_token = dsl.get_sessions_by_token("first").count();
    if count_by_token.ne(&2) {
        return Err(format!(
            "A non-unique hash index should find both sessions, got: {count_by_token}"
        ));
    }

    let count_by_region_and_shard = dsl.get_sessions_by_region_and_shard(&1, &2).count();
    if count_by_region_and_shard.ne(&2) {
        return Err(format!(
            "A multi-column hash index should find both sessions, got: {count_by_region_and_shard}"
        ));
    }

    let session = dsl
        .get_session_by_device_id(&22)
        .map_err(|e| format!("A unique hash index should find one session! Got:\n{e}"))?;

    if session.get_token().ne("first") {
        return Err(format!(
            "The session found by device_id should carry the token 'first', got: {}",
            session.get_token()
        ));
    }

    dsl.delete_session_by_device_id(&22)
        .map_err(|e| format!("Should be able to delete by a unique hash index! Got:\n{e}"))?;

    let count_after_delete = dsl.get_sessions_by_token("first").count();
    if count_after_delete.ne(&1) {
        return Err(format!(
            "One session should remain after deleting by device_id, got: {count_after_delete}"
        ));
    }

    dsl.delete_sessions_by_token("first")
        .map_err(|e| format!("Should be able to delete by a non-unique hash index! Got:\n{e}"))?;

    let count_at_end = dsl.get_sessions_by_token("first").count();
    if count_at_end.ne(&0) {
        return Err(format!(
            "No session should remain after deleting by token, got: {count_at_end}"
        ));
    }

    Ok(())
}
