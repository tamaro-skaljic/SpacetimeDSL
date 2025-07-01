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
                SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex,
            },
            foreign_key::OnDeleteStrategy,
            method::{SpacetimeDSLMethod, SpacetimeDSLMethodArg},
            table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
            wrapper::WrapperType,
        },
        rust::{table::RustStruct, visibility::RustVisibility},
    },
    internal::{
        column::InternalColumn, dsl::wrapper::map_wrapper_type_option_to_wrapped_type_option,
    },
};
use ident_case::RenameRule;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display},
};
use strum::IntoEnumIterator;
use syn::{Ident, Path, parse_str};

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

// FIXME: Ensure that any panic! / expect() / unwrap() is replaced by proper error handling, either returning spacetimedsl::SpacetimeDSLError during runtime or syn::Error during compilation time

#[derive(Debug)]
pub enum OneOrMultiple {
    One,
    Multiple,
}

impl quote::ToTokens for OneOrMultiple {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use proc_macro2::{Punct, Spacing};
        use quote::{TokenStreamExt, format_ident};

        tokens.append(format_ident!("spacetimedsl"));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(format_ident!("OneOrMultiple"));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(format_ident!(
            "{}",
            match self {
                OneOrMultiple::One => "One",
                OneOrMultiple::Multiple => "Multiple",
            },
        ));
    }
}

#[derive(PartialEq)]
enum CreateOrUpdate {
    Create,
    Update,
}

impl Display for CreateOrUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreateOrUpdate::Create => write!(f, "Create"),
            CreateOrUpdate::Update => write!(f, "Update"),
        }
    }
}

#[derive(PartialEq)]
enum Action {
    Create,
    Get,
    Update,
    Delete,
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Create => write!(f, "Create"),
            Action::Get => write!(f, "Get"),
            Action::Update => write!(f, "Update"),
            Action::Delete => write!(f, "Delete"),
        }
    }
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
                    internal_columns,
                );

                let delete_many = for_method(
                    DSLMethod::DeleteMany(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
                );

                SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
                    get_many,
                    delete_many,
                })
            }
            true => {
                let get_one_option = for_method(
                    DSLMethod::GetOneOption(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
                );

                let update = match spacetimedsl_table.is_mutable {
                    false => None,
                    true => Some(for_method(
                        DSLMethod::Update(index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        internal_columns,
                    )),
                };

                let delete_one = for_method(
                    DSLMethod::DeleteOne(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
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

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn try_parse(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &SpacetimeDSLTable,
        columns: &Vec<Column>,
        internal_columns: &Vec<InternalColumn>,
    ) -> syn::Result<SpacetimeDSLTableMethods> {
        let create = for_method(
            DSLMethod::Create,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
        );

        let get_all = for_method(
            DSLMethod::GetAll,
            rust_struct,
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
        );

        let get_count = for_method(
            DSLMethod::GetCount,
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
                &OneOrMultiple::One,
                spacetimedb_table,
                spacetimedsl_table,
                columns,
            ));

            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted = Some(for_referenced_by(
                &OneOrMultiple::Multiple,
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
                            rust_struct,
                            &OneOrMultiple::One,
                            spacetimedb_table,
                            spacetimedsl_table,
                            columns,
                            referenced_table_name,
                            &columns_with_foreign_key,
                        )
                    );
                    execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted.push(
                        for_foreign_key(
                            rust_struct,
                            &OneOrMultiple::Multiple,
                            spacetimedb_table,
                            spacetimedsl_table,
                            columns,
                            referenced_table_name,
                            &columns_with_foreign_key,
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
                        internal_columns,
                    );
                    let delete_many = for_method(
                        DSLMethod::DeleteMany(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        internal_columns,
                    );

                    multi_column_indices.push(SpacetimeDSLColumnMethods::ForIndex(
                        SpacetimeDSLColumnMethodsForIndex {
                            get_many,
                            delete_many,
                        },
                    ));
                }
                true => {
                    let get_one_option = for_method(
                        DSLMethod::GetOneOption(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        internal_columns,
                    );

                    let update = match spacetimedsl_table.is_mutable {
                        false => None,
                        true => Some(for_method(
                            DSLMethod::Update(multi_column_index),
                            rust_struct,
                            spacetimedb_table,
                            spacetimedsl_table,
                            internal_columns,
                        )),
                    };

                    let delete_one = for_method(
                        DSLMethod::DeleteOne(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        internal_columns,
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
            execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted,
            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted,
            execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted,
            execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted,
            multi_column_indices,
        };

        Ok(methods)
    }
}

fn process_columns_for_create_and_update_method(
    create_or_update: CreateOrUpdate,
    internal_column: &InternalColumn,
) -> (
    Option<SpacetimeDSLMethodArg>,
    Option<TokenStream>,
    Option<TokenStream>,
    TokenStream,
) {
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
        CreateOrUpdate::Create => {
            if internal_column.spacetimedb_column_is_auto_inc
                || internal_column
                    .rust_field_name
                    .to_string()
                    .eq(&"created_at")
                || internal_column
                    .rust_field_name
                    .to_string()
                    .eq(&"modified_at")
            {
                if internal_column.spacetimedb_column_is_auto_inc {
                    constructor_arg = Some(quote! {
                        let #column_name = #column_type::default();
                    });
                } else if internal_column
                    .rust_field_name
                    .to_string()
                    .eq(&"created_at")
                {
                    constructor_arg = Some(quote! {
                        let created_at = self.ctx().timestamp;
                    });
                } else if internal_column
                    .rust_field_name
                    .to_string()
                    .eq(&"modified_at")
                {
                    // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/37
                    constructor_arg = Some(quote! {
                        let modified_at = self.ctx().timestamp;
                    });
                }
                return (
                    method_arg,
                    wrapper_type_option_to_wrapped_type_option_mapper,
                    constructor_arg,
                    constructor_arg_name,
                );
            }
        }
        CreateOrUpdate::Update => {}
    };

    match &internal_column.spacetimedsl_column_wrapper_type {
        Some(wrapper_type) => match wrapper_type {
            WrapperType::Wrap(wrapper_type) => {
                if internal_column
                    .rust_field_type_name_or_path
                    .to_token_stream()
                    .to_string()
                    .eq(&"String")
                {
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { &str },
                    });
                    match create_or_update {
                        CreateOrUpdate::Create => {
                            constructor_arg = Some(quote! {
                                let #column_name = #column_name.to_string();
                            });
                        }
                        CreateOrUpdate::Update => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#getter_name();
                            });
                        }
                    };
                } else {
                    let wrapped_type_name_or_path = WrapperType::map_to_wrapped_type(wrapper_type);

                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { #wrapped_type_name_or_path },
                    });
                }
            }
            WrapperType::Wrapped(_) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if internal_column.spacetimedsl_column_is_option {
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { impl Into<Option<#wrapper_type_name_or_path>> },
                    });
                    wrapper_type_option_to_wrapped_type_option_mapper =
                        Some(map_wrapper_type_option_to_wrapped_type_option(
                            &column_name,
                            wrapper_type_name_or_path,
                        ));
                } else {
                    method_arg = Some(SpacetimeDSLMethodArg {
                        is_mut: false,
                        arg_name: column_name.clone(),
                        arg_type: quote! { impl Into<#wrapper_type_name_or_path> },
                    });
                    match create_or_update {
                        CreateOrUpdate::Create => {
                            constructor_arg = Some(quote! {
                                let #column_name = #column_name.into().value();
                            });
                        }
                        CreateOrUpdate::Update => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#getter_name().value();
                            });
                        }
                    };
                }
            }
        },
        None => {
            if internal_column
                .rust_field_type_name_or_path
                .to_token_stream()
                .to_string()
                .eq(&"String")
            {
                method_arg = Some(SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: column_name.clone(),
                    arg_type: quote! { &str },
                });

                match create_or_update {
                    CreateOrUpdate::Create => {
                        constructor_arg = Some(quote! {
                            let #column_name = #column_name.to_string();
                        });
                    }
                    CreateOrUpdate::Update => {
                        constructor_arg = Some(quote! {
                            let #column_name = #singular_table_name.#getter_name();
                        });
                    }
                };
            } else {
                method_arg = Some(SpacetimeDSLMethodArg {
                    is_mut: false,
                    arg_name: column_name.clone(),
                    arg_type: quote! { #column_type },
                });
            }
        }
    };

    (
        method_arg,
        wrapper_type_option_to_wrapped_type_option_mapper,
        constructor_arg,
        constructor_arg_name,
    )
}

pub(in crate::internal) fn for_method(
    dsl_method: DSLMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_columns: &Vec<InternalColumn>,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_as_string = singular_table_name.to_string();
    let singular_table_name_pascal_case =
        RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;

    let one = OneOrMultiple::One;
    let multiple = OneOrMultiple::Multiple;

    // TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/35
    let doc_comment;
    let trait_name;
    let method_name;
    // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/36
    let return_type;

    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let mut paths_of_traits_to_extend =
        vec![parse_str("spacetimedsl::DSLContext").expect("parsing should have worked")];
    let mut method_args = vec![];
    let method_impl;

    match dsl_method {
        DSLMethod::Create => {
            doc_comment = format!("Create a row in the `{singular_table_name}` table.");

            trait_name = format_ident!("Create{singular_table_name_pascal_case}Row");

            method_name = format_ident!("create_{}", singular_table_name);

            return_type = quote! {
                Result<#struct_name, spacetimedsl::SpacetimeDSLError>
            };

            let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
            let mut constructor_args = vec![];
            let mut constructor_arg_names = vec![];

            for internal_column in internal_columns {
                let (
                    method_arg,
                    wrapper_type_option_to_wrapped_type_option_mapper,
                    constructor_arg,
                    constructor_arg_name,
                ) = process_columns_for_create_and_update_method(
                    CreateOrUpdate::Create,
                    &internal_column,
                );
                match method_arg {
                    Some(method_arg) => method_args.push(method_arg),
                    None => {}
                }

                match wrapper_type_option_to_wrapped_type_option_mapper {
                    Some(wrapper_type_option_to_wrapped_type_option_mapper) => {
                        wrapper_type_option_to_wrapped_type_option_mappers
                            .push(wrapper_type_option_to_wrapped_type_option_mapper)
                    }
                    None => {}
                }

                match constructor_arg {
                    Some(constructor_arg) => constructor_args.push(constructor_arg),
                    None => {}
                }

                constructor_arg_names.push(constructor_arg_name)
            }

            let mut column_names_and_row_values = String::new();
            column_names_and_row_values.push_str("{{ ");
            column_names_and_row_values.push_str(&format!("{singular_table_name} : "));
            column_names_and_row_values.push_str("{} }}");

            let multi_column_index_checks = multi_column_index_checks(
                Action::Create,
                &struct_name,
                &singular_table_name,
                &spacetimedb_table,
                &column_names_and_row_values,
            );

            let use_itertools = if multi_column_index_checks.len() > 0 {
                quote! {
                    use spacetimedsl::itertools::Itertools;
                }
            } else {
                TokenStream::default()
            };

            let res = reference_integrity_checks_on_create_or_update(
                CreateOrUpdate::Create,
                spacetimedb_table,
                &internal_columns,
                paths_of_traits_to_extend,
                (&column_names_and_row_values, None),
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

                let mut #field_name_for_found_value: Option<#struct_name> = None;

                #(#multi_column_index_checks)*

                #(#reference_integrity_checks)*

                match self
                    .ctx()
                    .db()
                    .#singular_table_name()
                    .try_insert(#singular_table_name.clone()) {
                    Ok(entity) => Ok(entity),
                    Err(error) => match error {
                        spacetimedb::TryInsertError::UniqueConstraintViolation(_) => {
                            Err(spacetimedsl::SpacetimeDSLError::UniqueConstraintViolation {
                                table_name: #singular_table_name_as_string.into(),
                                action: spacetimedsl::Action::Create,
                                error_from: spacetimedsl::ErrorFrom::SpacetimeDB,
                                one_or_multiple: #one,
                                column_names_and_row_values: format!("{:?}", #singular_table_name).into(),
                            })
                        }
                        spacetimedb::TryInsertError::AutoIncOverflow(_) => {
                            Err(spacetimedsl::SpacetimeDSLError::AutoIncOverflow {
                                table_name: #singular_table_name_as_string.into(),
                            })
                        }
                    },
                }
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
        DSLMethod::GetMany(index)
        | DSLMethod::DeleteMany(index)
        | DSLMethod::GetOneOption(index)
        | DSLMethod::Update(index)
        | DSLMethod::DeleteOne(index) => {
            let index_name = &index.name;
            let index_name_pascal_case =
                RenameRule::PascalCase.apply_to_field(index_name.to_string());

            let is_unique_index = index.is_unique;
            let is_multi_column_index;
            let mut index_columns = vec![];

            let value_matches_or_values_match;
            let single_or_multi;
            let index_documentation;
            let mut documentation_on_column_or_columns;

            let mut column_names_and_row_values = String::new();
            column_names_and_row_values.push_str("{{ ");
            match &index.index_type {
                IndexType::BTreeSingleColumn { column } => {
                    is_multi_column_index = false;
                    index_columns.push(column.clone());
                    value_matches_or_values_match = "value matches the value from";
                    single_or_multi = "single";
                    index_documentation = format!("btree index");
                    documentation_on_column_or_columns = format!("`{column}` column");
                    // FIXME: column_names_and_row_values
                }
                IndexType::BTreeMultiColumn { columns } => {
                    is_multi_column_index = true;
                    value_matches_or_values_match = "values match the values from";
                    single_or_multi = "multi";
                    index_documentation = format!("btree index `{index_name}`");

                    documentation_on_column_or_columns = String::new();
                    documentation_on_column_or_columns.push_str(&format!("columns"));

                    let mut columns: VecDeque<Ident> = columns.clone().into();

                    let first_column = columns.pop_front().expect(
                        "There should be a first column in Vec<Ident> of BTreeMultiColumn.",
                    );
                    let last_column = columns
                        .pop_back()
                        .expect("There should be a last column in Vec<Ident> of BTreeMultiColumn.");
                    let any_other_column = columns;

                    documentation_on_column_or_columns.push_str(&format!(" `{first_column}`"));
                    column_names_and_row_values.push_str(&format!("{first_column} : "));
                    column_names_and_row_values.push_str("{} ");
                    index_columns.push(first_column);

                    for any_other_column in any_other_column {
                        documentation_on_column_or_columns
                            .push_str(&format!(", `{any_other_column}`"));
                        column_names_and_row_values.push_str(&format!(", {any_other_column} : "));
                        column_names_and_row_values.push_str("{} ");
                        index_columns.push(any_other_column);
                    }

                    documentation_on_column_or_columns.push_str(&format!(" and `{last_column}`"));
                    column_names_and_row_values.push_str(&format!(", {last_column} : "));
                    column_names_and_row_values.push_str("{}");
                    index_columns.push(last_column);
                }
                IndexType::Direct { column } => {
                    is_multi_column_index = false;
                    index_columns.push(column.clone());
                    value_matches_or_values_match = "value matches";
                    single_or_multi = "single";
                    index_documentation = format!("direct index");
                    documentation_on_column_or_columns = format!("`{column}` column");
                    // FIXME: column_names_and_row_values
                }
            };
            column_names_and_row_values.push_str(" }}");

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
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => panic!(
                    "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                ),
            };

            trait_name = match dsl_method {
                DSLMethod::GetMany(_) => format_ident!(
                    "Get{singular_table_name_pascal_case}RowsBy{index_name_pascal_case}"
                ),
                DSLMethod::DeleteMany(_) => format_ident!(
                    "Delete{singular_table_name_pascal_case}RowsBy{index_name_pascal_case}"
                ),
                DSLMethod::GetOneOption(_) => format_ident!(
                    "Get{singular_table_name_pascal_case}RowOptionBy{index_name_pascal_case}"
                ),
                DSLMethod::Update(_) => format_ident!(
                    "Update{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"
                ),
                DSLMethod::DeleteOne(_) => format_ident!(
                    "Delete{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"
                ),
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => panic!(
                    "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                ),
            };

            method_name = match dsl_method {
                DSLMethod::GetMany(_) => format_ident!("get_{plural_table_name}_by_{index_name}"),
                DSLMethod::DeleteMany(_) => {
                    format_ident!("delete_{plural_table_name}_by_{index_name}")
                }
                DSLMethod::GetOneOption(_) => {
                    format_ident!("get_{singular_table_name}_by_{index_name}")
                }
                DSLMethod::Update(_) => {
                    format_ident!("update_{singular_table_name}_by_{index_name}")
                }
                DSLMethod::DeleteOne(_) => {
                    format_ident!("delete_{singular_table_name}_by_{index_name}")
                }
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => panic!(
                    "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                ),
            };

            return_type = match dsl_method {
                DSLMethod::GetMany(_) => quote! {
                    impl Iterator<Item = #struct_name>
                },
                DSLMethod::DeleteMany(_) => quote! {
                    Result<spacetimedsl::DeletionResult, spacetimedsl::SpacetimeDSLError>
                },
                DSLMethod::GetOneOption(_) => quote! {
                    Result<#struct_name, spacetimedsl::SpacetimeDSLError>
                },
                DSLMethod::Update(_) => quote! {
                    Result<#struct_name, spacetimedsl::SpacetimeDSLError>
                },
                DSLMethod::DeleteOne(_) => quote! {
                    Result<spacetimedsl::DeletionResult, spacetimedsl::SpacetimeDSLError>
                },
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => panic!(
                    "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                ),
            };

            match dsl_method {
                DSLMethod::Update(_) => {
                    method_args.push(SpacetimeDSLMethodArg {
                        is_mut: true,
                        arg_name: singular_table_name.clone(),
                        arg_type: quote! { #struct_name },
                    });

                    let multi_column_index_checks = multi_column_index_checks(
                        Action::Update,
                        &struct_name,
                        &singular_table_name,
                        &spacetimedb_table,
                        &column_names_and_row_values,
                    );

                    let mut row_value_getters = vec![];

                    internal_columns
                        .iter()
                        .filter(|internal_column| {
                            internal_column.spacetimedsl_column_foreign_key.is_some()
                                && internal_column
                                    .rust_field_visibility
                                    .to_string()
                                    .ne(&RustVisibility::Private.to_string())
                        })
                        .for_each(|internal_column| {
                            let (_, _, column_getter, _) =
                                process_columns_for_create_and_update_method(
                                    CreateOrUpdate::Update,
                                    &internal_column,
                                );
                            match column_getter {
                                Some(column_getter) => row_value_getters.push(column_getter),
                                None => {}
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

                    let res = reference_integrity_checks_on_create_or_update(
                        CreateOrUpdate::Update,
                        spacetimedb_table,
                        internal_columns,
                        paths_of_traits_to_extend,
                        (&column_names_and_row_values, Some(&row_value_getters)),
                    );
                    paths_of_traits_to_extend = res.0;
                    let reference_integrity_checks = res.1;

                    let index_name = match is_multi_column_index {
                        true => &format_ident!("id"),
                        false => index_name,
                    };

                    method_impl = quote! {
                        #use_itertools

                        let mut #field_name_for_found_value: Option<#struct_name> = None;

                        #(#multi_column_index_checks)*

                        #(#row_value_getters)*
                        #(#reference_integrity_checks)*

                        #modified_at

                        // FIXME: try_update instead of update
                        // FIXME: on error return Err(spacetimedsl::SpacetimeDSLError);
                        Ok(self
                            .ctx()
                            .db()
                            .#singular_table_name()
                            .#index_name()
                            .update(#singular_table_name)
                        )
                    };
                }
                dsl_method => {
                    let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
                    let mut row_value_getters = vec![];

                    for column in internal_columns {
                        let column_name = &column.rust_field_name;
                        let column_is_string = column
                            .rust_field_type_name_or_path
                            .to_token_stream()
                            .to_string()
                            .eq(&"String");

                        if !&index_columns.contains(&column_name) {
                            continue;
                        }

                        let wrapper_type_option_to_wrapped_type_option_mapper;
                        let method_arg;
                        let row_value_getter;

                        match &column.spacetimedsl_column_wrapper_type {
                            Some(wrapper_type) => {
                                let wrapper_type = &WrapperType::map(wrapper_type);

                                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                if column_is_string {
                                    wrapper_type_option_to_wrapped_type_option_mapper =
                                        TokenStream::default();

                                    match &dsl_method {
                                        DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { &str },
                                            };
                                            row_value_getter = quote! { #column_name };
                                        }
                                        DSLMethod::GetOneOption(_) | DSLMethod::DeleteOne(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { &str },
                                            };
                                            row_value_getter = quote! { #column_name.to_string() };
                                        }
                                        DSLMethod::Update(_) => {
                                            panic!(
                                                "DSLColumnMethod::Update should already be processed!"
                                            )
                                        }
                                        DSLMethod::Create
                                        | DSLMethod::GetAll
                                        | DSLMethod::GetCount => panic!(
                                            "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                                        ),
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
                                        arg_type: quote! { &impl Into<Option<#wrapper_type>> },
                                    };

                                    row_value_getter = quote! { #column_name };
                                } else {
                                    wrapper_type_option_to_wrapped_type_option_mapper =
                                        TokenStream::default();

                                    match &dsl_method {
                                        DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { impl Into<#wrapper_type> },
                                            };
                                            row_value_getter =
                                                quote! { #column_name.into().value() };
                                        }
                                        DSLMethod::GetOneOption(_) | DSLMethod::DeleteOne(_) => {
                                            method_arg = SpacetimeDSLMethodArg {
                                                is_mut: false,
                                                arg_name: column_name.clone(),
                                                arg_type: quote! { impl Into<#wrapper_type> + Clone },
                                            };
                                            row_value_getter =
                                                quote! { #column_name.clone().into().value() };
                                        }
                                        DSLMethod::Update(_) => {
                                            panic!(
                                                "DSLColumnMethod::Update should already be processed!"
                                            )
                                        }
                                        DSLMethod::Create
                                        | DSLMethod::GetAll
                                        | DSLMethod::GetCount => panic!(
                                            "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                                        ),
                                    }
                                }
                            }
                            None => {
                                wrapper_type_option_to_wrapped_type_option_mapper =
                                    TokenStream::default();

                                let column_type;

                                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                if column_is_string {
                                    column_type =
                                        parse_str("str").expect("parsing should have worked");
                                } else {
                                    column_type = column.rust_field_type_name_or_path.clone();
                                }

                                match dsl_method {
                                    DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                        method_arg = SpacetimeDSLMethodArg {
                                            is_mut: false,
                                            arg_name: column_name.clone(),
                                            arg_type: quote! { &'a #column_type },
                                        };

                                        row_value_getter = quote! { #column_name };
                                    }
                                    DSLMethod::GetOneOption(_) | DSLMethod::DeleteOne(_) => {
                                        method_arg = SpacetimeDSLMethodArg {
                                            is_mut: false,
                                            arg_name: column_name.clone(),
                                            arg_type: quote! { &#column_type },
                                        };

                                        // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                        // TODO: Does that String stuff also work for GetMany and DeleteMany?
                                        if column_is_string {
                                            row_value_getter = quote! { #column_name.to_string() };
                                        } else {
                                            row_value_getter = quote! { #column_name };
                                        }
                                    }
                                    DSLMethod::Update(_) => {
                                        panic!(
                                            "DSLColumnMethod::Update should already be processed!"
                                        )
                                    }
                                    DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => {
                                        panic!(
                                            "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                                        )
                                    }
                                }
                            }
                        }

                        wrapper_type_option_to_wrapped_type_option_mappers
                            .push(wrapper_type_option_to_wrapped_type_option_mapper);
                        method_args.push(method_arg);
                        row_value_getters.push(row_value_getter);
                    }

                    let method_impl_prefix = quote! {
                        self
                            .ctx()
                            .db()
                            .#singular_table_name()
                            .#index_name()
                    };

                    match dsl_method {
                        DSLMethod::GetMany(_) => match is_multi_column_index {
                            true => {
                                method_impl = quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    #method_impl_prefix
                                        .filter((#(#row_value_getters),*))
                                }
                            }
                            false => {
                                method_impl = quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    #method_impl_prefix
                                        .filter(#(#row_value_getters),*)
                                }
                            }
                        },
                        DSLMethod::DeleteMany(_) => {
                            let let_index_name = match is_multi_column_index {
                                true => quote! {
                                    let #index_name = (#(#row_value_getters),*);
                                },
                                false => quote! {
                                    let #index_name = #(#row_value_getters),*;
                                },
                            };

                            let impl_until_return_ok_on_is_empty = quote! {
                                use itertools::Itertools;

                                #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                #let_index_name

                                let primary_key_column_values_of_referencing_table = #method_impl_prefix
                                    .filter(#index_name)
                                    .map(|row| row.id)
                                    .collect();

                                if primary_key_column_values_of_referencing_table.is_empty() {
                                    return Ok(spacetimedsl::DeletionResult {
                                        table_name: #singular_table_name_as_string.into(),
                                        one_or_multiple: #multiple,
                                        entries: vec![],
                                    });
                                }
                            };

                            let map_primary_key_column_values_of_referencing_table_to_deletion_result_entries = quote! {
                                let mut entries = std::collections::HashMap::new();

                                for primary_key_column_value_of_referencing_table in &primary_key_column_values_of_referencing_table {
                                    entries.insert(
                                        primary_key_column_value_of_referencing_table,
                                        spacetimedsl::DeletionResultEntry {
                                            table_name: #singular_table_name_as_string.into(),
                                            column_name: "id".into(),
                                            strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                            row_value: format!("{primary_key_column_value_of_referencing_table}").into(),
                                            child_entries: vec![],
                                        }
                                    );
                                }
                            };

                            let delete_many_and_return_result_impl = quote! {
                                let count_of_rows_to_delete = primary_key_column_values_of_referencing_table.len();
                                let count_of_deleted_rows = #method_impl_prefix.delete(#index_name);

                                if count_of_rows_to_delete.ne(&count_of_deleted_rows) {
                                    return Err(
                                        spacetimedsl::SpacetimeDSLError::Error(
                                            format!(
                                                "Delete Many Error: `count_of_rows_to_delete ( {} ) != ( {} ) count_of_deleted_rows`!",
                                                &count_of_rows_to_delete,
                                                &count_of_deleted_rows
                                            )
                                        )
                                    );
                                }

                                return Ok(spacetimedsl::DeletionResult {
                                    table_name: #singular_table_name_as_string.into(),
                                    one_or_multiple: spacetimedsl::OneOrMultiple::Multiple,
                                    entries: entries.into_values().collect_vec(),
                                });
                            };

                            if spacetimedsl_table.referencing_tables.is_empty() {
                                method_impl = quote! {
                                    #impl_until_return_ok_on_is_empty

                                    #map_primary_key_column_values_of_referencing_table_to_deletion_result_entries

                                    #delete_many_and_return_result_impl
                                };
                            } else {
                                let referenced_table_function_name = get_referenced_table_function_name(
                                    &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
                                    &singular_table_name
                                );

                                method_impl = quote! {
                                    #impl_until_return_ok_on_is_empty

                                    #map_primary_key_column_values_of_referencing_table_to_deletion_result_entries

                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, entries) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::Multiple,
                                                entries: e.into_values().collect_vec(),
                                            });
                                        },
                                        Ok(e) => entries = e,
                                    };

                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, entries) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::Multiple,
                                                entries: e.into_values().collect_vec(),
                                            });
                                        },
                                        Ok(e) => entries = e,
                                    };

                                    /* // FIXME: If SetNone is implemented
                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, entry) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::Multiple,
                                                entries: e.into_values().collect_vec(),
                                            });
                                        },
                                        Ok(e) => entries = e,
                                    };
                                    */

                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, entries) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::Multiple,
                                                entries: e.into_values().collect_vec(),
                                            });
                                        },
                                        Ok(e) => entries = e,
                                    };

                                    #delete_many_and_return_result_impl
                                };
                            }
                        }
                        DSLMethod::GetOneOption(_) => match is_multi_column_index {
                            true => {
                                let multi_column_index_check = get_unique_multi_column_index_check(
                                    &Action::Get,
                                    &singular_table_name,
                                    &index_name,
                                    &column_names_and_row_values,
                                    &row_value_getters,
                                );

                                method_impl = quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    use spacetimedsl::itertools::Itertools;

                                    let mut #field_name_for_found_value: Option<#struct_name> = None;

                                    #multi_column_index_check

                                    match #field_name_for_found_value {
                                        Some(#singular_table_name) => Ok(#singular_table_name),
                                        None => {
                                            return Err(
                                                spacetimedsl::SpacetimeDSLError::NotFoundError {
                                                    table_name: #singular_table_name_as_string,
                                                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*)
                                                }
                                            );
                                        }
                                    };
                                };
                            }
                            false => {
                                method_impl = quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    match #method_impl_prefix.find(#(#row_value_getters),*) {
                                        Some(#singular_table_name) => Ok(#singular_table_name),
                                        None => return Err(
                                            spacetimedsl::SpacetimeDSLError::NotFoundError {
                                                table_name: #singular_table_name_as_string,
                                                column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*)
                                            }
                                        )
                                    };
                                }
                            }
                        },
                        DSLMethod::DeleteOne(_) => {
                            let get_primary_key_column_value_of_referencing_table =
                                match is_multi_column_index {
                                    true => {
                                        let multi_column_index_check =
                                            get_unique_multi_column_index_check(
                                                &Action::Delete,
                                                &singular_table_name,
                                                &index_name,
                                                &column_names_and_row_values,
                                                &row_value_getters,
                                            );

                                        quote! {
                                            use spacetimedsl::itertools::Itertools;

                                            let #index_name = (#(#row_value_getters),*);

                                            let mut #field_name_for_found_value: Option<#struct_name> = None;

                                            #multi_column_index_check

                                            let primary_key_column_value_of_referencing_table = #field_name_for_found_value;
                                        }
                                    }
                                    false => quote! {
                                        let #index_name = #(#row_value_getters),*;

                                        let primary_key_column_value_of_referencing_table = #method_impl_prefix.find(#index_name);
                                    },
                                };

                            let impl_until_return_err_on_is_none = quote! {
                                use itertools::Itertools;

                                #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                #get_primary_key_column_value_of_referencing_table

                                let primary_key_column_value_of_referencing_table = match primary_key_column_value_of_referencing_table {
                                    None => return Err(spacetimedsl::DeletionResult {
                                        table_name: #singular_table_name_as_string.into(),
                                        one_or_multiple: spacetimedsl::OneOrMultiple::One,
                                        entries: vec![],
                                    }),
                                    Some(primary_key_column_value_of_referencing_table) => primary_key_column_value_of_referencing_table,
                                };
                            };

                            let map_primary_key_column_value_of_referencing_table_to_deletion_result_entry = quote! {
                                let mut entry = (&primary_key_column_value_of_referencing_table, spacetimedsl::DeletionResultEntry {
                                    table_name: #singular_table_name_as_string.into(),
                                    column_name: "id".into(),
                                    strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                    row_value: format!("{primary_key_column_value_of_referencing_table}").into(),
                                    child_entries: vec![],
                                });
                            };

                            let delete_one_and_return_result_impl = quote! {
                                match self
                                        .ctx()
                                        .db()
                                        .#singular_table_name()
                                        .delete(primary_key_column_value_of_referencing_table) {
                                        .id()
                                    false => {
                                        return Err(
                                            spacetimedsl::SpacetimeDSLError::Error(
                                                "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string(),
                                            )
                                        );
                                    },
                                    true => {
                                        return Ok(spacetimedsl::DeletionResult {
                                            table_name: #singular_table_name_as_string.into(),
                                            entries = vec![entry],
                                            one_or_multiple: #one,
                                        });
                                    },
                                };
                            };

                            if spacetimedsl_table.referencing_tables.is_empty() {
                                method_impl = quote! {
                                    #impl_until_return_err_on_is_none

                                    #map_primary_key_column_value_of_referencing_table_to_deletion_result_entry

                                    #delete_one_and_return_result_impl
                                };
                            } else {
                                let referenced_table_function_name = get_referenced_table_function_name(
                                    &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
                                    &singular_table_name
                                );

                                method_impl = quote! {
                                    #impl_until_return_err_on_is_none

                                    #map_primary_key_column_value_of_referencing_table_to_deletion_result_entry

                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, entry) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::One,
                                                entries: vec![entry],
                                            });
                                        },
                                        Ok(e) => entry = e,
                                    };

                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, entry) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::One,
                                                entries: vec![entry],
                                            });
                                        },
                                        Ok(e) => entry = e,
                                    };

                                    /* // FIXME: If SetNone is implemented
                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, entry) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::One,
                                                entries: vec![entry],
                                            });
                                        },
                                        Ok(e) => entry = e,
                                    };
                                    */

                                    match spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, entry) {
                                        Err(e) => {
                                            return Err(spacetimedsl::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: spacetimedsl::OneOrMultiple::One,
                                                entries: vec![entry],
                                            });
                                        },
                                        Ok(e) => entry = e,
                                    };

                                    #delete_one_and_return_result_impl
                                };
                            }
                        }
                        DSLMethod::Create
                        | DSLMethod::GetAll
                        | DSLMethod::GetCount
                        | DSLMethod::Update(_) => panic!(
                            "DSLColumnMethod Create / GetAll / GetCount / Update should already be processed!"
                        ),
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

fn reference_integrity_checks_on_create_or_update(
    create_or_update_dsl_method: CreateOrUpdate,
    spacetimedb_table: &SpacetimeDBTable,
    columns: &Vec<InternalColumn>,
    mut paths_of_traits_to_extend: Vec<Path>,
    column_names_and_row_value_getters: (&String, Option<&Vec<TokenStream>>),
) -> (Vec<Path>, Vec<TokenStream>) {
    let mut reference_integrity_checks = vec![];

    let column_names_and_row_values = column_names_and_row_value_getters.0;
    let row_value_getters = column_names_and_row_value_getters.1;

    for column in columns {
        // Checks of private columns only need to happen in checks for create methods, because they can't be changed, they don't need to be checked during updates
        if create_or_update_dsl_method.eq(&CreateOrUpdate::Update)
            && column
                .rust_field_visibility
                .to_string()
                .eq(&crate::api::rust::visibility::RustVisibility::Private.to_string())
        {
            continue;
        }

        let foreign_key;

        match &column.spacetimedsl_column_foreign_key {
            Some(fk) => foreign_key = fk,
            None => continue,
        };

        let referenced_table_name = &foreign_key.table_name;
        let referenced_table_name_pascal_case = format_ident!(
            "{}",
            RenameRule::PascalCase.apply_to_field(referenced_table_name.to_string())
        );
        let referenced_table_primary_key_column_name_pascal_case = format_ident!("Id");
        let get_row_of_referenced_table_by_primary_key_trait_name = format_ident!(
            "Get{referenced_table_name_pascal_case}RowOptionBy{referenced_table_primary_key_column_name_pascal_case}"
        );
        let get_row_of_referenced_table_by_primary_key_method_name =
            format_ident!("get_{referenced_table_name}_by_id");

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_getter_name =
            format_ident!("get_{referencing_table_column_name}");

        let referencing_table_column_type = column
            .rust_field_type_name_or_path
            .to_token_stream()
            .to_string();

        paths_of_traits_to_extend.push(
            parse_str(&format!(
                "{}::{get_row_of_referenced_table_by_primary_key_trait_name}",
                &foreign_key.path.to_token_stream().to_string()
            ))
            .expect("should be parseable"),
        );

        let field_name_for_found_value =
            format_ident!("the_same_or_another_{referencing_table_name}");

        let check = match &create_or_update_dsl_method {
            CreateOrUpdate::Create => {
                quote! {
                    match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                        Some(_) => {},
                        None => {
                            return Err(
                                spacetimedsl::SpacetimeDSLError::ReferenceIntegrityViolation(
                                    spacetimedsl::ReferenceIntegrityViolationError::OnCreateOrUpdate {
                                        table_name: #referencing_table_name_as_string,
                                        create_or_update: spacetimedsl::Action::Create,
                                        column_names_and_row_values: format!(#column_names_and_row_values, #referencing_table_name.#referencing_table_column_getter_name())
                                    }
                                )
                            );
                        }
                    };
                }
            }
            CreateOrUpdate::Update => {
                let row_value_getters =
                    row_value_getters.expect("Update Method should have row value getters");
                quote! {
                    if #field_name_for_found_value.is_none() {
                        #field_name_for_found_value = match self.ctx().db().#referencing_table_name().id().find(#referencing_table_name.get_id().value()) {
                            Some(#referencing_table_name) => Some(#referencing_table_name),
                            None => {
                                return Err(
                                    spacetimedsl::SpacetimeDSLError::NotFoundError {
                                        table_name: #referencing_table_name_as_string,
                                        column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*)
                                    }
                                );
                            }
                        };
                    }
                    if #field_name_for_found_value.as_ref().unwrap().#referencing_table_column_getter_name().ne(&#referencing_table_name.#referencing_table_column_getter_name()) {
                        match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                            Some(_) => {},
                            None => return Err(
                                spacetimedsl::SpacetimeDSLError::ReferenceIntegrityViolation(
                                    spacetimedsl::ReferenceIntegrityViolationError::OnCreateOrUpdate {
                                        table_name: #referencing_table_name_as_string,
                                        create_or_update: spacetimedsl::Action::Update,
                                        column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*)
                                    }
                                )
                            )
                        };
                    }
                }
            }
        };

        reference_integrity_checks.push(match referencing_table_column_type.trim() {
            "u8" | "u16" | "u32" | "u64" | "u128" => quote! {
                if #referencing_table_column_name.ne(&0) {
                    #check
                }
            },
            "Option" => quote! {
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

fn multi_column_index_checks(
    action: Action,
    struct_name: &Ident,
    singular_table_name: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
    column_names_and_row_values: &String,
) -> Vec<TokenStream> {
    let mut multi_column_index_checks = vec![];
    let singular_table_name_as_string = singular_table_name.to_string();

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

        let index_name = &multi_column_index.name;

        let mut row_value_getters = vec![];

        // TODO: Is it possible to use the row_value_getters multiple times or do they move the values?
        for column_name in index_column_names {
            let column_name = format_ident!("{column_name}");
            row_value_getters.push(quote! {#singular_table_name.#column_name});
        }

        let mut multi_column_index_check = get_unique_multi_column_index_check(
            &action,
            &singular_table_name,
            &index_name,
            &column_names_and_row_values,
            &row_value_getters,
        );

        let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

        let action_as_ident = format_ident!("{action}");

        let multiple = OneOrMultiple::Multiple;

        let return_unique_constraint_violation_error = quote! {
            return Err(
                spacetimedsl::SpacetimeDSLError::UniqueConstraintViolation {
                    table_name: #singular_table_name_as_string.into(),
                    action: spacetimedsl::Action::#action_as_ident,
                    error_from: spacetimedsl::ErrorFrom::SpacetimeDSL,
                    one_or_multiple: #multiple,
                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*)
                }
            );
        };

        let on_some = match action {
            Action::Create | Action::Get | Action::Delete => {
                return_unique_constraint_violation_error
            }
            Action::Update => {
                quote! {
                    if #field_name_for_found_value.id.ne(&#singular_table_name.id) {
                        #return_unique_constraint_violation_error
                    }
                }
            }
        };

        multi_column_index_check.append_all(quote! {
            match &#field_name_for_found_value {
                Some(#field_name_for_found_value) => {
                    #on_some
                },
                _ => {},
            };
        });

        multi_column_index_checks.push(multi_column_index_check);
    }

    multi_column_index_checks
}

pub(in crate::internal::dsl::method) fn get_unique_multi_column_index_check(
    action: &Action,
    singular_table_name: &Ident,
    index_name: &Ident,
    column_names_and_row_values: &String,
    row_value_getters: &Vec<TokenStream>,
) -> TokenStream {
    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let singular_table_name_as_string = singular_table_name.to_string();

    let action = format_ident!("{action}");

    let multiple = OneOrMultiple::Multiple;

    quote! {
        #field_name_for_found_value = match self.ctx().db().#singular_table_name().#index_name().filter((#(#row_value_getters),*)).at_most_one() {
            Ok(#singular_table_name) => #singular_table_name,
            Err(_) => return Err(
                spacetimedsl::SpacetimeDSLError::UniqueConstraintViolation {
                    table_name: #singular_table_name_as_string.into(),
                    action: spacetimedsl::Action::#action,
                    error_from: spacetimedsl::ErrorFrom::SpacetimeDSL,
                    one_or_multiple: #multiple,
                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*)
                }
            ),
        };
    }
}

fn for_referenced_by(
    one_or_multiple: &OneOrMultiple,
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
        .find(|c| c.rust_field.name.to_string().eq(&"id"))
        .expect("should have a primary key");

    let primary_key_column_type = &primary_key_column.rust_field.type_name_or_path;

    let doc_comment;
    let trait_name =
        get_referenced_table_trait_name(&one_or_multiple, &singular_table_name_pascal_case);
    let function_name = get_referenced_table_function_name(&one_or_multiple, &singular_table_name);

    let paths_of_traits_to_extend = vec![];
    let mut function_args = vec![
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("ctx"),
            arg_type: quote! { &spacetimedb::ReducerContext },
        },
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("strategy"),
            arg_type: quote! { spacetimedsl::OnDeleteStrategy },
        },
    ];

    let return_type;

    let entry_or_entries;

            doc_comment = format!("Execute On Delete Strategies of all referencing tables after one row of the referenced table `{singular_table_name}` was deleted.");
            entry_or_entries = format_ident!("entry");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: true,
                    arg_name: entry_or_entries.clone(),
                    arg_type: quote! { (&#primary_key_column_type, spacetimedsl::DeletionResultEntry>) }
                },
    match one_or_multiple {
        OneOrMultiple::One => {
            );
            return_type = quote! {
                Result<
                    (&#primary_key_column_type, spacetimedsl::DeletionResultEntry>),
                    (&#primary_key_column_type, spacetimedsl::DeletionResultEntry>)
                >
            };
        }
            doc_comment = format!("Execute On Delete Strategies of all referencing tables after multiple rows of the referenced table `{singular_table_name}` were deleted.");
            entry_or_entries = format_ident!("entries");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: true,
                    arg_name: entry_or_entries.clone(),
                    arg_type: quote! {
                        std::collections::HashMap<&#primary_key_column_type, spacetimedsl::DeletionResultEntry>
                    }
                },
        OneOrMultiple::Multiple => {
            );
            return_type = quote! {
                Result<
                    std::collections::HashMap<&#primary_key_column_type, spacetimedsl::DeletionResultEntry>,
                    std::collections::HashMap<&#primary_key_column_type, spacetimedsl::DeletionResultEntry>
                >
            };
        }
    };

    let doc_comment = doc_comment.into();

    let function_impl;

    let mut strategy_calls = vec![];

    for referencing_table in &spacetimedsl_table.referencing_tables {
        let referencing_table_name = &referencing_table.table_name;

        let referencing_table_name_pascal_case = format_ident!(
            "{}",
            RenameRule::PascalCase.apply_to_field(referencing_table_name.to_string())
        );

        let referencing_table_path = &referencing_table.path;

        let referencing_table_trait_name = get_referencing_table_trait_name(
            &one_or_multiple,
            &referencing_table_name_pascal_case,
            &singular_table_name_pascal_case,
        );

        let referencing_table_function_name = get_referencing_table_function_name(
            &one_or_multiple,
            &referencing_table_name,
            &singular_table_name,
        );


        strategy_calls.push(quote! {
            use #referencing_table_path::#referencing_table_trait_name;
            match spacetimedsl::internal::DSLInternals::#referencing_table_function_name(ctx, &strategy, #entry_or_entries) {
                Err(e) => {
                    return Err(#entry_or_entries);
                },
                Ok(e) => #entry_or_entries = e,
            };
        });
    }

    function_impl = quote! {
        #(#strategy_calls)*

        Ok(#entry_or_entries)
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
    rust_struct: &RustStruct,
    one_or_multiple: &OneOrMultiple,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &Vec<Column>,
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &Vec<&&Column>,
) -> SpacetimeDSLMethod {
    let primary_key_column_type = &columns
        .iter()
        .find(|c| c.rust_field.name.to_string().eq(&"id"))
        .expect("should have a primary key")
        .rust_field
        .type_name_or_path;

    let first_foreign_key_column = columns_with_foreign_key
        .first()
        .expect("there should be a column with foreign key");

    let referenced_table_primary_key_column_type =
        &first_foreign_key_column.rust_field.type_name_or_path;

    let mut columns_by_on_delete_strategies = HashMap::new();

    for column_with_foreign_key in columns_with_foreign_key {
        if column_with_foreign_key
            .rust_field
            .type_name_or_path
            .to_token_stream()
            .to_string()
            .ne(&referenced_table_primary_key_column_type
                .to_token_stream()
                .to_string())
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

        if !columns_by_on_delete_strategies.contains_key(on_delete_strategy) {
            columns_by_on_delete_strategies.insert(on_delete_strategy, vec![]);
        }

        columns_by_on_delete_strategies
            .get_mut(on_delete_strategy)
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
        &one_or_multiple,
        &singular_table_name_pascal_case,
        &referenced_table_name_pascal_case,
    );

    let function_name = get_referencing_table_function_name(
        &one_or_multiple,
        &singular_table_name,
        &referenced_table_name,
    );

    let paths_of_traits_to_extend = vec![];
    let mut function_args = vec![
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("ctx"),
            arg_type: quote! { &spacetimedb::ReducerContext },
        },
        SpacetimeDSLMethodArg {
            is_mut: false,
            arg_name: format_ident!("strategy"),
            arg_type: quote! { &spacetimedsl::OnDeleteStrategy },
        },
    ];

    let return_type;

    let entry_or_entries;

            doc_comment = format!("Execute On Delete Strategies of the referencing table `{singular_table_name}` after one row of the referenced table `{referenced_table_name}` was deleted.");
            entry_or_entries = format_ident!("entry");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: true,
                    arg_name: entry_or_entries.clone(),
                    arg_type: quote! { (&#primary_key_column_type, spacetimedsl::DeletionResultEntry>) }
                },
    match one_or_multiple {
        OneOrMultiple::One => {
            );
            return_type = quote! {
                Result<
                    (&#primary_key_column_type, spacetimedsl::DeletionResultEntry>),
                    (&#primary_key_column_type, spacetimedsl::DeletionResultEntry>)
                >
            };
        }
            doc_comment = format!("Execute On Delete Strategies of the referencing table `{singular_table_name}` after multiple rows of the referenced table `{referenced_table_name}` were deleted.");
            entry_or_entries = format_ident!("entries");
            function_args.push(
                SpacetimeDSLMethodArg {
                    is_mut: true,
                    arg_name: entry_or_entries.clone(),
                    arg_type: quote! {
                        std::collections::HashMap<&#primary_key_column_type, spacetimedsl::DeletionResultEntry>
                    }
                },
        OneOrMultiple::Multiple => {
            );
            return_type = quote! {
                Result<
                    std::collections::HashMap<&#primary_key_column_type, spacetimedsl::DeletionResultEntry>,
                    std::collections::HashMap<&#primary_key_column_type, spacetimedsl::DeletionResultEntry>
                >
            };
        }
    };

    let doc_comment = doc_comment.into();
    let function_impl;

    let mut on_delete_strategy_match_arms = HashMap::new();

    for on_delete_strategy in OnDeleteStrategy::iter() {
        on_delete_strategy_match_arms.insert(
            on_delete_strategy.clone(),
            quote! {
                #on_delete_strategy => { }
            },
        );
    }

    for (on_delete_strategy, columns_by_on_delete_strategy) in columns_by_on_delete_strategies {
        on_delete_strategy_match_arms.insert(
            on_delete_strategy.clone(),
            get_on_delete_strategy_implementation(
                struct_name,
                singular_table_name,
                primary_key_column_type,
                spacetimedsl_table,
                on_delete_strategy,
                columns_by_on_delete_strategy,
                &one_or_multiple,
                &entry_or_entries,
            ),
        );
    }

    let on_delete_strategy_match_arms = on_delete_strategy_match_arms.values().collect_vec();

    function_impl = quote! {
        match &strategy {
            #(#on_delete_strategy_match_arms),*
        };

        Ok(#entry_or_entries)
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
    primary_key_column_type: &Path,
    spacetimedsl_table: &SpacetimeDSLTable,
    on_delete_strategy: &OnDeleteStrategy,
    columns_by_on_delete_strategy: Vec<&&&Column>,
    one_or_multiple: &OneOrMultiple,
) -> TokenStream {
    let mut strategy_before_all_columns = TokenStream::default();
    let mut strategies = vec![];
    let mut strategy_after_all_columns = TokenStream::default();

    let singular_table_name_as_string = singular_table_name.to_string();

    for column in &columns_by_on_delete_strategy {
        let column_name = &column.rust_field.name;
        let column_name_as_string = column_name.to_string();

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
                    #spacetimedb_call_prefix.#column_name().find(referenced_row_primary_key_value)
                }
            }
            false => {
                quote! {
                    #spacetimedb_call_prefix.#column_name().filter(referenced_row_primary_key_value)
                }
            }
        };

        let referenced_table_one_function_name = get_referenced_table_function_name(
            &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThisTableWasDeleted,
            &singular_table_name
        );

        let referenced_table_multiple_function_name = get_referenced_table_function_name(
            &DSLInternalReferencedByFunction::ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThisTableWereDeleted,
            &singular_table_name
        );


        let get_child_entries = match on_delete_strategy {
            OnDeleteStrategy::Error | OnDeleteStrategy::SetZero | OnDeleteStrategy::Ignore => {
                quote! {
                    let child_entries = vec![];
                }
            }
            OnDeleteStrategy::Delete => quote! {
                // FIXME: I need better recursive functions for the result creation...

                let mut child_entries = vec![];

                 match spacetimedsl::internal::DSLInternals::#referenced_table_one_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Error, #entry_or_entries) {
                    Err(e) => {
                        return Err(spacetimedsl::DeletionResult {
                            table_name: #singular_table_name_as_string.into(),
                            one_or_multiple: spacetimedsl::OneOrMultiple::#one_or_multiple_as_ident,
                            entries: e.into_values().collect_vec(),
                        });
                    },
                    Ok(e) => entries = e,
                };

                match spacetimedsl::internal::DSLInternals::#referenced_table_one_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::Delete, #entry_or_entries) {
                    Err(e) => {
                        return Err(spacetimedsl::DeletionResult {
                            table_name: #singular_table_name_as_string.into(),
                            one_or_multiple: spacetimedsl::OneOrMultiple::#one_or_multiple_as_ident,
                            entries: e.into_values().collect_vec(),
                        });
                    },
                    Ok(e) => entries = e,
                };

                /* // FIXME: If SetNone is implemented
                match spacetimedsl::internal::DSLInternals::#referenced_table_one_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetNone, #entry_or_entries) {
                    Err(e) => {
                        return Err(spacetimedsl::DeletionResult {
                            table_name: #singular_table_name_as_string.into(),
                            one_or_multiple: spacetimedsl::OneOrMultiple::#one_or_multiple_as_ident,
                            entries: e.into_values().collect_vec(),
                        });
                    },
                    Ok(e) => entries = e,
                };
                */

                match spacetimedsl::internal::DSLInternals::#referenced_table_one_function_name(self.ctx(), spacetimedsl::OnDeleteStrategy::SetZero, #entry_or_entries) {
                    Err(e) => {
                        return Err(spacetimedsl::DeletionResult {
                            table_name: #singular_table_name_as_string.into(),
                            one_or_multiple: spacetimedsl::OneOrMultiple::#one_or_multiple_as_ident,
                            entries: e.into_values().collect_vec(),
                        });
                    },
                    Ok(e) => entries = e,
                };
            },
        };

        let create_child_entry = quote! {
            spacetimedsl::DeletionResultEntry {
                table_name: #singular_table_name_as_string.into(),
                column_name: #column_name_as_string.into(),
                strategy: #on_delete_strategy,
                row_value: format!("{}", id).into(),
                child_entries,
            }
        };

        let add_row_as_child_entry_to_referenced_row;

        match one_or_multiple {
            OneOrMultiple::One => {
                add_row_as_child_entry_to_referenced_row = quote! {
                    #get_child_entries

                    #entry_or_entries.1.child_entries.push(
                        #create_child_entry
                    );
                };
            }
            OneOrMultiple::Multiple => {
                add_row_as_child_entry_to_referenced_row = quote! {
                    #get_child_entries

                    #entry_or_entries.get_mut(&referenced_row_primary_key_value).unwrap().child_entries.push(
                        #create_child_entry
                    );
                };
            }
        };

        let strategy;

        match on_delete_strategy {
            OnDeleteStrategy::Error => {
                strategy_before_all_columns = quote! {
                    let mut is_err = false;
                };

                let strategy_per_row = quote! {
                    is_err = true;

                    #add_row_as_child_entry_to_referenced_row
                };

                strategy_after_all_columns = quote! {
                    match is_err {
                        false => return Ok(#entry_or_entries),
                        true => return Err(#entry_or_entries),
                    }
                };

                strategy = match is_unique_index {
                    true => quote! {
                        let row = #row_finder;

                        if row.is_some() {
                            #strategy_per_row
                        };
                    },
                    false => quote! {
                        let rows = #row_finder;

                        for row in &rows {
                            #strategy_per_row
                        }
                    },
                };
            }
            OnDeleteStrategy::Delete => {
                strategy_before_all_columns = quote! {
                    let mut primary_key_referencing_rows = std::collections::HashSet::new();
                };

                strategy_after_all_columns = quote! {
                    for primary_key_referencing_row in primary_key_referencing_rows {
                        #add_row_as_child_entry_to_referenced_row

                        // FIXME: On false Error
                        #spacetimedb_call_prefix
                            .#primary_key_column_name()
                            .delete(primary_key_referencing_row.#primary_key_column_name);
                    }
                };

                strategy = match is_unique_index {
                    true => quote! {
                        match #row_finder {
                            None => {},
                            Some(row) => primary_key_referencing_rows.insert(row),
                        };
                    },
                    false => quote! {
                        for row in #row_finder {
                            primary_key_referencing_rows.insert(row);
                        }
                    },
                };
            }
            OnDeleteStrategy::SetZero => {
                let strategy_per_row = quote! {
                    row.#column_name = 0;

                    #add_row_as_child_entry_to_referenced_row

                    // FIXME: try_update instead of update
                    // FIXME: on error return Err(spacetimedsl::SpacetimeDSLError);
                    #spacetimedb_call_prefix.#primary_key_column_name().update(row);
                };

                strategy = match is_unique_index {
                    true => quote! {
                        match #row_finder {
                            None => {}
                            Some(mut row) => { #strategy_per_row }
                        };
                    },
                    false => quote! {
                        for mut row in #row_finder {
                            #strategy_per_row
                        }
                    },
                };
            }
            OnDeleteStrategy::Ignore => {
                let strategy_per_row = quote! {
                    #add_row_as_child_entry_to_referenced_row
                };

                strategy = match is_unique_index {
                    true => quote! {
                        match #row_finder {
                            None => {}
                            Some(row) => { #strategy_per_row }
                        };
                    },
                    false => quote! {
                        for row in #row_finder {
                            #strategy_per_row
                        }
                    },
                };
            }
        };

        strategies.push(strategy);
    }

    let strategies = match one_or_multiple {
        OneOrMultiple::One => quote! {
            let referenced_row_primary_key_value = entry.0;

            #(#strategies)*
        },
        OneOrMultiple::Multiple => quote! {
            for referenced_row_primary_key_value in entries.keys() {
                #(#strategies)*
            }
        },
    };

    quote! {
        #on_delete_strategy => {
            #strategy_before_all_columns

            #strategies

            #strategy_after_all_columns
        }
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
    one_or_multiple: &OneOrMultiple,
    referenced_table_name_pascal_case: &Ident,
) -> Ident {
    match one_or_multiple {
        OneOrMultiple::One => {
            format_ident!(
                "ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThe{referenced_table_name_pascal_case}TableWasDeleted"
            )
        }
        OneOrMultiple::Multiple => {
            format_ident!(
                "ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThe{referenced_table_name_pascal_case}TableWereDeleted"
            )
        }
    }
}

fn get_referenced_table_function_name(
    one_or_multiple: &OneOrMultiple,
    referenced_table_name: &Ident,
) -> Ident {
    match one_or_multiple {
        OneOrMultiple::One => {
            format_ident!(
                "execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_{referenced_table_name}_table_was_deleted"
            )
        }
        OneOrMultiple::Multiple => {
            format_ident!(
                "execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_{referenced_table_name}_table_were_deleted"
            )
        }
    }
}

fn get_referencing_table_trait_name(
    one_or_multiple: &OneOrMultiple,
    referencing_table_name_pascal_case: &Ident,
    referenced_table_name_pascal_case: &Ident,
) -> Ident {
    match one_or_multiple {
        OneOrMultiple::One => {
            format_ident!(
                "ExecuteOnDeleteStrategiesOfThe{referencing_table_name_pascal_case}TableAfterOneRowOfThe{referenced_table_name_pascal_case}TableWasDeleted"
            )
        }
        OneOrMultiple::Multiple => {
            format_ident!(
                "ExecuteOnDeleteStrategiesOfThe{referencing_table_name_pascal_case}TableAfterMultipleRowsOfThe{referenced_table_name_pascal_case}TableWereDeleted"
            )
        }
    }
}

fn get_referencing_table_function_name(
    one_or_multiple: &OneOrMultiple,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    match one_or_multiple {
        OneOrMultiple::One => {
            format_ident!(
                "execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_one_row_of_the_{referenced_table_name}_table_was_deleted"
            )
        }
        OneOrMultiple::Multiple => {
            format_ident!(
                "execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_multiple_rows_of_the_{referenced_table_name}_table_were_deleted"
            )
        }
    }
}
