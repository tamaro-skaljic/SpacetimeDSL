use crate::api::db::{Index, IndexType, ScheduledReducer, SpacetimeDBTableVisibility};
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{
    IndexArg, IndexType as SpacetimeIndexType, ScheduledArg, TableAccess,
};

pub mod column;

impl SpacetimeDBTableVisibility {
    pub(in crate::internal) fn map(access: &Option<TableAccess>) -> SpacetimeDBTableVisibility {
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
    pub(in crate::internal) fn map(index: &IndexArg) -> Index {
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
    pub(in crate::internal) fn map(scheduled: &ScheduledArg) -> ScheduledReducer {
        let name = scheduled.at.as_ref().unwrap().to_string().into();

        let path_to_reducer = scheduled.reducer.to_token_stream().to_string().into();

        ScheduledReducer {
            name,
            reducer_name: path_to_reducer,
        }
    }
}
