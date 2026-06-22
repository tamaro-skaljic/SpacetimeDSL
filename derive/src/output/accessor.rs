use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::{getter::Getter, mut_getter::MutGetter, setter::Setter};
use syn::{Ident, Visibility, parse_str, token};

use crate::output::doc_comment;

pub(super) enum Accessor<'a> {
    Getter(&'a Getter),
    MutGetter(&'a MutGetter),
    Setter(&'a Setter),
}

pub(super) fn build(accessor: Accessor<'_>) -> syn::Result<TokenStream> {
    let accessor = accessor.definition()?;
    let method = accessor.method_tokens();
    let doc_comment = doc_comment::implementation_doc_comment(method.clone());

    Ok(quote! {
        #[doc = #doc_comment]
        #method
    })
}

struct AccessorDefinition<'a> {
    method_visibility: Visibility,
    method_name: &'a Ident,
    method_args: Vec<TokenStream>,
    return_type: &'a TokenStream,
    method_impl: &'a TokenStream,
}

impl<'a> Accessor<'a> {
    fn definition(&self) -> syn::Result<AccessorDefinition<'a>> {
        match self {
            Self::Getter(getter) => Ok(AccessorDefinition {
                method_visibility: Visibility::Public(token::Pub::default()),
                method_name: &getter.method_name,
                method_args: vec![quote! { &self }],
                return_type: &getter.return_type,
                method_impl: &getter.method_impl,
            }),
            Self::MutGetter(mut_getter) => Ok(AccessorDefinition {
                method_visibility: parse_str(&mut_getter.method_visibility.to_string())?,
                method_name: &mut_getter.method_name,
                method_args: vec![quote! { &mut self }],
                return_type: &mut_getter.return_type,
                method_impl: &mut_getter.method_impl,
            }),
            Self::Setter(setter) => Ok(AccessorDefinition {
                method_visibility: parse_str(&setter.method_visibility.to_string())?,
                method_name: &setter.method_name,
                method_args: vec![quote! { &mut self }, setter.method_arg.clone()],
                return_type: &setter.return_type,
                method_impl: &setter.method_impl,
            }),
        }
    }
}

impl AccessorDefinition<'_> {
    fn method_tokens(&self) -> TokenStream {
        let method_visibility = &self.method_visibility;
        let method_name = self.method_name;
        let method_args = &self.method_args;
        let return_type = self.return_type;
        let method_impl = self.method_impl;

        quote! {
            #method_visibility fn #method_name(#(#method_args),*) -> #return_type {
                use spacetimedsl::Wrapper;
                #method_impl
            }
        }
    }
}
