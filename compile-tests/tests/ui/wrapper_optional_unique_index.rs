//! UNSUPPORTED COMBINATION - this file pins a SpacetimeDB limitation, not a SpacetimeDSL
//! bug and not a rule SpacetimeDSL enforces itself.
//!
//! A column typed `Option<T>` carrying a unique index cannot work against SpacetimeDB:
//! `&Option<T>` does not implement `FilterableValue`, so `UniqueColumn::find` rejects it,
//! and `Option<T>` does not implement `Display`, so the error message builder cannot
//! format it. Neither is something SpacetimeDSL can generate its way out of, so this stays
//! a `compile_fail` case.
//!
//! The doubled `Option` this file used to pin - `Option<Option<spacetimedb::Timestamp>>`,
//! produced by wrapping a created wrapper's `value()` in `Some` although that wrapper
//! wraps the whole `Option` and `value()` already yields it - was a SpacetimeDSL bug. It
//! is fixed, and the `.stderr` shrank accordingly.
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
