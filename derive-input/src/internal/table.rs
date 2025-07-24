use crate::api::{
    Table,
    db::table::SpacetimeDBTable,
    dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
};
use quote::format_ident;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::DeriveInput;

pub(in crate::internal) fn try_parse(
    input: &DeriveInput,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
    plural_name: syn::Ident,
    unique_indices: Vec<syn::Ident>,
) -> syn::Result<Table> {
    let rust_struct = crate::internal::rust::table::map_struct(input);

    let spacetimedb_table = SpacetimeDBTable::map(table_args);

    let (spacetimedb_table, spacetimedsl_table) =
        SpacetimeDSLTable::try_parse(column_args, spacetimedb_table, plural_name, unique_indices)?;

    let (
        spacetimedb_table,
        columns,
        primary_key_column,
        internal_columns,
        internal_primary_key_column,
    ) = super::column::try_parse(
        column_args,
        &rust_struct,
        spacetimedb_table,
        &spacetimedsl_table,
    )?;

    let (spacetimedsl_methods, spacetimedsl_table) = SpacetimeDSLTableMethods::try_parse(
        &rust_struct,
        &spacetimedb_table,
        spacetimedsl_table,
        &columns,
        &internal_columns,
        &internal_primary_key_column,
    )?;

    Ok(Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        primary_key_column,
        spacetimedsl_methods,
    })
}

pub fn rm_rsharp(ident: syn::Ident) -> syn::Ident {
    let mut ident_as_str = ident.to_string();
    if ident_as_str.starts_with("r#") {
        format_ident!("{}", ident_as_str.split_off(2))
    } else {
        ident
    }
}
