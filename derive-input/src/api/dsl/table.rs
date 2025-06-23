use super::reference::ReferencingTable;
use crate::api::dsl::{column::SpacetimeDSLColumnMethods, method::SpacetimeDSLMethod};

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLTable {
    pub plural_name: Box<str>,
    pub is_mutable: bool,
    pub has_created_at_column: bool,
    pub has_modified_at_column: bool,
    pub referencing_tables: Vec<ReferencingTable>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLTableMethods {
    pub create: SpacetimeDSLMethod,
    pub get_all: SpacetimeDSLMethod,
    pub get_count: SpacetimeDSLMethod,
    pub actions_after_delete_one: Option<SpacetimeDSLMethod>,
    pub actions_after_delete_many: Option<SpacetimeDSLMethod>,
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
}
