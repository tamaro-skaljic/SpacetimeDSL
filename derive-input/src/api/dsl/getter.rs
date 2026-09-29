use {proc_macro2::TokenStream, syn::Ident};

/// The getter `get_<column>`, which every column gets except the primary key SpacetimeDSL
/// injects into a singleton.
#[derive(Clone)]
pub struct Getter {
    /// What the column references and the strategies it declares, when it has a foreign
    /// key; empty otherwise.
    pub doc_comment: String,
    /// `get_<column>`.
    pub method_name: Ident,
    /// What the getter returns: the wrapper type for a column with a wrapper (in an `Option`
    /// for an optional column), a reference to the column type otherwise.
    pub return_type: TokenStream,
    /// The body of the getter.
    pub method_impl: TokenStream,
}
