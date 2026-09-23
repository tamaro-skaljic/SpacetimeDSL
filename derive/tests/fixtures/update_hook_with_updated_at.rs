//! Pairs a `before_update`/`after_update` hook with an `#[updated_at]` column on a
//! non-singleton table - the one combination the rest of the corpus never puts on the same
//! table. `update_<table>_by_<primary_key>` shadows the row with the hook's own, non-`mut`
//! binding before it writes the timestamp; only this fixture proves the generator rebinds it
//! `mut` first.

#[spacetimedsl::dsl(
    plural_name = audited_entries,
    method(update = true),
    hook(before(update), after(update)),
)]
#[spacetimedb::table(accessor = audited_entry, public)]
pub struct AuditedEntry {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub label: String,

    #[updated_at]
    modified_at: Option<spacetimedb::Timestamp>,
}
