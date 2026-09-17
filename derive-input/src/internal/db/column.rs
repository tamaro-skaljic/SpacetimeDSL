use crate::api::{
    db::{column::SpacetimeDBColumn, index::IndexType, table::SpacetimeDBTable},
    rust::column::RustField,
};
use syn::{Error, Ident};

impl SpacetimeDBColumn {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        mut spacetimedb_table: SpacetimeDBTable,
        auto_inc_column_names: &[Ident],
        primary_key_column_name: &Ident,
        is_singleton: bool,
    ) -> Result<(SpacetimeDBTable, SpacetimeDBColumn), Error> {
        let column_name = &rust_field.name;

        let is_primary_key = column_name.eq(primary_key_column_name);

        if is_primary_key
            && column_name
                .to_string()
                .starts_with(&spacetimedb_table.singular_name.to_string())
        {
            return Err(Error::new_spanned(
                &rust_field.name,
                format!(
                    "A #[primary_key] column must not be prefixed with the table's name! Use `{}` instead of `{}`.",
                    column_name
                        .to_string()
                        .strip_prefix(&format!("{}_", spacetimedb_table.singular_name))
                        .unwrap_or("id"),
                    column_name,
                ),
            ));
        }

        // Singleton validation: user-defined columns must not have #[primary_key], #[index], or #[unique]
        // (the injected `id: u8` pk is allowed)
        if is_singleton && !is_primary_key {
            // Check if this column has a single-column index (which means #[index], #[unique], or #[primary_key] was used)
            for index in spacetimedb_table.multi_column_indices.iter() {
                match &index.index_type {
                    IndexType::BTreeSingleColumn { column }
                    | IndexType::Direct { column }
                    | IndexType::HashSingleColumn { column } => {
                        if column.eq(column_name) {
                            return Err(Error::new_spanned(
                                &rust_field.name,
                                format!(
                                    "`#[index]` and `#[unique]` are not allowed on singleton tables! Found index on column `{column_name}`.",
                                ),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut single_column_index = None;

        for (i, index) in spacetimedb_table.multi_column_indices.iter().enumerate() {
            match &index.index_type {
                IndexType::BTreeSingleColumn { column }
                | IndexType::HashSingleColumn { column }
                | IndexType::Direct { column } => {
                    if column.eq(column_name) {
                        single_column_index = Some(i);
                        break;
                    }
                }
                IndexType::BTreeMultiColumn { .. } | IndexType::HashMultiColumn { .. } => {}
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
