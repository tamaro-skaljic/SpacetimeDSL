use crate::api::{
    Column,
    db::{Index, IndexType, SpacetimeDBColumn, SpacetimeDBTable},
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
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

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
        primary_key_column_name: &Box<str>,
    ) -> syn::Result<SpacetimeDSLTableMethods> {
        let create = create::build(
            rust_struct,
            spacetimedb_table,
            columns,
            primary_key_column_name,
        );

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
                        primary_key_column_name,
                    );

                    let update = match spacetimedsl_table.is_mutable {
                        false => None,
                        true => Some(update::for_multi_column_index(
                            rust_struct,
                            spacetimedb_table,
                            multi_column_index,
                            spacetimedsl_table,
                            primary_key_column_name,
                        )),
                    };

                    let delete_one = delete_one::for_multi_column_index(
                        rust_struct,
                        spacetimedb_table,
                        multi_column_index,
                        spacetimedsl_table,
                        columns,
                        primary_key_column_name,
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
    pub(in crate::internal) fn map(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        rust_field: &RustField,
        spacetimedb_column: &SpacetimeDBColumn,
        spacetimedsl_column: &SpacetimeDSLColumn,
        primary_key_column_name: &Box<str>,
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
                        primary_key_column_name,
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

pub(in crate::internal::dsl::method) fn get_unique_multi_column_index_checks(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    primary_key_column_name: &Box<str>,
) -> Vec<TokenStream> {
    let struct_name = &rust_struct.name;
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);

    let mut multi_column_index_checks = vec![];

    for multi_column_index in &spacetimedb_table.multi_column_indices {
        let index_column_names = match &multi_column_index.index_type {
            IndexType::BTreeMultiColumn { columns } => columns,
            _ => {
                continue;
            }
        };

        let mut column_values = vec![];

        for column_name in index_column_names {
            column_values.push(quote! {#table_name.#column_name}.into());
        }

        multi_column_index_checks.push(get_unique_multi_column_index_check(
            struct_name,
            &table_name,
            multi_column_index,
            primary_key_column_name,
            column_values,
        ));
    }

    multi_column_index_checks
}

pub(in crate::internal::dsl::method) fn get_unique_multi_column_index_check(
    struct_name: &Box<str>,
    table_name: &Ident,
    multi_column_index: &Index,
    primary_key_column_name: &Box<str>,
    column_values: Vec<TokenStream>,
) -> TokenStream {
    let index_name = format_ident!("{}", multi_column_index.name.to_string());
    match multi_column_index.is_unique {
        false => {
            return TokenStream::default();
        }
        true => {
            let field_name_for_found_value = format!("the_same_or_another_{table_name}");

            let reasons = "There can be two reasons for this: You are inserting or updating somewhere using spacetimedb::ReducerContext instead of spacetimedsl::DSL or the unique multi-column index SpacetimeDSL feature is broken. Found: {:#?}";

            let mut more_than_one_panic_msg = format!(
                "There must be only one {struct_name} row inside the {table_name} table when filtering on the unique multi-column index {index_name} with value "
            );
            more_than_one_panic_msg.push_str("{:?}. Found more than one. ");
            more_than_one_panic_msg.push_str(reasons);

            let mut another_one_panic_msg = format!(
                "There must be only one {struct_name} row inside the {table_name} table when filtering on the unique multi-column index {index_name} with value "
            );
            another_one_panic_msg.push_str(
                "{:?}. Found another one with a different value in the primary key column. ",
            );
            another_one_panic_msg.push_str(reasons);

            return quote! {
                let #field_name_for_found_value = match ctx.db.#table_name().#index_name().filter((#(#column_values),*)).at_most_one() {
                    Ok(#table_name) => #table_name,
                    Err(e) => {
                        panic!(
                            #more_than_one_panic_msg,
                            (#(#column_values),*),
                            e.collect::<#struct_name>()
                        );
                    }
                };

                match &#field_name_for_found_value {
                    Some(#field_name_for_found_value) => {
                        if #field_name_for_found_value.#primary_key_column_name.ne(&#table_name.#primary_key_column_name) {
                            panic!(
                                #another_one_panic_msg,
                                (#(#column_values),*),
                                #field_name_for_found_value
                            );
                        }
                    },
                    _ => {},
                };
            };
        }
    }
}
