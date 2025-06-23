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
