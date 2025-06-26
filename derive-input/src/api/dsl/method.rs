#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct SpacetimeDSLMethod {
    pub doc_comment: Box<str>,
    pub trait_name: Box<str>,
    pub method_name: Box<str>,
    pub method_args: Vec<Box<str>>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}
