use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, PrettyPlease};

pub fn build(create_dsl_method_arg: &TokenStream) -> syn::Result<TokenStream> {
    let doc_comment = add_impl_doc(create_dsl_method_arg);

    Ok(quote! {
        #[doc=#doc_comment]
        #create_dsl_method_arg
    })
}

fn add_impl_doc(create_dsl_method_arg: &TokenStream) -> String {
    let pretty_please = PrettyPlease::default();

    let implementation_docs = pretty_please
        .format_tokens(create_dsl_method_arg.clone())
        .unwrap_or_else(|_| panic!("implementation doc formatting should work - got:\n\n```no_run\n{create_dsl_method_arg}\n```"));

    let doc_comment = format!("\n\nImplementation:\n\n```no_run\n{implementation_docs}\n```",);

    doc_comment
}
