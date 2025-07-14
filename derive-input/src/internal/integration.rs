use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use spacetime_bindings_macro_input::{match_meta, util::check_duplicate};
use crate::internal::dsl::plural_name;
use syn::{
    DeriveInput, Error, Ident,
    meta::parser,
    parse::Parser,
};

#[cfg(test)]
pub fn spacetime_bindings_macro_input<'a>(
    item: &'a DeriveInput,
    dsl_args: &proc_macro2::TokenStream,
) -> syn::Result<(TableArgs, ColumnArgs<'a>)> {
    let input = get_table_attribute_macro(item, dsl_args)?;

    let table_args = TableArgs::parse(input, item)?;

    let (table_args, column_args) = ColumnArgs::parse(table_args, item)?;

    Ok((table_args, column_args))
}

#[cfg(not(test))]
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
    // Parse DSL arguments to extract plural_name
    let plural_name_value = parse_dsl_args(dsl_args)?;
    
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

    // If only one table, return it
    if table_attrs.len() == 1 {
        return Ok(table_attrs[0].clone());
    }

    // For multiple table attributes, try to match based on plural_name
    if let Some(plural_name_ident) = plural_name_value {
        // Convert plural name to singular to match table name
        let singular_name = plural_to_singular(&plural_name_ident.to_string());
        
        for table_attr in &table_attrs {
            if table_contains_name(&table_attr, &singular_name) {
                return Ok(table_attr.clone());
            }
        }
    }

    // Fallback: use deterministic selection to ensure consistency
    let selection_index = deterministic_selection(dsl_args, table_attrs.len());
    Ok(table_attrs[selection_index].clone())
}

// Parse DSL arguments using established parsing patterns
fn parse_dsl_args(args: &proc_macro2::TokenStream) -> syn::Result<Option<Ident>> {
    let mut plural_name_value: Option<Ident> = None;

    parser(|meta| {
        match_meta!(match meta {
            plural_name => {
                check_duplicate(&plural_name_value, &meta)?;
                let value = meta.value()?;
                plural_name_value = Some(value.parse()?);
            }
        });
        Ok(())
    })
    .parse2(args.clone())?;

    Ok(plural_name_value)
}

// Convert plural name to singular (simple heuristic)
fn plural_to_singular(plural: &str) -> String {
    if plural.ends_with("ies") {
        format!("{}y", &plural[..plural.len()-3])
    } else if plural.ends_with("es") && plural.len() > 2 {
        plural[..plural.len()-2].to_string()
    } else if plural.ends_with("s") && plural.len() > 1 {
        plural[..plural.len()-1].to_string()
    } else {
        plural.to_string()
    }
}

// Check if table attribute contains the given name
fn table_contains_name(table_attr: &TokenStream, name: &str) -> bool {
    table_attr.to_string().contains(name)
}

// Deterministic selection based on hash of dsl args
fn deterministic_selection(dsl_args: &proc_macro2::TokenStream, table_count: usize) -> usize {
    let arg_str = dsl_args.to_string();
    let hash: usize = arg_str.chars().map(|c| c as usize).sum();
    hash % table_count
}
