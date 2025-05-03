#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLTable {
    pub plural_name: Box<str>,
    // TODO: Implement On Delete Hooks
    pub on_delete_hooks: Vec<OnDeleteHook>,
    pub is_mutable: bool,
    pub has_created_at_column: bool,
    pub has_modified_at_column: bool,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct OnDeleteHook {
    pub function_path: Box<str>,
}
