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
use syn::{Ident, Path, Type, Visibility, parse_str};

pub(crate) fn output(input: &Table) -> syn::Result<TokenStream> {
    let struct_name = format_ident!("{}", &input.rust_struct.name.to_string());
    let mut wrapper_types = vec![];

    for column in &input.columns {
        match &column.spacetimedsl_column.wrapper_type {
            Some(wrapper_type) => match wrapper_type {
                WrapperType::Wrap(wrapper_type) => {
                    let wrapper_type_impl: TokenStream = parse_str(&wrapper_type.wrapper_impl)?;
                    wrapper_types.push(wrapper_type_impl);
                }
                _ => {}
            },
            None => {}
        }
    }
    let mut table_methods = vec![];
    let mut dsl_methods = vec![];

    dsl_methods.push(build_without_lifetime(&input.spacetimedsl_methods.create)?);
    dsl_methods.push(build_with_lifetime(&input.spacetimedsl_methods.get_all)?);
    dsl_methods.push(build_with_lifetime(&input.spacetimedsl_methods.get_count)?);

    match &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted
    {
        Some(method) => {
            dsl_methods.push(build_internal(method)?);
        }
        None => {}
    }

    match &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted
    {
        Some(method) => {
            dsl_methods.push(build_internal(method)?);
        }
        None => {}
    }

    for execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted {
        dsl_methods.push(build_internal(
            execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted
        )?);
    }

    for execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted {
        dsl_methods.push(build_internal(
            execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted
        )?);
    }

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        dsl_methods.push(get_column_dsl_methods(multi_column_index)?);
    }

    for column in &input.columns {
        table_methods.push(getter(&column.spacetimedsl_column.getter)?);

        match &column.spacetimedsl_column.setter {
            Some(data) => table_methods.push(setter(data)?),
            None => {}
        }

        match &column.spacetimedsl_methods {
            Some(methods) => dsl_methods.push(get_column_dsl_methods(methods)?),
            None => {}
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

            match &methods.update {
                Some(method) => token_streams.push(build_without_lifetime(method)?),
                None => {}
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

    let trait_name = format_ident!("{}", *method.trait_name);
    let paths_of_traits_to_extend: Vec<Path> = method
        .paths_of_traits_to_extend
        .iter()
        .map(|p| parse_str(p).expect("parsing should have worked"))
        .collect();
    let method_name = format_ident!("{}", *method.method_name);

    let mut method_args: Vec<TokenStream> = vec![];
    for method_arg in &method.method_args {
        method_args.push(parse_str(&method_arg)?);
    }

    let return_type: Type = parse_str(&method.return_type)?;
    let method_impl: TokenStream = parse_str(&method.method_impl)?;

    doc_comment = add_impl_doc(
        &trait_name,
        &method_name,
        &method_args,
        &return_type,
        &method_impl,
        doc_comment,
    );

    // TODO: The trait doc comment should link to the method doc comment
    let method = quote! {
        #[doc=#doc_comment]
        pub trait #trait_name: #(#paths_of_traits_to_extend)+* {
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
    
    let paths_of_traits_to_extend: Vec<Path> = method
        .paths_of_traits_to_extend
        .iter()
        .map(|p| parse_str(p).expect("parsing should have worked"))
        .collect();
    let method_name = format_ident!("{}", *method.method_name);

    let mut method_args: Vec<TokenStream> = vec![];
    for method_arg in &method.method_args {
        method_args.push(parse_str(&method_arg)?);
    }

    let return_type: Type = parse_str(&method.return_type)?;
    let method_impl: TokenStream = parse_str(&method.method_impl)?;

    doc_comment = add_impl_doc(
        &trait_name,
        &method_name,
        &method_args,
        &return_type,
        &method_impl,
        doc_comment,
    );

    // TODO: The trait doc comment should link to the method doc comment
    let method = quote! {
        #[doc=#doc_comment]
        pub trait #trait_name: #(#paths_of_traits_to_extend)+* {
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

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build_internal(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
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

    doc_comment = add_impl_doc(
        &trait_name,
        &method_name,
        &method_args,
        &return_type,
        &method_impl,
        doc_comment,
    );

    // TODO: The trait doc comment should link to the method doc comment
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

fn add_impl_doc(
    trait_name: &Ident,
    method_name: &Ident,
    method_args: &Vec<TokenStream>,
    return_type: &Type,
    method_impl: &TokenStream,
    mut doc_comment: String,
) -> String {
    let pretty_please = PrettyPlease::default();
    let implementation_docs = quote! {
        pub trait #trait_name {
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext,Table};
                #method_impl
            }
        }
    };

    let implementation_docs = pretty_please
        .format_tokens(implementation_docs)
        .expect("implementation doc formatting should work");

    doc_comment.push_str(&format!(
        "\n\nImplementation:\n\n```rust\n{implementation_docs}\n```",
    ));

    doc_comment
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
