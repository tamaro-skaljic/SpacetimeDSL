use proc_macro2::TokenStream;
use quote::quote;
use rust_format::{Formatter, PrettyPlease};
use spacetimedsl_derive_input::api::dsl::{getter::Getter, setter::Setter};
use syn::{Ident, Visibility, parse_str, token};

use crate::output::malformed_code_generation_result;

pub fn getter(getter: &Getter) -> syn::Result<TokenStream> {
    let method_name = &getter.method_name;
    let self_token = quote! { &self };
    let return_type = &getter.return_type;
    let method_impl = &getter.method_impl;
    let doc_comment = add_impl_doc(
        &Visibility::Public(token::Pub::default()),
        method_name,
        &self_token,
        &TokenStream::default(),
        return_type,
        method_impl,
    );

    Ok(quote! {
        #[doc=#doc_comment]
        pub fn #method_name(#self_token) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}

pub fn setter(setter: &Setter) -> syn::Result<TokenStream> {
    let method_visibility: Visibility = parse_str(&setter.method_visibility.to_string())?;
    let method_name = &setter.method_name;
    let self_token = quote! { &mut self };
    let method_arg = &setter.method_arg;
    let return_type = &setter.return_type;
    let method_impl = &setter.method_impl;
    let doc_comment = add_impl_doc(
        &method_visibility,
        method_name,
        &self_token,
        method_arg,
        return_type,
        method_impl,
    );

    Ok(quote! {
        #[doc=#doc_comment]
        #method_visibility fn #method_name(#self_token, #method_arg) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}

fn add_impl_doc(
    method_visibility: &Visibility,
    method_name: &Ident,
    self_token: &TokenStream,
    method_arg: &TokenStream,
    return_type: &TokenStream,
    method_impl: &TokenStream,
) -> String {
    let pretty_please = PrettyPlease::default();
    let implementation_docs = quote! {
        #method_visibility fn #method_name(#self_token, #method_arg) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
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

    let doc_comment = format!("\n\nImplementation:\n\n```no_run\n{implementation_docs}\n```",);

    doc_comment
}
