use crate::api::{
    Table,
    db::SpacetimeDBTable,
    dsl::{method::SpacetimeDSLTableMethods, table::SpacetimeDSLTable},
};
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::DeriveInput;

mod rust;

mod db;

mod dsl;

pub(in crate::internal) fn try_parse(
    item: &DeriveInput,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
) -> syn::Result<Table> {
    let rust_struct = rust::map(&item);

    let spacetimedb_table = SpacetimeDBTable::map(&table_args);

    let (spacetimedb_table, spacetimedsl_table) =
        SpacetimeDSLTable::try_parse(&item, spacetimedb_table)?;

    let (spacetimedb_table, spacetimedsl_table, columns, primary_key_column_name) =
        super::column::try_parse(
            &column_args,
            &rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
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
