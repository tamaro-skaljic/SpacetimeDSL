use crate::api::{
    Column,
    db::{SpacetimeDBColumn, SpacetimeDBTable},
    dsl::{
        column::SpacetimeDSLColumn,
        method::{
            SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
            SpacetimeDSLColumnMethodsForUniqueIndex, SpacetimeDSLTableMethods,
        },
        table::SpacetimeDSLTable,
    },
    rust::{RustField, RustStruct},
};

pub mod create;

pub mod get_all;

pub mod get_count;

pub mod get_many;

pub mod delete_many;

pub mod get_one_option;

pub mod update;

pub mod delete_one;

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn try_parse(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        columns: &Vec<Column>,
    ) -> syn::Result<SpacetimeDSLTableMethods> {
        let create = create::build(rust_struct, spacetimedb_table, columns);
        let get_all = get_all::build(rust_struct, spacetimedb_table, spacetimedsl_table);
        let get_count = get_count::build(rust_struct, spacetimedb_table, spacetimedsl_table);
        let mut multi_column_indices = vec![];

        for multi_column_index in &spacetimedb_table.multi_column_indices {
            match multi_column_index.is_unique {
                false => {
                    let get_many = get_many::for_multi_column_index(
                        rust_struct,
                        spacetimedb_table,
                        multi_column_index,
                        spacetimedsl_table,
                        columns,
                    );
                    let delete_many = delete_many::for_multi_column_index(
                        rust_struct,
                        spacetimedb_table,
                        multi_column_index,
                        spacetimedsl_table,
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
                        rust_struct,
                        spacetimedb_table,
                        multi_column_index,
                        spacetimedsl_table,
                        columns,
                    );

                    let update = match spacetimedsl_table.is_mutable {
                        false => None,
                        true => Some(update::for_multi_column_index(
                            rust_struct,
                            spacetimedb_table,
                            multi_column_index,
                            spacetimedsl_table,
                            columns,
                        )),
                    };

                    let delete_one = delete_one::for_multi_column_index(
                        rust_struct,
                        spacetimedb_table,
                        multi_column_index,
                        spacetimedsl_table,
                        columns,
                    );

                    multi_column_indices.push(SpacetimeDSLColumnMethods::ForUniqueIndex(
                        SpacetimeDSLColumnMethodsForUniqueIndex {
                            get_one_option,
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
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        rust_field: &RustField,
        spacetimedb_column: &SpacetimeDBColumn,
        spacetimedsl_column: &SpacetimeDSLColumn,
    ) -> Option<SpacetimeDSLColumnMethods> {
        let index = match &spacetimedb_column.single_column_index {
            None => {
                return None;
            }
            Some(index) => index,
        };

        let methods = match &index.is_unique {
            &false => {
                let get_many = get_many::for_single_column_index(
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                );
                let delete_many = delete_many::for_single_column_index(
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                );
                SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
                    get_many,
                    delete_many,
                })
            }
            &true => {
                let get_one_option = get_one_option::for_single_column_index(
                    rust_struct,
                    spacetimedb_table,
                    rust_field,
                    spacetimedsl_column,
                );

                let update = match spacetimedsl_table.is_mutable {
                    false => None,
                    true => Some(update::for_single_column_index(
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        rust_field,
                    )),
                };
                let delete_one = delete_one::for_single_column_index(
                    rust_struct,
                    spacetimedb_table,
                    rust_field,
                    spacetimedsl_column,
                );
                SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                    get_one_option,
                    update,
                    delete_one,
                })
            }
        };

        Some(methods)
    }
}
