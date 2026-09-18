//! Covers an `Option<Timestamp>` column carrying an index under both wrapper kinds. No table
//! in `examples/test` reaches this branch, because the index there is commented out, so
//! these snapshots are the only thing pinning it.
//!
//! `#[create_wrapper]` wraps the whole `Option`, so the generated wrapper's `value()`
//! already yields an `Option`. `#[use_wrapper(...)]` reuses a wrapper another table
//! wrote around the inner type, so its `value()` yields that inner type. The index path
//! has to treat the two differently, and both are pinned here.

use spacetimedb::Timestamp;

#[spacetimedsl::dsl(plural_name = reminders, method(update = true))]
#[spacetimedb::table(accessor = reminder, public)]
pub struct Reminder {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[create_wrapper]
    pub acknowledged_at: Option<Timestamp>,

    #[index(btree)]
    #[use_wrapper(ReminderDueAt)]
    pub due_at: Option<Timestamp>,
}
