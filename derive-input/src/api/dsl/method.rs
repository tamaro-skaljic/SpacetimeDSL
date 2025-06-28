use proc_macro2::TokenStream;
use syn::{Ident, Path};

#[derive(Clone, Debug)]
pub struct SpacetimeDSLMethod {
    pub doc_comment: Box<str>,
    pub trait_name: Ident,
    pub paths_of_traits_to_extend: Vec<Path>,
    pub method_name: Ident,
    pub method_args: Vec<TokenStream>,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}
