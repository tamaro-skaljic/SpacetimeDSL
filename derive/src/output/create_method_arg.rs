use proc_macro2::TokenStream;
use quote::quote;

pub fn build(create_dsl_method_arg: &TokenStream) -> syn::Result<TokenStream> {
    Ok(quote! {
        #create_dsl_method_arg
    })
}
