use crate::api::{
    Table,
    db::table::SpacetimeDBTable,
    dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
};
use quote::format_ident;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use spacetime_bindings_macro_input::{match_meta, util::check_duplicate};
use syn::{DeriveInput, Ident, meta::parser, parse::Parser};

pub(in crate::internal) fn try_parse(
    args: proc_macro2::TokenStream,
    input: &DeriveInput,
    all_tables: &[(TableArgs, ColumnArgs<'_>)],
) -> syn::Result<Table> {
    // Select the appropriate table based on DSL args (plural_name)
    let (table_args, column_args) = select_table_for_dsl(&args, all_tables)?;

    let rust_struct = crate::internal::rust::table::map_struct(&input);

    let spacetimedb_table = SpacetimeDBTable::map(table_args);

    // Parse plural_name from args for DSL table
    let plural_name = parse_plural_name_from_args(&args)?
        .ok_or_else(|| syn::Error::new(
            proc_macro2::Span::call_site(),
            format_args!("PluralName must be set in `#[dsl(plural_name = PluralName)]`, e.g. `plural_name = {}s`.", spacetimedb_table.singular_name),
        ))?;

    let (spacetimedb_table, spacetimedsl_table) =
        SpacetimeDSLTable::try_parse_with_plural_name(column_args, spacetimedb_table, plural_name)?;

    let (spacetimedb_table, columns, internal_columns) = super::column::try_parse(
        &column_args,
        &rust_struct,
        spacetimedb_table,
        &spacetimedsl_table,
    )?;

    let primary_key_column = internal_columns
        .iter()
        .find(|c| c.rust_field_name.to_string().eq(&"id"))
        .expect("should have a primary key");

    let spacetimedsl_methods = SpacetimeDSLTableMethods::try_parse(
        &rust_struct,
        &spacetimedb_table,
        &spacetimedsl_table,
        &columns,
        &internal_columns,
        primary_key_column,
    )?;

    Ok(Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        spacetimedsl_methods,
    })
}

// Select the appropriate table based on DSL args (plural_name)
fn select_table_for_dsl<'a>(
    dsl_args: &proc_macro2::TokenStream,
    all_tables: &'a [(TableArgs, ColumnArgs<'a>)],
) -> syn::Result<&'a (TableArgs, ColumnArgs<'a>)> {
    if all_tables.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "No table attributes found"
        ));
    }

    // If only one table, return it
    if all_tables.len() == 1 {
        return Ok(&all_tables[0]);
    }

    // Parse plural_name from DSL args
    if let Ok(Some(plural_name)) = parse_plural_name_from_args(dsl_args) {
        // Convert plural name to singular to match table name
        let singular_name = plural_to_singular(&plural_name.to_string());

        // Try to match based on table name
        for table_entry in all_tables {
            let (table_args, _) = table_entry;
            if table_args.name.to_string() == singular_name {
                return Ok(table_entry);
            }
        }
    }

    // Fallback: use deterministic selection to ensure consistency
    let selection_index = deterministic_selection(dsl_args, all_tables.len());
    Ok(&all_tables[selection_index])
}

// Parse plural_name from DSL arguments
fn parse_plural_name_from_args(args: &proc_macro2::TokenStream) -> syn::Result<Option<Ident>> {
    use crate::internal::dsl::plural_name;
    
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
        format!("{}y", &plural[..plural.len() - 3])
    } else if plural.ends_with("es") && plural.len() > 2 {
        plural[..plural.len() - 2].to_string()
    } else if plural.ends_with("s") && plural.len() > 1 {
        plural[..plural.len() - 1].to_string()
    } else {
        plural.to_string()
    }
}

// Deterministic selection based on hash of dsl args
fn deterministic_selection(dsl_args: &proc_macro2::TokenStream, table_count: usize) -> usize {
    let arg_str = dsl_args.to_string();
    let hash: usize = arg_str.chars().map(|c| c as usize).sum();
    hash % table_count
}

pub fn rm_rsharp(ident: syn::Ident) -> syn::Ident {
    let mut ident_as_str = ident.to_string();
    if ident_as_str.starts_with("r#") {
        format_ident!("{}", ident_as_str.split_off(2))
    } else {
        ident
    }
}
