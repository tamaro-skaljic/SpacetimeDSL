use crate::{
    api::{
        db::{
            column::SpacetimeDBColumn,
            index::{Index, IndexType},
            table::SpacetimeDBTable,
        }, dsl::{
            column::{
                SpacetimeDSLColumn, SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex, SpacetimeDSLDeletionResult,
            },
            foreign_key::OnDeleteStrategy,
            method::SpacetimeDSLMethod,
            table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
            wrapper::WrapperType,
        }, rust::{column::RustField, table::RustStruct}, Column
    },
    internal::{column::InternalColumn, dsl::wrapper::wrapper_type_into_option},
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::collections::HashMap;
use syn::{parse_str, Ident, Path};

#[derive(Debug)]
pub(in crate::internal) enum DSLTableMethod {
    Create,
    GetAll,
    GetCount,
}

#[derive(Debug)]
pub(in crate::internal) enum DSLColumnMethod {
    GetMany,
    DeleteMany,
    GetOneOption,
    Update,
    DeleteOne,
}

//TODO: Remove if all methods are in one enum
#[derive(PartialEq)]
enum CreateOrUpdateDSLMethod {
    Create,
    Update
}

#[derive(Debug)]
pub(in crate::internal) enum DSLInternalReferencedByFunction {
    ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
    ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
}

#[derive(Debug)]
pub(in crate::internal) enum DSLInternalForeignKeyFunction {
    ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted,
    ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted,
}

impl SpacetimeDSLColumnMethods {
    pub(in crate::internal) fn map(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        rust_field: &RustField,
        spacetimedb_column: &SpacetimeDBColumn,
        spacetimedsl_column: &SpacetimeDSLColumn,
        primary_key_column_name: &Ident,
        internal_columns: &Vec<InternalColumn>,
    ) -> Option<SpacetimeDSLColumnMethods> {
        let index = match &spacetimedb_column.single_column_index {
            None => {
                return None;
            }
            Some(index) => index,
        };

        let methods = match index.is_unique {
            false => {
                let get_many = for_single_column_index(
                    DSLColumnMethod::GetMany,
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                    primary_key_column_name,
                    internal_columns,
                );

                let delete_many = for_single_column_index(
                    DSLColumnMethod::DeleteMany,
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                    primary_key_column_name,
                    internal_columns,
                );

                SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
                    get_many,
                    delete_many,
                    delete_many_result_type: SpacetimeDSLDeletionResult {}, // TODO
                })
            }
            true => {
                let get_one_option = for_single_column_index(
                    DSLColumnMethod::GetOneOption,
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                    primary_key_column_name,
                    internal_columns,
                );

                let update = match spacetimedsl_table.is_mutable {
                    false => None,
                    true => Some(for_single_column_index(
                        DSLColumnMethod::Update,
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        rust_field,
                        spacetimedsl_column,
                        primary_key_column_name,
                        internal_columns,
                    )),
                };

                let delete_one = for_single_column_index(
                    DSLColumnMethod::DeleteOne,
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                    primary_key_column_name,
                    internal_columns,
                );

                SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                    get_one_option,
                    update,
                    delete_one,
                    delete_one_result_type: SpacetimeDSLDeletionResult {}, // TODO
                })
            }
        };

        Some(methods)
    }
}

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn try_parse(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        columns: &Vec<Column>,
        primary_key_column_name: &Ident,
        internal_columns: &Vec<InternalColumn>,
    ) -> syn::Result<SpacetimeDSLTableMethods> {
        let create = for_table(
            DSLTableMethod::Create,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
        );

        let get_all = for_table(
            DSLTableMethod::GetAll,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
        );

        let get_count = for_table(
            DSLTableMethod::GetCount,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
        );

        let execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted;
        let execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted;

        if spacetimedsl_table.referencing_tables.is_empty() {
            execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted =
                None;
            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted = None;
        } else {
            execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted = Some(for_referenced_by(
                DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
                spacetimedb_table,
                spacetimedsl_table,
                columns,
                primary_key_column_name,
            ));

            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted = Some(for_referenced_by(
                DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
                spacetimedb_table,
                spacetimedsl_table,
                columns,
                primary_key_column_name,
            ));
        }

        let mut
        execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted =
            vec![];
        let mut
        execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted =
            vec![];

        let columns_with_foreign_keys: Vec<&Column> = columns
            .iter()
            .filter(|c| c.spacetimedsl_column.foreign_key.is_some())
            .collect();

        if !columns_with_foreign_keys.is_empty() {
            let mut columns_with_foreign_keys_by_table = HashMap::new();

            columns_with_foreign_keys.iter().for_each(|c| {
                let name_of_another_table = &c
                    .spacetimedsl_column
                    .foreign_key
                    .as_ref()
                    .expect("foreign key should exist")
                    .table_name;

                if !columns_with_foreign_keys_by_table.contains_key(name_of_another_table) {
                    columns_with_foreign_keys_by_table.insert(name_of_another_table, vec![]);
                }

                columns_with_foreign_keys_by_table
                    .get_mut(name_of_another_table)
                    .expect("key should exist in HashMap")
                    .push(c);
            });

            columns_with_foreign_keys_by_table
                .into_iter()
                .for_each(|(referenced_table_name, columns_with_foreign_key)| {
                    execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted.push(
                        for_foreign_key(
                            DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted,
                            rust_struct,
                            spacetimedb_table,
                            spacetimedsl_table,
                            columns,
                            referenced_table_name,
                            &columns_with_foreign_key,
                            primary_key_column_name,
                        )
                    );
                    execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted.push(
                        for_foreign_key(
                            DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted,
                            rust_struct,
                            spacetimedb_table,
                            spacetimedsl_table,
                            columns,
                            referenced_table_name,
                            &columns_with_foreign_key,
                            primary_key_column_name,
                        )
                    );
                });
        }

        let mut multi_column_indices = vec![];

        for multi_column_index in &spacetimedb_table.multi_column_indices {
            match multi_column_index.is_unique {
                false => {
                    let get_many = for_multi_column_index(
                        DSLColumnMethod::GetMany,
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                        primary_key_column_name,
                        internal_columns,
                    );
                    let delete_many = for_multi_column_index(
                        DSLColumnMethod::DeleteMany,
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                        primary_key_column_name,
                        internal_columns,
                    );

                    multi_column_indices.push(SpacetimeDSLColumnMethods::ForIndex(
                        SpacetimeDSLColumnMethodsForIndex {
                            get_many,
                            delete_many,
                            delete_many_result_type: SpacetimeDSLDeletionResult {}, // TODO
                        },
                    ));
                }
                true => {
                    let get_one_option = for_multi_column_index(
                        DSLColumnMethod::GetOneOption,
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                        primary_key_column_name,
                        internal_columns,
                    );

                    let update = match spacetimedsl_table.is_mutable {
                        false => None,
                        true => Some(for_multi_column_index(
                            DSLColumnMethod::Update,
                            rust_struct,
                            spacetimedb_table,
                            spacetimedsl_table,
                            multi_column_index,
                            columns,
                            primary_key_column_name,
                            internal_columns,
                        )),
                    };

                    let delete_one = for_multi_column_index(
                        DSLColumnMethod::DeleteOne,
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                        primary_key_column_name,
                        internal_columns,
                    );

                    multi_column_indices.push(SpacetimeDSLColumnMethods::ForUniqueIndex(
                        SpacetimeDSLColumnMethodsForUniqueIndex {
                            get_one_option,
                            update,
                            delete_one,
                            delete_one_result_type: SpacetimeDSLDeletionResult {}, // TODO
                        },
                    ));
                }
            };
        }

        let methods = SpacetimeDSLTableMethods {
            create,
            get_all,
            get_count,
            execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted,
            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted,
            execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted,
            execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted,
            multi_column_indices,
        };

        Ok(methods)
    }
}

fn process_columns_for_create_and_update_method(create_or_update: CreateOrUpdateDSLMethod, internal_column: &InternalColumn) -> (Option<TokenStream>, Option<TokenStream>, Option<TokenStream>, TokenStream) {
    let mut method_arg = None;
    let mut into_option = None;
    let mut constructor_arg = None;
    let constructor_arg_name;

    let singular_table_name = &internal_column.spacetimedb_table_singular_name;
    let column_name = &internal_column.rust_field_name;
    let getter_name = format_ident!("get_{column_name}");
    constructor_arg_name = quote! { #column_name };

    let column_type = &internal_column.rust_field_type_name_or_path;

    match create_or_update {
        CreateOrUpdateDSLMethod::Create => {
            // TODO: Allow Option<Timestamp> as modified_at column type
            if internal_column.spacetimedb_is_auto_inc
                || internal_column.rust_field_name.to_string().eq(&"created_at")
                || internal_column.rust_field_name.to_string().eq(&"modified_at")
            {
                if internal_column.spacetimedb_is_auto_inc {
                    constructor_arg = Some(quote! {
                        let #column_name = #column_type::default();
                    });
                } else if internal_column.rust_field_name.to_string().eq(&"created_at") {
                    constructor_arg = Some(quote! {
                        let created_at = self.ctx().timestamp;
                    });
                } else if internal_column.rust_field_name.to_string().eq(&"modified_at") {
                    constructor_arg = Some(quote! {
                        let modified_at = self.ctx().timestamp;
                    });
                }
                return (method_arg, into_option, constructor_arg, constructor_arg_name);
            }
        },
        CreateOrUpdateDSLMethod::Update => {

        },
    };

    match &internal_column.spacetimedsl_wrapper_type {
        Some(wrapper_type) => match wrapper_type {
            WrapperType::Wrap(wrapper_type) => {
                if internal_column.rust_field_type_name_or_path.to_token_stream().to_string().eq(&"String") {
                    method_arg = Some(quote! {
                        #column_name: &str
                    });
                    match create_or_update {
                        CreateOrUpdateDSLMethod::Create => {
                            constructor_arg = Some(quote! {
                                let #column_name = #column_name.to_string();
                            });
                        },
                        CreateOrUpdateDSLMethod::Update => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#getter_name();
                            });
                        },
                    };
                } else {
                    let wrapped_type_name_or_path =
                        WrapperType::map_to_wrapped_type(wrapper_type);
                    method_arg = Some(quote! {
                        #column_name: #wrapped_type_name_or_path
                    });
                }
            }
            WrapperType::Wrapped(_) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if internal_column.spacetimedsl_is_option {
                    method_arg = Some(quote! {
                        #column_name: impl Into<Option<#wrapper_type_name_or_path>>
                    });
                    into_option = Some(wrapper_type_into_option(
                        &column_name,
                        wrapper_type_name_or_path,
                    ));
                } else {
                    method_arg = Some(quote! {
                        #column_name: impl Into<#wrapper_type_name_or_path>
                    });
                    match create_or_update {
                        CreateOrUpdateDSLMethod::Create => {
                            constructor_arg = Some(quote! {
                                let #column_name = #column_name.into().value();
                            });
                        },
                        CreateOrUpdateDSLMethod::Update => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#getter_name().value();
                            });
                        },
                    };
                }
            }
        },
        None => {
            if internal_column.rust_field_type_name_or_path.to_token_stream().to_string().eq(&"String") {
                method_arg = Some(quote! {
                    #column_name: &str
                });

                match create_or_update {
                    CreateOrUpdateDSLMethod::Create => {
                        constructor_arg = Some(quote! {
                            let #column_name = #column_name.to_string();
                        });
                    },
                    CreateOrUpdateDSLMethod::Update => {
                        constructor_arg = Some(quote! {
                            let #column_name = #singular_table_name.#getter_name();
                        });
                    },
                };
            } else {
                method_arg = Some(quote! {
                    #column_name: #column_type
                });
            }
        }
    };

    (method_arg, into_option, constructor_arg, constructor_arg_name)
}

pub(in crate::internal) fn for_table(
    dsl_table_method: DSLTableMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_columns: &Vec<InternalColumn>,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;

    let doc_comment;
    let trait_name;
    let method_name;
    let return_type;

    match dsl_table_method {
        // TODO: Let foreign_key's influence the doc comment
        DSLTableMethod::Create => {
            doc_comment = format!("Create a row in the `{singular_table_name}` table.");
            trait_name = format_ident!("Create{singular_table_name_pascal_case}Row");
            method_name = format_ident!("create_{}", singular_table_name);

            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            return_type = quote! {
                Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
            };
        }
        DSLTableMethod::GetAll => {
            doc_comment = format!("Get all rows inside the `{singular_table_name}` table.");
            trait_name = format_ident!("GetAll{singular_table_name_pascal_case}Rows");
            method_name = format_ident!("get_all_{}", plural_table_name);
            return_type = quote! {
                impl Iterator<Item = #struct_name>
            };
        }
        DSLTableMethod::GetCount => {
            doc_comment = format!("Count all rows inside the `{singular_table_name}` table.");
            trait_name = format_ident!("CountOfAll{singular_table_name_pascal_case}Rows");
            method_name = format_ident!("count_of_all_{}", plural_table_name);
            return_type = quote! {
                u64
            };
        }
    }

    let doc_comment = doc_comment.into();

    let mut paths_of_traits_to_extend = vec![ parse_str("spacetimedsl::DSLContext").expect("parsing should have worked") ];
    let mut method_args = vec![];
    let method_impl;

    match dsl_table_method {
        DSLTableMethod::Create => {
            let mut into_options = vec![];
            let mut constructor_args = vec![];
            let mut constructor_arg_names = vec![];

            for internal_column in internal_columns {
                let (method_arg, into_option, constructor_arg, constructor_arg_name) = process_columns_for_create_and_update_method(CreateOrUpdateDSLMethod::Create, &internal_column);
                match method_arg {
                    Some(method_arg) => method_args.push(method_arg),
                    None => {},
                }

                match into_option {
                    Some(into_option) => into_options.push(into_option),
                    None => {},
                }

                match constructor_arg {
                    Some(constructor_arg) => constructor_args.push(constructor_arg),
                    None => {},
                }

                constructor_arg_names.push(constructor_arg_name)
            }

            let mut multi_column_index_checks = get_unique_multi_column_index_checks(
                &struct_name,
                &singular_table_name,
                &spacetimedb_table,
            );

            for multi_column_index_check in &mut multi_column_index_checks {
                let field_name_for_found_value =
                    format_ident!("the_same_or_another_{singular_table_name}");
                let index_name = &multi_column_index_check.index_name;

                let mut another_one_panic_msg = format!(
                    "There must be no {struct_name} row inside the {singular_table_name} table when filtering on the unique multi-column index {index_name} with value "
                );
                another_one_panic_msg.push_str(
                "{:?}. Found another one with a different value in the primary key column: {:?}. There can be two reasons for this: You are inserting or updating somewhere using spacetimedb::ReducerContext instead of spacetimedsl::DSL or the unique multi-column index SpacetimeDSL feature is broken.",
            );

                multi_column_index_check.check.append_all(quote! {
                    match &#field_name_for_found_value {
                        Some(#field_name_for_found_value) => {
                            use spacetimedb::table::MaybeError;
                            return Err(spacetimedb::UniqueConstraintViolation::get()
                                .map(spacetimedb::TryInsertError::UniqueConstraintViolation)
                                .expect("Mapping should have worked"));
                        },
                        _ => {},
                    };
                });
            }

            let multi_column_index_checks: Vec<TokenStream> = multi_column_index_checks
                .into_iter()
                .map(|mcic| mcic.check)
                .collect();

            let use_itertools = if multi_column_index_checks.len() > 0 {
                quote! {
                    use spacetimedsl::itertools::Itertools;
                }
            } else {
                TokenStream::default()
            };

            let res = reference_integrity_checks(
                CreateOrUpdateDSLMethod::Create,
                spacetimedb_table,
                &internal_columns,
                paths_of_traits_to_extend
            );
            paths_of_traits_to_extend = res.0;
            let reference_integrity_checks = res.1;

            method_impl = quote! {
                #use_itertools

                #(#constructor_args)*
                #(#into_options)*
                let #singular_table_name = #struct_name {
                    #(#constructor_arg_names),*
                };

                #(#multi_column_index_checks)*

                #(#reference_integrity_checks)*

                self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .try_insert(#singular_table_name)
            }
        }
        DSLTableMethod::GetAll => {
            method_impl = quote! {
                self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .iter()
            };
        }
        DSLTableMethod::GetCount => {
            method_impl = quote! {
                self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .count()
            };
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        paths_of_traits_to_extend,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
}

pub(in crate::internal) fn for_single_column_index(
    dsl_method: DSLColumnMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    rust_field: &RustField,
    spacetimedsl_column: &SpacetimeDSLColumn,
    primary_key_column_name: &Ident,
    internal_columns: &Vec<InternalColumn>,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;
    let column_name = &rust_field.name;
    let column_name_pascal_case = RenameRule::PascalCase.apply_to_field(column_name.to_string());
    let primary_key_column_name = format_ident!("{primary_key_column_name}");

    let doc_comment;
    let trait_name;
    let method_name;
    let return_type;

    match dsl_method {
        DSLColumnMethod::GetMany => {
            doc_comment = format!(
                "Get all {struct_name} rows inside the {singular_table_name} table filtered by the single-column index on the {column_name} column."
            );
            trait_name = format_ident!("Get{singular_table_name_pascal_case}RowsBy{column_name_pascal_case}");
            method_name = format_ident!("get_{plural_table_name}_by_{column_name}");
            return_type = quote! {
                impl Iterator<Item = #struct_name>
            };
        }
        DSLColumnMethod::DeleteMany => {
            // TODO: Let referenced_by's influence the doc comment
            doc_comment = format!(
                "Delete all {struct_name} rows inside the {singular_table_name} table filtered by the single-column index on the {column_name} column."
            );
            trait_name = format_ident!("Delete{singular_table_name_pascal_case}RowsBy{column_name_pascal_case}");
            method_name = format_ident!("delete_{plural_table_name}_by_{column_name}");
            // TODO: Result<spacetimedsl::DeletionResult, spacetimedsl::ReferenceIntegrityViolationError>
            return_type = quote! {Result<u64,()>};
        }
        DSLColumnMethod::GetOneOption => {
            doc_comment = format!(
                "Get an Option<{struct_name}> row inside the {singular_table_name} table filtered by the unique single-column index on the {column_name} column."
            );
            trait_name = format_ident!("Get{singular_table_name_pascal_case}RowOptionBy{column_name_pascal_case}");
            method_name = format_ident!("get_{singular_table_name}_by_{column_name}");
            return_type = quote! {
                Option<#struct_name>
            };
        }
        DSLColumnMethod::Update => {
            // TODO: Let foreign_key's influence the doc comment
            doc_comment = format!(
                "Update a {struct_name} row inside the {singular_table_name} table by the unique single-column index on the {column_name} column."
            );
            trait_name = format_ident!("Update{singular_table_name_pascal_case}RowBy{column_name_pascal_case}");
            method_name = format_ident!("update_{singular_table_name}_by_{column_name}");

            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            return_type = quote! {
                Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
            };
        }
        DSLColumnMethod::DeleteOne => {
            // TODO: Let referenced_by's influence the doc comment
            doc_comment = format!(
                "Delete a {struct_name} row inside the {singular_table_name} table filtered by the unique single-column index on the {column_name} column."
            );
            trait_name = format_ident!("Delete{singular_table_name_pascal_case}RowBy{column_name_pascal_case}");
            method_name = format_ident!("delete_{singular_table_name}_by_{column_name}");
            // TODO: Result<spacetimedsl::DeletionResult, spacetimedsl::ReferenceIntegrityViolationError>
            return_type = quote! {Result<bool,()>};
        }
    }

    let doc_comment = doc_comment.into();

    let mut paths_of_traits_to_extend = vec![ parse_str("spacetimedsl::DSLContext").expect("parsing should have worked") ];
    let mut method_args = vec![];
    let method_impl;

    match dsl_method {
        DSLColumnMethod::Update => {
            method_args.push(quote! { mut #singular_table_name: #struct_name });

            let multi_column_index_checks = multi_column_index_checks(
                &struct_name,
                &singular_table_name,
                &spacetimedb_table,
                &primary_key_column_name,
            );
            
            let mut column_getters = vec![];

            internal_columns.iter().filter(|internal_column| internal_column.spacetimedsl_column_foreign_key.is_some()).for_each(|internal_column| {
                let (_, _, column_getter, _) = process_columns_for_create_and_update_method(CreateOrUpdateDSLMethod::Update, &internal_column);

                match column_getter {
                    Some(column_getter) => column_getters.push(column_getter),
                    None => {},
                };
            });
            
            // TODO: Allow Option<Timestamp> as modified_at column type
            let modified_at = match spacetimedsl_table.has_modified_at_column {
                false => TokenStream::default(),
                true => {
                    quote! {
                        #singular_table_name.modified_at = self.ctx().timestamp;
                    }
                }
            };

            let use_itertools = if multi_column_index_checks.len() > 0 {
                quote! {
                    use spacetimedsl::itertools::Itertools;
                }
            } else {
                TokenStream::default()
            };

            let res = reference_integrity_checks(
                CreateOrUpdateDSLMethod::Update,
                spacetimedb_table,
                internal_columns,
                paths_of_traits_to_extend
            );
            paths_of_traits_to_extend = res.0;
            let reference_integrity_checks = res.1;

            method_impl = quote! {
                #use_itertools

                #(#multi_column_index_checks)*
                
                #(#column_getters)*
                #(#reference_integrity_checks)*

                #modified_at

                Ok(self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .#column_name()
                    .update(#singular_table_name)
                )
            };
        }
        dsl_method => {
            let mut into_option = TokenStream::default();
            let method_arg;
            let column_value;

            match &spacetimedsl_column.wrapper_type {
                Some(wrapper_type) => {
                    let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);
                    // TODO: Also special cases for Strings in multi-column indices?
                    if rust_field.type_name_or_path.to_token_stream().to_string().eq(&"String") {
                        match &dsl_method {
                            DSLColumnMethod::GetMany | DSLColumnMethod::DeleteMany => {
                                method_arg = quote! { #column_name: &str };
                                column_value = quote! { #column_name };
                            }
                            DSLColumnMethod::GetOneOption | DSLColumnMethod::DeleteOne => {
                                method_arg = quote! { #column_name: &str };
                                column_value = quote! { #column_name.to_string() };
                            }
                            DSLColumnMethod::Update => {
                                panic!("Update DSLColumnMethod should already be processed.")
                            }
                        }
                    } else {
                        if spacetimedsl_column.is_option {
                            into_option =
                                wrapper_type_into_option(&column_name, wrapper_type_name_or_path);

                            method_arg = quote! { #column_name: &impl Into<Option<#wrapper_type_name_or_path>> };

                            column_value = quote! { #column_name };
                        } else {
                            method_arg =
                                quote! { #column_name: impl Into<#wrapper_type_name_or_path> };
                            column_value = quote! { #column_name.into().value() };
                        }
                    }
                }
                None => match dsl_method {
                    DSLColumnMethod::GetMany => {
                        let column_type;
                        if rust_field.type_name_or_path.to_token_stream().to_string().eq(&"String") {
                            column_type = parse_str("str").expect("parsing should have worked");
                        } else {
                            column_type = rust_field.type_name_or_path.clone();
                        }

                        let ma = quote! {
                            &'a #column_type
                        };

                        method_arg = quote! { #column_name: #ma };

                        column_value = quote! {
                            #column_name
                        };
                    }
                    DSLColumnMethod::DeleteMany => {
                        let column_type;
                        if rust_field.type_name_or_path.to_token_stream().to_string().eq(&"String") {
                            column_type = parse_str("str").expect("parsing should have worked");
                        } else {
                            column_type = rust_field.type_name_or_path.clone();
                        }

                        method_arg = quote! { #column_name: &'a #column_type };

                        column_value = quote! {
                                #column_name
                        };
                    }
                    DSLColumnMethod::GetOneOption => {
                        if rust_field.type_name_or_path.to_token_stream().to_string().eq(&"String") {
                            method_arg = quote! { #column_name: &str };

                            column_value = quote! { #column_name.to_string() };
                        } else {
                            let column_type = &rust_field.type_name_or_path;
                            method_arg = quote! { #column_name: &#column_type };

                            column_value = quote! {
                                #column_name
                            };
                        }
                    }
                    DSLColumnMethod::DeleteOne => {
                        let column_type = &rust_field.type_name_or_path;
                        method_arg = quote! { #column_name: &#column_type };

                        column_value = quote! {
                                #column_name
                        };
                    }
                    DSLColumnMethod::Update => {
                        panic!("Update DSLColumnMethod should already be processed.")
                    }
                },
            };

            method_args.push(method_arg);

            let method_impl_prefix = quote! {
                    self
                        .ctx()
                        .db()
                        .#singular_table_name()
                        .#column_name()
            };

            method_impl = match dsl_method {
                DSLColumnMethod::GetMany => quote! {
                    #into_option
                    #method_impl_prefix
                        .filter(#column_value)
                },
                DSLColumnMethod::DeleteMany => {
                    if spacetimedsl_table.referencing_tables.is_empty() {
                        quote! {
                            #into_option
                            Ok(
                                #method_impl_prefix
                                    .delete(#column_value)
                            )
                        }
                    } else {
                        let referenced_table_function_name = get_referenced_table_function_name(
                            &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
                            &singular_table_name
                        );

                        let primary_key_column = internal_columns
                                .iter()
                                .find(|c| c.rust_field_name.eq(&primary_key_column_name))
                                .expect("should have a primary key");

                        let primary_key_column_type = &primary_key_column.rust_field_type_name_or_path;

                        quote! {
                            #into_option

                            let #column_name = #column_value;

                            let primary_key_values_of_rows_to_delete: Vec<#primary_key_column_type> = #method_impl_prefix
                                .filter(#column_name)
                                .map(|row| row.#primary_key_column_name)
                                .collect(); // TODO: maybe some types need a .clone() after #column_name

                            if primary_key_values_of_rows_to_delete.is_empty() {
                                return Ok(0);
                            }

                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, &primary_key_values_of_rows_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, &primary_key_values_of_rows_to_delete)?;
                            //TODO: spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, &primary_key_values_of_rows_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, &primary_key_values_of_rows_to_delete)?;

                            Ok(
                                #method_impl_prefix
                                    .delete(#column_name)
                            )
                        }
                    }
                }
                DSLColumnMethod::GetOneOption => quote! {
                    #into_option
                    #method_impl_prefix
                        .find(#column_value)
                },
                DSLColumnMethod::DeleteOne => {
                    if spacetimedsl_table.referencing_tables.is_empty() {
                        quote! {
                            #into_option
                            Ok(
                                #method_impl_prefix
                                    .delete(#column_value)
                            )
                        }
                    } else {
                        let referenced_table_function_name = get_referenced_table_function_name(
                            &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
                            &singular_table_name
                        );

                        quote! {
                            #into_option

                            let #column_name = #column_value;

                            let row_to_delete = #method_impl_prefix
                                .find(#column_name); // TODO: maybe some types need a .clone() after #column_name

                            let primary_key_value_of_row_to_delete;

                            match row_to_delete {
                                None => { return Ok(false); },
                                Some(row) => { primary_key_value_of_row_to_delete = row.#primary_key_column_name; },
                            };

                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, &primary_key_value_of_row_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, &primary_key_value_of_row_to_delete)?;
                            //TODO: spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, &primary_key_value_of_row_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, &primary_key_value_of_row_to_delete)?;

                            Ok(
                                #method_impl_prefix
                                    .delete(#column_name)
                            )
                        }
                    }
                }
                DSLColumnMethod::Update => {
                    panic!("Update DSLColumnMethod should already be processed.")
                }
            }
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        paths_of_traits_to_extend,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
}

pub(in crate::internal) fn for_multi_column_index(
    dsl_method: DSLColumnMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    multi_column_index: &Index,
    columns: &[Column],
    primary_key_column_name: &Ident,
    internal_columns: &Vec<InternalColumn>,
) -> SpacetimeDSLMethod {
    let index_columns = match &multi_column_index.index_type {
        IndexType::BTreeMultiColumn { columns } => columns,
        i => {
            panic!(
                "There shouldn't be an index with another type when this code is running. Found: {:#?}",
                i
            )
        }
    };

    let struct_name = &rust_struct.name;
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;
    let index_name = &multi_column_index.name;
    let index_name_pascal_case = RenameRule::PascalCase.apply_to_field(index_name.to_string());
    let primary_key_column_name = format_ident!("{primary_key_column_name}");

    let panic_reason = "Panics if it finds more than one, because then the unique constraint is violated somewhere";
    let doc_comment = match dsl_method {
        DSLColumnMethod::GetMany => format!("Get all {struct_name} rows inside the {singular_table_name} table filtered by the multi-column index {index_name}."),
        DSLColumnMethod::DeleteMany => format!("Delete all {struct_name} rows inside the {singular_table_name} table filtered by the multi-column index {index_name}."),
        DSLColumnMethod::GetOneOption => format!("Get an Option<{struct_name}> row inside the {singular_table_name} table filtered by the unique multi-column index {index_name}.\n\n{panic_reason}."),
        DSLColumnMethod::Update => format!("Update a {struct_name} row inside the {singular_table_name} table by the unique multi-column index {index_name}.\n\n{panic_reason}."),
        DSLColumnMethod::DeleteOne => format!("Delete a {struct_name} row inside the {singular_table_name} table by the unique multi-column index {index_name}.\n\n{panic_reason}."),
    }
    .into();

    let trait_name = match dsl_method {
        DSLColumnMethod::GetMany => format_ident!("Get{singular_table_name_pascal_case}RowsBy{index_name_pascal_case}"),
        DSLColumnMethod::DeleteMany => format_ident!("Delete{singular_table_name_pascal_case}RowsBy{index_name_pascal_case}"),
        DSLColumnMethod::GetOneOption => {
            format_ident!("Get{singular_table_name_pascal_case}RowOptionBy{index_name_pascal_case}")
        }
        DSLColumnMethod::Update => format_ident!("Update{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"),
        DSLColumnMethod::DeleteOne => format_ident!("Delete{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"),
    };

    let method_name = match dsl_method {
        DSLColumnMethod::GetMany => format_ident!("get_{plural_table_name}_by_{index_name}"),
        DSLColumnMethod::DeleteMany => format_ident!("delete_{plural_table_name}_by_{index_name}"),
        DSLColumnMethod::GetOneOption => format_ident!("get_{singular_table_name}_by_{index_name}"),
        DSLColumnMethod::Update => format_ident!("update_{singular_table_name}_by_{index_name}"),
        DSLColumnMethod::DeleteOne => format_ident!("delete_{singular_table_name}_by_{index_name}"),
    };

    let return_type = match dsl_method {
        DSLColumnMethod::GetMany => quote! {impl Iterator<Item = #struct_name>},
        // TODO: Result<spacetimedsl::DeletionResult, spacetimedsl::ReferenceIntegrityViolationError>
        DSLColumnMethod::DeleteMany => quote! {Result<u64,()>},
        DSLColumnMethod::GetOneOption => quote! {Option<#struct_name>},
        DSLColumnMethod::Update => {
            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            quote! {Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>}
        },
        // TODO: Result<spacetimedsl::DeletionResult, spacetimedsl::ReferenceIntegrityViolationError>
        DSLColumnMethod::DeleteOne => quote! {Result<bool,()>},
    };

    let mut paths_of_traits_to_extend = vec![ parse_str("spacetimedsl::DSLContext").expect("parsing should have worked") ];
    let mut method_args = vec![];
    let method_impl;

    match dsl_method {
        DSLColumnMethod::Update => {
            method_args.push(quote! { mut #singular_table_name: #struct_name });

            let multi_column_index_checks = multi_column_index_checks(
                &struct_name,
                &singular_table_name,
                &spacetimedb_table,
                &primary_key_column_name,
            );

            let mut column_getters = vec![];

            internal_columns.iter().filter(|internal_column| internal_column.spacetimedsl_column_foreign_key.is_some()).for_each(|internal_column| {
                let (_, _, column_getter, _) = process_columns_for_create_and_update_method(CreateOrUpdateDSLMethod::Update, &internal_column);

                match column_getter {
                    Some(column_getter) => column_getters.push(column_getter),
                    None => {},
                };
            });

            let modified_at = match spacetimedsl_table.has_modified_at_column {
                false => TokenStream::default(),
                true => {
                    quote! {
                        #singular_table_name.modified_at = self.ctx().timestamp;
                    }
                }
            };

            let use_itertools = if multi_column_index_checks.len() > 0 {
                quote! {
                    use spacetimedsl::itertools::Itertools;
                }
            } else {
                TokenStream::default()
            };

            let res = reference_integrity_checks(
                CreateOrUpdateDSLMethod::Update,
                spacetimedb_table,
                internal_columns,
                paths_of_traits_to_extend
            );
            paths_of_traits_to_extend = res.0;
            let reference_integrity_checks = res.1;

            method_impl = quote! {
                #use_itertools

                #(#multi_column_index_checks)*
                
                #(#column_getters)*
                #(#reference_integrity_checks)*

                #modified_at

                Ok(self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .#primary_key_column_name()
                    .update(#singular_table_name)
                )
            };
        }
        dsl_method => {
            let mut into_options = vec![];

            let mut column_values = vec![];

            for column in columns {
                let mut into_option = TokenStream::default();
                let method_arg;
                let column_value;

                if !index_columns.contains(&column.rust_field.name) {
                    continue;
                }

                let column_name = &column.rust_field.name;

                match &column.spacetimedsl_column.wrapper_type {
                    Some(wrapper_type) => {
                        let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                        if column.spacetimedsl_column.is_option {
                            into_option =
                                wrapper_type_into_option(&column_name, wrapper_type_name_or_path);

                            method_arg = quote! { #column_name: &impl Into<Option<#wrapper_type_name_or_path>> };

                            match &dsl_method {
                                DSLColumnMethod::GetMany | DSLColumnMethod::DeleteMany => {
                                    column_value = quote! { #column_name };
                                }
                                DSLColumnMethod::GetOneOption | DSLColumnMethod::DeleteOne => {
                                    column_value = quote! { #column_name };
                                }
                                DSLColumnMethod::Update => {
                                    panic!("Update DSLColumnMethod should already be processed.")
                                }
                            }
                        } else {
                            match &dsl_method {
                                DSLColumnMethod::GetMany | DSLColumnMethod::DeleteMany => {
                                    method_arg = quote! { #column_name: impl Into<#wrapper_type_name_or_path> };
                                    column_value = quote! { #column_name.into().value() };
                                }
                                DSLColumnMethod::GetOneOption | DSLColumnMethod::DeleteOne => {
                                    method_arg = quote! { #column_name: impl Into<#wrapper_type_name_or_path> + Clone };
                                    column_value = quote! { #column_name.clone().into().value() };
                                }
                                DSLColumnMethod::Update => {
                                    panic!("Update DSLColumnMethod should already be processed.")
                                }
                            }
                        }
                    }
                    None => {
                        let column_type = &column.rust_field.type_name_or_path;

                        match dsl_method {
                            DSLColumnMethod::GetMany | DSLColumnMethod::DeleteMany => {
                                method_arg = quote! { #column_name: &'a #column_type };
                            }
                            DSLColumnMethod::GetOneOption | DSLColumnMethod::DeleteOne => {
                                method_arg = quote! { #column_name: &#column_type };
                            }
                            DSLColumnMethod::Update => {
                                panic!("Update DSLColumnMethod should already be processed.")
                            }
                        }

                        column_value = quote! { #column_name };
                    }
                };

                method_args.push(method_arg);
                into_options.push(into_option);
                column_values.push(column_value);
            }

            match dsl_method {
                DSLColumnMethod::GetMany | DSLColumnMethod::DeleteMany => {
                    let method_impl_prefix = quote! {
                        self
                            .ctx()
                            .db()
                            .#singular_table_name()
                            .#index_name()
                    };

                    method_impl = match dsl_method {
                        DSLColumnMethod::GetMany => quote! {
                            #(#into_options)*
                            #method_impl_prefix
                                .filter((#(#column_values),*))
                        },

                        DSLColumnMethod::DeleteMany => {
                            if spacetimedsl_table.referencing_tables.is_empty() {
                                quote! {
                                    #(#into_options)*
                                    Ok(#method_impl_prefix
                                        .delete((#(#column_values),*)))
                                }
                            } else {
                                let referenced_table_function_name = get_referenced_table_function_name(
                                    &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
                                    &singular_table_name
                                );

                                let primary_key_column = internal_columns
                                    .iter()
                                    .find(|c| c.rust_field_name.eq(&primary_key_column_name))
                                    .expect("should have a primary key");

                                let primary_key_column_type = &primary_key_column.rust_field_type_name_or_path;
                                quote! {
                                    #(#into_options)*

                                    let #index_name = (#(#column_values),*);

                                    let primary_key_values_of_rows_to_delete: Vec<#primary_key_column_type> = #method_impl_prefix
                                        .filter(#index_name)
                                        .map(|row| row.#primary_key_column_name)
                                        .collect();

                                    if primary_key_values_of_rows_to_delete.is_empty() {
                                        return Ok(0);
                                    }

                                    spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, &primary_key_values_of_rows_to_delete)?;
                                    spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, &primary_key_values_of_rows_to_delete)?;
                                    //TODO: spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, &primary_key_values_of_rows_to_delete)?;
                                    spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, &primary_key_values_of_rows_to_delete)?;

                                    Ok(
                                        #method_impl_prefix
                                            .delete(#index_name)
                                    )
                                }
                            }
                        },
                        _ => {
                            panic!("Should be processed elsewhere.")
                        }
                    }
                }
                DSLColumnMethod::GetOneOption | DSLColumnMethod::DeleteOne => {
                    let multi_column_index_check = get_unique_multi_column_index_check(
                        &struct_name,
                        &singular_table_name,
                        &index_name,
                        &column_values,
                    )
                    .check;

                    let method_impl_prefix = quote! {
                        use spacetimedsl::itertools::Itertools;

                        #(#into_options)*

                        #multi_column_index_check
                    };

                    let field_name_for_found_value =
                        format_ident!("the_same_or_another_{singular_table_name}");

                    method_impl = match dsl_method {
                        DSLColumnMethod::GetOneOption => quote! {
                            #method_impl_prefix

                                #field_name_for_found_value
                        },
                        DSLColumnMethod::DeleteOne => {
                            if spacetimedsl_table.referencing_tables.is_empty() {
                                quote! {
                                    #method_impl_prefix

                                    Ok(
                                        self
                                            .ctx()
                                            .db()
                                            .#singular_table_name()
                                            .#primary_key_column_name()
                                            .delete(#field_name_for_found_value.expect("value should be found").#primary_key_column_name)
                                    )
                                }
                            } else {
                                let referenced_table_function_name = get_referenced_table_function_name(
                                    &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
                                    &singular_table_name
                                );

                                quote! {
                                    #(#into_options)*

                                    let #index_name = (#(#column_values),*);

                                    let row_to_delete = #method_impl_prefix
                                        .find(#index_name);

                                    let primary_key_value_of_row_to_delete;

                                    match row_to_delete {
                                        None => { return Ok(false); },
                                        Some(row) => { primary_key_value_of_row_to_delete = row.#primary_key_column_name; },
                                    };

                                    spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, &primary_key_value_of_row_to_delete)?;
                                    spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, &primary_key_value_of_row_to_delete)?;
                                    //TODO: spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, &primary_key_value_of_row_to_delete)?;
                                    spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, &primary_key_value_of_row_to_delete)?;

                                    Ok(
                                        #method_impl_prefix
                                            .delete(#index_name)
                                    )
                                }
                            }
                        },
                        _ => {
                            panic!("Should be processed elsewhere.")
                        }
                    }
                }
                DSLColumnMethod::Update => {
                    panic!("Update DSLColumnMethod should already be processed.")
                }
            }
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        paths_of_traits_to_extend,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
}

fn reference_integrity_checks(
    create_or_update_dsl_method: CreateOrUpdateDSLMethod,
    spacetimedb_table: &SpacetimeDBTable,
    columns: &Vec<InternalColumn>,
    mut paths_of_traits_to_extend: Vec<Path>,
) -> (Vec<Path>, Vec<TokenStream>) {
    let mut reference_integrity_checks = vec![];

    let reasons = "There can be two reasons for this: You are inserting or updating somewhere using spacetimedb::ReducerContext instead of spacetimedsl::DSL or the Foreign Key / Referenced By SpacetimeDSL feature is broken.";

    for column in columns {
        // Checks of private columns only need to happen in checks for create methods, because they can't be changed, they don't need to be checked during updates
        if create_or_update_dsl_method.eq(&CreateOrUpdateDSLMethod::Update) && column.rust_field_visibility.eq(&crate::api::rust::visibility::RustVisibility::Private) {
            continue;
        }

        let foreign_key;

        match &column.spacetimedsl_column_foreign_key {
            Some(fk) => foreign_key = fk,
            None => continue,
        };

        let referenced_table_name = &foreign_key.table_name;
        let referenced_table_name_pascal_case = format_ident!("{}", RenameRule::PascalCase.apply_to_field(referenced_table_name.to_string()));
        let referenced_table_primary_key_column_name_pascal_case = format_ident!("Id");
        let get_row_of_referenced_table_by_primary_key_trait_name = format_ident!("Get{referenced_table_name_pascal_case}RowOptionBy{referenced_table_primary_key_column_name_pascal_case}");
        let get_row_of_referenced_table_by_primary_key_method_name = format_ident!("get_{referenced_table_name}_by_id");

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_getter_name = format_ident!("get_{referencing_table_column_name}");

        let mut reference_integrity_violation_error_panic_message = format!(
            "There must be a row inside the `{referenced_table_name}` table when trying to find one with primary key column `id` value "
        );
        reference_integrity_violation_error_panic_message.push_str("`{:?}`. Found none. ");
        reference_integrity_violation_error_panic_message.push_str(reasons);

        let referencing_table_column_type = column.rust_field_type_name_or_path.to_token_stream().to_string();

        paths_of_traits_to_extend.push(parse_str(&format!("{}::{get_row_of_referenced_table_by_primary_key_trait_name}", &foreign_key.path.to_token_stream().to_string())).expect("should be parseable"));

        let check = quote! {
            match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                Some(_) => {},
                None => {
                    panic!(
                        #reference_integrity_violation_error_panic_message,
                        #referencing_table_name.#referencing_table_column_getter_name(),
                    );
                }
            };
        };

        reference_integrity_checks.push(
            match referencing_table_column_type.trim() {
            "u8" | "u16" | "u32" | "u64" | "u128"  => quote! {
                if #referencing_table_column_name.ne(&0) {
                    #check
                }
            },
            "Option"  => quote! {
                if #referencing_table_column_name.is_some() {
                    #check
                }
            },
            _ => quote! {
                #check
            },
        });
    }

    (paths_of_traits_to_extend, reference_integrity_checks)
}

pub(in crate::internal::dsl::method) struct MultiColumnIndexCheck {
    index_name: Ident,
    check: TokenStream,
}

fn multi_column_index_checks(
    struct_name: &Ident,
    singular_table_name: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
    primary_key_column_name: &Ident,
) -> Vec<TokenStream> {
    let mut multi_column_index_checks =
        get_unique_multi_column_index_checks(struct_name, singular_table_name, spacetimedb_table);

    for multi_column_index_check in &mut multi_column_index_checks {
        let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

        multi_column_index_check.check.append_all(quote! {
            match &#field_name_for_found_value {
                Some(#field_name_for_found_value) => {
                    if #field_name_for_found_value.#primary_key_column_name.ne(&#singular_table_name.#primary_key_column_name) {
                        use spacetimedb::table::MaybeError;
                        return Err(spacetimedb::UniqueConstraintViolation::get()
                            .map(spacetimedb::TryInsertError::UniqueConstraintViolation)
                            .expect("Mapping should have worked"));
                    }
                },
                _ => {},
            };
        });
    }

    let multi_column_index_checks: Vec<TokenStream> = multi_column_index_checks
        .into_iter()
        .map(|mcic| mcic.check)
        .collect();

    multi_column_index_checks
}

pub(in crate::internal::dsl::method) fn get_unique_multi_column_index_checks(
    struct_name: &Ident,
    singular_table_name: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
) -> Vec<MultiColumnIndexCheck> {
    let mut multi_column_index_checks = vec![];

    for multi_column_index in &spacetimedb_table.multi_column_indices {
        let index_column_names = match &multi_column_index.index_type {
            IndexType::BTreeMultiColumn { columns } => columns,
            _ => {
                continue;
            }
        };

        if !multi_column_index.is_unique {
            continue;
        }

        let mut column_values = vec![];

        for column_name in index_column_names {
            let column_name = format_ident!("{column_name}");
            column_values.push(quote! {#singular_table_name.#column_name});
        }

        multi_column_index_checks.push(get_unique_multi_column_index_check(
            struct_name,
            &singular_table_name,
            &multi_column_index.name,
            &column_values,
        ));
    }

    multi_column_index_checks
}

pub(in crate::internal::dsl::method) fn get_unique_multi_column_index_check(
    struct_name: &Ident,
    singular_table_name: &Ident,
    index_name: &Ident,
    column_values: &Vec<TokenStream>,
) -> MultiColumnIndexCheck {
    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let reasons = "There can be two reasons for this: You are inserting or updating somewhere using spacetimedb::ReducerContext instead of spacetimedsl::DSL or the unique multi-column index SpacetimeDSL feature is broken.";

    let mut more_than_one_panic_msg = format!(
        "There must be only one {struct_name} row inside the {singular_table_name} table when filtering on the unique multi-column index {index_name} with value "
    );
    more_than_one_panic_msg.push_str("{:?}. Found more than one. ");
    more_than_one_panic_msg.push_str(reasons);

    MultiColumnIndexCheck {
        index_name: index_name.clone(),
        check: quote! {
                let #field_name_for_found_value = match self.ctx().db().#singular_table_name().#index_name().filter((#(#column_values),*)).at_most_one() {
                    Ok(#singular_table_name) => #singular_table_name,
                    Err(_) => {
                        panic!(
                            #more_than_one_panic_msg,
                            (#(#column_values),*),
                        );
                    }
                };
        },
    }
}

fn for_referenced_by(
    dsl_internal_referenced_by_function: DSLInternalReferencedByFunction,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &Vec<Column>,
    primary_key_column_name: &Ident,
) -> SpacetimeDSLMethod {
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = format_ident!(
        "{}",
        RenameRule::PascalCase.apply_to_field(spacetimedb_table.singular_name.to_string())
    );

    let primary_key_column = columns
        .iter()
        .find(|c| c.rust_field.name.eq(primary_key_column_name))
        .expect("should have a primary key");

    let primary_key_column_type = &primary_key_column.rust_field.type_name_or_path;

    let doc_comment;
    let trait_name = get_referenced_table_trait_name(
        &dsl_internal_referenced_by_function,
        &singular_table_name_pascal_case,
    );
    let function_name = get_referenced_table_function_name(
        &dsl_internal_referenced_by_function,
        &singular_table_name,
    );
    let primary_key_value_arg_name;
    let paths_of_traits_to_extend = vec![];
    let mut function_args = vec![];
    // TODO: Result Type
    let return_type = quote! { Result<(), ()> };

    match dsl_internal_referenced_by_function {
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of referencing tables after one row of the `{singular_table_name}` table was deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_value_of_row_to_delete");
            function_args.push(quote! {
                ctx: &spacetimedb::ReducerContext,
                strategy: spacetimedsl::OnDeleteStrategy,
                #primary_key_value_arg_name: &#primary_key_column_type,
            });
        }
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of referencing tables after multiple rows of the `{singular_table_name}` table were deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_values_of_rows_to_delete");
            function_args.push(quote! {
                ctx: &spacetimedb::ReducerContext,
                strategy: spacetimedsl::OnDeleteStrategy,
                #primary_key_value_arg_name: &Vec<#primary_key_column_type>,
            });
        }
    };

    let doc_comment = doc_comment.into();

    let function_impl;

    let mut strategy_calls = vec![];

    for referencing_table in &spacetimedsl_table.referencing_tables {
        let referencing_table_path = &referencing_table.path;
        let referencing_table_name = &referencing_table.table_name;
        let referencing_table_name_pascal_case = format_ident!(
            "{}",
            RenameRule::PascalCase.apply_to_field(referencing_table_name.to_string())
        );

        let dsl_internal_foreign_key_function = get_foreign_key_function_by_referenced_by_function(
            &dsl_internal_referenced_by_function,
        );

        let referencing_table_trait_name = get_referencing_table_trait_name(
            &dsl_internal_foreign_key_function,
            &referencing_table_name_pascal_case,
            &singular_table_name_pascal_case,
        );

        let referencing_table_function_name = get_referencing_table_function_name(
            &dsl_internal_foreign_key_function,
            &referencing_table_name,
            &singular_table_name,
        );

        strategy_calls.push(quote! {
            use #referencing_table_path::#referencing_table_trait_name;
            spacetimedsl::internal::DSLInternals::#referencing_table_function_name(ctx, &strategy, #primary_key_value_arg_name)?;
        });
    }

    function_impl = quote! {
        #(#strategy_calls)*

        Ok(())
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        paths_of_traits_to_extend,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
    }
}

fn for_foreign_key(
    dsl_internal_foreign_key_function: DSLInternalForeignKeyFunction,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &Vec<Column>,
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &Vec<&&Column>,
    primary_key_column_name: &Ident,
) -> SpacetimeDSLMethod {
    let primary_key_column_type = 
        &columns
        .iter()
        .find(|c| {
            c.rust_field
                .name
                .eq(primary_key_column_name)
        })
        .expect("should have a primary key")
            .rust_field
            .type_name_or_path;

    let first_foreign_key_column = columns_with_foreign_key
        .first()
        .expect("there should be a column with foreign key");

    let primary_key_column_name = format_ident!("{primary_key_column_name}");
    let referenced_table_primary_key_column_type =
        &first_foreign_key_column.rust_field.type_name_or_path;

    let mut columns_by_on_delete_strategies = HashMap::new();

    for column_with_foreign_key in columns_with_foreign_key {
        if column_with_foreign_key
            .rust_field
            .type_name_or_path.to_token_stream().to_string()
            .ne(&referenced_table_primary_key_column_type.to_token_stream().to_string())
        {
            // TODO: If Option is supported, the type of the primary key values needs to be without option and it's allowed to have both, option and non-option columns. There is already a function to remove option from the type representation, search for `Option <`` in the code.
            panic!(
                "All foreign key columns which reference the same primary key of another table should have the same type"
            );
        }

        let on_delete_strategy = &column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .expect(&format!(
                "the column {} should have a foreign key",
                column_with_foreign_key.rust_field.name
            ))
            .on_delete_strategy;

        if !columns_by_on_delete_strategies.contains_key(&on_delete_strategy) {
            columns_by_on_delete_strategies.insert(on_delete_strategy, vec![]);
        }

        columns_by_on_delete_strategies
            .get_mut(&on_delete_strategy)
            .expect("The key OnDeleteStrategy should exist!")
            .push(column_with_foreign_key);
    }

    let struct_name = &rust_struct.name;

    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = format_ident!(
        "{}",
        RenameRule::PascalCase.apply_to_field(spacetimedb_table.singular_name.to_string())
    );
    let referenced_table_name = format_ident!("{}", *referenced_table_name);
    let referenced_table_name_pascal_case = format_ident!(
        "{}",
        RenameRule::PascalCase.apply_to_field(referenced_table_name.to_string())
    );

    let doc_comment;

    let trait_name = get_referencing_table_trait_name(
        &dsl_internal_foreign_key_function,
        &singular_table_name_pascal_case,
        &referenced_table_name_pascal_case,
    );

    let function_name = get_referencing_table_function_name(
        &dsl_internal_foreign_key_function,
        &singular_table_name,
        &referenced_table_name,
    );

    let primary_key_value_arg_name;

    let paths_of_traits_to_extend = vec![];
    let mut function_args = vec![
        quote! {
            ctx: &spacetimedb::ReducerContext
        },
        quote! {
            strategy: &spacetimedsl::OnDeleteStrategy
        },
    ];

    // TODO: Result Type
    let return_type = quote! {
        Result<(),()>
    };

    match dsl_internal_foreign_key_function {
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of the `{singular_table_name}` table after one row of the `{referenced_table_name}` table was deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_value_of_row_to_delete");
            function_args.push(quote! {
                #primary_key_value_arg_name: &#referenced_table_primary_key_column_type
            });
        }
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of the `{singular_table_name}` table after multiple rows of the `{referenced_table_name}` table were deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_values_of_rows_to_delete");
            function_args.push(quote! {
                #primary_key_value_arg_name: &Vec<#referenced_table_primary_key_column_type>
            });
        }
    };

    let doc_comment = doc_comment.into();
    let function_impl;

    let mut on_delete_strategy_match_arms = vec![];

    for (on_delete_strategy, columns_by_on_delete_strategy) in columns_by_on_delete_strategies {
        on_delete_strategy_match_arms.push(get_on_delete_strategy_implementation(
            &struct_name,
            &singular_table_name,
            &primary_key_column_name,
            primary_key_column_type,
            &spacetimedsl_table,
            &dsl_internal_foreign_key_function,
            on_delete_strategy,
            columns_by_on_delete_strategy,
        ));
    }

    function_impl = quote! {
        match &strategy {
            #(#on_delete_strategy_match_arms)*
            _ => {}
        };

        Ok(())
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        paths_of_traits_to_extend,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
    }
}

fn get_on_delete_strategy_implementation(
    struct_name: &Ident,
    singular_table_name: &Ident,
    primary_key_column_name: &Ident,
    primary_key_column_type: &Path,
    spacetimedsl_table: &SpacetimeDSLTable,
    dsl_internal_foreign_key_function: &DSLInternalForeignKeyFunction,
    on_delete_strategy: &OnDeleteStrategy,
    columns_by_on_delete_strategy: Vec<&&&Column>,
) -> TokenStream {
    let mut strategy_before_all_columns = TokenStream::default();
    let mut strategy_by_column = vec![];
    let mut strategy_after_all_columns = TokenStream::default();

    for column in &columns_by_on_delete_strategy {
        let column_name = &column.rust_field.name;

        let is_unique_index = column
            .spacetimedb_column
            .single_column_index
            .as_ref()
            .unwrap()
            .is_unique;

        let spacetimedb_call_prefix = quote! {
            ctx
                .db()
                .#singular_table_name()
        };

        let row_finder = match is_unique_index {
            true => {
                quote! {
                    #spacetimedb_call_prefix.#column_name().find(primary_key_value_of_row_to_delete)
                }
            }
            false => {
                quote! {
                    #spacetimedb_call_prefix.#column_name().filter(primary_key_value_of_row_to_delete)
                }
            }
        };

        strategy_by_column.push(match on_delete_strategy {
            OnDeleteStrategy::Error => {
                let strategy_per_row = quote! {
                    return Err(());
                };

                match is_unique_index {
                    true => {
                        quote! {
                            if #row_finder.is_some() {
                                #strategy_per_row
                            };
                        }
                    }
                    false => {
                        quote! {
                            if #row_finder.next().is_some() {
                                #strategy_per_row
                            }
                        }
                    }
                }
            }
            OnDeleteStrategy::Delete => {
                let optional_primary_key_value_setter;

                if spacetimedsl_table.referencing_tables.is_empty() {
                    optional_primary_key_value_setter = TokenStream::default();

                    strategy_before_all_columns = quote! {
                        let mut rows_to_delete: Vec<#struct_name> = vec![];
                    };

                    strategy_after_all_columns = quote! {
                        for row_to_delete in rows_to_delete {
                            #spacetimedb_call_prefix
                                .#primary_key_column_name()
                                .delete(row_to_delete.#primary_key_column_name);
                        }
                    };
                } else {
                    match is_unique_index {
                        true => {
                            optional_primary_key_value_setter = quote! {
                                primary_key_values_of_rows_to_delete.push(row.#primary_key_column_name);
                            };
                        }
                        false => {
                            optional_primary_key_value_setter = quote! {
                                let mut primary_keys = rows.iter().map(|row| row.#primary_key_column_name).collect();
                                primary_key_values_of_rows_to_delete.append(&mut primary_keys);
                            }
                        }
                    };

                    strategy_before_all_columns = quote! {
                        let mut primary_key_values_of_rows_to_delete: Vec<#primary_key_column_type> = vec![];
                        let mut rows_to_delete: Vec<#struct_name> = vec![];
                    };

                    let delete_one_hooks = get_referenced_table_function_name(
                        &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
                        &singular_table_name
                    );

                    let delete_many_hooks = get_referenced_table_function_name(
                        &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
                        &singular_table_name
                    );

                    strategy_after_all_columns = quote! {
                        if primary_key_values_of_rows_to_delete.len().eq(&1) {
                            let primary_key_value_of_row_to_delete = primary_key_values_of_rows_to_delete[0];

                            spacetimedsl::internal::DSLInternals::#delete_one_hooks(ctx, spacetimedsl::OnDeleteStrategy::Error, &primary_key_value_of_row_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#delete_one_hooks(ctx, spacetimedsl::OnDeleteStrategy::Delete, &primary_key_value_of_row_to_delete)?;
                            //TODO: spacetimedsl::internal::DSLInternals::#delete_one_hooks(ctx, spacetimedsl::OnDeleteStrategy::SetNone, &primary_key_value_of_row_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#delete_one_hooks(ctx, spacetimedsl::OnDeleteStrategy::SetZero, &primary_key_value_of_row_to_delete)?;

                            let row_to_delete = &rows_to_delete[0];

                            #spacetimedb_call_prefix
                                .#primary_key_column_name()
                                .delete(row_to_delete.#primary_key_column_name);
                        } else {
                            spacetimedsl::internal::DSLInternals::#delete_many_hooks(ctx, spacetimedsl::OnDeleteStrategy::Error, &primary_key_values_of_rows_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#delete_many_hooks(ctx, spacetimedsl::OnDeleteStrategy::Delete, &primary_key_values_of_rows_to_delete)?;
                            //TODO: spacetimedsl::internal::DSLInternals::#delete_many_hooks(ctx, spacetimedsl::OnDeleteStrategy::SetNone, &primary_key_values_of_rows_to_delete)?;
                            spacetimedsl::internal::DSLInternals::#delete_many_hooks(ctx, spacetimedsl::OnDeleteStrategy::SetZero, &primary_key_values_of_rows_to_delete)?;
                            for row_to_delete in rows_to_delete {
                                #spacetimedb_call_prefix
                                    .#primary_key_column_name()
                                    .delete(row_to_delete.#primary_key_column_name);
                            }
                        };
                    };
                }

                match is_unique_index {
                    true => {
                        quote! {
                            match #row_finder {
                                None => {}
                                Some(row) => { 
                                    #optional_primary_key_value_setter
                                    rows_to_delete.push(row);
                                }
                            };
                        }
                    }
                    false => {
                        quote! {
                            let mut rows: Vec<#struct_name> = #row_finder.collect();
                            #optional_primary_key_value_setter
                            rows_to_delete.append(&mut rows);
                        }
                    }
                }
            }
            OnDeleteStrategy::SetZero => {
                let strategy_per_row = quote! {
                    row.#column_name = 0;
                    #spacetimedb_call_prefix.#primary_key_column_name().update(row);
                };

                match is_unique_index {
                    true => {
                        quote! {
                            match #row_finder {
                                None => {}
                                Some(mut row) => { #strategy_per_row }
                            };
                        }
                    }
                    false => {
                        quote! {
                            #row_finder.for_each(|mut row| {
                                #strategy_per_row
                            });
                        }
                    }
                }
            }
        })
    }

    match dsl_internal_foreign_key_function {
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted => {
            quote! {
                #on_delete_strategy => {
                    #strategy_before_all_columns

                    #(#strategy_by_column)*

                    #strategy_after_all_columns
                }
            }
        },
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => {
            quote! {
                #on_delete_strategy => {
                    #strategy_before_all_columns

                    for primary_key_value_of_row_to_delete in primary_key_values_of_rows_to_delete {
                        #(#strategy_by_column)*
                    }

                    #strategy_after_all_columns
                }
            }
        },
    }
}

fn get_foreign_key_function_by_referenced_by_function(
    dsl_internal_referenced_by_function: &DSLInternalReferencedByFunction,
) -> DSLInternalForeignKeyFunction {
    match *dsl_internal_referenced_by_function {
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted => {
            DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted
        }
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted => {
            DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted
        }
    }
}

fn get_referenced_table_trait_name(
    dsl_internal_referenced_by_function: &DSLInternalReferencedByFunction,
    referenced_table_name_pascal_case: &Ident,
) -> Ident {
    match dsl_internal_referenced_by_function {
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted => {
            format_ident!("ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThe{referenced_table_name_pascal_case}TableWasDeleted")
        },
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted => {
            format_ident!("ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThe{referenced_table_name_pascal_case}TableWereDeleted")
        }
    }
}

fn get_referenced_table_function_name(
    dsl_internal_referenced_by_function: &DSLInternalReferencedByFunction,
    referenced_table_name: &Ident,
) -> Ident {
    match dsl_internal_referenced_by_function {
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted => {
            format_ident!("execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_{referenced_table_name}_table_was_deleted")
        },
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted => {
            format_ident!("execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_{referenced_table_name}_table_were_deleted")
        }
    }
}

fn get_referencing_table_trait_name(
    dsl_internal_foreign_key_function: &DSLInternalForeignKeyFunction,
    referencing_table_name_pascal_case: &Ident,
    referenced_table_name_pascal_case: &Ident,
) -> Ident {
    match dsl_internal_foreign_key_function {
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted => {
            format_ident!("ExecuteOnDeleteStrategiesOfThe{referencing_table_name_pascal_case}TableAfterOneRowOfThe{referenced_table_name_pascal_case}TableWasDeleted")
        },
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => {
            format_ident!("ExecuteOnDeleteStrategiesOfThe{referencing_table_name_pascal_case}TableAfterMultipleRowsOfThe{referenced_table_name_pascal_case}TableWereDeleted")
        }
    }
}

fn get_referencing_table_function_name(
    dsl_internal_foreign_key_function: &DSLInternalForeignKeyFunction,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    match dsl_internal_foreign_key_function {
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted => {
            format_ident!("execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_one_row_of_the_{referenced_table_name}_table_was_deleted")
        },
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => {
            format_ident!("execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_multiple_rows_of_the_{referenced_table_name}_table_were_deleted")
        }
    }
}
