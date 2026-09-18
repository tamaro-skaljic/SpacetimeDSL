//! Covers `method(update = false, delete = false)`.
//!
//! The snapshots pin what the two flags do. `update = false` removes the setters and every
//! update method; `delete = false` removes every delete method, so this table generates
//! only `create`, the getters and the count.
//!
//! `delete = false` additionally rejects three things this table does not have: an
//! `on_delete = Delete` foreign key, a before- or after-delete hook, and a
//! `#[referenced_by]` attribute. Each is pinned by its own case in `compile-tests`.
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
