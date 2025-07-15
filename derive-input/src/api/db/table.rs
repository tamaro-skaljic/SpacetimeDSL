use crate::api::db::{index::Index, reducer::ScheduledReducer};
use syn::Ident;

pub struct SpacetimeDBTable {
    pub singular_name: Ident,
    pub visibility: SpacetimeDBTableVisibility,
    pub multi_column_indices: Vec<Index>,
    pub scheduled_reducer: Option<ScheduledReducer>,
}

pub enum SpacetimeDBTableVisibility {
    Public,
    Private,
}
