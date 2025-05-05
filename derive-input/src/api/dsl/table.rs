use super::reference::Reference;

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLTable {
    pub plural_name: Box<str>,
    pub is_mutable: bool,
    pub has_created_at_column: bool,
    pub has_modified_at_column: bool,
    pub references: Vec<Reference>,
}
