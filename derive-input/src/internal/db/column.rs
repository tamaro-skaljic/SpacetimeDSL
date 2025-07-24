use crate::api::{
    db::{column::SpacetimeDBColumn, index::IndexType, table::SpacetimeDBTable},
    rust::column::RustField,
};
use proc_macro2::Span;
use syn::{Error, Ident};

impl SpacetimeDBColumn {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        mut spacetimedb_table: SpacetimeDBTable,
        auto_inc_column_names: &[Ident],
        primary_key_column_name: &Ident,
    ) -> Result<(SpacetimeDBTable, SpacetimeDBColumn), Error> {
        let column_name = &rust_field.name;

        let is_primary_key = column_name.eq(primary_key_column_name);

        if is_primary_key
            && column_name
                .to_string()
                .starts_with(&format!("{}", &spacetimedb_table.singular_name.to_string()))
        {
            return Err(Error::new(
                Span::call_site(),
                format!(
                    "A #[primary_key] column must not be prefixed with the table's name! Use `{}` instead of `{}`.",
                    column_name
                        .to_string()
                        .strip_prefix(&spacetimedb_table.singular_name.to_string())
                        .unwrap_or("id"),
                    column_name.to_string(),
                ),
            ));
        }

        let mut single_column_index = None;

        for (i, index) in spacetimedb_table.multi_column_indices.iter().enumerate() {
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
        }

        let single_column_index =
            single_column_index.map(|i| spacetimedb_table.multi_column_indices.swap_remove(i));

        let is_auto_inc = auto_inc_column_names
            .iter()
            .any(|c| c.to_string().eq(&column_name.to_string()));

        Ok((
            spacetimedb_table,
            SpacetimeDBColumn {
                is_primary_key,
                single_column_index,
                is_auto_inc,
            },
        ))
    }
}
