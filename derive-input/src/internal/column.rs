use crate::api::{
    Column,
    db::{column::SpacetimeDBColumn, table::SpacetimeDBTable},
    dsl::{
        column::{SpacetimeDSLColumn, SpacetimeDSLColumnMethods},
        foreign_key::ForeignKey,
        table::SpacetimeDSLTable,
        wrapper::WrapperType,
    },
    rust::{column::RustField, table::RustStruct, visibility::RustVisibility},
};
use itertools::izip;
use proc_macro2::Span;
use spacetime_bindings_macro_input::table::ColumnArgs;
use syn::{Ident, Path};

#[allow(clippy::type_complexity)]
pub(in crate::internal) fn try_parse(
    column_args: &ColumnArgs,
    rust_struct: &RustStruct,
    mut spacetimedb_table: SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> syn::Result<(
    SpacetimeDBTable,
    Vec<Column>,
    Column,
    Vec<InternalColumn>,
    InternalColumn,
)> {
    let primary_key_column_name = match get_primary_key_column_name(column_args) {
        Some(pk) => pk,
        None => {
            return Err(syn::Error::new(
                Span::call_site(),
                "The table should have a column with `#[primary_key]` helper attribute!",
            ));
        }
    };

    let auto_inc_column_names = get_auto_inc_column_names(column_args);

    let mut rust_fields = vec![];
    let mut spacetimedb_columns = vec![];
    let mut spacetimedsl_columns = vec![];
    let mut columns = vec![];
    let mut internal_columns = vec![];

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

        let spacetimedsl_column =
            SpacetimeDSLColumn::try_parse(field, rust_struct, &rust_field, &spacetimedb_column)?;

        let internal_column = InternalColumn {
            spacetimedb_table_singular_name: spacetimedb_table.singular_name.clone(),
            rust_field_visibility: rust_field.visibility.clone(),
            rust_field_name: rust_field.name.clone(),
            rust_field_type_name_or_path: rust_field.type_name_or_path.clone(),
            spacetimedsl_column_foreign_key: spacetimedsl_column.foreign_key.clone(),
            spacetimedb_column_is_auto_inc: spacetimedb_column.is_auto_inc,
            spacetimedsl_column_is_option: spacetimedsl_column.is_option,
            spacetimedsl_column_wrapper_type: spacetimedsl_column.wrapper_type.clone(),
        };

        rust_fields.push(rust_field);
        spacetimedb_columns.push(spacetimedb_column);
        spacetimedsl_columns.push(spacetimedsl_column);
        internal_columns.push(internal_column);
    }

    let internal_primary_key_column = internal_columns
        .iter()
        .find(|c| {
            c.rust_field_name
                .to_string()
                .eq(&primary_key_column_name.to_string())
        })
        .expect("PK column should be present")
        .clone();

    for (rust_field, spacetimedb_column, spacetimedsl_column) in
        izip!(rust_fields, spacetimedb_columns, spacetimedsl_columns)
    {
        let spacetimedsl_methods = SpacetimeDSLColumnMethods::map(
            rust_struct,
            &spacetimedb_table,
            spacetimedsl_table,
            &spacetimedb_column,
            &internal_columns,
            &internal_primary_key_column,
        );

        columns.push(Column {
            rust_field,
            spacetimedb_column,
            spacetimedsl_column,
            spacetimedsl_methods,
        });
    }

    let primary_key_column = columns
        .iter()
        .find(|c| {
            c.rust_field
                .name
                .to_string()
                .eq(&primary_key_column_name.to_string())
        })
        .expect("PK column should be present")
        .clone();

    Ok((
        spacetimedb_table,
        columns,
        primary_key_column,
        internal_columns,
        internal_primary_key_column,
    ))
}

#[derive(Clone)]
pub(in crate::internal) struct InternalColumn {
    pub spacetimedb_table_singular_name: Ident,
    pub rust_field_visibility: RustVisibility,
    pub rust_field_name: Ident,
    pub rust_field_type_name_or_path: Path,
    pub spacetimedb_column_is_auto_inc: bool,
    pub spacetimedsl_column_is_option: bool,
    pub spacetimedsl_column_foreign_key: Option<ForeignKey>,
    pub spacetimedsl_column_wrapper_type: Option<WrapperType>,
}

fn get_auto_inc_column_names(column_args: &ColumnArgs<'_>) -> Vec<Ident> {
    column_args
        .sequenced_columns
        .iter()
        .map(|c| c.ident.clone())
        .collect()
}

pub(in crate::internal) fn get_primary_key_column_name(
    column_args: &ColumnArgs<'_>,
) -> Option<Ident> {
    column_args
        .primary_key_column
        .as_ref()
        .map(|c| c.ident.clone())
}
