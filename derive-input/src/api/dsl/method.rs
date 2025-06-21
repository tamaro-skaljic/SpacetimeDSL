#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLMethod {
    pub doc_comment: Box<str>,
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_args: Vec<Box<str>>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
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

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub enum SpacetimeDSLColumnMethods {
    ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex),
    ForIndex(SpacetimeDSLColumnMethodsForIndex),
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLColumnMethodsForUniqueIndex {
    pub get_one_option: SpacetimeDSLMethod,
    pub update: Option<SpacetimeDSLMethod>,
    pub delete_one: SpacetimeDSLMethod,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLColumnMethodsForIndex {
    pub get_many: SpacetimeDSLMethod,
    pub delete_many: SpacetimeDSLMethod,
}
