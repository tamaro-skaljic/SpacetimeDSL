use crate::api::db::SpacetimeDBTable;
use crate::api::db::{Index, IndexType, ScheduledReducer, SpacetimeDBTableVisibility};
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{
    IndexArg, IndexType as SpacetimeIndexType, ScheduledArg, TableAccess, TableArgs,
};

impl SpacetimeDBTable {
    pub(in crate::internal) fn map(table: &TableArgs) -> SpacetimeDBTable {
        let singular_name = table.name.to_string().into();
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
        let name = index.name.to_string().into();
        let is_unique = index.is_unique;
        let r#type = match &index.kind {
            SpacetimeIndexType::Direct { column } => {
                let column = column.to_string().into();
                IndexType::Direct { column }
            }
            SpacetimeIndexType::BTree { columns } => {
                let columns: Vec<Box<str>> = columns.iter().map(|c| c.to_string().into()).collect();

                match columns.len() {
                    1 => IndexType::BTreeSingleColumn {
                        column: columns.get(0).unwrap().clone(),
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
        let reducer_name = scheduled.reducer.to_token_stream().to_string().into();

        ScheduledReducer { reducer_name }
    }
}
