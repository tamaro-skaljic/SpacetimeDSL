use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rust_format::{Formatter, PrettyPlease};
use spacetimedsl_derive_input::api::{
    Table,
    dsl::{
        column::SpacetimeDSLColumnMethods, getter::Getter, method::SpacetimeDSLMethod,
        setter::Setter, wrapper::WrapperType,
    },
};
use syn::{Ident, Visibility, parse_str};

pub(crate) fn output(
    input: &Table,
    should_generate_wrapper_types_and_accessors: bool,
) -> syn::Result<TokenStream> {
    let struct_name = format_ident!("{}", &input.rust_struct.name.to_string());
    let mut wrapper_types = vec![];

    // Only generate wrapper types if this is the last DSL attribute to avoid conflicts
    if should_generate_wrapper_types_and_accessors {
        for column in &input.columns {
            if let Some(WrapperType::Created(wrapper_type)) =
                &column.spacetimedsl_column.wrapper_type
            {
                wrapper_types.push(&wrapper_type.wrapper_impl);
            }
        }
    }

    let mut table_methods = vec![];
    let mut dsl_methods = vec![];

    dsl_methods.push(build_without_lifetime(&input.spacetimedsl_methods.create)?);
    dsl_methods.push(build_with_lifetime(&input.spacetimedsl_methods.get_all)?);
    dsl_methods.push(build_with_lifetime(&input.spacetimedsl_methods.get_count)?);

    if let Some(method) = &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted
    {
        dsl_methods.push(build_internal(method)?);
    }

    if let Some(method) = &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted {
        dsl_methods.push(build_internal(method)?);
    }

    for execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted {
        dsl_methods.push(build_internal(execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted)?);
    }

    for execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted {
        dsl_methods.push(build_internal(execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted)?);
    }

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        dsl_methods.push(get_column_dsl_methods(multi_column_index)?);
    }

    for column in &input.columns {
        if should_generate_wrapper_types_and_accessors {
            table_methods.push(getter(&column.spacetimedsl_column.getter)?);

            if let Some(data) = &column.spacetimedsl_column.setter {
                table_methods.push(setter(data)?)
            }
        }

        if let Some(methods) = &column.spacetimedsl_methods {
            dsl_methods.push(get_column_dsl_methods(methods)?)
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

fn get_column_dsl_methods(methods: &SpacetimeDSLColumnMethods) -> syn::Result<TokenStream> {
    let mut token_streams = vec![];

    match methods {
        SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => {
            token_streams.push(build_without_lifetime(&methods.get_one_option)?);

            if let Some(method) = &methods.update {
                token_streams.push(build_without_lifetime(method)?)
            };

            token_streams.push(build_without_lifetime(&methods.delete_one)?);
        }
        SpacetimeDSLColumnMethods::ForIndex(methods) => {
            token_streams.push(build_with_lifetime(&methods.get_many)?);

            token_streams.push(build_with_lifetime(&methods.delete_many)?);
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

    // TODO: The trait doc comment should link to the method doc comment
    let method = quote! {
        pub trait #trait_name: #(#paths_of_traits_to_extend)+* {
            #[doc=#doc_comment]
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
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

    // TODO: The trait doc comment should link to the method doc comment
    let method = quote! {
        pub trait #trait_name: #(#paths_of_traits_to_extend)+* {
            #[allow(clippy::too_many_arguments)]
            #[doc=#doc_comment]
            fn #method_name(
                &self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    };

    Ok(method)
}

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build_internal(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
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

    // TODO: The trait doc comment should link to the method doc comment
    let method = quote! {
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
        .format_tokens(implementation_docs)
        .expect("implementation doc formatting should work");

    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```no_run\n{implementation_docs}\n```",
    ));

    doc_comment
}

fn getter(getter: &Getter) -> syn::Result<TokenStream> {
    let method_name = &getter.method_name;
    let return_type = &getter.return_type;
    let method_impl = &getter.method_impl;

    Ok(quote! {
        pub fn #method_name(&self) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}

fn setter(setter: &Setter) -> syn::Result<TokenStream> {
    let method_visibility: Visibility = parse_str(&setter.method_visibility.to_string())?;
    let method_name = &setter.method_name;
    let method_arg = &setter.method_arg;
    let return_type = &setter.return_type;
    let method_impl = &setter.method_impl;

    Ok(quote! {
        #method_visibility fn #method_name(&mut self, #method_arg) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}

fn map_method_args(method: &SpacetimeDSLMethod) -> Vec<TokenStream> {
    let mut method_args = vec![];

    for method_arg in &method.method_args {
        let arg_name = &method_arg.arg_name;
        let arg_type = &method_arg.arg_type;
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
