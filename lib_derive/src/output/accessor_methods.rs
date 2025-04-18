use crate::input::{ColumnSchema, TableSchema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Visibility;

use super::get_column_type;

pub fn build(table: &TableSchema) -> TokenStream {
    let struct_name = &table.struct_name;
    let mut accessors: Vec<TokenStream> = vec![];

    table.columns.iter().for_each(|column| {
        accessors.push(getter(column));
        accessors.push(setter(column));
    });

    quote! {
        impl #struct_name {
            #(#accessors)*
        }
    }
}

//region return type

fn return_type(column: &ColumnSchema) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        normal_return_type(column)
    } else {
        wrapped_return_type(column)
    }
}

fn normal_return_type(column: &ColumnSchema) -> TokenStream {
    let column_type = &column.column_type;
    quote! {
        &#column_type
    }
}

fn wrapped_return_type(column: &ColumnSchema) -> TokenStream {
    let column_type = column.column_type_wrapper.as_ref().unwrap();
    quote! {
        #column_type
    }
}

//endregion return type

//region getter implementation

fn getter(column: &ColumnSchema) -> TokenStream {
    let column_name = &column.column_name;
    let method_name = format_ident!("get_{column_name}");
    let return_type = return_type(column);
    let getter_impl = getter_impl(column);

    quote! {
        pub fn #method_name(&self) -> #return_type {
            #getter_impl
        }
    }
}

fn getter_impl(column: &ColumnSchema) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        normal_getter_impl(column)
    } else {
        getter_impl_for_wrapper_types(column)
    }
}

fn normal_getter_impl(column: &ColumnSchema) -> TokenStream {
    let column_name = &column.column_name;

    quote! {
        &self.#column_name
    }
}

fn getter_impl_for_wrapper_types(column: &ColumnSchema) -> TokenStream {
    let column_type = column.column_type_wrapper.as_ref().unwrap();
    let column_name = &column.column_name;

    quote! {
        #column_type::new(self.#column_name.clone())
    }
}

//endregion getter implementation

//region setter implementation

fn setter(column: &ColumnSchema) -> TokenStream {
    let visibility = &column.visibility;

    match &visibility {
        Visibility::Inherited => {
            return TokenStream::default();
        }
        _ => {}
    }

    let column_name = &column.column_name;
    let method_name = format_ident!("set_{column_name}");
    let column_type = get_column_type(column);
    let setter_impl = setter_impl(column);

    quote! {
        #visibility fn #method_name(&mut self, #column_name: #column_type) {
            #setter_impl;
        }
    }
}

fn setter_impl(column: &ColumnSchema) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        normal_setter_impl(column)
    } else {
        setter_impl_for_wrapper_types(column)
    }
}

fn normal_setter_impl(column: &ColumnSchema) -> TokenStream {
    let column_name = &column.column_name;

    quote! {
        self.#column_name = #column_name
    }
}

fn setter_impl_for_wrapper_types(column: &ColumnSchema) -> TokenStream {
    let column_name = &column.column_name;

    quote! {
        self.#column_name = #column_name.value()
    }
}

//endregion setter implementation
