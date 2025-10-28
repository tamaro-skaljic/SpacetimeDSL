use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, PrettyPlease};
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;
use syn::Ident;

pub mod associated;

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = &method.trait_name;
    let paths_of_traits_to_extend = &method.paths_of_traits_to_extend;
    let method_name = &method.method_name;

    let method_args = map_method_args(method);

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
            fn #method_name<'a>(
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
            fn #method_name<'a>(
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
        .unwrap_or_else(|_| panic!("implementation doc formatting should work - got:\n\n```no_run\n{implementation_docs}\n```"));

    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```no_run\n{implementation_docs}\n```",
    ));

    doc_comment
}

fn map_method_args(method: &SpacetimeDSLMethod) -> Vec<TokenStream> {
    let mut method_args = vec![];

    for method_arg in &method.method_args {
        let arg_name = &method_arg.arg_name;
        let arg_type = match &method_arg.arg_type {
            spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLArgType::Normal(
                actual_type,
            ) => actual_type,
            spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLArgType::Wrapped {
                wrapped_type: _,
                actual_type,
            } => actual_type,
        };

        if method_arg.is_mut {
            method_args.push(quote! {
                mut #arg_name: #arg_type
            });
        } else {
            method_args.push(quote! {
                #arg_name: #arg_type
            });
        }
    }

    method_args
}
