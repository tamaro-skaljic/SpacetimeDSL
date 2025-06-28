use syn::Path;

#[derive(Clone)]
pub enum RustVisibility {
    /// `pub`
    Public,
    /// `pub(crate)`, `pub(super)` or `pub(in path::to::module)`
    Restricted(Path),
    /// Default
    Private,
}
