use crate::input::dsl::method::SpacetimeDSLMethod;
use crate::input::{Column, Table};
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, format_ident, quote};

mod accessor_methods;
mod wrapper_types;

pub fn output(input: Table) -> syn::Result<TokenStream> {
    let mut output: Vec<TokenStream> = vec![];

    output.push(wrapper_types::build(&input));
    output.push(accessor_methods::build(&input));

    Ok(quote! {
        use spacetimedsl::Wrapper as _;
        use spacetimedb::{DbContext as _, Table as _};
        #(#output)*
    })
}

// get_many, delete_many, get_many_options
pub fn build_with_lifetime(method: &SpacetimeDSLMethod) -> TokenStream {
    let doc_comment = &method.doc_comment;
    let trait_name = &method.trait_name;
    let method_name = &method.method_name;
    let method_args = &method.method_args;
    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    quote! {
        #[doc=#doc_comment]
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#doc_comment]
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}

// get_one_option, update, delete_one, create
pub fn build_without_lifetime(method: &SpacetimeDSLMethod) -> TokenStream {
    let doc_comment = &method.doc_comment;
    let trait_name = &method.trait_name;
    let method_name = &method.method_name;
    let method_args = &method.method_args;
    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    quote! {
        #[doc=#doc_comment]
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#doc_comment]
            fn #method_name(
                &self,
                #(#method_args),*
            ) -> #return_type {
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
