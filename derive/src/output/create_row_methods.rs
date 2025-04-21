use crate::input::{ColumnSchema, TableSchema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use super::get_column_type;

pub fn build(table: &TableSchema) -> TokenStream {
    let table_name = &table.singular_table_name;
    let struct_name = &table.struct_name;
    let trait_name = format_ident!("Create{struct_name}");
    let comment = format!("Create a {struct_name}.");
    let method_name = format_ident!("create_{table_name}");
    let try_insert_error_generic_type = format_ident!("{table_name}__TableHandle");

    let mut method_arguments = vec![];
    let mut initializer_arguments = vec![];

    table.columns.iter().for_each(|column| {
        method_arguments.push(method_arg(column));
        initializer_arguments.push(init_arg(column));
    });

    let trait_definition = quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name(
                &self,
                #(#method_arguments)*
            ) -> Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>> {
                let #table_name = #struct_name {
                    #(#initializer_arguments)*
                };

                return self
                        .ctx()
                        .db()
                        .#table_name()
                        .try_insert(#table_name);
            }
        }
    };

    quote! {
        #trait_definition

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}

//region method arg

fn method_arg(column: &ColumnSchema) -> TokenStream {
    let method_arg;

    if column.is_auto_inc {
        method_arg = TokenStream::default();
    } else {
        let column_name = &column.column_name;
        let column_type = get_column_type(column);

        method_arg = quote! {
            #column_name: #column_type,
        }
    }

    method_arg
}

//endregion method arg

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
        #column_name,
    }
}

//endregion init arg
