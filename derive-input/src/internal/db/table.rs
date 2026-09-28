use {
    crate::api::db::{
        index::{Index, IndexType},
        reducer::ScheduledReducer,
        table::{SpacetimeDBTable, SpacetimeDBTableVisibility},
    },
    spacetime_bindings_macro_input::table::{
        IndexArg, IndexType as SpacetimeIndexType, ScheduledArg, TableAccess, TableArgs,
    },
    std::collections::BTreeSet,
    syn::{Ident, ext::IdentExt},
};

impl SpacetimeDBTable {
    /// Assembles the table from what `#[table]` declares and the index assignment
    /// `internal::table` computed: `multi_column_indices` are the indices no column claimed,
    /// in declaration order, and a B-tree multi-column index named in
    /// `dsl_unique_index_names` is unique.
    pub(crate) fn map(
        table: &TableArgs,
        multi_column_indices: Vec<Index>,
        dsl_unique_index_names: &BTreeSet<Ident>,
    ) -> syn::Result<SpacetimeDBTable> {
        let singular_name = table.accessor.unraw();
        let visibility = SpacetimeDBTableVisibility::map(&table.access);
        let scheduled_reducer = table.scheduled.as_ref().map(ScheduledReducer::map);

        let multi_column_indices = multi_column_indices
            .into_iter()
            .map(|index| Index {
                is_unique: index.is_unique || dsl_unique_index_names.contains(&index.name),
                ..index
            })
            .collect();

        Ok(SpacetimeDBTable {
            singular_name,
            visibility,
            multi_column_indices,
            scheduled_reducer,
        })
    }
}

impl SpacetimeDBTableVisibility {
    fn map(access: &Option<TableAccess>) -> SpacetimeDBTableVisibility {
        match &access {
            Some(a) => match a {
                TableAccess::Public(_) => SpacetimeDBTableVisibility::Public,
                TableAccess::Private(_) => SpacetimeDBTableVisibility::Private,
            },
            None => SpacetimeDBTableVisibility::Private,
        }
    }
}

impl Index {
    pub(crate) fn map(index: &IndexArg) -> Index {
        let name = index.accessor.clone();
        let is_unique = index.is_unique;
        let r#type = match &index.kind {
            SpacetimeIndexType::Direct { column } => {
                let column = column.clone();
                IndexType::Direct { column }
            }

            SpacetimeIndexType::Hash { columns } => {
                let columns: Vec<Ident> = columns.to_vec();

                match columns.len() {
                    1 => IndexType::HashSingleColumn {
                        column: columns.first().expect("column 0 should exist").clone(),
                    },
                    _ => IndexType::HashMultiColumn { columns },
                }
            }
            SpacetimeIndexType::BTree { columns } => {
                let columns: Vec<Ident> = columns.to_vec();

                match columns.len() {
                    1 => IndexType::BTreeSingleColumn {
                        column: columns.first().expect("column 0 should exist").clone(),
                    },
                    _ => IndexType::BTreeMultiColumn { columns },
                }
            }
        };

        Index {
            name,
            is_unique,
            index_type: r#type,
        }
    }
}

impl ScheduledReducer {
    fn map(scheduled: &ScheduledArg) -> ScheduledReducer {
        ScheduledReducer {
            reducer_path: scheduled.reducer_or_procedure.clone(),
        }
    }
}
