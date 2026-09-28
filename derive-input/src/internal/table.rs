use crate::{
    api::{
        Table,
        db::table::SpacetimeDBTable,
        dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
    },
    internal::{DSLData, dsl::method::MethodGenerationContext},
};
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::DeriveInput;

use crate::internal::rust::column::column_type_path;

pub fn try_parse(
    input: &DeriveInput,
    dsl_data: DSLData,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
) -> syn::Result<Table> {
    // Every other column analysis assumes a path type, so an unsupported one is reported
    // before any of them runs.
    for field in &column_args.fields {
        column_type_path(field.ty)?;
    }

    let rust_struct = crate::internal::rust::table::map_struct(input);

    let spacetimedb_table = SpacetimeDBTable::map(table_args, dsl_data.kind.singleton().is_some())?;

    let (spacetimedb_table, mut spacetimedsl_table) =
        SpacetimeDSLTable::try_parse(dsl_data, column_args, spacetimedb_table)?;

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

    let context = MethodGenerationContext::new(
        &rust_struct,
        &spacetimedb_table,
        &spacetimedsl_table,
        &internal_columns,
        &internal_primary_key_column,
    );

    let (spacetimedsl_methods, contributions) =
        SpacetimeDSLTableMethods::generate(&context, &columns)?;

    contributions.apply_to(&mut spacetimedsl_table);

    Ok(Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        primary_key_column,
        spacetimedsl_methods,
    })
}
