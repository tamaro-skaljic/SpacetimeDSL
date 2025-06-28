use syn::Path;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum RustVisibility {
    /// `pub`
    Public,
    /// `pub(crate)`, `pub(super)` or `pub(in path::to::module)`
    Restricted(Path),
    /// Default
    Private,
}
