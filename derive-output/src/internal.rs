use crate::api::{
    Column, Table,
    db::{DBColumn, SpacetimeDBTable},
    dsl::table::DSLTable,
    rust::{RustField, RustStruct},
};
use db::{column::ParseSpacetimeColumn, table::ParseSpacetimeTable};
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};

mod rust;

mod db;

mod dsl;

pub(crate) fn try_parse(args: syn::Attribute, item: syn::DeriveInput) -> syn::Result<Table> {
    let table_args = TableArgs::try_parse(&item)?;
    let (table_args, column_args) = ColumnArgs::try_parse(&item, table_args)?;

    let rust = RustStruct::map(&item);

    let spacetimedb = SpacetimeDBTable::map(&table_args);

    let spacetimedsl = crate::api::dsl::table::DSLTable::try_parse(&args)?;

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
        mut spacetimedsl_table: Option<DSLTable>,
    ) -> syn::Result<(SpacetimeDBTable, Option<DSLTable>, Vec<Column>)> {
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
            let spacetimedb = res.1;

            let mut spacetimedsl = None;
            if spacetimedsl_table.is_some() {
                let res = crate::api::dsl::column::DSLColumn::try_parse(
                    item,
                    &spacetimedb,
                    spacetimedsl_table.unwrap(),
                )?;
                spacetimedsl_table = Some(res.0);
                spacetimedsl = Some(res.1);
            }

            columns.push(Column {
                rust,
                spacetimedb,
                spacetimedsl,
            });
        }

        Ok((spacetimedb_table, spacetimedsl_table, columns))
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
