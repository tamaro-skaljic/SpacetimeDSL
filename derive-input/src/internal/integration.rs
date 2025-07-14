use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::{DeriveInput, Error};

pub(in crate::internal) fn spacetime_bindings_macro_input<'a>(
    item: &'a DeriveInput,
    dsl_args: &proc_macro2::TokenStream,
) -> syn::Result<(TableArgs, ColumnArgs<'a>)> {
    let input = get_table_attribute_macro(item, dsl_args)?;

    let table_args = TableArgs::parse(input, item)?;

    let (table_args, column_args) = ColumnArgs::parse(table_args, item)?;

    Ok((table_args, column_args))
}

fn get_table_attribute_macro(input: &DeriveInput, dsl_args: &proc_macro2::TokenStream) -> syn::Result<TokenStream> {
    // Parse the dsl arguments to get the plural_name or other identifying info
    let dsl_args_str = dsl_args.to_string();
    
    // Find all table attributes
    let mut table_attrs = Vec::new();
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
                    table_attrs.push(list.tokens.clone());
                }
            }
            Err(_) => {}
        }
    }

    if table_attrs.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            format!(
                "Haven't found `#[table]`/`#[spacetimedb::table]` attribute macro! Make sure `#[dsl]`/`#[spacetimedsl::dsl]` is directly above one."
            ),
        ));
    }

    // Use a simple heuristic: if there are multiple table attributes and multiple dsl args,
    // try to match them based on order or content
    if table_attrs.len() == 1 {
        return Ok(table_attrs[0].clone());
    }

    // For multiple table attributes, use a deterministic selection based on dsl args
    // This is a simple heuristic - select table based on hash of dsl args
    let selection_index = simple_hash(&dsl_args_str) % table_attrs.len();
    Ok(table_attrs[selection_index].clone())
}

// Simple hash function to deterministically select table based on dsl args
fn simple_hash(s: &str) -> usize {
    s.chars().map(|c| c as usize).sum()
}
