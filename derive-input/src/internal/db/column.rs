use crate::api::db::{IndexType, SpacetimeDBColumn, SpacetimeDBTable};
use spacetime_bindings_macro_input::{
    sats::SatsField,
    table::{ColumnArgs, TableArgs},
};
use syn::DeriveInput;

impl SpacetimeDBColumn {
    pub(in crate::internal) fn map(
        mut spacetimedb_table: SpacetimeDBTable,
        primary_key_column: &Option<String>,
        sequenced_columns: &Vec<String>,
        field: &SatsField<'_>,
    ) -> (SpacetimeDBTable, SpacetimeDBColumn) {
        let name = field.name.as_ref().unwrap();

        let is_primary_key = match &primary_key_column {
            Some(primary_key_column) => primary_key_column.eq(name),
            None => false,
        };

        let mut i: usize = 0;
        let mut single_column_index = None;

        for index in &spacetimedb_table.multi_column_indices {
            match &index.index_type {
                IndexType::BTreeSingleColumn { column } => {
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
            SpacetimeDBColumn {
                is_primary_key,
                single_column_index,
                is_auto_inc,
            },
        )
    }
}
