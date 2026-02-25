use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, PrettyPlease};
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;
use syn::Ident;

use crate::output::{malformed_code_generation_result, map_args};

pub mod associated;

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = &method.trait_name;
    let paths_of_traits_to_extend = &method.paths_of_traits_to_extend;
    let method_name = &method.method_name;

    let method_args = map_args(&method.method_args);

    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    doc_comment = add_impl_doc(
        trait_name,
        paths_of_traits_to_extend,
        method_name,
        &method_args,
        return_type,
        method_impl,
        doc_comment,
    );

    let trait_comment = format!("See [`Self::{method_name}`] for details.");
    let method = quote! {
        #[doc = #trait_comment]
        pub trait #trait_name {
            #[doc=#doc_comment]
            fn #method_name<'a, T: spacetimedsl::WriteContext>(
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::internal::DSLInternals {}
    };

    Ok(method)
}

fn add_impl_doc(
    trait_name: &Ident,
    paths_of_traits_to_extend: &Vec<syn::Path>,
    method_name: &Ident,
    method_args: &Vec<TokenStream>,
    return_type: &TokenStream,
    method_impl: &TokenStream,
    mut doc_comment: String,
) -> String {
    let pretty_please = PrettyPlease::default();
    let implementation_docs = quote! {
        pub trait #trait_name: #(#paths_of_traits_to_extend)+* {
            fn #method_name<'a, T: spacetimedsl::WriteContext>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #method_impl
            }
        }
    };

    let implementation_docs = pretty_please
        .format_tokens(implementation_docs.clone())
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                malformed_code_generation_result(implementation_docs.to_string())
            )
        });

    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```no_run\n{implementation_docs}\n```",
    ));

    doc_comment
}
