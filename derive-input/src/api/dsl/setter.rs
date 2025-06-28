use proc_macro2::TokenStream;
use syn::Ident;

use crate::api::rust::visibility::RustVisibility;


pub struct Setter {
    pub method_visibility: RustVisibility,
    pub method_name: Ident,
    pub method_arg: TokenStream,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}
