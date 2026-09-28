use {
    crate::api::db::{index::Index, reducer::ScheduledReducer},
    syn::Ident,
};

/// What `#[spacetimedb::table]` declares for the struct.
#[derive(Clone)]
pub struct SpacetimeDBTable {
    /// The `accessor` of the table, such as `entity`.
    pub singular_name: Ident,
    /// Whether clients can read the table.
    pub visibility: SpacetimeDBTableVisibility,
    /// The indices which cover more than one column, in declaration order. An index covering
    /// one column is held by that column's `SpacetimeDBColumn::single_column_index` instead.
    pub multi_column_indices: Vec<Index>,
    /// `Some` for a table with `scheduled(...)`.
    pub scheduled_reducer: Option<ScheduledReducer>,
}

/// Whether clients can read the table.
#[derive(Clone)]
pub enum SpacetimeDBTableVisibility {
    /// The table carries `public`.
    Public,
    /// The table carries `private`, or neither.
    Private,
}
