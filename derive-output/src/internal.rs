use crate::api::{
    Column, Table,
    db::{DBColumn, SpacetimeDBTable},
    dsl::table::DSLTable,
    rust::{RustField, RustStruct},
};
use db::{column::ParseSpacetimeColumn, table::ParseSpacetimeTable};
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use std::fmt::Display;

mod rust;

mod db;

#[cfg(feature = "spacetimedsl")]
mod dsl;

pub(crate) fn try_parse(args: syn::Attribute, item: syn::DeriveInput) -> syn::Result<Table> {
    let table_args = TableArgs::try_parse(&item)?;
    let (table_args, column_args) = ColumnArgs::try_parse(&item, table_args)?;

    let rust_struct = RustStruct::map(&item);

    let spacetimedb_table = SpacetimeDBTable::map(&table_args);

    #[cfg(feature = "spacetimedsl")]
    let spacetimedsl_table = crate::api::dsl::table::DSLTable::try_parse(
        &args,
        &item,
        &column_args,
        &rust_struct,
        &spacetimedb_table,
    )?;

    let (spacetimedb_table, columns) = Column::try_parse(
        &item,
        &column_args,
        spacetimedb_table,
        #[cfg(feature = "spacetimedsl")]
        &spacetimedsl_table,
    )?;

    Ok(Table {
        rust: rust_struct,
        spacetimedb: spacetimedb_table,
        #[cfg(feature = "spacetimedsl")]
        spacetimedsl: spacetimedsl_table,
        columns,
    })
}

impl Column {
    fn try_parse(
        item: &syn::DeriveInput,
        column_args: &ColumnArgs,
        mut spacetimedb_table: SpacetimeDBTable,
        #[cfg(feature = "spacetimedsl")] spacetimedsl_table: &Option<DSLTable>,
    ) -> syn::Result<(SpacetimeDBTable, Vec<Column>)> {
        let sequenced_columns = &columns_to_string(&column_args.sequenced_columns);
        let primary_key_column = &column_args
            .primary_key_column
            .as_ref()
            .map(|c| column_to_string(c));

        let mut columns = vec![];

        for field in &column_args.fields {
            let rust = RustField::map(field);

            let res = DBColumn::map(
                spacetimedb_table,
                primary_key_column,
                sequenced_columns,
                field,
            );
            spacetimedb_table = res.0;
            let spacetimedb_column = res.1;
            #[cfg(feature = "spacetimedsl")]
            let spacetimedsl = crate::api::dsl::column::DSLColumn::try_parse(
                item,
                spacetimedsl_table,
                &spacetimedb_column,
            )?;

            columns.push(Column {
                rust,
                spacetimedb: spacetimedb_column,
                #[cfg(feature = "spacetimedsl")]
                spacetimedsl,
            });
        }

        Ok((spacetimedb_table, columns))
    }
}

fn columns_to_string(
    columns: &Vec<spacetime_bindings_macro_input::table::Column<'_>>,
) -> Vec<String> {
    columns.iter().map(|c| column_to_string(c)).collect()
}

fn column_to_string(column: &spacetime_bindings_macro_input::table::Column<'_>) -> String {
    column.ident.to_string()
}

trait IntoStringError<T> {
    fn error_into_str(self) -> Result<T, Box<str>>;
}

impl<T, E: Display> IntoStringError<T> for Result<T, E> {
    fn error_into_str(self) -> Result<T, Box<str>> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => return Err(err.to_string().into()),
        }
    }
}
