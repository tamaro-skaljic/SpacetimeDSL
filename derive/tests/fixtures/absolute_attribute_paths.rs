//! Covers the spellings of the `#[dsl]` and `#[table]` attribute paths the other fixtures
//! do not use: an absolute path with a leading `::`, and the bare name a `use` brings into
//! scope.
//!
//! Every spelling has to be recognised, otherwise the struct generates nothing.

#[::spacetimedsl::dsl(plural_name = gauges, method(update = false))]
#[::spacetimedb::table(accessor = gauge, public)]
pub struct Gauge {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
}

#[dsl(plural_name = dials, method(update = false))]
#[table(accessor = dial, public)]
pub struct Dial {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
}
