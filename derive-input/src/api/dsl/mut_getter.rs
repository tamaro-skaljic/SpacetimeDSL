use proc_macro2::TokenStream;
use syn::Ident;

use crate::api::rust::visibility::RustVisibility;

#[derive(Clone)]
pub struct MutGetter {
    pub method_visibility: RustVisibility,
    pub method_name: Ident,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}
