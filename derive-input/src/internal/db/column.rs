use crate::api::{
    db::{column::SpacetimeDBColumn, index::IndexType, table::SpacetimeDBTable},
    rust::column::RustField,
};

impl SpacetimeDBColumn {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        mut spacetimedb_table: SpacetimeDBTable,
        auto_inc_column_names: &Vec<Box<str>>,
        primary_key_column_name: &Box<str>,
    ) -> (SpacetimeDBTable, SpacetimeDBColumn) {
        let column_name = &rust_field.name;

        let is_primary_key = column_name.eq(primary_key_column_name);

        let mut i: usize = 0;
        let mut single_column_index = None;

        for index in &spacetimedb_table.multi_column_indices {
            match &index.index_type {
                IndexType::BTreeSingleColumn { column } => {
                    if column.eq(column_name) {
                        single_column_index = Some(i);
                        break;
                    }
                }
                IndexType::Direct { column } => {
                    if column.eq(column_name) {
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

        let is_auto_inc = auto_inc_column_names
            .iter()
            .find(|c| c.eq(&column_name))
            .is_some();

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
