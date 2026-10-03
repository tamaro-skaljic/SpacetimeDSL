//! `#[creation_default(<expression>)]`: the value `create_<table>` fills a column with,
//! instead of asking the caller for it in `Create<Table>`, and the columns it is not
//! allowed on.

use {
    super::creation_default,
    crate::{
        api::{
            db::column::SpacetimeDBColumn,
            dsl::{auto_gen::UUIDVersion, table::SpacetimeDSLTable},
        },
        internal::error,
    },
    spacetime_bindings_macro_input::sats::SatsField,
    syn::{Attribute, Expr, Ident},
};

/// Reads `#[creation_default(<expression>)]` from a column. A column may have one, where the
/// caller would otherwise supply the value and where two rows may hold the same one.
pub fn try_parse(
    field: &SatsField<'_>,
    column_name: &Ident,
    spacetimedb_column: &SpacetimeDBColumn,
    spacetimedsl_table: &SpacetimeDSLTable,
    auto_generated_uuid_version: Option<UUIDVersion>,
) -> syn::Result<Option<Expr>> {
    let mut creation_default_attributes = field
        .original_attrs
        .iter()
        .filter(|attribute| attribute.path() == creation_default);

    let Some(creation_default_attribute) = creation_default_attributes.next() else {
        return Ok(None);
    };

    if let Some(repeated_attribute) = creation_default_attributes.next() {
        return Err(error::multiple_creation_default_attributes(
            repeated_attribute,
        ));
    }

    let expression = parse_expression(creation_default_attribute)?;

    reject_where_create_fills_the_column(
        creation_default_attribute,
        column_name,
        spacetimedb_column,
        spacetimedsl_table,
        auto_generated_uuid_version,
    )?;

    reject_where_rows_would_share_the_value(creation_default_attribute, spacetimedb_column)?;

    Ok(Some(expression))
}

fn parse_expression(creation_default_attribute: &Attribute) -> syn::Result<Expr> {
    creation_default_attribute
        .meta
        .require_list()
        .and_then(|list| list.parse_args())
        .map_err(|_| error::creation_default_without_expression(creation_default_attribute))
}

/// A default would never be used where `create_<table>` fills the column itself, or where
/// the table has no create method at all.
fn reject_where_create_fills_the_column(
    creation_default_attribute: &Attribute,
    column_name: &Ident,
    spacetimedb_column: &SpacetimeDBColumn,
    spacetimedsl_table: &SpacetimeDSLTable,
    auto_generated_uuid_version: Option<UUIDVersion>,
) -> syn::Result<()> {
    let names_this_column =
        |role_column_name: &Option<Ident>| role_column_name.as_ref() == Some(column_name);

    if spacetimedsl_table.singleton_has_default() {
        return Err(error::creation_default_on_singleton_with_default(
            creation_default_attribute,
        ));
    }

    if spacetimedb_column.is_auto_inc {
        return Err(error::creation_default_on_auto_inc_column(
            creation_default_attribute,
        ));
    }

    if auto_generated_uuid_version.is_some() {
        return Err(error::creation_default_on_auto_gen_column(
            creation_default_attribute,
        ));
    }

    if names_this_column(&spacetimedsl_table.on_insert_set_current_timestamp_column_name) {
        return Err(error::creation_default_on_set_on_create_column(
            creation_default_attribute,
        ));
    }

    if names_this_column(&spacetimedsl_table.on_update_set_current_timestamp_column_name) {
        return Err(error::creation_default_on_set_on_update_column(
            creation_default_attribute,
        ));
    }

    if spacetimedsl_table
        .soft_delete_marker
        .as_ref()
        .is_some_and(|marker| marker.column_name == *column_name)
    {
        return Err(error::creation_default_on_marker_column(
            creation_default_attribute,
        ));
    }

    Ok(())
}

/// Every created row would get the same value, which a unique column holds only once.
fn reject_where_rows_would_share_the_value(
    creation_default_attribute: &Attribute,
    spacetimedb_column: &SpacetimeDBColumn,
) -> syn::Result<()> {
    if spacetimedb_column.is_primary_key {
        return Err(error::creation_default_on_primary_key_column(
            creation_default_attribute,
        ));
    }

    if spacetimedb_column
        .single_column_index
        .as_ref()
        .is_some_and(|index| index.is_unique)
    {
        return Err(error::creation_default_on_unique_column(
            creation_default_attribute,
        ));
    }

    Ok(())
}
