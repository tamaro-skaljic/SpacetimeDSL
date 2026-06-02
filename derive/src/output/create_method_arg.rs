use proc_macro2::TokenStream;
use quote::quote;

use crate::output::doc_comment;

pub fn build(create_dsl_method_arg: &TokenStream) -> syn::Result<TokenStream> {
    let doc_comment = doc_comment::implementation_doc_comment(create_dsl_method_arg.clone());

    Ok(quote! {
        #[doc = #doc_comment]
        #create_dsl_method_arg
    })
}
