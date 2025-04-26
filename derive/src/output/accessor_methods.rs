use crate::input::{Column, Table};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Visibility;

use super::{get_column_type, into_option, is_option};

pub fn build(table: &Table) -> TokenStream {
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

fn return_type(column: &Column) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        normal_return_type(column)
    } else {
        wrapped_return_type(column)
    }
}

fn normal_return_type(column: &Column) -> TokenStream {
    let column_type = &column.column_type;
    quote! {
        &#column_type
    }
}

fn wrapped_return_type(column: &Column) -> TokenStream {
    let column_type = column
        .column_type_wrapper
        .as_ref()
        .expect("Expected column_type_wrapper in wrapped_return_type(), found None!");

    if is_option(column) {
        quote! {
            Option<#column_type>
        }
    } else {
        quote! {
            #column_type
        }
    }
}

//endregion return type

//region getter implementation

fn getter(column: &Column) -> TokenStream {
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

fn getter_impl(column: &Column) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        normal_getter_impl(column)
    } else {
        getter_impl_for_wrapper_types(column)
    }
}

fn normal_getter_impl(column: &Column) -> TokenStream {
    let column_name = &column.column_name;

    quote! {
        &self.#column_name
    }
}

fn getter_impl_for_wrapper_types(column: &Column) -> TokenStream {
    let column_type = column
        .column_type_wrapper
        .as_ref()
        .expect("Expected column_type_wrapper in getter_impl_for_wrapper_types(), found None!");
    let column_name = &column.column_name;

    if is_option(column) {
        quote! {
            if self.#column_name.is_none() {
                None
            } else {
                Some(#column_type::new(self.#column_name.unwrap()))
            }
        }
    } else {
        quote! {
            #column_type::new(self.#column_name.clone())
        }
    }
}

//endregion getter implementation

//region setter implementation

fn setter(column: &Column) -> TokenStream {
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
            #setter_impl
        }
    }
}

fn setter_impl(column: &Column) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        normal_setter_impl(column)
    } else {
        setter_impl_for_wrapper_types(column)
    }
}

fn normal_setter_impl(column: &Column) -> TokenStream {
    let column_name = &column.column_name;

    quote! {
        self.#column_name = #column_name;
    }
}

fn setter_impl_for_wrapper_types(column: &Column) -> TokenStream {
    let column_name = &column.column_name;

    if is_option(column) {
        let into_option = into_option(column);
        let column_value_name = format_ident!("{column_name}_value");
        quote! {
            #into_option
            self.#column_name = #column_value_name;
        }
    } else {
        quote! {
            self.#column_name = #column_name.into().value();
        }
    }
}

//endregion setter implementation
