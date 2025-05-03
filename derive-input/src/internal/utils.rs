use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{DeriveInput, Error};

pub(in crate::internal) fn get_table_attribute_macro(
    item: &DeriveInput,
    path: &str,
) -> syn::Result<TokenStream> {
    let mut table = None;

    for attr in item.attrs.iter() {
        match attr.meta.require_list() {
            Ok(list) => {
                if list.path.to_token_stream().to_string().eq(path) {
                    table = Some(attr);
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
    column_name: &Box<str>,
    wrapper_type_name_or_path: &Box<str>,
) -> TokenStream {
    let column_option_name = &format!("{column_name}_option");
    quote! {
        let #column_name = #column_name.into();
        let mut #column_option_name = None;
        if #column_name.is_some() {
            #column_option_name = Some(Into::<#wrapper_type_name_or_path>::into(#column_name.unwrap()).value());
        }
        let #column_name = #column_option_name;
    }
}
