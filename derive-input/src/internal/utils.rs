use crate::api::{dsl::wrapper::WrapperType, rust::RustVisibility};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::fmt::Display;
use syn::{parse_str, DeriveInput, Error, Ident, Type, Visibility};

impl RustVisibility {
    pub(in crate::internal) fn map(value: &Visibility) -> RustVisibility {
        match value {
            Visibility::Public(_) => RustVisibility::Public,
            Visibility::Restricted(vis) => {
                RustVisibility::Restricted(vis.path.to_token_stream().to_string().into())
            }
            Visibility::Inherited => RustVisibility::Private,
        }
    }
}

impl WrapperType {
    pub(in crate::internal) fn map(value: &WrapperType) -> Type {
        match value {
            WrapperType::Wrap(w) => parse_str(&w.wrapper_struct_name).expect(&format!(
                "Failed to parse {} as Ident in WrapperType::map for WrapperType::Wrap.",
                &w.wrapper_struct_name
            )),
            WrapperType::Wrapped(w) => parse_str(&w.wrapper_struct_name_or_path).expect(&format!(
                "Failed to parse {} as Path in WrapperType::map for WrapperType::Wrapped.",
                &w.wrapper_struct_name_or_path
            )),
        }
    }
}

pub(in crate::internal) fn get_table_attribute_macro(
    input: &DeriveInput,
    path: &str,
) -> syn::Result<TokenStream> {
    let mut table = None;

    for attr in input.attrs.iter() {
        match attr.meta.require_list() {
            Ok(list) => {
                if list.path.to_token_stream().to_string().eq(path) {
                    table = Some(list.tokens.to_token_stream());
                }
            }
            Err(_) => {}
        }
    }

    match table {
        Some(table) => Ok(table.to_token_stream()),
        None => Err(Error::new(
            Span::call_site(),
            format!("Haven't found #[{path}] attribute macro!"),
        )),
    }
}

pub(in crate::internal) fn wrapper_type_into_option(
    column_name: &Ident,
    wrapper_type_name_or_path: &Type,
) -> TokenStream {
    let column_option_name = &format_ident!("{column_name}_option");
    quote! {
        let #column_name = #column_name.into();
        let mut #column_option_name = None;
        if #column_name.is_some() {
            #column_option_name = Some(Into::<#wrapper_type_name_or_path>::into(#column_name.unwrap()).value());
        }
        let #column_name = #column_option_name;
    }
}

impl Display for RustVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => {
                write!(f, "pub")
            }
            Self::Restricted(str) => {
                let str: &str = str;

                match str {
                    "crate" => {
                        write!(f, "pub (crate)")
                    }
                    "super" => {
                        write!(f, "pub (super)")
                    }
                    str => {
                        write!(f, "pub (in {str})")
                    }
                }
            }
            Self::Private => {
                write!(f, "")
            }
        }
    }
}
