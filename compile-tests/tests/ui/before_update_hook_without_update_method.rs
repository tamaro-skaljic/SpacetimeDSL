//! A `before_update` hook can never run when the update method it would wrap is not
//! generated.
//!
//! The hook function is here for the fix: once `update` is enabled, the table calls it.
//! Until then a rejected `#[dsl]` emits nothing else, so the function misses the
//! `AuditEntry` struct and the `BeforeAuditEntryUpdateHook` trait it implements, which is
//! where the errors after the first come from.

::spacetimedsl::spacetimedsl!();

pub mod audit_entry {
    #[spacetimedsl::dsl(
        plural_name = audit_entries,
        method(update = false),
        hook(before(update)),
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
    fn before_audit_entry_update(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _old_audit_entry: &AuditEntry,
        new_audit_entry: AuditEntry,
    ) -> Result<AuditEntry, crate::spacetimedsl::SpacetimeDSLError> {
        Ok(new_audit_entry)
    }
}

fn main() {}
