use {crate::api::rust::visibility::RustVisibility, proc_macro2::TokenStream, syn::Ident};

/// The setter `set_<column>` a column gets when its field is not private.
#[derive(Clone)]
pub struct Setter {
    /// What the column references and the strategies it declares, when it has a foreign
    /// key; empty otherwise.
    pub doc_comment: String,
    /// The visibility of the field, which the setter takes over.
    pub method_visibility: RustVisibility,
    /// `set_<column>`.
    pub method_name: Ident,
    /// The parameter of the setter, including its name and type.
    pub method_arg: TokenStream,
    /// What the setter returns: the value the column held before.
    pub return_type: TokenStream,
    /// The body of the setter.
    pub method_impl: TokenStream,
}
