#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct TableDSLMethods {
    pub create_row: CreateRowDSLMethod,
    pub get_all_rows: GetAllRowsDSLMethod,
    pub get_count_of_rows: GetCountOfRowsDSLMethod,
    // For multi-column indices
    pub get_many_rows_by: Vec<GetManyRowsByDSLMethod>,
    pub delete_many_rows_by: Vec<DeleteManyRowsByDSLMethod>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct CreateRowDSLMethod {
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
pub struct GetAllRowsDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct GetCountOfRowsDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
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
    pub get_many_rows_by: GetManyRowsByDSLMethod,
    pub delete_many_rows_by: DeleteManyRowsByDSLMethod,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct GetManyRowsByDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct DeleteManyRowsByDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct ColumnDSLMethodsForUniqueConstraints {
    pub get_one_row_option_by: GetOneRowOptionByDSLMethod,
    pub get_many_row_options_by: GetManyRowOptionsByDSLMethod,
    pub update_row_by: UpdateRowByDSLMethod,
    pub delete_one_row_by: DeleteOneRowByDSLMethod,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct GetOneRowOptionByDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct GetManyRowOptionsByDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct UpdateRowByDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct DeleteOneRowByDSLMethod {
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}
