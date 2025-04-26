use crate::api::db::{DBColumn, IndexType, SpacetimeDBTable};
use spacetime_bindings_macro_input::{
    sats::SatsField,
    table::{ColumnArgs, TableArgs},
};
use syn::DeriveInput;

pub(in crate::internal) trait ParseSpacetimeColumn {
    fn try_parse(item: &DeriveInput, table_args: TableArgs)
    -> syn::Result<(TableArgs, ColumnArgs)>;
}

impl ParseSpacetimeColumn for ColumnArgs<'_> {
    fn try_parse(
        item: &DeriveInput,
        table_args: TableArgs,
    ) -> syn::Result<(TableArgs, ColumnArgs)> {
        let (table_args, column_args) = ColumnArgs::parse(table_args, item)?;

        Ok((table_args, column_args))
    }
}

impl DBColumn {
    pub(in crate::internal) fn map(
        mut spacetimedb_table: SpacetimeDBTable,
        primary_key_column: &Option<String>,
        sequenced_columns: &Vec<String>,
        field: &SatsField<'_>,
    ) -> (SpacetimeDBTable, DBColumn) {
        let name = field.name.as_ref().unwrap();

        let is_primary_key = match &primary_key_column {
            Some(primary_key_column) => primary_key_column.eq(name),
            None => false,
        };

        let mut i: usize = 0;
        let mut single_column_index = None;

        for index in &spacetimedb_table.multi_column_indices {
            match &index.r#type {
                IndexType::SingleColumnBTree { column } => {
                    if column.to_string().eq(name) {
                        single_column_index = Some(i);
                        break;
                    }
                }
                IndexType::Direct { column } => {
                    if column.to_string().eq(name) {
                        single_column_index = Some(i);
                        break;
                    }
                }
                _ => {}
            }

            i = i + 1;
        }

        let single_column_index =
            single_column_index.map(|i| spacetimedb_table.multi_column_indices.swap_remove(i));

        let is_auto_inc = sequenced_columns.iter().find(|c| c.eq(&name)).is_some();

        (
            spacetimedb_table,
            DBColumn {
                is_primary_key,
                single_column_index,
                is_auto_inc,
            },
        )
    }
}
