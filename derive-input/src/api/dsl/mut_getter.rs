use {crate::api::rust::visibility::RustVisibility, proc_macro2::TokenStream, syn::Ident};

/// The getter `get_<column>_mut` a column gets when its field is not private and it has
/// no wrapper type.
#[derive(Clone)]
pub struct MutGetter {
    /// The visibility of the field, which the mut getter takes over.
    pub method_visibility: RustVisibility,
    /// `get_<column>_mut`.
    pub method_name: Ident,
    /// What the mut getter returns: a mutable reference to the column type.
    pub return_type: TokenStream,
    /// The body of the mut getter.
    pub method_impl: TokenStream,
}
