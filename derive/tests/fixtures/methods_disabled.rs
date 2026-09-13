//! Covers `method(update = false, delete = false)`.
//!
//! The snapshots pin what the two flags really do, which is less than their names
//! suggest. `update = false` does remove the setters and every update method.
//! `delete = false` removes nothing from the generated output: the delete methods are
//! still there. It is only read by `foreign_key.rs:115`, to reject an
//! `on_delete = Delete` foreign key, and by `internal.rs:177`, to reject a before- or
//! after-delete hook - neither of which this table has. (FIXME)
//!
//! All columns are private, which is what `update = false` requires: a non-private column
//! would get a setter and therefore need an update method.

#[spacetimedsl::dsl(plural_name = audit_entries, method(update = false, delete = false))]
#[spacetimedb::table(accessor = audit_entry, public)]
pub struct AuditEntry {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    actor_id: u64,

    message: String,
}
