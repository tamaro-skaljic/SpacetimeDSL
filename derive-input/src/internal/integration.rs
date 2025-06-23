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

fn get_table_attribute_macro(input: &DeriveInput) -> syn::Result<TokenStream> {
    let mut table = None;

    // TODO: Because multiple `#[spacetimedb::table]` attribute macros are possible per struct, this should get the first one below the calling DeriveInput.
    for attr in &input.attrs {
        match attr.meta.require_list() {
            Ok(list) => {
                if list.path.to_token_stream().to_string().eq("table")
                    || list
                        .path
                        .to_token_stream()
                        .to_string()
                        .eq("spacetimedb :: table")
                {
                    table = Some(list.tokens.clone());
                }
            }
            Err(_) => {}
        }
    }

    match table {
        Some(table) => Ok(table),
        None => Err(Error::new(
            // TODO: span should be the dsl attribute macro
            Span::call_site(),
            format!(
                "Haven't found `#[table]`/`#[spacetimedb::table]` attribute macro! Make sure `#[dsl]`/`#[spacetimedsl::dsl]` is directly above one."
            ),
        )),
    }
}
