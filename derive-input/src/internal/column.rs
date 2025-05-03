use crate::api::{
    Column,
    db::{SpacetimeDBColumn, SpacetimeDBTable},
    dsl::{method::SpacetimeDSLColumnMethods, table::SpacetimeDSLTable},
    rust::{RustField, RustStruct},
};
use spacetime_bindings_macro_input::table::ColumnArgs;

mod rust;

mod db;

mod dsl;

pub(in crate::internal) fn try_parse(
    item: &syn::DeriveInput,
    column_args: &ColumnArgs,
    rust_struct: &RustStruct,
    mut spacetimedb_table: SpacetimeDBTable,
    mut spacetimedsl_table: SpacetimeDSLTable,
) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable, Vec<Column>)> {
    let sequenced_columns = &columns_to_string(&column_args.sequenced_columns);
    let primary_key_column = &column_args
        .primary_key_column
        .as_ref()
        .map(|c| column_to_string(c));

    let mut columns = vec![];

    for field in &column_args.fields {
        let rust_field = RustField::map(field);

        let res = SpacetimeDBColumn::map(
            spacetimedb_table,
            primary_key_column,
            sequenced_columns,
            field,
        );
        spacetimedb_table = res.0;
        let spacetimedb_column = res.1;

        let res = crate::api::dsl::column::SpacetimeDSLColumn::try_parse(
            item,
            field,
            &rust_field,
            spacetimedsl_table,
        )?;
        spacetimedsl_table = res.0;
        let spacetimedsl_column = res.1;

        let spacetimedsl_methods = SpacetimeDSLColumnMethods::try_parse(
            &rust_struct,
            &spacetimedb_table,
            &spacetimedsl_table,
            &rust_field,
            &spacetimedb_column,
            &spacetimedsl_column,
        );

        columns.push(Column {
            rust_field,
            spacetimedb_column,
            spacetimedsl_column,
            spacetimedsl_methods,
        });
    }

    Ok((spacetimedb_table, spacetimedsl_table, columns))
}

pub(in crate::internal) fn columns_to_string(
    columns: &Vec<spacetime_bindings_macro_input::table::Column<'_>>,
) -> Vec<String> {
    columns.iter().map(|c| column_to_string(c)).collect()
}

pub(in crate::internal) fn column_to_string(
    column: &spacetime_bindings_macro_input::table::Column<'_>,
) -> String {
    column.ident.to_string()
}
