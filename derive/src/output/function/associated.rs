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
    doc_comment: &str,
) -> TokenStream {
    let method_name = &method.method_name;
    let additional_paths_to_use = &method.additional_paths_to_use;
    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    quote! {
        impl<T: #context_bound> #dsl_type {
            #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
            #[doc = #doc_comment]
            pub fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use ::spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #(use #additional_paths_to_use as _;)*
                #method_impl
            }
        }
    }
}

pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let method_name = &method.method_name;
    let method_args = crate::output::map_args(&method.method_args);
    let return_type = &method.return_type;

    // Build the write-context impl to generate the doc comment from it.
    let dsl_impl_for_doc = {
        let method_impl = &method.method_impl;
        let additional_paths_to_use = &method.additional_paths_to_use;
        quote! {
            impl<T: crate::spacetimedsl::WriteContext> crate::spacetimedsl::DSL<'_, T> {
                #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
                pub fn #method_name<'a>(
                    &'a self,
                    #(#method_args),*
                ) -> #return_type {
                    use ::spacetimedsl::Wrapper;
                    use spacetimedb::{DbContext, Table as _};
                    #(use #additional_paths_to_use as _;)*
                    #method_impl
                }
            }
        }
    };

    let impl_doc = PrettyPlease::default()
        .format_tokens(dsl_impl_for_doc.clone())
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                malformed_code_generation_result(dsl_impl_for_doc.to_string())
            )
        });

    let doc_comment = format!(
        "{}\n\nImplementation:\n\n```no_run\n{impl_doc}\n```",
        method.doc_comment
    );

    let mut output = build_impl(
        method,
        &method_args,
        quote! { crate::spacetimedsl::WriteContext },
        quote! { crate::spacetimedsl::DSL<'_, T> },
        &doc_comment,
    );

    if method.read_context_compatible {
        output.extend(build_impl(
            method,
            &method_args,
            quote! { crate::spacetimedsl::ReadContext },
            quote! { crate::spacetimedsl::ReadOnlyDSL<'_, T> },
            &doc_comment,
        ));
    }

    Ok(output)
}
