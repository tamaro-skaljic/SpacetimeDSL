use crate::{
    api::db::{
        index::{Index, IndexType},
        reducer::ScheduledReducer,
        table::{SpacetimeDBTable, SpacetimeDBTableVisibility},
    },
    internal::table::rm_rsharp,
};
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::table::{
    IndexArg, IndexType as SpacetimeIndexType, ScheduledArg, TableAccess, TableArgs,
};
use syn::Ident;

impl SpacetimeDBTable {
    pub(in crate::internal) fn map(table: &TableArgs) -> SpacetimeDBTable {
        let singular_name = rm_rsharp(table.accessor.clone());
        let visibility = SpacetimeDBTableVisibility::map(&table.access);
        let indices = table.indices.iter().map(Index::map).collect();
        let scheduled_reducer = table.scheduled.as_ref().map(ScheduledReducer::map);

        SpacetimeDBTable {
            singular_name,
            visibility,
            // Contains all indices during processing for the moment, but all single column indices are removed from the Vector after the columns are processed.
            multi_column_indices: indices,
            scheduled_reducer,
        }
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
    fn map(index: &IndexArg) -> Index {
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
        let reducer_name = format_ident!(
            "{}",
            scheduled.reducer_or_procedure.to_token_stream().to_string()
        );

        ScheduledReducer { reducer_name }
    }
}
