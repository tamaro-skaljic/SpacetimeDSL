use crate::api::db::{index::Index, reducer::ScheduledReducer};

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct SpacetimeDBTable {
    pub singular_name: Box<str>,
    pub visibility: SpacetimeDBTableVisibility,
    pub multi_column_indices: Vec<Index>,
    pub scheduled_reducer: Option<ScheduledReducer>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub enum SpacetimeDBTableVisibility {
    Public,
    Private,
}
