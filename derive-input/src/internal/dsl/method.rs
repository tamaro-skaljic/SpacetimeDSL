use crate::{
    api::{
        Column,
        db::{Index, IndexType, SpacetimeDBColumn, SpacetimeDBTable},
        dsl::{
            column::SpacetimeDSLColumn,
            method::{
                SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex, SpacetimeDSLMethod,
                SpacetimeDSLTableMethods,
            },
            table::SpacetimeDSLTable,
            wrapper::WrapperType,
        },
        rust::{RustField, RustStruct},
    },
    internal::dsl::wrapper::wrapper_type_into_option,
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, format_ident, quote};
use syn::{Ident, Path, Type, parse_str};

#[derive(Debug)]
pub(in crate::internal) enum DSLTableMethod {
    Create,
    GetAll,
    GetCount,
    ActionsAfterDeleteOne,
    ActionsAfterDeleteMany,
}

#[derive(Debug)]
pub(in crate::internal) enum DSLColumnMethod {
    GetMany,
    DeleteMany,
    GetOneOption,
    Update,
    DeleteOne,
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
            primary_key_column_name,
        );

        let get_all = for_table(
            DSLTableMethod::GetAll,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            columns,
            primary_key_column_name,
        );

        let get_count = for_table(
            DSLTableMethod::GetCount,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            columns,
            primary_key_column_name,
        );

        let actions_after_delete_one;
        let actions_after_delete_many;

        if spacetimedsl_table.referencing_tables.is_empty() {
            actions_after_delete_one = None;
            actions_after_delete_many = None;
        } else {
            actions_after_delete_one = Some(for_table(
                DSLTableMethod::ActionsAfterDeleteOne,
                rust_struct,
                spacetimedb_table,
                spacetimedsl_table,
                columns,
                primary_key_column_name,
            ));

            actions_after_delete_many = Some(for_table(
                DSLTableMethod::ActionsAfterDeleteMany,
                rust_struct,
                spacetimedb_table,
                spacetimedsl_table,
                columns,
                primary_key_column_name,
            ));
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
                        },
                    ));
                }
            };
        }

        let methods = SpacetimeDSLTableMethods {
            create,
            get_all,
            get_count,
            actions_after_delete_one,
            actions_after_delete_many,
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
                })
            }
        };

        Some(methods)
    }
}

pub(in crate::internal) fn for_table(
    dsl_table_method: DSLTableMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &Vec<Column>,
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = format_ident!("{}", *rust_struct.name);
    let singular_table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let singular_table_name_pascal_case =
        RenameRule::PascalCase.apply_to_field(spacetimedb_table.singular_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;

    let doc_comment = match dsl_table_method {
        DSLTableMethod::Create => format!("Create a row in the `{singular_table_name}` table."),
        DSLTableMethod::GetAll => {
            format!("Get all rows inside the `{singular_table_name}` table.")
        }
        DSLTableMethod::GetCount => {
            format!("Get the count of all rows inside the `{singular_table_name}` table.")
        }
        DSLTableMethod::ActionsAfterDeleteOne => {
            format!("Execute OnDeleteStrategies of referencing tables after deleting 1 row in the `{singular_table_name}` table.")
        }
        DSLTableMethod::ActionsAfterDeleteMany => {
            format!("Execute OnDeleteStrategies of referencing tables after deleting multiple rows in the `{singular_table_name}` table.")
        }
    }
    .into();

    let trait_name = match dsl_table_method {
        DSLTableMethod::Create => format!("Create{}Row", struct_name),
        DSLTableMethod::GetAll => format!("GetAll{}Rows", struct_name),
        DSLTableMethod::GetCount => format!("GetCountOf{}Rows", struct_name),
        DSLTableMethod::ActionsAfterDeleteOne => {
            format!("ExecuteOnDeleteStrategiesAfterOne{singular_table_name_pascal_case}RowWasDeleted")
        }
        DSLTableMethod::ActionsAfterDeleteMany => {
            format!("ExecuteOnDeleteStrategiesAfterMultiple{singular_table_name_pascal_case}RowsWereDeleted")
        }
    }
    .into();

    let method_name = match dsl_table_method {
        DSLTableMethod::Create => format!("create_{}", singular_table_name),
        DSLTableMethod::GetAll => format!("get_all_{}", plural_table_name),
        DSLTableMethod::GetCount => format!("get_count_of_{}", plural_table_name),
        DSLTableMethod::ActionsAfterDeleteOne => {
            format!("execute_on_delete_strategies_after_one_{singular_table_name}_row_was_deleted")
        }
        DSLTableMethod::ActionsAfterDeleteMany => {
            format!("execute_on_delete_strategies_after_multiple_{singular_table_name}_rows_were_deleted")
        }
    }
    .into();

    let return_type = match dsl_table_method {
        DSLTableMethod::Create => {
            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            quote! {
                Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
            }
        }
        DSLTableMethod::GetAll => quote! {
            impl Iterator<Item = #struct_name>
        },
        DSLTableMethod::GetCount => quote! {
            u64
        },
        DSLTableMethod::ActionsAfterDeleteOne => quote! {
            Result<(), spacetimedsl::ReferenceIntegrityViolationError>
        },
        DSLTableMethod::ActionsAfterDeleteMany => quote! {
            Result<(), spacetimedsl::ReferenceIntegrityViolationError>
        },
    }
    .to_string()
    .into();

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
                                .unwrap());
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
            method_impl = quote! {
            #use_itertools

            #(#into_options)*
            let #singular_table_name = #struct_name {
                #(#constructor_args),*
            };

            #(#multi_column_index_checks)*

            return self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .try_insert(#singular_table_name);
                };
        }
        DSLTableMethod::GetAll => {
            method_impl = quote! {
                return self
                        .ctx()
                        .db()
                        .#singular_table_name()
                        .iter();
            };
        }
        DSLTableMethod::GetCount => {
            method_impl = quote! {
                return self
                        .ctx()
                        .db()
                        .#singular_table_name()
                        .count();
            };
        }
        DSLTableMethod::ActionsAfterDeleteOne | DSLTableMethod::ActionsAfterDeleteMany => {
            let primary_key_column = columns
                .iter()
                .find(|c| c.rust_field.name.eq(primary_key_column_name))
                .unwrap();

            let primary_key_column_type: Type =
                parse_str(&primary_key_column.rust_field.type_name_or_path).unwrap();

            method_args.push(match dsl_table_method {
                DSLTableMethod::ActionsAfterDeleteOne => quote! {
                    #primary_key_column_name: &#primary_key_column_type
                },
                DSLTableMethod::ActionsAfterDeleteMany => quote! {
                    #primary_key_column_name: Vec<&#primary_key_column_type>
                },
                dsl_table_method => {
                    panic!("DSLTableMethod {dsl_table_method:?} should already be processed.")
                }
            });

            let mut use_clauses = vec![];
            let mut on_error_strategy_calls = vec![];
            let mut cascade_strategy_calls = vec![];
            let set_none_strategy_calls: Vec<TokenStream> = vec![];
            let mut set_zero_strategy_calls = vec![];

            for referencing_table in spacetimedsl_table.referencing_tables.iter() {
                let referencing_table_path: Path = parse_str(&referencing_table.path).unwrap();
                let referencing_table_name = &referencing_table.table_name;
                let referencing_table_name_pascal_case =
                    RenameRule::PascalCase.apply_to_field(referencing_table_name.to_string());

                // Use clauses
                let foreign_trait_name;
                let function_name;

                match dsl_table_method {
                    DSLTableMethod::ActionsAfterDeleteOne => {
                        foreign_trait_name = format_ident!(
                            "ExecuteOnDeleteStrategiesOf{referencing_table_name_pascal_case}AfterOne{singular_table_name_pascal_case}RowWasDeleted"
                        );
                        function_name = format_ident!(
                            "execute_on_delete_strategies_of_{referencing_table_name}_after_one_{singular_table_name}_row_was_deleted",
                        );
                    }
                    DSLTableMethod::ActionsAfterDeleteMany => {
                        foreign_trait_name = format_ident!(
                            "ExecuteOnDeleteStrategiesOf{referencing_table_name_pascal_case}AfterMultiple{singular_table_name_pascal_case}RowsWereDeleted"
                        );
                        function_name = format_ident!(
                            "execute_on_delete_strategies_of_{referencing_table_name}_after_multiple_{singular_table_name}_rows_were_deleted",
                        );
                    }
                    dsl_table_method => {
                        panic!("DSLTableMethod {dsl_table_method:?} should already be processed.")
                    }
                };

                use_clauses.push(quote! {
                    use #referencing_table_path::#foreign_trait_name;
                });

                on_error_strategy_calls.push(quote! {
                    spacetimedsl::internal::DSLInternals::#function_name(dsl, spacetimedsl::OnDeleteStrategy::Error, #primary_key_column_name)?;
                });
                cascade_strategy_calls.push(quote! {
                    spacetimedsl::internal::DSLInternals::#function_name(dsl, spacetimedsl::OnDeleteStrategy::Cascade, #primary_key_column_name)?;
                });
                /* TODO: Because Option is currently not allowed on primary_key and unique/btree indices this strategy isn't used and implemented yet.
                set_none_strategy_calls.push(quote! {
                    spacetimedsl::internal::DSLInternals::#method_name(dsl, spacetimedsl::OnDeleteStrategy::SetNone, #primary_key_column_name)?;
                });
                */
                set_zero_strategy_calls.push(quote! {
                    spacetimedsl::internal::DSLInternals::#function_name(dsl, spacetimedsl::OnDeleteStrategy::SetZero, #primary_key_column_name)?;
                });
            }

            // TODO: Handle OnError Strategy Calls to return an ReferenceIntegrityViolationError on Failure.
            method_impl = quote! {
                #(#use_clauses)*
                #(#on_error_strategy_calls)*
                #(#cascade_strategy_calls)*
                #(#set_none_strategy_calls)*
                #(#set_zero_strategy_calls)*
                Ok(())
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

    let doc_comment = match dsl_method {
        DSLColumnMethod::GetMany => format!("Get all {struct_name} rows inside the {singular_table_name} table filtered by the single-column index on the {column_name} column."),
        DSLColumnMethod::DeleteMany => format!("Delete all {struct_name} rows inside the {singular_table_name} table filtered by the single-column index on the {column_name} column."),
        DSLColumnMethod::GetOneOption => format!("Get an Option<{struct_name}> row inside the {singular_table_name} table filtered by the unique single-column index on the {column_name} column."),
        DSLColumnMethod::Update => format!("Update a {struct_name} row inside the {singular_table_name} table by the unique single-column index on the {column_name} column."),
        DSLColumnMethod::DeleteOne => format!("Delete a {struct_name} row inside the {singular_table_name} table filtered by the unique single-column index on the {column_name} column."),
    }.into();

    let trait_name = match dsl_method {
        DSLColumnMethod::GetMany => format!("Get{struct_name}RowsBy{column_name_pascal_case}"),
        DSLColumnMethod::DeleteMany => {
            format!("Delete{struct_name}RowsBy{column_name_pascal_case}")
        }
        DSLColumnMethod::GetOneOption => {
            format!("Get{struct_name}RowOptionBy{column_name_pascal_case}")
        }
        DSLColumnMethod::Update => format!("Update{struct_name}RowBy{column_name_pascal_case}"),
        DSLColumnMethod::DeleteOne => format!("Delete{struct_name}RowBy{column_name_pascal_case}"),
    }
    .into();

    let method_name = match dsl_method {
        DSLColumnMethod::GetMany => format!("get_{plural_table_name}_by_{column_name}"),
        DSLColumnMethod::DeleteMany => format!("delete_{plural_table_name}_by_{column_name}"),
        DSLColumnMethod::GetOneOption => format!("get_{singular_table_name}_by_{column_name}"),
        DSLColumnMethod::Update => format!("update_{singular_table_name}_by_{column_name}"),
        DSLColumnMethod::DeleteOne => format!("delete_{singular_table_name}_by_{column_name}"),
    }
    .into();

    let return_type = match dsl_method {
        DSLColumnMethod::GetMany => quote! {impl Iterator<Item = #struct_name>},
        DSLColumnMethod::DeleteMany => quote! {u64},
        DSLColumnMethod::GetOneOption => quote! {Option<#struct_name>},
        DSLColumnMethod::Update => {
            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            quote! {Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>}
        },
        DSLColumnMethod::DeleteOne => quote! {bool},
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

            method_impl = quote! {
                #use_itertools

                #(#multi_column_index_checks)*

                #modified_at
                return Ok(self
                        .ctx()
                        .db()
                        .#singular_table_name()
                        .#column_name()
                        .update(#singular_table_name));
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
                            column_type = parse_str("str").unwrap();
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
                            column_type = parse_str("str").unwrap();
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
                #into_option
                    return self
                        .ctx()
                        .db()
                        .#singular_table_name()
                        .#column_name()
            };

            method_impl = match dsl_method {
                DSLColumnMethod::GetMany => quote! {
                    #method_impl_prefix
                        .filter(#column_value);
                },
                DSLColumnMethod::DeleteMany => quote! {
                    #method_impl_prefix
                        .delete(#column_value);
                },
                DSLColumnMethod::GetOneOption => quote! {
                    #method_impl_prefix
                        .find(#column_value);
                },
                DSLColumnMethod::DeleteOne => quote! {
                    #method_impl_prefix
                        .delete(#column_value);
                },
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
        DSLColumnMethod::DeleteMany => quote! {u64},
        DSLColumnMethod::GetOneOption => quote! {Option<#struct_name>},
        DSLColumnMethod::Update => {
            let try_insert_error_generic_type = format_ident!("{singular_table_name}__TableHandle");
            quote! {Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>}
        },
        DSLColumnMethod::DeleteOne => quote! {bool},
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

            method_impl = quote! {
                #use_itertools

                #(#multi_column_index_checks)*

                #modified_at
                return Ok(self
                        .ctx()
                        .db()
                        .#singular_table_name()
                        .#primary_key_column_name()
                        .update(#singular_table_name));
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
                                    column_value = quote! { #column_name: #column_name };
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
                                    column_value =
                                        quote! { #column_name: #column_name.into().value() };
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
                            #(#into_options)*
                            return self
                                .ctx()
                                .db()
                                .#singular_table_name()
                                .#index_name()
                    };

                    method_impl = match dsl_method {
                        DSLColumnMethod::GetMany => quote! {
                            #method_impl_prefix
                                .filter((#(#column_values),*));
                        },
                        DSLColumnMethod::DeleteMany => quote! {
                            #method_impl_prefix
                                .delete((#(#column_values),*));
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
                        DSLColumnMethod::DeleteOne => quote! {
                            #method_impl_prefix

                            return self
                                .ctx()
                                .db()
                                .#singular_table_name()
                                .#primary_key_column_name()
                                .delete(#field_name_for_found_value.unwrap().#primary_key_column_name);
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
                            .unwrap());
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
