use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::{DeriveInput, Error};

pub(in crate::internal) fn spacetime_bindings_macro_input<'a>(
    item: &'a DeriveInput,
    plural_name: &syn::Ident,
) -> syn::Result<(TableArgs, ColumnArgs<'a>)> {
    select_table_with_heuristics(item, plural_name)
}

fn get_all_table_attributes<'a>(
    input: &'a DeriveInput,
) -> syn::Result<Vec<(TableArgs, ColumnArgs<'a>)>> {
    // Find all table attributes
    let mut table_attrs = Vec::new();
    for attr in &input.attrs {
        if let Ok(list) = attr.meta.require_list() {
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
    }

    if table_attrs.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "Haven't found `#[table]`/`#[spacetimedb::table]` attribute macro! Make sure `#[dsl]`/`#[spacetimedsl::dsl]` is directly above one.".to_string(),
        ));
    }

    // Parse all table attributes and return them
    let mut results = vec![];
    for table_attr in table_attrs {
        let table_args = TableArgs::parse(table_attr, input)?;
        let (table_args, column_args) = ColumnArgs::parse(table_args, input)?;
        results.push((table_args, column_args));
    }

    Ok(results)
}

// Select table using heuristics or index-based fallback
fn select_table_with_heuristics<'a>(
    input: &'a DeriveInput,
    plural_name: &syn::Ident,
) -> syn::Result<(TableArgs, ColumnArgs<'a>)> {
    let all_tables = get_all_table_attributes(input)?;

    if all_tables.is_empty() {
        return Err(Error::new(Span::call_site(), "No table attributes found"));
    }

    // If only one table, return it
    if all_tables.len() == 1 {
        return Ok(all_tables.into_iter().next().unwrap());
    }

    // Use heuristics since plural_name is provided
    let plural_str = plural_name.to_string();

    // Try exact match first
    for (i, table_entry) in all_tables.iter().enumerate() {
        let (table_args, _) = table_entry;
        let table_name = table_args.name.to_string();
        if table_name == plural_str {
            return Ok(all_tables.into_iter().nth(i).unwrap());
        }
    }

    // Try intelligent matching: find table name that is most similar
    // This handles cases like test_tables1 -> test_table1
    for (i, table_entry) in all_tables.iter().enumerate() {
        let (table_args, _) = table_entry;
        let table_name = table_args.name.to_string();

        // Check if the plural name matches the table name with some smart heuristics
        if is_plural_match(&plural_str, &table_name) {
            return Ok(all_tables.into_iter().nth(i).unwrap());
        }
    }

    // Fallback: use deterministic selection based on plural_name
    let selection_index =
        deterministic_selection_by_name(&plural_name.to_string(), all_tables.len());

    Ok(all_tables.into_iter().nth(selection_index).unwrap())
}

// Check if a plural name matches a table name using intelligent heuristics
fn is_plural_match(plural_name: &str, table_name: &str) -> bool {
    // Remove trailing digits/numbers from both
    let plural_base = remove_trailing_digits(plural_name);
    let table_base = remove_trailing_digits(table_name);

    // Check if the suffixes (digits) match
    let plural_suffix = &plural_name[plural_base.len()..];
    let table_suffix = &table_name[table_base.len()..];

    if plural_suffix != table_suffix {
        return false;
    }

    // Now try different pluralization rules
    // 1. Direct conversion: tables -> table
    if plural_base == "tables" && table_base == "table" {
        return true;
    }

    // 2. Standard pluralization rules
    let singular = plural_to_singular(&plural_base);
    if singular == table_base {
        return true;
    }

    // 3. Check if table is a substring of plural (e.g., "table" in "test_tables")
    if plural_base.contains(&table_base) {
        return true;
    }

    false
}

// Convert plural name to singular (simple heuristic)
fn plural_to_singular(plural: &str) -> String {
    if let Some(stripped) = plural.strip_suffix("ies") {
        format!("{stripped}y")
    } else if plural.ends_with("es") && plural.len() > 2 {
        plural[..plural.len() - 2].to_string()
    } else if plural.ends_with("s") && plural.len() > 1 {
        plural[..plural.len() - 1].to_string()
    } else {
        plural.to_string()
    }
}

// Remove trailing digits from a string
fn remove_trailing_digits(s: &str) -> String {
    let mut result = s.to_string();
    while let Some(last_char) = result.chars().last() {
        if last_char.is_ascii_digit() {
            result.pop();
        } else {
            break;
        }
    }
    result
}

// Deterministic selection based on hash of plural name
fn deterministic_selection_by_name(name: &str, table_count: usize) -> usize {
    let hash: usize = name.chars().map(|c| c as usize).sum();
    hash % table_count
}
