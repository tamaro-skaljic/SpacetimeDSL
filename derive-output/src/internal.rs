use crate::api::{
    Column, Table,
    db::{SpacetimeDBColumn, SpacetimeDBTable},
    dsl::table::SpacetimeDSLTable,
    rust::{RustField, RustStruct},
};
use db::{column::ParseSpacetimeColumn, table::ParseSpacetimeTable};
use proc_macro2::TokenStream;
use quote::quote;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};

mod rust;

mod db;

mod dsl;

pub(crate) fn try_parse(args: syn::Attribute, item: syn::DeriveInput) -> syn::Result<Table> {
    let table_args = TableArgs::try_parse(&item)?;
    let (table_args, column_args) = ColumnArgs::try_parse(&item, table_args)?;

    let rust = RustStruct::map(&item);

    let spacetimedb = SpacetimeDBTable::map(&table_args);

    let (spacetimedb, spacetimedsl) =
        crate::api::dsl::table::SpacetimeDSLTable::try_parse(&args, spacetimedb)?;

    let (spacetimedb, spacetimedsl, columns) =
        Column::try_parse(&item, &column_args, spacetimedb, spacetimedsl)?;

    Ok(Table {
        rust,
        spacetimedb,
        spacetimedsl,
        columns,
    })
}

impl Column {
    fn try_parse(
        item: &syn::DeriveInput,
        column_args: &ColumnArgs,
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
            let rust = RustField::map(field);

            let res = SpacetimeDBColumn::map(
                spacetimedb_table,
                primary_key_column,
                sequenced_columns,
                field,
            );
            spacetimedb_table = res.0;
            let spacetimedb = res.1;

            let res = crate::api::dsl::column::SpacetimeDSLColumn::try_parse(
                item,
                field,
                &spacetimedb,
                spacetimedsl_table,
            )?;
            spacetimedsl_table = res.0;
            let spacetimedsl = res.1;

            columns.push(Column {
                rust,
                spacetimedb,
                spacetimedsl,
            });
        }

        Ok((spacetimedb_table, spacetimedsl_table, columns))
    }
}

// TODO: Anything under this should probably be refactored

fn columns_to_string(
    columns: &Vec<spacetime_bindings_macro_input::table::Column<'_>>,
) -> Vec<String> {
    columns.iter().map(|c| column_to_string(c)).collect()
}

fn column_to_string(column: &spacetime_bindings_macro_input::table::Column<'_>) -> String {
    column.ident.to_string()
}

pub(in crate::internal) fn wrapper_type_into_option(
    column_name: &Box<str>,
    column_option_name: &Box<str>,
    wrapper_type_name_or_path: &Box<str>,
) -> TokenStream {
    quote! {
        let #column_name = #column_name.into();
        let mut #column_option_name = None;
        if #column_name.is_some() {
            #column_option_name = Some(Into::<#wrapper_type_name_or_path>::into(#column_name.unwrap()).value());
        }
    }
}
