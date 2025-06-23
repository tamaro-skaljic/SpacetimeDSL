use crate::api::{
    Table,
    db::table::SpacetimeDBTable,
    dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
};
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::DeriveInput;

pub(in crate::internal) fn try_parse(
    args: proc_macro2::TokenStream,
    input: &DeriveInput,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
) -> syn::Result<Table> {
    let rust_struct = crate::internal::rust::table::map_struct(&input);

    let spacetimedb_table = SpacetimeDBTable::map(table_args);

    let (spacetimedb_table, spacetimedsl_table) =
        SpacetimeDSLTable::try_parse(args, column_args, spacetimedb_table)?;

    let (spacetimedb_table, columns, primary_key_column_name) = super::column::try_parse(
        &column_args,
        &rust_struct,
        spacetimedb_table,
        &spacetimedsl_table,
    )?;

    let spacetimedsl_methods = SpacetimeDSLTableMethods::try_parse(
        &rust_struct,
        &spacetimedb_table,
        &spacetimedsl_table,
        &columns,
        &primary_key_column_name,
    )?;

    Ok(Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        spacetimedsl_methods,
    })
}
