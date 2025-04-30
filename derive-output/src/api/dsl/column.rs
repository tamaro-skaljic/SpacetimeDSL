use super::method::ColumnDSLMethods;

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct DSLColumn {
    pub getter: Getter,
    // Only Some(T) if mutable
    pub setter: Option<Setter>,
    // Only Some(T) if there is a single-column index referencing the column
    pub dsl_methods: Option<ColumnDSLMethods>,
    // Only Some(T) if its wrapped.
    pub wrapper_type: Option<WrapperType>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct Getter {
    pub method_name: Box<str>,
    pub return_type: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct Setter {
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct WrapperType {
    pub wrapper_struct_name: Box<str>,
    pub wrapped_type_name: Box<str>,
}
