use proc_macro2::TokenStream;
use syn::{Ident, Path};

#[derive(Clone)]
pub struct SpacetimeDSLMethod {
    pub doc_comment: String,
    pub trait_name: Ident,
    pub paths_of_traits_to_extend: Vec<Path>,
    pub method_name: Ident,
    pub method_args: Vec<SpacetimeDSLArg>,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}

#[derive(Clone)]
pub struct SpacetimeDSLArg {
    pub is_mut: bool,
    pub is_option: bool,
    pub arg_name: Ident,
    pub arg_type: SpacetimeDSLArgType,
}

#[derive(Clone)]
pub enum SpacetimeDSLArgType {
    Normal(TokenStream),
    Wrapped {
        wrapped_type: TokenStream,
        actual_type: TokenStream,
    },
}
