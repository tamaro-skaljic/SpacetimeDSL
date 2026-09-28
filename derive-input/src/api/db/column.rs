use crate::api::db::index::Index;

/// What `#[spacetimedb::table]` declares for one field of the struct.
#[derive(Clone)]
pub struct SpacetimeDBColumn {
    /// Whether the field carries `#[primary_key]`, or is the `id: u8` column SpacetimeDSL
    /// injects into a singleton.
    pub is_primary_key: bool,
    /// The index whose only column this field is: `#[primary_key]`, `#[unique]`,
    /// `#[index(...)]` on the field, or an `index(...)` of the table naming the field alone.
    /// `None` when there is no such index.
    pub single_column_index: Option<Index>,
    /// Whether the field carries `#[auto_inc]`.
    pub is_auto_inc: bool,
}
