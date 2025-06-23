#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub enum RustVisibility {
    /// `pub`
    Public,
    /// `pub(crate)`, `pub(super)` or `pub(in path::to::module)`
    Restricted(Box<str>),
    /// Default
    Private,
}
