use crate::input::{Column, Table};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

mod accessor_methods;
mod create_row_methods;
mod delete_many_rows_by_methods;
mod delete_one_row_by_methods;
mod get_all_rows_method;
mod get_count_of_rows_method;
mod get_many_row_options_by_methods;
mod get_many_rows_by_methods;
mod get_one_row_option_by_methods;
mod update_row_by_methods;
mod wrapper_types;

pub fn output(input: Table) -> syn::Result<TokenStream> {
    let mut output: Vec<TokenStream> = vec![];

    output.push(wrapper_types::build(&input));
    output.push(accessor_methods::build(&input));

    output.push(create_row_methods::build(&input));
    output.push(get_one_row_option_by_methods::build(&input));
    output.push(get_many_row_options_by_methods::build(&input));
    output.push(get_many_rows_by_methods::build(&input));
    output.push(get_all_rows_method::build(&input));
    output.push(get_count_of_rows_method::build(&input));
    output.push(update_row_by_methods::build(&input));
    output.push(delete_one_row_by_methods::build(&input));
    output.push(delete_many_rows_by_methods::build(&input));

    Ok(quote! {
        use spacetimedsl::Wrapper as _;
        use spacetimedb::{DbContext as _, Table as _};
        #(#output)*
    })
}

pub fn get_column_type(column: &Column) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        let column_type = &column.column_type;
        quote! {
            #column_type
        }
    } else {
        let column_type = column
            .column_type_wrapper
            .as_ref()
            .expect("Expected column_type_wrapper in get_column_type(), found None!");

        if is_option(column) {
            quote! {
                impl Into<Option<#column_type>>
            }
        } else {
            quote! {
                impl Into<#column_type>
            }
        }
    }
}

pub fn get_column_value(column: &Column) -> TokenStream {
    let column_name = &column.column_name;
    if column.column_type_wrapper.is_none() {
        quote! {
            #column_name
        }
    } else {
        if is_option(column) {
            let column_value_name = format_ident!("{column_name}_value");
            quote! {
                #column_name: #column_value_name
            }
        } else {
            quote! {
                #column_name.into().value()
            }
        }
    }
}

pub fn is_option(column: &Column) -> bool {
    column
        .column_type
        .to_token_stream()
        .to_string()
        .contains("Option")
}
