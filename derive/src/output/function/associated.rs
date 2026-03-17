use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, PrettyPlease};
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;

use crate::output::malformed_code_generation_result;

fn build_impl(
    method: &SpacetimeDSLMethod,
    method_args: &[TokenStream],
    context_bound: TokenStream,
    dsl_type: TokenStream,
    context_import: TokenStream,
) -> TokenStream {
    let trait_name = &method.trait_name;
    let trait_dep_paths = &method.trait_dep_paths;
    let method_name = &method.method_name;
    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    quote! {
        impl<T: #context_bound> #trait_name for #dsl_type {
            #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                use #context_import;
                #(use #trait_dep_paths as _;)*
                #method_impl
            }
        }
    }
}

pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let trait_name = &method.trait_name;
    let method_name = &method.method_name;
    let method_args = crate::output::map_args(&method.method_args);
    let return_type = &method.return_type;

    let mut dsl_impl = build_impl(
        method,
        &method_args,
        quote! { spacetimedsl::WriteContext },
        quote! { spacetimedsl::DSL<'_, T> },
        quote! { spacetimedsl::DSLContext },
    );

    if method.read_context_compatible {
        dsl_impl.extend(build_impl(
            method,
            &method_args,
            quote! { spacetimedsl::ReadContext },
            quote! { spacetimedsl::ReadOnlyDSL<'_, T> },
            quote! { spacetimedsl::ReadOnlyDSLContext },
        ));
    }

    let impl_doc = PrettyPlease::default()
        .format_tokens(dsl_impl.clone())
        .unwrap_or_else(|_| panic!("{}", malformed_code_generation_result(dsl_impl.to_string())));

    let doc_comment = format!(
        "{}\n\nImplementation:\n\n```no_run\n{impl_doc}\n```",
        method.doc_comment
    );
    let trait_comment = format!("See [`Self::{method_name}`] for details.");

    Ok(quote! {
        #[doc = #trait_comment]
        pub trait #trait_name {
            #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
            #[doc=#doc_comment]
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type;
        }

        #dsl_impl
    })
}
