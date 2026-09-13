//! KNOWN DEFECT - this file pins a SpacetimeDSL bug, not a rule.
//!
//! A column typed `Option<T>` with both `#[create_wrapper]` and a unique index generates
//! code which does not compile: the error message builder formats the column with `{}`
//! after the wrapper has already wrapped it once, so it asks for
//! `Option<Option<spacetimedb::Timestamp>>: Display`.
//!
//! The defect is therefore *expected* here, and this test passes while it exists. Fixing
//! it turns this test red. That is the signal to convert this file into a `t.pass()` case
//! and move it out of `tests/ui`, not to regenerate the `.stderr`.
//!
//! Maintenance note: unlike every other `.stderr` here, this one quotes source lines out
//! of SpacetimeDB's own `src/table.rs` - the signature of `UniqueColumn::find` and the
//! bound it requires. `trybuild` normalizes the path and drops the line numbers, so the
//! version does not leak in, but a reworded signature does. This file therefore churns on
//! SpacetimeDB upgrades, independently of the compiler pinned in `rust-toolchain.toml`,
//! and regenerating it with `TRYBUILD=overwrite` is part of the upgrade routine.

::spacetimedsl::spacetimedsl!();

pub mod reminder {
    use spacetimedb::Timestamp;

    #[spacetimedsl::dsl(plural_name = reminders, method(update = true))]
    #[spacetimedb::table(accessor = reminder, public)]
    pub struct Reminder {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[unique]
        #[create_wrapper]
        pub acknowledged_at: Option<Timestamp>,
    }
}

fn main() {}
