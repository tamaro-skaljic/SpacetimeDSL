use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;

use crate::output::{doc_comment, map_args};

pub mod associated;

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let method_name = &method.method_name;
    let additional_paths_to_use = &method.additional_paths_to_use;

    let method_args = map_args(&method.method_args);

    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    let doc_comment = doc_comment::doc_comment_with_implementation(
        &method.doc_comment,
        quote! {
            impl crate::spacetimedsl::internal::DSLInternals {
                pub fn #method_name<'a, T: crate::spacetimedsl::WriteContext>(
                    #(#method_args),*
                ) -> #return_type {
                    use ::spacetimedsl::Wrapper;
                    use spacetimedb::{DbContext, Table as _};
                    #(use #additional_paths_to_use as _;)*
                    #method_impl
                }
            }
        },
    );

    let method = quote! {
        impl crate::spacetimedsl::internal::DSLInternals {
            #[doc = #doc_comment]
            pub fn #method_name<'a, T: crate::spacetimedsl::WriteContext>(
                #(#method_args),*
            ) -> #return_type {
                use ::spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #method_impl
            }
        }
    };

    Ok(method)
}
