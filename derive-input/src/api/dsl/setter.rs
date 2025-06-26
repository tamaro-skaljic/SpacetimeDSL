use crate::api::rust::visibility::RustVisibility;

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct Setter {
    pub method_visibility: RustVisibility,
    pub method_name: Box<str>,
    pub method_arg: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}
