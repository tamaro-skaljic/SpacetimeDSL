use crate::api::rust::visibility::RustVisibility;

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct RustField {
    pub visibility: RustVisibility,
    pub name: Box<str>,
    pub type_name_or_path: Box<str>,
}
