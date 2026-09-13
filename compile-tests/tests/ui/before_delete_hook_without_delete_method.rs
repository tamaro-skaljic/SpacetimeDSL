//! A `before_delete` hook is rejected together with `method(delete = false)`.
//!
//! Note that the flag does not actually remove the delete methods - the snapshot fixture
//! `methods_disabled` pins that - so this diagnostic is the only place where
//! `delete = false` is visible to the user besides `on_delete = Delete`.

::spacetimedsl::spacetimedsl!();

pub mod audit_entry {
    #[spacetimedsl::dsl(
        plural_name = audit_entries,
        method(update = false, delete = false),
        hook(before(delete)),
    )]
    #[spacetimedb::table(accessor = audit_entry, public)]
    pub struct AuditEntry {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        message: String,
    }
}

fn main() {}
