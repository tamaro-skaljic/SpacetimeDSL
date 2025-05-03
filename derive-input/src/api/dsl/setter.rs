use crate::api::rust::RustVisibility;

/// TODO: Doc comment field
#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct Setter {
    pub method_visibility: RustVisibility,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}
