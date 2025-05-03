use crate::api::{
    db::SpacetimeDBColumn,
    dsl::method::{
        SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex, SpacetimeDSLColumnMethodsForUniqueIndex,
        SpacetimeDSLTableMethods,
    },
};
use spacetime_bindings_macro_input::sats::SatsField;

pub mod create;

pub mod get_all;

pub mod get_count;

pub mod get_many;

pub mod delete_many;

pub mod get_one_option;

pub mod get_many_options;

pub mod update;

pub mod delete_one;

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn try_parse() -> syn::Result<SpacetimeDSLTableMethods> {
        todo!()
    }
}

pub(in crate::internal) fn get_column_dsl_methods(
    item: &syn::DeriveInput,
    field: &SatsField<'_>,
    spacetimedb_column: &SpacetimeDBColumn,
) -> Option<SpacetimeDSLColumnMethods> {
    let index = match &spacetimedb_column.single_column_index {
        None => {
            return None;
        }
        Some(index) => index,
    };

    let methods = match &index.is_unique {
        &false => {
            let get_many = get_many::build(item, field, spacetimedb_column);
            let delete_many = delete_many::build(item, field, spacetimedb_column);
            SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
                get_many,
                delete_many,
            })
        }
        &true => {
            let get_one_option = get_one_option::build(item, field, spacetimedb_column);
            let get_many_options = get_many_options::build(item, field, spacetimedb_column);
            let update = update::build(item, field, spacetimedb_column);
            let delete_one = delete_one::build(item, field, spacetimedb_column);
            SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                get_one_option,
                get_many_options,
                update,
                delete_one,
            })
        }
    };

    Some(methods)
}
