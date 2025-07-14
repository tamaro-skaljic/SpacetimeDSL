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

    // For multiple table attributes, match each DSL macro with the appropriate table
    // based on the naming convention in the DSL arguments (e.g., plural_name)
    if let Some(plural_name) = extract_plural_name(&dsl_args.to_string()) {
        for table_attr in &table_attrs {
            let table_str = table_attr.to_string();
            let base_name = if plural_name.contains("tables") {
                plural_name.replace("tables", "table")
            } else if plural_name.ends_with("s") && plural_name.len() > 1 {
                plural_name[..plural_name.len()-1].to_string()
            } else {
                plural_name.clone()
            };
            
            if table_str.contains(&base_name) {
                return Ok(table_attr.clone());
            }
        }
    }

    // Fallback: use deterministic selection based on dsl args to ensure
    // different DSL macros select different tables consistently
    let selection_index = simple_hash(&dsl_args.to_string()) % table_attrs.len();
    Ok(table_attrs[selection_index].clone())
}

// Extract plural_name from DSL arguments
fn extract_plural_name(dsl_args: &str) -> Option<String> {
    // Look for pattern like "plural_name = some_name"
    if let Some(start) = dsl_args.find("plural_name") {
        if let Some(equals_pos) = dsl_args[start..].find('=') {
            let after_equals = &dsl_args[start + equals_pos + 1..];
            // Find the next word/identifier, more carefully
            let trimmed = after_equals.trim();
            // Split by whitespace and commas to find the identifier
            for part in trimmed.split(|c: char| c.is_whitespace() || c == ',') {
                let cleaned = part.trim();
                if !cleaned.is_empty() && cleaned.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(cleaned.to_string());
                }
            }
        }
    }
    None
}

// Simple hash function to deterministically select table based on dsl args
fn simple_hash(s: &str) -> usize {
    s.chars().map(|c| c as usize).sum()
}
