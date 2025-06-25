use crate::{
    api::{
        Column,
        db::{
            column::SpacetimeDBColumn,
            index::{Index, IndexType},
            table::SpacetimeDBTable,
        },
        dsl::{
            column::{
                SpacetimeDSLColumn, SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex, SpacetimeDSLDeletionResult,
            },
            foreign_key::OnDeleteStrategy,
            method::SpacetimeDSLMethod,
            table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
            wrapper::WrapperType,
        },
        rust::{column::RustField, table::RustStruct},
    },
    internal::dsl::wrapper::wrapper_type_into_option,
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, format_ident, quote};
use std::collections::HashMap;
use syn::{Ident, Path, Type, parse_str};

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
        primary_key_column_name: &Box<str>,
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
                );

                let delete_many = for_single_column_index(
                    DSLColumnMethod::DeleteMany,
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    rust_field,
                    spacetimedsl_column,
                    primary_key_column_name,
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
        primary_key_column_name: &Box<str>,
    ) -> syn::Result<SpacetimeDSLTableMethods> {
        let create = for_table(
            DSLTableMethod::Create,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            columns,
        );

        let get_all = for_table(
            DSLTableMethod::GetAll,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            columns,
        );

        let get_count = for_table(
            DSLTableMethod::GetCount,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            columns,
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
                .for_each(|(name_of_another_table, columns_with_foreign_key)| {
                    execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted.push(
                        for_foreign_key(
                            DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted,
                            rust_struct,
                            spacetimedb_table,
                            spacetimedsl_table,
                            columns,
                            primary_key_column_name,
                            name_of_another_table,
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
                            primary_key_column_name,
                            name_of_another_table,
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
                    );
                    let delete_many = for_multi_column_index(
                        DSLColumnMethod::DeleteMany,
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        multi_column_index,
                        columns,
                        primary_key_column_name,
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

pub(in crate::internal) fn for_table(
    dsl_table_method: DSLTableMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &Vec<Column>,
) -> SpacetimeDSLMethod {
    let struct_name = format_ident!("{}", *rust_struct.name);
    let singular_table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let plural_table_name = &spacetimedsl_table.plural_name;

    let doc_comment;
    let trait_name;
    let method_name;
    let return_type;

    match dsl_table_method {
        // TODO: Let foreign_key's influence the doc comment
        DSLTableMethod::Create => {
            doc_comment = format!("Create a row in the `{singular_table_name}` table.");
            trait_name = format!("Create{}Row", struct_name);
            method_name = format!("create_{}", singular_table_name);

            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            return_type = quote! {
                Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
            };
        }
        DSLTableMethod::GetAll => {
            doc_comment = format!("Get all rows inside the `{singular_table_name}` table.");
            trait_name = format!("GetAll{}Rows", struct_name);
            method_name = format!("get_all_{}", plural_table_name);
            return_type = quote! {
                impl Iterator<Item = #struct_name>
            };
        }
        DSLTableMethod::GetCount => {
            doc_comment = format!("Count all rows inside the `{singular_table_name}` table.");
            trait_name = format!("CountOfAll{}Rows", struct_name);
            method_name = format!("count_of_all_{}", plural_table_name);
            return_type = quote! {
                u64
            };
        }
    }

    let doc_comment = doc_comment.into();
    let trait_name = trait_name.into();
    let method_name = method_name.into();
    let return_type = return_type.to_string().into();

    let mut method_args = vec![];
    let method_impl;

    match dsl_table_method {
        DSLTableMethod::Create => {
            let mut into_options = vec![];
            let mut constructor_args = vec![];

            for column in columns {
                let column_name = format_ident!("{}", *column.rust_field.name);
                let column_type: Type =
                    parse_str(&column.rust_field.type_name_or_path).expect("create");

                // TODO: Allow Option<Timestamp> as modified_at column type
                if column.spacetimedb_column.is_auto_inc
                    || column.rust_field.name.eq(&"created_at".to_string().into())
                    || column.rust_field.name.eq(&"modified_at".to_string().into())
                {
                    if column.spacetimedb_column.is_auto_inc {
                        constructor_args.push(quote! {
                            #column_name: #column_type::default()
                        });
                    } else if column.rust_field.name.eq(&"created_at".to_string().into()) {
                        constructor_args.push(quote! {
                            created_at: self.ctx().timestamp
                        });
                    } else if column.rust_field.name.eq(&"modified_at".to_string().into()) {
                        constructor_args.push(quote! {
                            modified_at: self.ctx().timestamp
                        });
                    }
                    continue;
                }

                match &column.spacetimedsl_column.wrapper_type {
                    Some(wrapper_type) => match wrapper_type {
                        WrapperType::Wrap(wrapper_type) => {
                            if column.rust_field.type_name_or_path.eq(&"String".into()) {
                                method_args.push(quote! {
                                    #column_name: &str
                                });

                                constructor_args.push(quote! {
                                    #column_name: #column_name.to_string()
                                });
                            } else {
                                let wrapped_type_name_or_path =
                                    WrapperType::map_to_wrapped_type(wrapper_type);

                                method_args.push(quote! {
                                    #column_name: #wrapped_type_name_or_path
                                });

                                constructor_args.push(quote! {
                                    #column_name
                                });
                            }
                        }
                        WrapperType::Wrapped(_) => {
                            let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                            if column.spacetimedsl_column.is_option {
                                method_args.push(quote! {
                                    #column_name: impl Into<Option<#wrapper_type_name_or_path>>
                                });

                                into_options.push(wrapper_type_into_option(
                                    &column_name,
                                    wrapper_type_name_or_path,
                                ));

                                constructor_args.push(quote! {
                                    #column_name
                                });
                            } else {
                                method_args.push(quote! {
                                    #column_name: impl Into<#wrapper_type_name_or_path>
                                });

                                constructor_args.push(quote! {
                                    #column_name: #column_name.into().value()
                                });
                            }
                        }
                    },
                    None => {
                        if column.rust_field.type_name_or_path.eq(&"String".into()) {
                            method_args.push(quote! {
                                #column_name: &str
                            });
                            constructor_args.push(quote! {
                                #column_name: #column_name.to_string()
                            });
                        } else {
                            method_args.push(quote! {
                                #column_name: #column_type
                            });
                            constructor_args.push(quote! {
                                #column_name
                            });
                        }
                    }
                };
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

            // TODO: Check foreign keys
            method_impl = quote! {
                #use_itertools

                #(#into_options)*
                let #singular_table_name = #struct_name {
                    #(#constructor_args),*
                };

                #(#multi_column_index_checks)*

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

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();
    let method_impl = method_impl.to_string().into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
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
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = format_ident!("{}", *rust_struct.name);
    let singular_table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let plural_table_name = &spacetimedsl_table.plural_name;
    let column_name = format_ident!("{}", *rust_field.name);
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
            trait_name = format!("Get{struct_name}RowsBy{column_name_pascal_case}");
            method_name = format!("get_{plural_table_name}_by_{column_name}");
            return_type = quote! {
                impl Iterator<Item = #struct_name>
            };
        }
        DSLColumnMethod::DeleteMany => {
            // TODO: Let referenced_by's influence the doc comment
            doc_comment = format!(
                "Delete all {struct_name} rows inside the {singular_table_name} table filtered by the single-column index on the {column_name} column."
            );
            trait_name = format!("Delete{struct_name}RowsBy{column_name_pascal_case}");
            method_name = format!("delete_{plural_table_name}_by_{column_name}");
            // TODO: Result<spacetimedsl::DeletionResult, spacetimedsl::ReferenceIntegrityViolationError>
            return_type = quote! {Result<u64,()>};
        }
        DSLColumnMethod::GetOneOption => {
            doc_comment = format!(
                "Get an Option<{struct_name}> row inside the {singular_table_name} table filtered by the unique single-column index on the {column_name} column."
            );
            trait_name = format!("Get{struct_name}RowOptionBy{column_name_pascal_case}");
            method_name = format!("get_{singular_table_name}_by_{column_name}");
            return_type = quote! {
                Option<#struct_name>
            };
        }
        DSLColumnMethod::Update => {
            // TODO: Let foreign_key's influence the doc comment
            doc_comment = format!(
                "Update a {struct_name} row inside the {singular_table_name} table by the unique single-column index on the {column_name} column."
            );
            trait_name = format!("Update{struct_name}RowBy{column_name_pascal_case}");
            method_name = format!("update_{singular_table_name}_by_{column_name}");

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
            trait_name = format!("Delete{struct_name}RowBy{column_name_pascal_case}");
            method_name = format!("delete_{singular_table_name}_by_{column_name}");
            // TODO: Result<spacetimedsl::DeletionResult, spacetimedsl::ReferenceIntegrityViolationError>
            return_type = quote! {Result<bool,()>};
        }
    }

    let doc_comment = doc_comment.into();
    let trait_name = trait_name.into();
    let method_name = method_name.into();
    let return_type = return_type.to_string().into();

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

            // TODO: Check foreign keys
            method_impl = quote! {
                #use_itertools

                #(#multi_column_index_checks)*

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
                    if rust_field.type_name_or_path.eq(&"String".into()) {
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
                        let column_type: Type;
                        if rust_field.type_name_or_path.eq(&"String".into()) {
                            column_type = parse_str("str").expect("parsing should have worked");
                        } else {
                            column_type = parse_str(&rust_field.type_name_or_path)
                                .expect("get_many.for_single_column_index");
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
                        let column_type: Type;
                        if rust_field.type_name_or_path.eq(&"String".into()) {
                            column_type = parse_str("str").expect("parsing should have worked");
                        } else {
                            column_type = parse_str(&rust_field.type_name_or_path)
                                .expect("delete_many.for_single_column_index");
                        }

                        method_arg = quote! { #column_name: &'a #column_type };

                        column_value = quote! {
                                #column_name
                        };
                    }
                    DSLColumnMethod::GetOneOption => {
                        if rust_field.type_name_or_path.eq(&"String".into()) {
                            method_arg = quote! { #column_name: &str };

                            column_value = quote! { #column_name.to_string() };
                        } else {
                            let column_type: Type = parse_str(&rust_field.type_name_or_path)
                                .expect("get_one_option.for_single_column_index");
                            method_arg = quote! { #column_name: &#column_type };

                            column_value = quote! {
                                #column_name
                            };
                        }
                    }
                    DSLColumnMethod::DeleteOne => {
                        let column_type: Type = parse_str(&rust_field.type_name_or_path)
                            .expect("delete_one.for_single_column_index");
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

                        quote! {
                            #into_option

                            let #column_name = #column_value;

                            let primary_key_values_of_rows_to_delete = #method_impl_prefix
                                .filter(#column_name)
                                .map(|row| row.#primary_key_column_name)
                                .collect(); // TODO: maybe some types need a .clone() after #column_name

                            if primary_key_values_of_rows_to_delete.is_empty() {
                                return Ok(0);
                            }

                            spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), &primary_key_values_of_rows_to_delete)?;

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

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();
    let method_impl = method_impl.to_string().into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
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
    primary_key_column_name: &Box<str>,
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

    let struct_name = format_ident!("{}", *rust_struct.name);
    let singular_table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let plural_table_name = format_ident!("{}", *spacetimedsl_table.plural_name);
    let index_name = format_ident!("{}", *multi_column_index.name);
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
        DSLColumnMethod::GetMany => format!("Get{struct_name}RowsBy{index_name_pascal_case}"),
        DSLColumnMethod::DeleteMany => format!("Delete{struct_name}RowsBy{index_name_pascal_case}"),
        DSLColumnMethod::GetOneOption => {
            format!("Get{struct_name}RowOptionBy{index_name_pascal_case}")
        }
        DSLColumnMethod::Update => format!("Update{struct_name}RowBy{index_name_pascal_case}"),
        DSLColumnMethod::DeleteOne => format!("Delete{struct_name}RowBy{index_name_pascal_case}"),
    }
    .into();

    let method_name = match dsl_method {
        DSLColumnMethod::GetMany => format!("get_{plural_table_name}_by_{index_name}"),
        DSLColumnMethod::DeleteMany => format!("delete_{plural_table_name}_by_{index_name}"),
        DSLColumnMethod::GetOneOption => format!("get_{singular_table_name}_by_{index_name}"),
        DSLColumnMethod::Update => format!("update_{singular_table_name}_by_{index_name}"),
        DSLColumnMethod::DeleteOne => format!("delete_{singular_table_name}_by_{index_name}"),
    }
    .into();

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
    }
    .to_string()
    .into();

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

            // TODO: Check foreign keys
            method_impl = quote! {
                #use_itertools

                #(#multi_column_index_checks)*

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

                let column_name = format_ident!("{}", *column.rust_field.name);

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
                        let column_type: Type = parse_str(&column.rust_field.type_name_or_path)
                            .expect("for_multi_column_index");

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

                        // TODO: If !referencing_tables.is_empty() { todo!("Call delete_many hooks before the current implementation"); }
                        DSLColumnMethod::DeleteMany => quote! {
                            #(#into_options)*
                            Ok(#method_impl_prefix
                                .delete((#(#column_values),*)))
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
                        column_values,
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
                        // TODO: If !referencing_tables.is_empty() { todo!("Call delete_one hooks before the current implementation"); }
                        DSLColumnMethod::DeleteOne => quote! {
                            #method_impl_prefix

                            Ok(
                                self
                                    .ctx()
                                    .db()
                                    .#singular_table_name()
                                    .#primary_key_column_name()
                                    .delete(#field_name_for_found_value.expect("value should be found").#primary_key_column_name)
                            )
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

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();
    let method_impl = method_impl.to_string().into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
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
            let cn = format_ident!("{column_name}");
            column_values.push(quote! {#singular_table_name.#cn});
        }

        multi_column_index_checks.push(get_unique_multi_column_index_check(
            struct_name,
            &singular_table_name,
            &format_ident!("{}", *multi_column_index.name),
            column_values,
        ));
    }

    multi_column_index_checks
}

pub(in crate::internal::dsl::method) fn get_unique_multi_column_index_check(
    struct_name: &Ident,
    singular_table_name: &Ident,
    index_name: &Ident,
    column_values: Vec<TokenStream>,
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
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let singular_table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let singular_table_name_pascal_case = format_ident!(
        "{}",
        RenameRule::PascalCase.apply_to_field(spacetimedb_table.singular_name.to_string())
    );

    let primary_key_column = columns
        .iter()
        .find(|c| c.rust_field.name.eq(primary_key_column_name))
        .expect("should have a primary key");

    let primary_key_column_type: Type = parse_str(&primary_key_column.rust_field.type_name_or_path)
        .expect("parsing should have worked");

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
    let trait_name = trait_name.to_string().into();
    let function_name = function_name.to_string().into();
    let return_type = return_type.to_string().into();

    let function_impl;

    let mut strategy_calls = vec![];

    for referencing_table in &spacetimedsl_table.referencing_tables {
        let referencing_table_path: Path =
            parse_str(&referencing_table.path).expect("parsing should have worked");
        let referencing_table_name = format_ident!("{}", *referencing_table.table_name);
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

    let function_args = function_args
        .iter()
        .map(|ts| ts.to_string().into())
        .collect();
    let function_impl = function_impl.to_string().into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
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
    referencing_table_primary_key_column_name: &Box<str>,
    referenced_table_name: &str,
    columns_with_foreign_key: &Vec<&&Column>,
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let referencing_table_primary_key_column = columns
        .iter()
        .find(|c| {
            c.rust_field
                .name
                .eq(referencing_table_primary_key_column_name)
        })
        .expect("should have a primary key");

    // TODO: Needed?
    let referencing_table_primary_key_column_type: Type = parse_str(
        &referencing_table_primary_key_column
            .rust_field
            .type_name_or_path,
    )
    .expect("parsing should have worked");

    let first_foreign_key_column = columns_with_foreign_key
        .first()
        .expect("there should be a column with foreign key");

    let referenced_table_primary_key_column_type =
        &first_foreign_key_column.rust_field.type_name_or_path;

    let mut columns_with_on_delete_error_strategy = vec![];
    let mut columns_with_on_delete_cascade_strategy = vec![];
    //TODO let mut columns_with_on_delete_set_none_strategy = vec![];
    let mut columns_with_on_delete_set_zero_strategy = vec![];

    for column_with_foreign_key in columns_with_foreign_key {
        match column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .expect(&format!(
                "the column {} should have a foreign key",
                column_with_foreign_key.rust_field.name
            ))
            .on_delete_strategy
        {
            OnDeleteStrategy::Error => {
                columns_with_on_delete_error_strategy.push(column_with_foreign_key);
            }
            OnDeleteStrategy::Delete => {
                columns_with_on_delete_cascade_strategy.push(column_with_foreign_key);
            }
            //TODO: OnDeleteStrategy::SetNone => {columns_with_on_delete_set_none_strategy.push(column_with_foreign_key);}
            OnDeleteStrategy::SetZero => {
                columns_with_on_delete_set_zero_strategy.push(column_with_foreign_key);
            }
        };

        if column_with_foreign_key
            .rust_field
            .type_name_or_path
            .ne(&referenced_table_primary_key_column_type)
        {
            // TODO: If Option is supported, the type of the primary key values needs to be without option and it's allowed to have both, option and non-option columns. There is already a function to remove option from the type representation, search for `Option <`` in the code.
            panic!(
                "All foreign key columns which reference the same primary key of another table should have the same type"
            );
        }
    }

    let referenced_table_primary_key_column_type: Type =
        parse_str(&referenced_table_primary_key_column_type).expect("parsing should have worked");

    let singular_table_name = format_ident!("{}", *spacetimedb_table.singular_name);
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
    )
    .to_string()
    .into();

    let function_name = get_referencing_table_function_name(
        &dsl_internal_foreign_key_function,
        &singular_table_name,
        &referenced_table_name,
    )
    .to_string()
    .into();

    let primary_key_value_arg_name;

    let mut function_args = vec![];

    // TODO: Result Type
    let return_type = quote! {
        Result<(),()>
    }
    .to_string()
    .into();

    match dsl_internal_foreign_key_function {
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of the `{singular_table_name}` table after one row of the `{referenced_table_name}` table was deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_value_of_row_to_delete");
            function_args.push(quote! {
                ctx: &spacetimedb::ReducerContext,
                strategy: &spacetimedsl::OnDeleteStrategy,
                #primary_key_value_arg_name: &#referenced_table_primary_key_column_type,
            });
        }
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of the `{singular_table_name}` table after multiple rows of the `{referenced_table_name}` table were deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_values_of_rows_to_delete");
            function_args.push(quote! {
                ctx: &spacetimedb::ReducerContext,
                strategy: &spacetimedsl::OnDeleteStrategy,
                #primary_key_value_arg_name: &Vec<#referenced_table_primary_key_column_type>,
            });
        }
    };

    let doc_comment = doc_comment.into();
    let function_impl;

    let on_delete_error_strategy;
    if columns_with_on_delete_error_strategy.is_empty() {
        on_delete_error_strategy = TokenStream::default();
    } else {
        on_delete_error_strategy = get_on_delete_strategy_implementation(
            &dsl_internal_foreign_key_function,
            columns_with_on_delete_error_strategy,
            &singular_table_name,
        );
    }

    let on_delete_cascade_strategy;
    if columns_with_on_delete_cascade_strategy.is_empty() {
        on_delete_cascade_strategy = TokenStream::default();
    } else {
        on_delete_cascade_strategy = get_on_delete_strategy_implementation(
            &dsl_internal_foreign_key_function,
            columns_with_on_delete_cascade_strategy,
            &singular_table_name,
        );
    }

    /* TODO
       let on_delete_set_none_strategy;
       if columns_with_on_delete_set_none_strategy.is_empty() {
           on_delete_set_none_strategy = TokenStream::default();
       } else {
           on_delete_set_none_strategy = get_row_finders(
                &dsl_internal_foreign_key_function,
                columns_with_on_delete_set_none_strategy,
                &singular_table_name,
            );
       }
    */

    let on_delete_set_zero_strategy;
    if columns_with_on_delete_set_zero_strategy.is_empty() {
        on_delete_set_zero_strategy = TokenStream::default();
    } else {
        on_delete_set_zero_strategy = get_on_delete_strategy_implementation(
            &dsl_internal_foreign_key_function,
            columns_with_on_delete_set_zero_strategy,
            &singular_table_name,
        );
    }

    let struct_name = format_ident!("{}", *rust_struct.name);
    function_impl = quote! {
        let mut rows_to_process: Vec<#struct_name> = vec![];

        match &strategy {
            spacetimedsl::OnDeleteStrategy::Error => {
                #on_delete_error_strategy
            }
            spacetimedsl::OnDeleteStrategy::Delete => {
                #on_delete_cascade_strategy
            }
            //TODO: spacetimedsl::OnDeleteStrategy::SetNone => {#on_delete_set_none_strategy}
            spacetimedsl::OnDeleteStrategy::SetZero => {
                #on_delete_set_zero_strategy
            }
            _ => {}
        };
        Ok(())
    };

    let function_args = function_args
        .iter()
        .map(|ts| ts.to_string().into())
        .collect();
    let function_impl = function_impl.to_string().into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
    }
}

fn get_on_delete_strategy_implementation(
    dsl_internal_foreign_key_function: &DSLInternalForeignKeyFunction,
    columns_with_same_strategy: Vec<&&&Column>,
    singular_table_name: &Ident,
) -> TokenStream {
    let strategy = &columns_with_same_strategy
        .first()
        .as_ref()
        .expect("there should be a column")
        .spacetimedsl_column
        .foreign_key
        .as_ref()
        .expect("there should be a foreign key")
        .on_delete_strategy;

    let mut row_finders = vec![];

    for column in &columns_with_same_strategy {
        let column_name = format_ident!("{}", *column.rust_field.name);
        let method_impl_prefix = quote! {
            ctx
                .db()
                .#singular_table_name()
                .#column_name()
        };

        let is_unique_index = column
            .spacetimedb_column
            .single_column_index
            .as_ref()
            .unwrap()
            .is_unique;

        /*
        let strategy_impl = match &strategy {
            OnDeleteStrategy::Error => {
                quote! {

                }
            }
            OnDeleteStrategy::Delete => {
                quote! {

                }
            }
            //OnDeleteStrategy::SetNone => { quote! {} },
            OnDeleteStrategy::SetZero => {
                quote! {
                    row_to_process.#column_name = 0;
                }
            }
        };
         */

        match is_unique_index {
            false => {
                if strategy.eq(&OnDeleteStrategy::Error) {
                    row_finders.push(quote! {
                        if #method_impl_prefix
                            .filter(primary_key_value_of_row_to_delete).next().is_some() {
                                return Err(());
                            }
                    });
                } else {
                    row_finders.push(quote! {
                        #method_impl_prefix
                            .filter(primary_key_value_of_row_to_delete).for_each(|row| {
                                // FIXME: Instead of pushing to a vec, implement the on delete strategy directly here
                                rows_to_process.push(row);
                        });
                    });
                }
            }
            true => {
                if strategy.eq(&OnDeleteStrategy::Error) {
                    row_finders.push(quote! {
                        if #method_impl_prefix.find(primary_key_value_of_row_to_delete).is_some() {
                            return Err(());
                        };
                    });
                } else {
                    row_finders.push(quote! {
                        match #method_impl_prefix.find(primary_key_value_of_row_to_delete) {
                            None => {}
                            Some(row) => { rows_to_process.push(row); }
                        };
                    });
                }
            }
        }
    }

    match dsl_internal_foreign_key_function {
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted => quote! {
            #(#row_finders)*

            if !rows_to_process.is_empty() {
                for row_to_process in rows_to_process {
                    //TODO#strategy_impl
                }
            }
        },
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => quote! {
            for primary_key_value_of_row_to_delete in primary_key_values_of_rows_to_delete {
                #(#row_finders)*
            }

            if !rows_to_process.is_empty() {
                for row_to_process in rows_to_process {
                    //TODO#strategy_impl
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
