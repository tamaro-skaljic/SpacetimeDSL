use crate::api::db::{index::Index, reducer::ScheduledReducer};

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDBTable {
    pub singular_name: Box<str>,
    pub visibility: SpacetimeDBTableVisibility,
    pub multi_column_indices: Vec<Index>,
    pub scheduled_reducer: Option<ScheduledReducer>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub enum SpacetimeDBTableVisibility {
    Public,
    Private,
}
