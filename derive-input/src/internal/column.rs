use crate::api::{
    Column,
    db::{column::SpacetimeDBColumn, table::SpacetimeDBTable},
    dsl::{
        column::{SpacetimeDSLColumn, SpacetimeDSLColumnMethods},
        table::SpacetimeDSLTable,
    },
    rust::{column::RustField, table::RustStruct},
};
use spacetime_bindings_macro_input::table::ColumnArgs;

pub(in crate::internal) fn try_parse(
    column_args: &ColumnArgs,
    rust_struct: &RustStruct,
    mut spacetimedb_table: SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> syn::Result<(SpacetimeDBTable, Vec<Column>, Box<str>)> {
    let primary_key_column_name = match get_primary_key_column_name(column_args) {
        Some(pk) => pk,
        None => {
            panic!("The table should have a column with `#[primary_key]` helper attribute!")
        }
    };

    let auto_inc_column_names = get_auto_inc_column_names(column_args);

    let mut columns = vec![];

    for field in &column_args.fields {
        let rust_field = RustField::map(field);

        let res = SpacetimeDBColumn::map(
            &rust_field,
            spacetimedb_table,
            &auto_inc_column_names,
            &primary_key_column_name,
        )?;
        spacetimedb_table = res.0;
        let spacetimedb_column = res.1;

        let spacetimedsl_column = SpacetimeDSLColumn::try_parse(field, &rust_struct, &rust_field, &spacetimedb_column)?;

        let spacetimedsl_methods = SpacetimeDSLColumnMethods::map(
            &rust_struct,
            &spacetimedb_table,
            &spacetimedsl_table,
            &rust_field,
            &spacetimedb_column,
            &spacetimedsl_column,
            &primary_key_column_name,
        );

        columns.push(Column {
            rust_field,
            spacetimedb_column,
            spacetimedsl_column,
            spacetimedsl_methods,
        });
    }

    Ok((spacetimedb_table, columns, primary_key_column_name))
}

fn get_auto_inc_column_names(column_args: &ColumnArgs<'_>) -> Vec<Box<str>> {
    column_args
        .sequenced_columns
        .iter()
        .map(|c| c.ident.to_string().into())
        .collect()
}

pub(in crate::internal) fn get_primary_key_column_name(
    column_args: &ColumnArgs<'_>,
) -> Option<Box<str>> {
    column_args
        .primary_key_column
        .as_ref()
        .map(|c| c.ident.to_string().into())
}
