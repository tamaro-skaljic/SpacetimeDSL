//! Covers an `Option<Timestamp>` column carrying both `#[create_wrapper]` and an index -
//! the branch fixed in `81ada87`, which no table in `examples/test` reaches because the
//! index there is commented out.
//!
//! The generated wrapper wraps the `Option`, so the index methods take
//! `Option<Timestamp>` rather than `Timestamp`.

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
}
