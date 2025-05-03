use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::{
    Table,
    dsl::{
        getter::Getter,
        method::{SpacetimeDSLColumnMethods, SpacetimeDSLMethod},
        setter::Setter,
        wrapper::WrapperType,
    },
};

pub(crate) fn output(input: &Table) -> syn::Result<TokenStream> {
    let mut token_streams = vec![];

    token_streams.push(build_without_lifetime(&input.spacetimedsl_methods.create));
    token_streams.push(build_with_lifetime(&input.spacetimedsl_methods.get_all));
    token_streams.push(build_with_lifetime(&input.spacetimedsl_methods.get_count));

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        token_streams.push(get_column_dsl_methods(multi_column_index));
    }

    for column in &input.columns {
        column
            .spacetimedsl_column
            .wrapper_type
            .as_ref()
            .map(|wrapper_type| match wrapper_type {
                WrapperType::Wrap(wrapper_type) => {
                    let wrapper_type_impl = &wrapper_type.wrapper_impl;
                    token_streams.push(quote! {
                        #wrapper_type_impl
                    });
                }
                _ => {}
            });

        token_streams.push(getter(&column.spacetimedsl_column.getter));

        column
            .spacetimedsl_column
            .setter
            .as_ref()
            .map(|s| token_streams.push(setter(s)));

        column
            .spacetimedsl_methods
            .as_ref()
            .map(|single_column_index| {
                token_streams.push(get_column_dsl_methods(single_column_index))
            });
    }

    Ok(quote! {
        use spacetimedsl::Wrapper as _;
        use spacetimedb::{DbContext as _, Table as _};
        #(#token_streams)*
    })
}

fn get_column_dsl_methods(index: &SpacetimeDSLColumnMethods) -> TokenStream {
    let mut token_streams = vec![];

    match index {
        SpacetimeDSLColumnMethods::ForUniqueIndex(index) => {
            token_streams.push(build_without_lifetime(&index.get_one_option));

            index
                .update
                .as_ref()
                .map(|update| token_streams.push(build_without_lifetime(update)));

            token_streams.push(build_without_lifetime(&index.delete_one));
        }
        SpacetimeDSLColumnMethods::ForIndex(index) => {
            token_streams.push(build_with_lifetime(&index.get_many));

            token_streams.push(build_with_lifetime(&index.delete_many));
        }
    };

    quote! {
        #(#token_streams)*
    }
}

// get_all, get_count, get_many, delete_many
fn build_with_lifetime(method: &SpacetimeDSLMethod) -> TokenStream {
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

// create, get_one_option, update, delete_one
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

fn getter(getter: &Getter) -> TokenStream {
    let method_name = &getter.method_name;
    let return_type = &getter.return_type;
    let method_impl = &getter.method_impl;

    quote! {
        pub fn #method_name(&self) -> #return_type {
            #method_impl
        }
    }
}

fn setter(setter: &Setter) -> TokenStream {
    let method_visibility = &setter.method_visibility.to_string();
    let method_name = &setter.method_name;
    let method_arg = &setter.method_arg;
    let return_type = &setter.return_type;
    let method_impl = &setter.method_impl;

    quote! {
        #method_visibility fn #method_name(&mut self, #method_arg) -> #return_type {
            #method_impl
        }
    }
}
