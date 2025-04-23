use crate::input::{ColumnSchema, TableSchema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

pub fn build(table: &TableSchema) -> TokenStream {
    let mut wrapper_types = vec![];

    table.columns.iter().for_each(|column| {
        wrapper_types.push(wrapper_type(table, column));
    });

    quote! {
        #(#wrapper_types)*
    }
}

fn wrapper_type(table: &TableSchema, column: &ColumnSchema) -> TokenStream {
    if column.column_type_wrapper.is_none() {
        return TokenStream::default();
    }

    let mut path = None;

    match column
        .column_type_wrapper
        .as_ref()
        .expect("Expected column_type_wrapper in wrapper_type(), found None!")
    {
        Type::Path(tp) => {
            let _ = tp.path.require_ident().inspect(|i| {
                path = Some(format_ident!("{}", i));
            });
        }
        _ => {}
    }

    if path.is_none() {
        return TokenStream::default();
    }

    let wrapper_struct_name = path
        .as_ref()
        .expect("Expected path in wrapper_type(), found None!");

    if wrapper_struct_name.to_string().contains("::") {
        return TokenStream::default();
    }

    let wrapped_struct_name = &column.column_type;
    let struct_name = &table.struct_name;
    let getter_name = format_ident!("get_{}", &column.column_name);

    quote! {
        #[derive(Clone, Debug, PartialEq, spacetimedb::SpacetimeType)]
        pub struct #wrapper_struct_name {
            value: #wrapped_struct_name,
        }

        impl From<&#struct_name> for #wrapper_struct_name {
            fn from(value: &#struct_name) -> Self {
                value.#getter_name()
            }
        }

        impl From<&#struct_name> for Option<#wrapper_struct_name> {
            fn from(value: &#struct_name) -> Option<#wrapper_struct_name> {
                Some(value.#getter_name())
            }
        }

        impl spacetimedsl::Wrapper<#wrapped_struct_name, #wrapper_struct_name> for #wrapper_struct_name {
            fn new(value: #wrapped_struct_name) -> Self {
                Self { value }
            }
            fn default() -> Self {
                Self {
                    value: #wrapped_struct_name::default(),
                }
            }
            fn value(&self) -> #wrapped_struct_name {
                self.value.clone()
            }
        }
    }
}
