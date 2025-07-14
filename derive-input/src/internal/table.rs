use crate::api::{
    Table,
    db::table::SpacetimeDBTable,
    dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
};
use quote::format_ident;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::DeriveInput;

pub(in crate::internal) fn try_parse(
    args: proc_macro2::TokenStream,
    input: &DeriveInput,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
    plural_name: syn::Ident,
) -> syn::Result<Table> {
    let rust_struct = crate::internal::rust::table::map_struct(&input);

    let spacetimedb_table = SpacetimeDBTable::map(table_args);

    let (spacetimedb_table, spacetimedsl_table) =
        SpacetimeDSLTable::try_parse(args, column_args, spacetimedb_table, plural_name)?;

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

pub fn rm_rsharp(ident: syn::Ident) -> syn::Ident {
    let mut ident_as_str = ident.to_string();
    if ident_as_str.starts_with("r#") {
        format_ident!("{}", ident_as_str.split_off(2))
    } else {
        ident
    }
}
