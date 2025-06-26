use crate::api::rust::visibility::RustVisibility;

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct RustStruct {
    pub visibility: RustVisibility,
    pub name: Box<str>,
}