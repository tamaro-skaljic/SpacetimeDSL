use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rust_format::{Formatter, PrettyPlease};
use spacetimedsl_derive_input::api::{
    Table,
    dsl::{
        getter::Getter,
        method::{SpacetimeDSLColumnMethods, SpacetimeDSLMethod},
        setter::Setter,
        wrapper::WrapperType,
    },
};
use syn::{Type, Visibility, parse_str};

pub(crate) fn output(input: &Table) -> syn::Result<TokenStream> {
    let struct_name = format_ident!("{}", &input.rust_struct.name.to_string());
    let mut wrapper_types = vec![];

    for column in &input.columns {
        if column.spacetimedsl_column.wrapper_type.is_some() {
            match column.spacetimedsl_column.wrapper_type.as_ref().unwrap() {
                WrapperType::Wrap(wrapper_type) => {
                    let wrapper_type_impl: TokenStream = parse_str(&wrapper_type.wrapper_impl)?;
                    wrapper_types.push(wrapper_type_impl);
                }
                _ => {}
            }
        }
    }
    let mut table_methods = vec![];
    let mut dsl_methods = vec![];

    dsl_methods.push(build_without_lifetime(&input.spacetimedsl_methods.create)?);
    dsl_methods.push(build_with_lifetime(&input.spacetimedsl_methods.get_all)?);
    dsl_methods.push(build_with_lifetime(&input.spacetimedsl_methods.get_count)?);
    dsl_methods.push(build_internal(
        &input.spacetimedsl_methods.actions_after_delete_one,
    )?);
    dsl_methods.push(build_internal(
        &input.spacetimedsl_methods.actions_after_delete_many,
    )?);

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        dsl_methods.push(get_column_dsl_methods(multi_column_index)?);
    }

    for column in &input.columns {
        table_methods.push(getter(&column.spacetimedsl_column.getter)?);
        if column.spacetimedsl_column.setter.is_some() {
            table_methods.push(setter(column.spacetimedsl_column.setter.as_ref().unwrap())?);
        }

        if column.spacetimedsl_methods.is_some() {
            dsl_methods.push(get_column_dsl_methods(
                column.spacetimedsl_methods.as_ref().unwrap(),
            )?);
        }
    }

    Ok(quote! {
        #(#wrapper_types)*

        impl #struct_name {
            #(#table_methods)*
        }

        #(#dsl_methods)*
    })
}

fn get_column_dsl_methods(index: &SpacetimeDSLColumnMethods) -> syn::Result<TokenStream> {
    let mut token_streams = vec![];

    match index {
        SpacetimeDSLColumnMethods::ForUniqueIndex(index) => {
            token_streams.push(build_without_lifetime(&index.get_one_option)?);
            if index.update.is_some() {
                token_streams.push(build_without_lifetime(index.update.as_ref().unwrap())?);
            }
            token_streams.push(build_without_lifetime(&index.delete_one)?);
        }
        SpacetimeDSLColumnMethods::ForIndex(index) => {
            token_streams.push(build_with_lifetime(&index.get_many)?);

            token_streams.push(build_with_lifetime(&index.delete_many)?);
        }
    };

    Ok(quote! {
        #(#token_streams)*
    })
}

// get_all, get_count, get_many, delete_many
fn build_with_lifetime(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = format_ident!("{}", *method.trait_name);
    let method_name = format_ident!("{}", *method.method_name);

    let mut method_args: Vec<TokenStream> = vec![];
    for method_arg in &method.method_args {
        method_args.push(parse_str(&method_arg)?);
    }

    let return_type: Type = parse_str(&method.return_type)?;
    let method_impl: TokenStream = parse_str(&method.method_impl)?;

    let pretty_please = PrettyPlease::default();
    let implementation_docs = quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    };
    let implementation_docs = pretty_please.format_tokens(implementation_docs).unwrap();
    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```rust\n{implementation_docs}\n```",
    ));

    let method = quote! {
        #[doc=#doc_comment]
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#doc_comment]
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    };

    Ok(method)
}

// create, get_one_option, update, delete_one
pub fn build_without_lifetime(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = format_ident!("{}", *method.trait_name);
    let method_name = format_ident!("{}", *method.method_name);

    let mut method_args: Vec<TokenStream> = vec![];
    for method_arg in &method.method_args {
        method_args.push(parse_str(&method_arg)?);
    }

    let return_type: Type = parse_str(&method.return_type)?;
    let method_impl: TokenStream = parse_str(&method.method_impl)?;

    let pretty_please = PrettyPlease::default();
    let implementation_docs = quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            fn #method_name(
                &self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    };
    let implementation_docs = pretty_please.format_tokens(implementation_docs).unwrap();
    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```rust\n{implementation_docs}\n```",
    ));

    let method = quote! {
        #[doc=#doc_comment]
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#doc_comment]
            fn #method_name(
                &self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    };

    Ok(method)
}

// actions_after_delete_one, actions_after_delete_many
pub fn build_internal(method: &Option<SpacetimeDSLMethod>) -> syn::Result<TokenStream> {
    //TODO: if method.is_none() {
    return Ok(TokenStream::default());
    //}

    let method = method.as_ref().unwrap();

    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = format_ident!("{}", *method.trait_name);
    let method_name = format_ident!("{}", *method.method_name);

    let mut method_args: Vec<TokenStream> = vec![];
    for method_arg in &method.method_args {
        method_args.push(parse_str(&method_arg)?);
    }

    let return_type: Type = parse_str(&method.return_type)?;
    let method_impl: TokenStream = parse_str(&method.method_impl)?;

    let pretty_please = PrettyPlease::default();
    let implementation_docs = quote! {
        pub trait #trait_name {
            fn #method_name(
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::internal::DSLInternals {}
    };
    let implementation_docs = pretty_please.format_tokens(implementation_docs).unwrap();
    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```rust\n{implementation_docs}\n```",
    ));

    let method = quote! {
        #[doc=#doc_comment]
        pub trait #trait_name {
            #[doc=#doc_comment]
            fn #method_name(
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::internal::DSLInternals {}
    };

    Ok(method)
}

fn getter(getter: &Getter) -> syn::Result<TokenStream> {
    let method_name = format_ident!("{}", *getter.method_name);
    let return_type: Type = parse_str(&getter.return_type)?;
    let method_impl: TokenStream = parse_str(&getter.method_impl)?;

    Ok(quote! {
        pub fn #method_name(&self) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}

fn setter(setter: &Setter) -> syn::Result<TokenStream> {
    let method_visibility: Visibility = parse_str(&setter.method_visibility.to_string())?;
    let method_name = format_ident!("{}", *setter.method_name);
    let method_arg: TokenStream = parse_str(&setter.method_arg)?;
    let return_type: Type = parse_str(&setter.return_type)?;
    let method_impl: TokenStream = parse_str(&setter.method_impl)?;

    Ok(quote! {
        #method_visibility fn #method_name(&mut self, #method_arg) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}
