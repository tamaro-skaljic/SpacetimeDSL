use crate::api::db::{
    Index, IndexType, ScheduledReducer, SpacetimeDBTable, SpacetimeDBTableVisibility,
};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use spacetime_bindings_macro_input::table::{
    IndexArg, IndexType as SpacetimeIndexType, ScheduledArg, TableAccess, TableArgs,
};
use syn::{DeriveInput, Error};

pub(in crate::internal) trait ParseSpacetimeTable {
    fn try_parse(item: &DeriveInput) -> syn::Result<TableArgs>;
}

impl ParseSpacetimeTable for TableArgs {
    fn try_parse(item: &DeriveInput) -> syn::Result<TableArgs> {
        let input = get_table_attribute_macro(item)?;

        let table_args = TableArgs::parse(input, item)?;
        Ok(table_args)
    }
}

fn get_table_attribute_macro(item: &DeriveInput) -> syn::Result<TokenStream> {
    let mut table = None;

    for attr in item.attrs.iter() {
        match attr.meta.require_list() {
            Ok(list) => {
                if list
                    .path
                    .to_token_stream()
                    .to_string()
                    .eq("spacetimedb :: table")
                {
                    table = Some(attr);
                }
            }
            Err(_) => {}
        }
    }

    match table {
        Some(table) => Ok(table.to_token_stream()),
        None => Err(Error::new(
            Span::call_site(),
            "Haven't found #[spacetimedb::table] attribute macro!".to_string(),
        )),
    }
}

impl SpacetimeDBTable {
    pub(in crate::internal) fn map(table: &TableArgs) -> SpacetimeDBTable {
        let singular_name = table.name.to_string().into();
        let visibility = SpacetimeDBTableVisibility::map(&table.access);
        let indices = table.indices.iter().map(|i| Index::map(i)).collect();
        let scheduled_reducer = table.scheduled.as_ref().map(|s| ScheduledReducer::map(s));

        SpacetimeDBTable {
            singular_name,
            visibility,
            // Contains all indices during processing for the moment, but all single column indices are removed from the Vector in Column::try_parse().
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
        let name = scheduled.at.as_ref().unwrap().to_string().into();

        let path_to_reducer = scheduled.reducer.to_token_stream().to_string().into();

        ScheduledReducer {
            name,
            reducer_name: path_to_reducer,
        }
    }
}
