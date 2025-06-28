use crate::api::db::{
    index::{Index, IndexType},
    reducer::ScheduledReducer,
    table::{SpacetimeDBTable, SpacetimeDBTableVisibility},
};
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::table::{
    IndexArg, IndexType as SpacetimeIndexType, ScheduledArg, TableAccess, TableArgs,
};
use syn::Ident;

impl SpacetimeDBTable {
    pub(in crate::internal) fn map(table: &TableArgs) -> SpacetimeDBTable {
        let singular_name = table.name.clone();
        let visibility = SpacetimeDBTableVisibility::map(&table.access);
        let indices = table.indices.iter().map(|i| Index::map(i)).collect();
        let scheduled_reducer = table.scheduled.as_ref().map(|s| ScheduledReducer::map(s));

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
        let name = index.name.clone();
        let is_unique = index.is_unique;
        let r#type = match &index.kind {
            SpacetimeIndexType::Direct { column } => {
                let column = column.clone();
                IndexType::Direct { column }
            }
            SpacetimeIndexType::BTree { columns } => {
                let columns: Vec<Ident> = columns.iter().map(|c| c.clone()).collect();

                match columns.len() {
                    1 => IndexType::BTreeSingleColumn {
                        column: columns.get(0).expect("column 0 should exist").clone(),
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
        let reducer_name = format_ident!("{}", scheduled.reducer.to_token_stream().to_string());

        ScheduledReducer { reducer_name }
    }
}
