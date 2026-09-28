use proc_macro2::TokenStream;
use syn::Ident;

/// The getter `get_<column>` every column gets.
#[derive(Clone)]
pub struct Getter {
    /// `get_<column>`.
    pub method_name: Ident,
    /// What the getter returns: the wrapper type for a column with a wrapper (in an `Option`
    /// for an optional column), a reference to the column type otherwise.
    pub return_type: TokenStream,
    /// The body of the getter.
    pub method_impl: TokenStream,
}
