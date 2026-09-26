use crate::spacetimedsl::prelude::*;

/// What the child's before-delete hook says when it refuses.
const LOCKED_MESSAGE: &str = "this lock holder is locked";

#[spacetimedsl::dsl(plural_name = lock_groups, method(update = false, delete = true))]
#[spacetimedb::table(accessor = lock_group)]
pub struct LockGroup {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(LockGroupId)]
    #[referenced_by(path = crate::cascade_hook_error_test, table = lock_holder)]
    id: u64,

    /// A non-unique index, so the many-row delete method exists.
    #[index(btree)]
    batch: u64,
}

#[spacetimedsl::dsl(
    plural_name = lock_holders,
    method(update = false, delete = true),
    hook(before(delete))
)]
#[spacetimedb::table(accessor = lock_holder)]
pub struct LockHolder {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(LockHolderId)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LockGroupId)]
    #[foreign_key(
        path = crate::cascade_hook_error_test,
        table = lock_group,
        column = id,
        on_delete = Delete
    )]
    group_id: u64,

    locked: bool,
}

#[spacetimedsl::hook]
fn before_lock_holder_delete(
    _dsl: &crate::spacetimedsl::DSL<'_, T>,
    row: &LockHolder,
) -> Result<(), SpacetimeDSLError> {
    match row.get_locked() {
        false => Ok(()),
        true => Err(SpacetimeDSLError::Error(LOCKED_MESSAGE.to_string())),
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let one_row_group = dsl.create_lock_group(CreateLockGroup { batch: 1 })?;
    dsl.create_lock_holder(CreateLockHolder {
        group_id: one_row_group.get_id(),
        locked: true,
    })?;

    match dsl.delete_lock_group_by_id(&one_row_group) {
        Ok(_) => {
            return Err(
                "Deleting a lock group whose holder's before_delete hook fails should fail!"
                    .to_string(),
            );
        }
        Err(error) => {
            let error = error.to_string();
            if !error.contains(LOCKED_MESSAGE) {
                return Err(format!(
                    "The error the before_delete hook raised should reach the caller! Got:\n{error}"
                ));
            }
        }
    };

    let many_rows_group = dsl.create_lock_group(CreateLockGroup { batch: 2 })?;
    dsl.create_lock_holder(CreateLockHolder {
        group_id: many_rows_group.get_id(),
        locked: true,
    })?;

    match dsl.delete_lock_groups_by_batch(&2) {
        Ok(_) => {
            return Err(
                "Deleting lock groups whose holders' before_delete hook fails should fail!"
                    .to_string(),
            );
        }
        Err(error) => {
            let error = error.to_string();
            if !error.contains(LOCKED_MESSAGE) {
                return Err(format!(
                    "The error the before_delete hook raised should reach the caller of the many-row delete! Got:\n{error}"
                ));
            }
        }
    };

    Ok(())
}
