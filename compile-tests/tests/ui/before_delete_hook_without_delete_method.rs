//! A `before_delete` hook is rejected together with `method(delete = false)`.
//!
//! `method(delete = false)` removes every delete method, which the snapshot fixture
//! `methods_disabled` pins, and rejects the three things that would have needed one: a
//! delete hook, an `on_delete = Delete` foreign key, and a `#[referenced_by]` attribute.

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
