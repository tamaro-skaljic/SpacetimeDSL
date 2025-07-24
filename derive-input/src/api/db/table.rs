use crate::api::db::{index::Index, reducer::ScheduledReducer};
use syn::Ident;

#[derive(Debug, Clone)]
pub struct SpacetimeDBTable {
    pub singular_name: Ident,
    pub visibility: SpacetimeDBTableVisibility,
    pub multi_column_indices: Vec<Index>,
    pub scheduled_reducer: Option<ScheduledReducer>,
}

#[derive(Debug, Clone)]
pub enum SpacetimeDBTableVisibility {
    Public,
    Private,
}
