use crate::api::db::index::Index;

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SpacetimeDBColumn {
    pub is_primary_key: bool,
    pub single_column_index: Option<Index>,
    pub is_auto_inc: bool,
}
