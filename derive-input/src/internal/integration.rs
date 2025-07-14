use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::{DeriveInput, Error};

#[cfg(test)]
pub fn spacetime_bindings_macro_input<'a>(
    item: &'a DeriveInput,
) -> syn::Result<Vec<(TableArgs, ColumnArgs<'a>)>> {
    get_all_table_attributes(item)
}

#[cfg(not(test))]
pub(in crate::internal) fn spacetime_bindings_macro_input<'a>(
    item: &'a DeriveInput,
) -> syn::Result<Vec<(TableArgs, ColumnArgs<'a>)>> {
    get_all_table_attributes(item)
}

fn get_all_table_attributes<'a>(
    input: &'a DeriveInput,
) -> syn::Result<Vec<(TableArgs, ColumnArgs<'a>)>> {
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

    // Parse all table attributes and return them
    let mut results = Vec::new();
    for table_attr in table_attrs {
        let table_args = TableArgs::parse(table_attr, input)?;
        let (table_args, column_args) = ColumnArgs::parse(table_args, input)?;
        results.push((table_args, column_args));
    }

    Ok(results)
}
