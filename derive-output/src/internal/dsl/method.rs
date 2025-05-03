use crate::api::{
    Column,
    db::{SpacetimeDBColumn, SpacetimeDBTable},
    dsl::{
        method::{
            SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
            SpacetimeDSLColumnMethodsForUniqueIndex, SpacetimeDSLTableMethods,
        },
        table::SpacetimeDSLTable,
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
    pub(in crate::internal) fn try_parse(
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        columns: &Vec<Column>,
    ) -> syn::Result<SpacetimeDSLTableMethods> {
        let create = create::build(spacetimedb_table, spacetimedsl_table, columns);
        let get_all = get_all::build(spacetimedb_table, spacetimedsl_table, columns);
        let get_count = get_count::build(spacetimedb_table, spacetimedsl_table, columns);
        let mut multi_column_indices = vec![];

        for multi_column_index in &spacetimedb_table.multi_column_indices {
            match multi_column_index.is_unique {
                false => {
                    let get_many = get_many::for_multi_column_index(
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                    );
                    let delete_many = delete_many::for_multi_column_index(
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                    );

                    multi_column_indices.push(SpacetimeDSLColumnMethods::ForIndex(
                        SpacetimeDSLColumnMethodsForIndex {
                            get_many,
                            delete_many,
                        },
                    ));
                }
                true => {
                    let get_one_option = get_one_option::for_multi_column_index(
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                    );
                    let get_many_options = get_many_options::for_multi_column_index(
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                    );
                    let update = update::for_multi_column_index(
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                    );
                    let delete_one = delete_one::for_multi_column_index(
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                    );

                    multi_column_indices.push(SpacetimeDSLColumnMethods::ForUniqueIndex(
                        SpacetimeDSLColumnMethodsForUniqueIndex {
                            get_one_option,
                            get_many_options,
                            update,
                            delete_one,
                        },
                    ));
                }
            };
        }

        let methods = SpacetimeDSLTableMethods {
            create,
            get_all,
            get_count,
            multi_column_indices,
        };

        Ok(methods)
    }
}

impl SpacetimeDSLColumnMethods {
    pub(in crate::internal) fn try_parse(
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
                let get_many = get_many::for_single_column_index(item, field, spacetimedb_column);
                let delete_many =
                    delete_many::for_single_column_index(item, field, spacetimedb_column);
                SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
                    get_many,
                    delete_many,
                })
            }
            &true => {
                let get_one_option =
                    get_one_option::for_single_column_index(item, field, spacetimedb_column);
                let get_many_options =
                    get_many_options::for_single_column_index(item, field, spacetimedb_column);
                let update = update::for_single_column_index(item, field, spacetimedb_column);
                let delete_one =
                    delete_one::for_single_column_index(item, field, spacetimedb_column);
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
}
