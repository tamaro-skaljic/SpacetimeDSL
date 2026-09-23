//! An `after_delete` hook is rejected together with `method(delete = false)`.

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
