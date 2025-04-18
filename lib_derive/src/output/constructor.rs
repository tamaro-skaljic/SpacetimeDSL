use crate::input::{ColumnSchema, TableSchema};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use super::get_column_type;

pub fn build(table: &mut TableSchema) -> TokenStream {
    let mut constructor_arguments = vec![];
    let mut initializer_arguments = vec![];

    table.columns.iter().for_each(|column| {
        constructor_arguments.push(constructor_arg(column));
        initializer_arguments.push(init_arg(column));
    });

    if constructor_arguments
        .iter()
        .find(|ts| !ts.is_empty())
        .is_none()
    {
        table.no_args_constructor = true;
    }

    let struct_name = &table.struct_name;

    quote! {
        impl #struct_name {
            pub fn new(
                #(#constructor_arguments)*
            ) -> #struct_name {
                #struct_name {
                    #(#initializer_arguments)*
                }
            }
        }
    }
}

fn constructor_arg(column: &ColumnSchema) -> TokenStream {
    let constructor_arg;

    if column.is_auto_inc {
        constructor_arg = TokenStream::default();
    } else {
        let column_name = &column.column_name;
        let column_type = get_column_type(column);

        constructor_arg = quote! {
            #column_name: #column_type,
        }
    }

    constructor_arg
}

//region init arg

fn init_arg(column: &ColumnSchema) -> TokenStream {
    if column.is_auto_inc {
        auto_inc_init_arg(&column.column_name, &column.column_type)
    } else {
        if column.column_type_wrapper.is_some() {
            column_type_wrapper_init_arg(&column.column_name)
        } else {
            normal_init_arg(&column.column_name)
        }
    }
}

fn auto_inc_init_arg(column_name: &Ident, column_type: &Type) -> TokenStream {
    quote! {
        #column_name: #column_type::default(),
    }
}

fn column_type_wrapper_init_arg(column_name: &Ident) -> TokenStream {
    quote! {
        #column_name: #column_name.into().value(),
    }
}

fn normal_init_arg(column_name: &Ident) -> TokenStream {
    quote! {
        #column_name: #column_name.clone(),
    }
}

//endregion init arg
