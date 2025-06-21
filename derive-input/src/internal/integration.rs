use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::{DeriveInput, Error};

pub(in crate::internal) fn spacetime_bindings_macro_input(
    item: &DeriveInput,
) -> syn::Result<(TableArgs, ColumnArgs)> {
    let input = get_table_attribute_macro(item)?;

    let table_args = TableArgs::parse(input, item)?;

    let (table_args, column_args) = ColumnArgs::parse(table_args, item)?;

    Ok((table_args, column_args))
}

fn get_table_attribute_macro(
    input: &DeriveInput,
) -> syn::Result<TokenStream> {
    let mut table = None;

    for attr in input.attrs.iter() {
        match attr.meta.require_list() {
            Ok(list) => {
                if list.path.to_token_stream().to_string().eq("table")
                    || list
                        .path
                        .to_token_stream()
                        .to_string()
                        .eq("spacetimedb :: table")
                {
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
            format!(
                "Haven't found `#[table]` or `#[spacetimedb::table]` attribute macro! Make sure `#[dsl]` is above it, not below it."
            ),
        )),
    }
}
