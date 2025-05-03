use crate::api::db::{Index, ScheduledReducer, SpacetimeDBTable, SpacetimeDBTableVisibility};
use spacetime_bindings_macro_input::table::TableArgs;

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
