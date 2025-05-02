#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct TableDSLMethods {
    pub create_row: DSLMethod,
    pub get_all_rows: DSLMethod,
    pub get_count_of_rows: DSLMethod,
    // For multi-column indices
    pub get_many_rows_by: Vec<DSLMethod>,
    pub delete_many_rows_by: Vec<DSLMethod>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct DSLMethod {
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
pub enum ColumnDSLMethods {
    ForBtreeIndex(ColumnDSLMethodsForBtreeIndices),
    ForUniqueConstraint(ColumnDSLMethodsForUniqueConstraints),
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct ColumnDSLMethodsForBtreeIndices {
    pub get_many_rows_by: DSLMethod,
    pub delete_many_rows_by: DSLMethod,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct ColumnDSLMethodsForUniqueConstraints {
    pub get_one_row_option_by: DSLMethod,
    pub get_many_row_options_by: DSLMethod,
    pub update_row_by: DSLMethod,
    pub delete_one_row_by: DSLMethod,
}
