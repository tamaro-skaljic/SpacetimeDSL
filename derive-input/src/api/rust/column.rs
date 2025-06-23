use crate::api::rust::visibility::RustVisibility;

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct RustField {
    pub visibility: RustVisibility,
    pub name: Box<str>,
    pub type_name_or_path: Box<str>,
}
