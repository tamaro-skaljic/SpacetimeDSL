use crate::{
    api::{
        db::{
            column::SpacetimeDBColumn,
            index::{Index, IndexType},
            table::SpacetimeDBTable,
        }, dsl::{
            column::{
                SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex, SpacetimeDSLDeletionResult,
            },
            foreign_key::OnDeleteStrategy,
            method::{SpacetimeDSLMethod, SpacetimeDSLMethodArg},
            table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
            wrapper::WrapperType,
        }, rust::{table::RustStruct, visibility::RustVisibility}, Column
    },
    internal::{column::InternalColumn, dsl::wrapper::map_wrapper_type_option_to_wrapped_type_option},
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::collections::{HashMap, VecDeque};
use syn::{parse_str, Ident, Path};

pub(in crate::internal) enum DSLMethod<'a> {
    Create,
    GetAll,
    GetCount,
    GetMany(&'a Index),
    DeleteMany(&'a Index),
    GetOneOption(&'a Index),
    Update(&'a Index),
    DeleteOne(&'a Index),
}

#[derive(PartialEq)]
enum CreateOrUpdateDSLMethod {
    Create,
    Update
}

pub(in crate::internal) enum DSLInternalReferencedByFunction {
    ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
    ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
}


pub(in crate::internal) enum DSLInternalForeignKeyFunction {
    ExecuteOnDeleteStrategiesOfThisTableAfterOneRowOfTheReferencedTableWasDeleted,
    ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted,
}

impl SpacetimeDSLColumnMethods {
    pub(in crate::internal) fn map(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        spacetimedb_column: &SpacetimeDBColumn,
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
                let get_many = for_method(
                    DSLMethod::GetMany(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    primary_key_column_name,
                    internal_columns,
                );

                let delete_many = for_method(
                    DSLMethod::DeleteMany(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
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
                let get_one_option = for_method(
                    DSLMethod::GetOneOption(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    primary_key_column_name,
                    internal_columns,
                );

                let update = match spacetimedsl_table.is_mutable {
                    false => None,
                    true => Some(for_method(
                        DSLMethod::Update(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    primary_key_column_name,
                    internal_columns,
                    )),
                };

                let delete_one = for_method(
                    DSLMethod::DeleteOne(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
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
        let create = for_method(
            DSLMethod::Create,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column_name,
            internal_columns,
        );

        let get_all = for_method(
            DSLMethod::GetAll,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column_name,
            internal_columns,
        );

        let get_count = for_method(
            DSLMethod::GetCount,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column_name,
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
                    let get_many = for_method(
                        DSLMethod::GetMany(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        primary_key_column_name,
                        internal_columns,
                    );
                    let delete_many = for_method(
                        DSLMethod::DeleteMany(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
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
                    let get_one_option = for_method(
                        DSLMethod::GetOneOption(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        primary_key_column_name,
                        internal_columns,
                    );

                    let update = match spacetimedsl_table.is_mutable {
                        false => None,
                        true => Some(for_method(
                            DSLMethod::Update(multi_column_index),
                            rust_struct,
                            spacetimedb_table,
                            spacetimedsl_table,
                            primary_key_column_name,
                            internal_columns,
                        )),
                    };

                    let delete_one = for_method(
                        DSLMethod::DeleteOne(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
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

fn process_columns_for_create_and_update_method(create_or_update: CreateOrUpdateDSLMethod, internal_column: &InternalColumn) -> (Option<SpacetimeDSLMethodArg>, Option<TokenStream>, Option<TokenStream>, TokenStream) {
    let mut method_arg = None;
    let mut wrapper_type_option_to_wrapped_type_option_mapper = None;
    let mut constructor_arg = None;
    let constructor_arg_name;

    let singular_table_name = &internal_column.spacetimedb_table_singular_name;
    let column_name = &internal_column.rust_field_name;
    let getter_name = format_ident!("get_{column_name}");
    constructor_arg_name = quote! { #column_name };

    let column_type = &internal_column.rust_field_type_name_or_path;

    match create_or_update {
        CreateOrUpdateDSLMethod::Create => {
            // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/37
            if internal_column.spacetimedb_column_is_auto_inc
                || internal_column.rust_field_name.to_string().eq(&"created_at")
                || internal_column.rust_field_name.to_string().eq(&"modified_at")
            {
                if internal_column.spacetimedb_column_is_auto_inc {
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
                return (method_arg, wrapper_type_option_to_wrapped_type_option_mapper, constructor_arg, constructor_arg_name);
            }
        },
        CreateOrUpdateDSLMethod::Update => {

        },
    };

    match &internal_column.spacetimedsl_column_wrapper_type {
        Some(wrapper_type) => match wrapper_type {
            WrapperType::Wrap(wrapper_type) => {
                if internal_column.rust_field_type_name_or_path.to_token_stream().to_string().eq(&"String") {
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { &str }
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
                        
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { #wrapped_type_name_or_path }
                    });
                }
            }
            WrapperType::Wrapped(_) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if internal_column.spacetimedsl_column_is_option {
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { impl Into<Option<#wrapper_type_name_or_path>> }
                    });
                    wrapper_type_option_to_wrapped_type_option_mapper = Some(map_wrapper_type_option_to_wrapped_type_option(
                        &column_name,
                        wrapper_type_name_or_path,
                    ));
                } else {
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { impl Into<#wrapper_type_name_or_path> }
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
                method_arg = Some(SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: column_name.clone(),
                    arg_type: quote! { &str }
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
                method_arg = Some(SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: column_name.clone(),
                    arg_type: quote! { #column_type }
                });
            }
        }
    };

    (method_arg, wrapper_type_option_to_wrapped_type_option_mapper, constructor_arg, constructor_arg_name)
}

pub(in crate::internal) fn for_method(
    dsl_method: DSLMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    primary_key_column_name: &Ident,
    internal_columns: &Vec<InternalColumn>
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;
    
    let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");

    // TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/35
    let doc_comment;
    let trait_name;
    let method_name;
    // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/36
    let return_type;

    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let mut paths_of_traits_to_extend = vec![ parse_str("spacetimedsl::DSLContext").expect("parsing should have worked") ];
    let mut method_args = vec![];
    let method_impl;

    match dsl_method {
        DSLMethod::Create => {
            doc_comment = format!("Create a row in the `{singular_table_name}` table.");

            trait_name = format_ident!("Create{singular_table_name_pascal_case}Row");

            method_name = format_ident!("create_{}", singular_table_name);

            return_type = quote! {
                Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
            };

            let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
            let mut constructor_args = vec![];
            let mut constructor_arg_names = vec![];

            for internal_column in internal_columns {
                let (method_arg, wrapper_type_option_to_wrapped_type_option_mapper, constructor_arg, constructor_arg_name) = process_columns_for_create_and_update_method(CreateOrUpdateDSLMethod::Create, &internal_column);
                match method_arg {
                    Some(method_arg) => method_args.push(method_arg),
                    None => {},
                }
                
                match wrapper_type_option_to_wrapped_type_option_mapper {
                    Some(wrapper_type_option_to_wrapped_type_option_mapper) => wrapper_type_option_to_wrapped_type_option_mappers.push(wrapper_type_option_to_wrapped_type_option_mapper),
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
                #(#wrapper_type_option_to_wrapped_type_option_mappers)*
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
        DSLMethod::GetAll => {
            doc_comment = format!("Get all rows inside the `{singular_table_name}` table.");

            trait_name = format_ident!("GetAll{singular_table_name_pascal_case}Rows");

            method_name = format_ident!("get_all_{}", plural_table_name);

            return_type = quote! {
                impl Iterator<Item = #struct_name>
            };
            
            method_impl = quote! {
                self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .iter()
            };
        }
        DSLMethod::GetCount => {
            doc_comment = format!("Count all rows inside the `{singular_table_name}` table.");

            trait_name = format_ident!("CountOfAll{singular_table_name_pascal_case}Rows");

            method_name = format_ident!("count_of_all_{}", plural_table_name);

            return_type = quote! {
                u64
            };

            method_impl = quote! {
                self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .count()
            };
        }
        DSLMethod::GetMany(index) | DSLMethod::DeleteMany(index) | DSLMethod::GetOneOption(index) | DSLMethod::Update(index) | DSLMethod::DeleteOne(index)  => {
            let index_name = &index.name;
            let index_name_pascal_case = RenameRule::PascalCase.apply_to_field(index_name.to_string());

            let is_unique_index = index.is_unique;
            let is_multi_column_index;
            let mut index_columns = vec![];

            let value_matches_or_values_match;
            let single_or_multi;
            let index_documentation;
            let mut documentation_on_column_or_columns;

            match &index.index_type {
                IndexType::BTreeSingleColumn { column } => {
                    is_multi_column_index = false;
                    index_columns.push(column.clone());
                    value_matches_or_values_match = "value matches the value from";
                    single_or_multi = "single";
                    index_documentation = format!("btree index");
                    documentation_on_column_or_columns = format!("`{column}` column");
                },
                IndexType::BTreeMultiColumn { columns } => {
                    is_multi_column_index = true;
                    value_matches_or_values_match = "values match the values from";
                    single_or_multi = "multi";
                    index_documentation = format!("btree index `{index_name}`");

                    documentation_on_column_or_columns = String::new();
                    documentation_on_column_or_columns.push_str(&format!("columns"));

                    let mut columns: VecDeque<Ident> = columns.clone().into();

                    let first_column = columns.pop_front().expect("There should be a first column in Vec<Ident> of BTreeMultiColumn.");
                    let last_column = columns.pop_back().expect("There should be a last column in Vec<Ident> of BTreeMultiColumn.");
                    let any_other_column = columns;
                    
                    documentation_on_column_or_columns.push_str(&format!(" `{first_column}`"));
                    index_columns.push(first_column);

                    for any_other_column in any_other_column {
                        documentation_on_column_or_columns.push_str(&format!(", `{any_other_column}`"));
                        index_columns.push(any_other_column);
                    }

                    documentation_on_column_or_columns.push_str(&format!(" and `{last_column}`"));
                    index_columns.push(last_column);
                },
                IndexType::Direct { column } => {
                    is_multi_column_index = false;
                    index_columns.push(column.clone());
                    value_matches_or_values_match = "value matches";
                    single_or_multi = "single";
                    index_documentation = format!("direct index");
                    documentation_on_column_or_columns = format!("`{column}` column");
                },
            };

            let unique_multi_column_index_hint;
            
            if is_unique_index && is_multi_column_index {
                unique_multi_column_index_hint = "Warning: The unique multi-column index feature of SpacetimeDSL is experimental.\n- It will be removed if unique multi-column indices are implemented in SpacetimeDB.\n- SpacetimeDSL is only able to enforce referential integrity if you never use the (mutating) `insert`, `update` and `delete` methods of `spacetimedb::ReducerContext` yourself.";
            } else {
                unique_multi_column_index_hint = "";
            };

            doc_comment = match dsl_method {
                DSLMethod::GetMany(_) => format!(
                    "Get a `{struct_name}` iterator that contains all rows in the `{singular_table_name}` table whose {value_matches_or_values_match} the {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::DeleteMany(_) => format!(
                    "Try to delete all `{struct_name}` rows in the `{singular_table_name}` table whose {value_matches_or_values_match} the {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::GetOneOption(_) => format!(
                    "{unique_multi_column_index_hint}\n\nTry to get a `{struct_name}` from the `{singular_table_name}` table whose {value_matches_or_values_match} the unique {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::Update(_) => format!(
                    "{unique_multi_column_index_hint}\n\nTry to update a `{struct_name}` row of the `{singular_table_name}` table whose {value_matches_or_values_match} the unique {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::DeleteOne(_) => format!(
                    "{unique_multi_column_index_hint}\n\nTry to delete a `{struct_name}` row in the `{singular_table_name}` table whose {value_matches_or_values_match} the unique {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
            };

            trait_name = match dsl_method {
                DSLMethod::GetMany(_) => format_ident!("Get{singular_table_name_pascal_case}RowsBy{index_name_pascal_case}"),
                DSLMethod::DeleteMany(_) => format_ident!("Delete{singular_table_name_pascal_case}RowsBy{index_name_pascal_case}"),
                DSLMethod::GetOneOption(_) => format_ident!("Get{singular_table_name_pascal_case}RowOptionBy{index_name_pascal_case}"),
                DSLMethod::Update(_) => format_ident!("Update{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"),
                DSLMethod::DeleteOne(_) => format_ident!("Delete{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"),
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
            };

            method_name = match dsl_method {
                DSLMethod::GetMany(_) => format_ident!("get_{plural_table_name}_by_{index_name}"),
                DSLMethod::DeleteMany(_) => format_ident!("delete_{plural_table_name}_by_{index_name}"),
                DSLMethod::GetOneOption(_) => format_ident!("get_{singular_table_name}_by_{index_name}"),
                DSLMethod::Update(_) => format_ident!("update_{singular_table_name}_by_{index_name}"),
                DSLMethod::DeleteOne(_) => format_ident!("delete_{singular_table_name}_by_{index_name}"),
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
            };

            return_type = match dsl_method {
                DSLMethod::GetMany(_) => quote! {
                    impl Iterator<Item = #struct_name>
                },
                DSLMethod::DeleteMany(_) => quote! {
                    Result<u64, ()>
                },
                DSLMethod::GetOneOption(_) => quote! {
                    Option<#struct_name>
                },
                DSLMethod::Update(_) => quote! {
                    Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
                },
                DSLMethod::DeleteOne(_) => quote! {
                    Result<bool, ()>
                },
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
            };

            match dsl_method {
                DSLMethod::Update(_) => {
                    method_args.push(
                        SpacetimeDSLMethodArg {
                            is_mut: true,
                            arg_name: singular_table_name.clone(),
                            arg_type: quote! { #struct_name }
                        }
                    );
                
                    let multi_column_index_checks = multi_column_index_checks(
                        &struct_name,
                        &singular_table_name,
                        &spacetimedb_table,
                        &primary_key_column_name,
                    );
                            
                    let mut column_getters = vec![];
                
                    internal_columns.iter().filter(|internal_column| internal_column.spacetimedsl_column_foreign_key.is_some() && internal_column.rust_field_visibility.to_string().ne(&RustVisibility::Private.to_string())).for_each(|internal_column| {
                        let (_, _, column_getter, _) = process_columns_for_create_and_update_method(CreateOrUpdateDSLMethod::Update, &internal_column);
                        match column_getter {
                            Some(column_getter) => column_getters.push(column_getter),
                            None => {},
                        };
                    });
                            
                    // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/37
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
                    
                    let index_name = match is_multi_column_index {
                        true => primary_key_column_name,
                        false => index_name,
                    };
                    
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
                            .#index_name()
                            .update(#singular_table_name)
                        )
                    };
                },
                dsl_method => {
                    let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
                    let mut column_value_getters = vec![];

                    for column in internal_columns {
                        let column_name = &column.rust_field_name;
                        let column_is_string = column.rust_field_type_name_or_path.to_token_stream().to_string().eq(&"String");

                        if !&index_columns.contains(&column_name) {
                            continue;
                        }

                        let wrapper_type_option_to_wrapped_type_option_mapper;
                        let method_arg;
                        let column_value_getter;

                        match &column.spacetimedsl_column_wrapper_type {
                            Some(wrapper_type) => {
                                let wrapper_type = &WrapperType::map(wrapper_type);

                                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                if column_is_string {
                                    wrapper_type_option_to_wrapped_type_option_mapper = TokenStream::default();

                                    match &dsl_method {
                                        DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { &str }
                                            };
                                            column_value_getter = quote! { #column_name };
                                        }
                                        DSLMethod::GetOneOption(_) | DSLMethod::DeleteOne(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { &str }
                                            };
                                            column_value_getter = quote! { #column_name.to_string() };
                                        }
                                        DSLMethod::Update(_) => {
                                            panic!("DSLColumnMethod::Update should already be processed!")
                                        }
                                        DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
                                    }
                                } else if column.spacetimedsl_column_is_option {
                                    wrapper_type_option_to_wrapped_type_option_mapper = quote! {
                                        let #column_name = match #column_name.into() {
                                            None => None,
                                            Some(#column_name) => Some(Into::<#wrapper_type>::into(#column_name).value());
                                        }
                                    };
                                    
                                    method_arg = SpacetimeDSLMethodArg {
                                        is_mut: false,
                                        arg_name: column_name.clone(),
                                        arg_type: quote! { &impl Into<Option<#wrapper_type>> }
                                    };

                                    column_value_getter = quote! { #column_name };
                                } else {
                                    wrapper_type_option_to_wrapped_type_option_mapper = TokenStream::default();

                                    match &dsl_method {
                                        DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { impl Into<#wrapper_type> }
                                            };
                                            column_value_getter = quote! { #column_name.into().value() };
                                        }
                                        DSLMethod::GetOneOption(_) | DSLMethod::DeleteOne(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { impl Into<#wrapper_type> + Clone }
                                            };
                                            column_value_getter = quote! { #column_name.clone().into().value() };
                                        }
                                        DSLMethod::Update(_) => {
                                            panic!("DSLColumnMethod::Update should already be processed!")
                                        }
                                        DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
                                    }
                                }
                            },
                            None => {
                                wrapper_type_option_to_wrapped_type_option_mapper = TokenStream::default();

                                let column_type;

                                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                if column_is_string {
                                    column_type = parse_str("str").expect("parsing should have worked");
                                } else {
                                    column_type = column.rust_field_type_name_or_path.clone();
                                }

                                match dsl_method {
                                    DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                        method_arg = SpacetimeDSLMethodArg {
                                            is_mut: false,
                                            arg_name: column_name.clone(),
                                            arg_type: quote! { &'a #column_type }
                                        };
                                        
                                        column_value_getter = quote! { #column_name };
                                    }
                                    DSLMethod::GetOneOption(_) | DSLMethod::DeleteOne(_) => {
                                        method_arg = SpacetimeDSLMethodArg {
                                            is_mut: false,
                                            arg_name: column_name.clone(),
                                            arg_type: quote! { &#column_type }
                                        };

                                        // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                        // TODO: Does that String stuff also work for GetMany and DeleteMany?
                                        if column_is_string {
                                            column_value_getter = quote! { #column_name.to_string() };
                                        } else {
                                            column_value_getter = quote! { #column_name };
                                        }
                                    }
                                    DSLMethod::Update(_) => {
                                        panic!("DSLColumnMethod::Update should already be processed!")
                                    }
                                    DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount=> panic!("DSLColumnMethod Create / GetAll / GetCount should already be processed!"),
                                }
                            },
                        }
                        
                        wrapper_type_option_to_wrapped_type_option_mappers.push(wrapper_type_option_to_wrapped_type_option_mapper);
                        method_args.push(method_arg);
                        column_value_getters.push(column_value_getter);
                    }

                    let method_impl_prefix= quote! {
                        self
                            .ctx()
                            .db()
                            .#singular_table_name()
                            .#index_name()
                    };

                    method_impl = match dsl_method {
                        DSLMethod::GetMany(_) => {
                            match is_multi_column_index {
                                true => quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    #method_impl_prefix
                                        .filter((#(#column_value_getters),*))
                                },
                                false => quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    #method_impl_prefix
                                        .filter(#(#column_value_getters),*)
                                }
                            }
                        },
                        DSLMethod::DeleteMany(_) => {
                            if spacetimedsl_table.referencing_tables.is_empty() {
                                match is_multi_column_index {
                                    true => quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                        Ok(#method_impl_prefix
                                            .delete((#(#column_value_getters),*))
                                        )
                                    },
                                    false => quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                        Ok(#method_impl_prefix
                                            .delete(#(#column_value_getters),*)
                                        )
                                    }
                                }
                            } else {
                                let referenced_table_function_name = get_referenced_table_function_name(
                                    &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
                                    &singular_table_name
                                );
                                
                                let primary_key_column = internal_columns
                                    .iter()
                                    .find(|c| c.rust_field_name.eq(primary_key_column_name))
                                    .expect("should have a primary key");

                                let primary_key_column_type = &primary_key_column.rust_field_type_name_or_path;

                                match is_multi_column_index {
                                    true => quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*
                                        
                                        let #index_name = (#(#column_value_getters),*);

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
                                    },
                                    false => quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                        let #index_name = #(#column_value_getters),*;

                                        let primary_key_values_of_rows_to_delete: Vec<#primary_key_column_type> = #method_impl_prefix
                                            .filter(#index_name)
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
                                                .delete(#index_name)
                                        )
                                    }
                                }

                                
                            }
                        },
                        DSLMethod::GetOneOption(_) => {
                            match is_multi_column_index {
                                true => {
                                    let multi_column_index_check = get_unique_multi_column_index_check(
                                        &struct_name,
                                        &singular_table_name,
                                        &index_name,
                                        &column_value_getters,
                                    )
                                    .check;

                                    quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                        use spacetimedsl::itertools::Itertools;

                                        #multi_column_index_check

                                        #field_name_for_found_value
                                    }
                                },
                                false => quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*
                                    
                                    #method_impl_prefix
                                        .find(#(#column_value_getters),*)
                                }
                            }
                        },
                        DSLMethod::DeleteOne(_) => {

                            let multi_column_index_check = get_unique_multi_column_index_check(
                                &struct_name,
                                &singular_table_name,
                                &index_name,
                                &column_value_getters,
                            )
                            .check;

                            if spacetimedsl_table.referencing_tables.is_empty() {
                                match is_multi_column_index {
                                    true => {
                                        quote! {
                                            #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                            use spacetimedsl::itertools::Itertools;

                                            #multi_column_index_check

                                            Ok(
                                                self
                                                    .ctx()
                                                    .db()
                                                    .#singular_table_name()
                                                    .#primary_key_column_name()
                                                    .delete(#field_name_for_found_value.expect("value should be found").#primary_key_column_name)
                                            )
                                        }
                                    },
                                    false => quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*
                                        Ok(
                                            #method_impl_prefix
                                                .delete(#(#column_value_getters),*)
                                        )
                                    }
                                }
                            } else {
                                let referenced_table_function_name = get_referenced_table_function_name(
                                    &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
                                    &singular_table_name
                                );

                                match is_multi_column_index {
                                    true => {
                                        quote! {
                                            #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                            let #index_name = (#(#column_value_getters),*);

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
                                    },
                                    false => quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                        let #index_name = #(#column_value_getters),*;

                                        let row_to_delete = #method_impl_prefix
                                            .find(#index_name); // TODO: maybe some types need a .clone() after #index_name

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
                            }
                        },
                        DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount | DSLMethod::Update(_)=> panic!("DSLColumnMethod Create / GetAll / GetCount / Update should already be processed!"),
                    };
                }
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
        if create_or_update_dsl_method.eq(&CreateOrUpdateDSLMethod::Update) && column.rust_field_visibility.to_string().eq(&crate::api::rust::visibility::RustVisibility::Private.to_string()) {
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
    column_value_getters: &Vec<TokenStream>,
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
                let #field_name_for_found_value = match self.ctx().db().#singular_table_name().#index_name().filter((#(#column_value_getters),*)).at_most_one() {
                    Ok(#singular_table_name) => #singular_table_name,
                    Err(_) => {
                        panic!(
                            #more_than_one_panic_msg,
                            (#(#column_value_getters),*),
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
    let mut function_args = vec![
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("ctx"),
            arg_type: quote! { &spacetimedb::ReducerContext }
        },
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("strategy"),
            arg_type: quote! { spacetimedsl::OnDeleteStrategy }
        },
    ];
    // TODO: Result Type
    let return_type = quote! { Result<(), ()> };

    match dsl_internal_referenced_by_function {
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of referencing tables after one row of the `{singular_table_name}` table was deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_value_of_row_to_delete");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: primary_key_value_arg_name.clone(),
                    arg_type: quote! { &#primary_key_column_type } }
            );
        }
        DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of referencing tables after multiple rows of the `{singular_table_name}` table were deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_values_of_rows_to_delete");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: primary_key_value_arg_name.clone(),
                    arg_type: quote! { &Vec<#primary_key_column_type> } }
            );
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
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("ctx"),
            arg_type: quote! { &spacetimedb::ReducerContext }
        },
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("strategy"),
            arg_type: quote! { &spacetimedsl::OnDeleteStrategy }
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
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: primary_key_value_arg_name.clone(),
                    arg_type: quote! { &#referenced_table_primary_key_column_type }
                }
            );
        }
        DSLInternalForeignKeyFunction::ExecuteOnDeleteStrategiesOfThisTableAfterMultipleRowsOfTheReferencedTableWereDeleted => {
            doc_comment = format!("Execute OnDeleteStrategies of the `{singular_table_name}` table after multiple rows of the `{referenced_table_name}` table were deleted.");
            primary_key_value_arg_name = format_ident!("primary_key_values_of_rows_to_delete");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: primary_key_value_arg_name.clone(),
                    arg_type: quote! { &Vec<#referenced_table_primary_key_column_type> }
                }
            );
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
