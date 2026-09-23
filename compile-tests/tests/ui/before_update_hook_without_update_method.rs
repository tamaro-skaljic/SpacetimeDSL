//! A `before_update` hook can never run when the update method it would wrap is not
//! generated.

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
