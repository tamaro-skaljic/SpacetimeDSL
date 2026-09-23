//! An `after_delete` hook is rejected together with `method(delete = false)`.
//!
//! The hook function is here for the fix: once `delete` is enabled, the table calls it.
//! Until then a rejected `#[dsl]` emits nothing else, so the function misses the
//! `AuditEntry` struct and the `AfterAuditEntryDeleteHook` trait it implements, which is
//! where the errors after the first come from.

::spacetimedsl::spacetimedsl!();

pub mod audit_entry {
    #[spacetimedsl::dsl(
        plural_name = audit_entries,
        method(update = false, delete = false),
        hook(after(delete)),
    )]
    #[spacetimedb::table(accessor = audit_entry, public)]
    pub struct AuditEntry {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        message: String,
    }

    #[spacetimedsl::hook]
    fn after_audit_entry_delete(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _old_audit_entry: &AuditEntry,
    ) -> Result<(), crate::spacetimedsl::SpacetimeDSLError> {
        Ok(())
    }
}

fn main() {}
