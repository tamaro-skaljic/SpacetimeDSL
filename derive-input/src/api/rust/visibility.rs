#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub enum RustVisibility {
    /// `pub`
    Public,
    /// `pub(crate)`, `pub(super)` or `pub(in path::to::module)`
    Restricted(Box<str>),
    /// Default
    Private,
}
