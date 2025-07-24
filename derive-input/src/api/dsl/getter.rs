use proc_macro2::TokenStream;
use syn::Ident;

#[derive(Clone)]
pub struct Getter {
    pub method_name: Ident,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}
