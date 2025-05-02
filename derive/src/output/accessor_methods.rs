use super::{get_column_type, into_option, is_option};
use crate::input::{Column, Table};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Visibility;

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

//region getter

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

//endregion getter

//region setter

fn setter(setter: &Setter) -> TokenStream {
    let method_visibility = &setter.method_visibility;
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

//endregion setter
