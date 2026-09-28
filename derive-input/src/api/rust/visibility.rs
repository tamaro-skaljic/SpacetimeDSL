use syn::Path;

/// The visibility of a struct, a field or a generated accessor.
#[derive(Clone)]
pub enum RustVisibility {
    /// `pub`
    Public,
    /// `pub(crate)`, `pub(super)` or `pub(in path::to::module)`, holding what is inside the
    /// parentheses.
    Restricted(Path),
    /// No visibility modifier.
    Private,
}
