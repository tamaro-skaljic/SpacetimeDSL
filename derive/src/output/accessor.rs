use {
    crate::output::doc_comment,
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    spacetimedsl_derive_input::api::{
        dsl::{getter::Getter, mut_getter::MutGetter, setter::Setter},
        runtime,
    },
    syn::Ident,
};

pub enum Accessor<'a> {
    Getter(&'a Getter),
    MutGetter(&'a MutGetter),
    Setter(&'a Setter),
}

pub fn build(accessor: Accessor<'_>) -> TokenStream {
    let accessor = accessor.definition();
    let method = accessor.method_tokens();
    let doc_comment = doc_comment::implementation_doc_comment(method.clone());

    quote! {
        #[doc = #doc_comment]
        #method
    }
}

struct AccessorDefinition<'a> {
    method_visibility: TokenStream,
    method_name: &'a Ident,
    method_args: Vec<TokenStream>,
    return_type: &'a TokenStream,
    method_impl: &'a TokenStream,
}

impl<'a> Accessor<'a> {
    fn definition(&self) -> AccessorDefinition<'a> {
        match self {
            Self::Getter(getter) => AccessorDefinition {
                method_visibility: quote! { pub },
                method_name: &getter.method_name,
                method_args: vec![quote! { &self }],
                return_type: &getter.return_type,
                method_impl: &getter.method_impl,
            },
            Self::MutGetter(mut_getter) => AccessorDefinition {
                method_visibility: mut_getter.method_visibility.to_token_stream(),
                method_name: &mut_getter.method_name,
                method_args: vec![quote! { &mut self }],
                return_type: &mut_getter.return_type,
                method_impl: &mut_getter.method_impl,
            },
            Self::Setter(setter) => AccessorDefinition {
                method_visibility: setter.method_visibility.to_token_stream(),
                method_name: &setter.method_name,
                method_args: vec![quote! { &mut self }, setter.method_arg.clone()],
                return_type: &setter.return_type,
                method_impl: &setter.method_impl,
            },
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
        let wrapper_trait_import = runtime::wrapper_trait_import();

        quote! {
            #method_visibility fn #method_name(#(#method_args),*) -> #return_type {
                #wrapper_trait_import
                #method_impl
            }
        }
    }
}
