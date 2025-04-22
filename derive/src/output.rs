use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

use crate::input::{ColumnSchema, TableSchema};

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

pub fn output(table: TableSchema) -> TokenStream {
    let mut output: Vec<TokenStream> = vec![];

    output.push(wrapper_types::build(&table));
    output.push(accessor_methods::build(&table));

    output.push(create_row_methods::build(&table));
    output.push(get_one_row_option_by_methods::build(&table));
    output.push(get_many_row_options_by_methods::build(&table));
    output.push(get_many_rows_by_methods::build(&table));
    output.push(get_all_rows_method::build(&table));
    output.push(get_count_of_rows_method::build(&table));
    output.push(update_row_by_methods::build(&table));
    output.push(delete_one_row_by_methods::build(&table));
    output.push(delete_many_rows_by_methods::build(&table));

    quote! {
        use spacetimedsl::Wrapper as _;
        use spacetimedb::{DbContext as _, Table as _};
        #(#output)*
    }
}

pub fn get_column_type(column: &ColumnSchema) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        let column_type = &column.column_type;
        quote! {
            #column_type
        }
    } else {
        let column_type = column.column_type_wrapper.as_ref().unwrap();

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

pub fn get_column_value(column: &ColumnSchema) -> TokenStream {
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

pub fn is_option(column: &ColumnSchema) -> bool {
    column
        .column_type
        .to_token_stream()
        .to_string()
        .contains("Option")
}

pub fn into_option(column: &ColumnSchema) -> TokenStream {
    let column_name = &column.column_name;
    let column_value_name = format_ident!("{column_name}_value");
    let wrapper_type = column.column_type_wrapper.as_ref();

    let wrapper_type = wrapper_type.unwrap();
    quote! {
        let #column_name = #column_name.into();
        let mut #column_value_name = None;
        if #column_name.is_some() {
            #column_value_name = Some(Into::<#wrapper_type>::into(#column_name.unwrap()).value());
        }
    }
}
