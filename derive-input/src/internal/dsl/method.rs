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
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            table::{CreateDSLMethodArg, SpacetimeDSLTable, SpacetimeDSLTableMethods},
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
    GetOne(&'a Index),
    Update(&'a Index),
    DeleteOne(&'a Index),
}

#[derive(Debug)]
pub enum OneOrMultiple {
    One,
    Multiple,
}

impl quote::ToTokens for OneOrMultiple {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            OneOrMultiple::One => quote! { crate::spacetimedsl::error::OneOrMultiple::One },
            OneOrMultiple::Multiple => quote! { crate::spacetimedsl::error::OneOrMultiple::Multiple },
        };
        tokens.extend(variant);
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

impl SpacetimeDSLColumnMethods {
    pub(in crate::internal) fn map(
        rust_struct: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
        spacetimedsl_table: &mut SpacetimeDSLTable,
        spacetimedb_column: &SpacetimeDBColumn,
        internal_columns: &Vec<InternalColumn>,
        primary_key_column: &InternalColumn,
    ) -> Option<SpacetimeDSLColumnMethods> {
        let index = match &spacetimedb_column.single_column_index {
            None => {
                return None;
            }
            Some(index) => index,
        };

        let methods = match index.is_unique {
            false => {
                if spacetimedsl_table.is_singleton {
                    return None;
                }

                let get_many = for_method(
                    DSLMethod::GetMany(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
                    primary_key_column,
                );

                let delete_many = for_method(
                    DSLMethod::DeleteMany(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
                    primary_key_column,
                );

                SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
                    get_many,
                    delete_many,
                })
            }
            true => {
                let get_one_option = for_method(
                    DSLMethod::GetOne(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
                    primary_key_column,
                );

                let method_is_for_primary_key = match &index.index_type {
                    IndexType::BTreeSingleColumn { column }
                    | IndexType::HashSingleColumn { column }
                    | IndexType::Direct { column } => column
                        .to_string()
                        .eq(&primary_key_column.rust_field_name.to_string()),
                    _ => panic!("When this code is called, it should be a single column index!"),
                };

                let update = match spacetimedsl_table.has_update_method && method_is_for_primary_key
                {
                    false => None,
                    true => Some(for_method(
                        DSLMethod::Update(index),
                        rust_struct,
                        spacetimedb_table,
                        spacetimedsl_table,
                        internal_columns,
                        primary_key_column,
                    )),
                };

                let delete_one = for_method(
                    DSLMethod::DeleteOne(index),
                    rust_struct,
                    spacetimedb_table,
                    spacetimedsl_table,
                    internal_columns,
                    primary_key_column,
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
        mut spacetimedsl_table: SpacetimeDSLTable,
        columns: &[Column],
        internal_columns: &Vec<InternalColumn>,
        primary_key_column: &InternalColumn,
    ) -> syn::Result<(SpacetimeDSLTableMethods, SpacetimeDSLTable)> {
        let is_singleton = spacetimedsl_table.is_singleton;

        let create = for_method(
            DSLMethod::Create,
            rust_struct,
            spacetimedb_table,
            &mut spacetimedsl_table,
            internal_columns,
            primary_key_column,
        );

        let get_all = if is_singleton {
            None
        } else {
            Some(for_method(
                DSLMethod::GetAll,
                rust_struct,
                spacetimedb_table,
                &mut spacetimedsl_table,
                internal_columns,
                primary_key_column,
            ))
        };

        let get_count = if is_singleton {
            None
        } else {
            Some(for_method(
                DSLMethod::GetCount,
                rust_struct,
                spacetimedb_table,
                &mut spacetimedsl_table,
                internal_columns,
                primary_key_column,
            ))
        };

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
                &mut spacetimedsl_table,
                primary_key_column,
            ));

            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted = Some(for_referenced_by(
                &OneOrMultiple::Multiple,
                spacetimedb_table,
                &mut spacetimedsl_table,
                primary_key_column,
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
                            &OneOrMultiple::One,
                            !spacetimedsl_table.referencing_tables.is_empty(),
                            spacetimedb_table,
                            referenced_table_name,
                            &columns_with_foreign_key,
                            primary_key_column,
                            &mut spacetimedsl_table,
                        )
                    );
                    execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted.push(
                        for_foreign_key(
                            &OneOrMultiple::Multiple,
                            !spacetimedsl_table.referencing_tables.is_empty(),
                            spacetimedb_table,
                            referenced_table_name,
                            &columns_with_foreign_key,
                            primary_key_column,
                            &mut spacetimedsl_table,
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
                        &mut spacetimedsl_table,
                        internal_columns,
                        primary_key_column,
                    );
                    let delete_many = for_method(
                        DSLMethod::DeleteMany(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        &mut spacetimedsl_table,
                        internal_columns,
                        primary_key_column,
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
                        DSLMethod::GetOne(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        &mut spacetimedsl_table,
                        internal_columns,
                        primary_key_column,
                    );

                    let update = match spacetimedsl_table.has_update_method {
                        false => None,
                        true => Some(for_method(
                            DSLMethod::Update(multi_column_index),
                            rust_struct,
                            spacetimedb_table,
                            &mut spacetimedsl_table,
                            internal_columns,
                            primary_key_column,
                        )),
                    };

                    let delete_one = for_method(
                        DSLMethod::DeleteOne(multi_column_index),
                        rust_struct,
                        spacetimedb_table,
                        &mut spacetimedsl_table,
                        internal_columns,
                        primary_key_column,
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

        Ok((methods, spacetimedsl_table))
    }
}

fn process_columns_for_create_and_update_method(
    spacetimedsl_table: &SpacetimeDSLTable,
    create_or_update: CreateOrUpdate,
    internal_column: &InternalColumn,
) -> (
    Option<SpacetimeDSLArg>,
    Option<TokenStream>,
    Option<TokenStream>,
    TokenStream,
) {
    let mut arg = None;
    let mut wrapper_type_option_to_wrapped_type_option_mapper = None;
    let mut constructor_arg = None;

    let singular_table_name = &internal_column.spacetimedb_table_singular_name;
    let column_name = &internal_column.rust_field_name;
    let getter_name = format_ident!("get_{column_name}");
    let constructor_arg_name = quote! { #column_name };

    let column_type = &internal_column.rust_field_type_name_or_path;

    match create_or_update {
        CreateOrUpdate::Create => {
            // Singleton PK column (id: u8) is auto-filled with 0
            if spacetimedsl_table.is_singleton
                && internal_column.rust_field_name == "id"
                && internal_column
                    .rust_field_type_name_or_path
                    .to_token_stream()
                    .to_string()
                    == "u8"
            {
                constructor_arg = Some(quote! {
                    let #column_name = 0u8;
                });
                return (
                    arg,
                    wrapper_type_option_to_wrapped_type_option_mapper,
                    constructor_arg,
                    constructor_arg_name,
                );
            }

            if internal_column.spacetimedb_column_is_auto_inc {
                constructor_arg = Some(quote! {
                    let #column_name = #column_type::default();
                });
            } else if let Some(column_name) =
                &spacetimedsl_table.on_insert_set_current_timestamp_column_name
                && { internal_column.rust_field_name.eq(column_name) }
            {
                constructor_arg = Some(quote! {
                    let #column_name = self.ctx.timestamp()?;
                });
            } else if let Some(column_name) =
                &spacetimedsl_table.on_update_set_current_timestamp_column_name
                && { internal_column.rust_field_name.eq(column_name) }
            {
                let column_type_str = internal_column
                    .rust_field_type_name_or_path
                    .to_token_stream()
                    .to_string();
                let timestamp_value = if column_type_str.starts_with("Option") {
                    quote! { None }
                } else {
                    quote! { self.ctx.timestamp()? }
                };
                constructor_arg = Some(quote! {
                    let #column_name = #timestamp_value;
                });
            }

            if constructor_arg.is_some() {
                return (
                    arg,
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
            WrapperType::Created(_) => {
                if internal_column
                    .rust_field_type_name_or_path
                    .to_token_stream()
                    .to_string()
                    .eq(&"String")
                {
                    arg = Some(SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(quote! { String }),
                    });
                    match create_or_update {
                        CreateOrUpdate::Create => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#column_name;
                            });
                        }
                        CreateOrUpdate::Update => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#getter_name();
                            });
                        }
                    };
                } else {
                    arg = Some(SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(
                            WrapperType::map_to_wrapped_type(wrapper_type).to_token_stream(),
                        ),
                    });

                    constructor_arg = Some(quote! {
                        let #column_name = #singular_table_name.#column_name;
                    });
                }
            }
            WrapperType::Used(_) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if internal_column.spacetimedsl_column_is_option {
                    arg = Some(SpacetimeDSLArg {
                        is_option: true,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Wrapped {
                            wrapped_type: WrapperType::map_to_wrapped_type(wrapper_type)
                                .to_token_stream(),
                            actual_type: quote! { Option<#wrapper_type_name_or_path> },
                        },
                    });
                    constructor_arg = Some(quote! {
                        let #column_name = #singular_table_name.#column_name;
                    });
                    wrapper_type_option_to_wrapped_type_option_mapper =
                        Some(map_wrapper_type_option_to_wrapped_type_option(
                            column_name,
                            wrapper_type_name_or_path,
                        ));
                } else {
                    let wrapped_type =
                        WrapperType::map_to_wrapped_type(wrapper_type).to_token_stream();

                    arg = Some(SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Wrapped {
                            wrapped_type: wrapped_type.clone(),
                            actual_type: quote! { #wrapper_type_name_or_path },
                        },
                    });

                    match create_or_update {
                        CreateOrUpdate::Create => {
                            constructor_arg = Some(quote! {
                                let #column_name = #singular_table_name.#column_name.value();
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
                arg = Some(SpacetimeDSLArg {
                    is_option: false,
                    arg_name: column_name.clone(),
                    arg_type: SpacetimeDSLArgType::Normal(quote! { String }),
                });

                match create_or_update {
                    CreateOrUpdate::Create => {
                        constructor_arg = Some(quote! {
                            let #column_name = #singular_table_name.#column_name;
                        });
                    }
                    CreateOrUpdate::Update => {
                        constructor_arg = Some(quote! {
                            let #column_name = #singular_table_name.#getter_name();
                        });
                    }
                };
            } else {
                arg = Some(SpacetimeDSLArg {
                    is_option: internal_column.spacetimedsl_column_is_option,
                    arg_name: column_name.clone(),
                    arg_type: SpacetimeDSLArgType::Normal(quote! { #column_type }),
                });
                constructor_arg = Some(quote! {
                    let #column_name = #singular_table_name.#column_name;
                });
            }
        }
    };

    (
        arg,
        wrapper_type_option_to_wrapped_type_option_mapper,
        constructor_arg,
        constructor_arg_name,
    )
}

pub(in crate::internal) fn for_method(
    dsl_method: DSLMethod,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &mut SpacetimeDSLTable,
    internal_columns: &Vec<InternalColumn>,
    primary_key_column: &InternalColumn,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_as_string = singular_table_name.to_string();
    let singular_table_name_pascal_case =
        RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());
    let plural_table_name = &spacetimedsl_table.plural_name;

    let primary_key_column_name = &primary_key_column.rust_field_name;
    let primary_key_column_name_as_string = &primary_key_column.rust_field_name.to_string();

    let one = OneOrMultiple::One;
    let multiple = OneOrMultiple::Multiple;

    // TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/35 Doc comments should be influenced by referenced_by and foreign_key attributes.
    let doc_comment;
    let trait_name;
    let method_name;
    let return_type;

    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let mut additional_paths_to_use: Vec<syn::Path> = vec![];
    let mut method_args = vec![];
    let method_impl;

    let read_context_compatible = match &dsl_method {
        DSLMethod::GetMany(_) | DSLMethod::GetOne(_) => true,
        DSLMethod::Create
        | DSLMethod::GetAll
        | DSLMethod::GetCount
        | DSLMethod::Update(_)
        | DSLMethod::DeleteOne(_)
        | DSLMethod::DeleteMany(_) => false,
    };

    match dsl_method {
        DSLMethod::Create => {
            doc_comment = format!("Create a row in the `{singular_table_name}` table.");

            trait_name = format_ident!("Create{singular_table_name_pascal_case}Row");

            method_name = format_ident!("create_{}", singular_table_name);

            return_type = quote! {
                Result<#struct_name, crate::spacetimedsl::error::SpacetimeDSLError>
            };

            let mut method_arg_members = vec![];

            let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
            let mut constructor_args = vec![];
            let mut constructor_arg_names = vec![];

            for internal_column in internal_columns {
                let (
                    method_arg_member,
                    wrapper_type_option_to_wrapped_type_option_mapper,
                    constructor_arg,
                    constructor_arg_name,
                ) = process_columns_for_create_and_update_method(
                    spacetimedsl_table,
                    CreateOrUpdate::Create,
                    internal_column,
                );

                if let Some(method_arg_member) = method_arg_member {
                    method_arg_members.push(method_arg_member)
                }

                if let Some(wrapper_type_option_to_wrapped_type_option_mapper) =
                    wrapper_type_option_to_wrapped_type_option_mapper
                {
                    wrapper_type_option_to_wrapped_type_option_mappers
                        .push(wrapper_type_option_to_wrapped_type_option_mapper)
                }

                if let Some(constructor_arg) = constructor_arg {
                    constructor_args.push(constructor_arg)
                }

                constructor_arg_names.push(constructor_arg_name)
            }

            if !method_arg_members.is_empty() {
                let method_arg_name = format_ident!("Create{singular_table_name_pascal_case}");

                method_args.push(SpacetimeDSLArg {
                    is_option: false,
                    arg_name: singular_table_name.clone(),
                    arg_type: SpacetimeDSLArgType::Normal(quote! {
                        #method_arg_name
                    }),
                });

                let method_arg_member_names_and_types = method_arg_members
                    .iter()
                    .map(|member| {
                        let member_name = &member.arg_name;
                        let member_type = match &member.arg_type {
                            SpacetimeDSLArgType::Normal(member_type) => member_type,
                            SpacetimeDSLArgType::Wrapped { actual_type, .. } => actual_type,
                        };
                        quote! {
                            pub #member_name : #member_type
                        }
                    })
                    .collect_vec();

                spacetimedsl_table.create_dsl_method_arg = Some(CreateDSLMethodArg {
                    struct_name: method_arg_name.clone(),
                    struct_members: method_arg_members,
                    struct_impl: quote! {
                        pub struct #method_arg_name {
                            #(#method_arg_member_names_and_types),*
                        }
                    },
                });
            }

            let mut column_names_and_row_values = String::new();
            column_names_and_row_values.push_str("{{ ");
            column_names_and_row_values.push_str(&format!("{singular_table_name} : "));
            column_names_and_row_values.push_str("{:?} }}");

            let multi_column_index_checks = multi_column_index_checks(
                Action::Create,
                singular_table_name,
                spacetimedb_table,
                internal_columns,
                primary_key_column_name,
            );

            let use_itertools = if !multi_column_index_checks.is_empty() {
                quote! {
                    use ::spacetimedsl::itertools::Itertools;
                }
            } else {
                TokenStream::default()
            };

            let res = reference_integrity_checks_on_create_or_update(
                CreateOrUpdate::Create,
                spacetimedb_table,
                internal_columns,
                additional_paths_to_use,
                None,
                &OneOrMultiple::One,
                primary_key_column,
            );
            additional_paths_to_use = res.0;
            let reference_integrity_checks = res.1;

            let let_field_name_for_found_value =
                if multi_column_index_checks.is_empty() && reference_integrity_checks.is_empty() {
                    TokenStream::default()
                } else {
                    quote! {
                        let mut #field_name_for_found_value: Option<#struct_name> = None;
                    }
                };

            let before_insert_hook = match &spacetimedsl_table.hooks.before_insert {
                None => TokenStream::default(),
                Some(before_insert_hook) => {
                    let hook_trait_name = &before_insert_hook.trait_name;
                    let hook_function_name = &before_insert_hook.function_name;

                    quote! {
                        use self::#hook_trait_name;
                        let #singular_table_name = crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, #singular_table_name)?;
                    }
                }
            };

            let after_insert_hook = match &spacetimedsl_table.hooks.after_insert {
                None => TokenStream::default(),
                Some(after_insert_hook) => {
                    let hook_trait_name = &after_insert_hook.trait_name;
                    let hook_function_name = &after_insert_hook.function_name;

                    quote! {
                        use self::#hook_trait_name;
                        crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &entity)?;
                    }
                }
            };

            method_impl = quote! {
                #use_itertools

                #before_insert_hook

                #(#constructor_args)*
                #(#wrapper_type_option_to_wrapped_type_option_mappers)*
                let #singular_table_name = #struct_name {
                    #(#constructor_arg_names),*
                };

                #let_field_name_for_found_value

                #(#multi_column_index_checks)*

                #(#reference_integrity_checks)*

                match self
                    .db
                    .#singular_table_name()
                    .try_insert(#singular_table_name.clone()) { // FIXME: No clone?
                    Ok(entity) => {
                        #after_insert_hook

                        Ok(entity)
                    },
                    Err(error) => match error {
                        spacetimedb::TryInsertError::UniqueConstraintViolation(_) => {
                            Err(crate::spacetimedsl::error::SpacetimeDSLError::UniqueConstraintViolation {
                                table_name: #singular_table_name_as_string.into(),
                                action: crate::spacetimedsl::error::Action::Create,
                                error_from: crate::spacetimedsl::error::ErrorFrom::SpacetimeDB,
                                one_or_multiple: #one,
                                column_names_and_row_values: format!(#column_names_and_row_values, #singular_table_name).into(), // FIXME: Only show unique columns here
                            })
                        }
                        spacetimedb::TryInsertError::AutoIncOverflow(_) => {
                            Err(crate::spacetimedsl::error::SpacetimeDSLError::AutoIncOverflow {
                                table_name: #singular_table_name_as_string.into(),
                            })
                        }
                    },
                }
            };
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
                    .db
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
                    .db
                    .#singular_table_name()
                    .count()
            };
        }
        DSLMethod::GetMany(index)
        | DSLMethod::DeleteMany(index)
        | DSLMethod::GetOne(index)
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
                IndexType::BTreeSingleColumn { column }
                | IndexType::HashSingleColumn { column } => {
                    is_multi_column_index = false;
                    index_columns.push(column.clone());
                    value_matches_or_values_match = "value matches the value from";
                    single_or_multi = "single";
                    index_documentation = "btree index".to_string();
                    documentation_on_column_or_columns = format!("`{column}` column");
                    column_names_and_row_values.push_str(&format!(", {column} : "));
                    column_names_and_row_values.push_str("{} ");
                }
                IndexType::BTreeMultiColumn { columns }
                | IndexType::HashMultiColumn { columns } => {
                    is_multi_column_index = true;
                    value_matches_or_values_match = "values match the values from";
                    single_or_multi = "multi";
                    index_documentation = format!("btree index `{index_name}`");

                    documentation_on_column_or_columns = String::new();
                    documentation_on_column_or_columns.push_str("columns");

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
                    index_documentation = "direct index".to_string();
                    documentation_on_column_or_columns = format!("`{column}` column");
                    column_names_and_row_values.push_str(&format!(", {column} : "));
                    column_names_and_row_values.push_str("{} ");
                }
            };
            column_names_and_row_values.push_str(" }}");

            let is_singleton_pk = spacetimedsl_table.is_singleton
                && !is_multi_column_index
                && index_columns
                    .first()
                    .is_some_and(|c| primary_key_column.rust_field_name == *c);

            let unique_multi_column_index_hint = if is_unique_index && is_multi_column_index {
                "Warning: The unique multi-column index feature of SpacetimeDSL is experimental.\n- It will be removed if unique multi-column indices are implemented in SpacetimeDB.\n- SpacetimeDSL is only able to enforce referential integrity if you never use the (mutating) `insert`, `update` and `delete` methods of `spacetimedb::ReducerContext` yourself."
            } else {
                ""
            };

            doc_comment = match dsl_method {
                DSLMethod::GetMany(_) => format!(
                    "Get a `{struct_name}` iterator that contains all rows in the `{singular_table_name}` table whose {value_matches_or_values_match} the {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::DeleteMany(_) => format!(
                    "Try to delete all `{struct_name}` rows in the `{singular_table_name}` table whose {value_matches_or_values_match} the {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                ),
                DSLMethod::GetOne(_) => {
                    if is_singleton_pk {
                        format!(
                            "Try to get the `{struct_name}` from the singleton `{singular_table_name}` table."
                        )
                    } else {
                        format!(
                            "{unique_multi_column_index_hint}\n\nTry to get a `{struct_name}` from the `{singular_table_name}` table whose {value_matches_or_values_match} the unique {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                        )
                    }
                }
                DSLMethod::Update(_) => {
                    if is_singleton_pk {
                        format!(
                            "Try to update the `{struct_name}` row of the singleton `{singular_table_name}` table."
                        )
                    } else {
                        format!(
                            "{unique_multi_column_index_hint}\n\nTry to update a `{struct_name}` row of the `{singular_table_name}` table whose {value_matches_or_values_match} the unique {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                        )
                    }
                }
                DSLMethod::DeleteOne(_) => {
                    if is_singleton_pk {
                        format!(
                            "Try to delete the `{struct_name}` row from the singleton `{singular_table_name}` table."
                        )
                    } else {
                        format!(
                            "{unique_multi_column_index_hint}\n\nTry to delete a `{struct_name}` row in the `{singular_table_name}` table whose {value_matches_or_values_match} the unique {single_or_multi}-column {index_documentation} on the {documentation_on_column_or_columns}."
                        )
                    }
                }
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
                DSLMethod::GetOne(_) => {
                    if is_singleton_pk {
                        format_ident!("Get{singular_table_name_pascal_case}Row")
                    } else {
                        format_ident!(
                            "Get{singular_table_name_pascal_case}RowOptionBy{index_name_pascal_case}"
                        )
                    }
                }
                DSLMethod::Update(_) => {
                    if is_singleton_pk {
                        format_ident!("Update{singular_table_name_pascal_case}Row")
                    } else {
                        format_ident!(
                            "Update{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"
                        )
                    }
                }
                DSLMethod::DeleteOne(_) => {
                    if is_singleton_pk {
                        format_ident!("Delete{singular_table_name_pascal_case}Row")
                    } else {
                        format_ident!(
                            "Delete{singular_table_name_pascal_case}RowBy{index_name_pascal_case}"
                        )
                    }
                }
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => panic!(
                    "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                ),
            };

            method_name = match dsl_method {
                DSLMethod::GetMany(_) => format_ident!("get_{plural_table_name}_by_{index_name}"),
                DSLMethod::DeleteMany(_) => {
                    format_ident!("delete_{plural_table_name}_by_{index_name}")
                }
                DSLMethod::GetOne(_) => {
                    if is_singleton_pk {
                        format_ident!("get_{singular_table_name}")
                    } else {
                        format_ident!("get_{singular_table_name}_by_{index_name}")
                    }
                }
                DSLMethod::Update(_) => {
                    if is_singleton_pk {
                        format_ident!("update_{singular_table_name}")
                    } else {
                        format_ident!("update_{singular_table_name}_by_{index_name}")
                    }
                }
                DSLMethod::DeleteOne(_) => {
                    if is_singleton_pk {
                        format_ident!("delete_{singular_table_name}")
                    } else {
                        format_ident!("delete_{singular_table_name}_by_{index_name}")
                    }
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
                    Result<crate::spacetimedsl::delete::DeletionResult, crate::spacetimedsl::error::SpacetimeDSLError>
                },
                DSLMethod::GetOne(_) => quote! {
                    Result<#struct_name, crate::spacetimedsl::error::SpacetimeDSLError>
                },
                DSLMethod::Update(_) => quote! {
                    Result<#struct_name, crate::spacetimedsl::error::SpacetimeDSLError>
                },
                DSLMethod::DeleteOne(_) => quote! {
                    Result<crate::spacetimedsl::delete::DeletionResult, crate::spacetimedsl::error::SpacetimeDSLError>
                },
                DSLMethod::Create | DSLMethod::GetAll | DSLMethod::GetCount => panic!(
                    "DSLColumnMethod Create / GetAll / GetCount should already be processed!"
                ),
            };

            match dsl_method {
                DSLMethod::Update(_) => {
                    method_args.push(SpacetimeDSLArg {
                        is_option: false,
                        arg_name: singular_table_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(quote! { #struct_name }),
                    });

                    let multi_column_index_checks = multi_column_index_checks(
                        Action::Update,
                        singular_table_name,
                        spacetimedb_table,
                        internal_columns,
                        primary_key_column_name,
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
                                    spacetimedsl_table,
                                    CreateOrUpdate::Update,
                                    internal_column,
                                );
                            if let Some(column_getter) = column_getter {
                                row_value_getters.push(column_getter)
                            };
                        });

                    let on_update_set_current_timestamp = match &spacetimedsl_table
                        .on_update_set_current_timestamp_column_name
                    {
                        None => TokenStream::default(),
                        Some(column_name) => {
                            let on_update_set_current_timestamp_column = internal_columns
                                .iter()
                                .find(|c| c.rust_field_name.eq(column_name))
                                .unwrap_or_else(|| {
                                    panic!("{column_name} column should exist in internal columns")
                                });

                            let column_type_str = on_update_set_current_timestamp_column
                                .rust_field_type_name_or_path
                                .to_token_stream()
                                .to_string();

                            let timestamp_value = if column_type_str.starts_with("Option") {
                                quote! { Some(self.ctx.timestamp()?) }
                            } else {
                                quote! { self.ctx.timestamp()? }
                            };

                            quote! {
                                #singular_table_name.#column_name = #timestamp_value;
                            }
                        }
                    };

                    let use_itertools = if !multi_column_index_checks.is_empty() {
                        quote! {
                            use ::spacetimedsl::itertools::Itertools;
                        }
                    } else {
                        TokenStream::default()
                    };

                    let one_or_multiple = match is_multi_column_index {
                        false => OneOrMultiple::One,
                        true => OneOrMultiple::Multiple,
                    };

                    let res = reference_integrity_checks_on_create_or_update(
                        CreateOrUpdate::Update,
                        spacetimedb_table,
                        internal_columns,
                        additional_paths_to_use,
                        Some((&column_names_and_row_values, &index_columns)),
                        &one_or_multiple,
                        primary_key_column,
                    );
                    additional_paths_to_use = res.0;
                    let reference_integrity_checks = res.1;

                    let let_field_name_for_found_value = if multi_column_index_checks.is_empty()
                        && reference_integrity_checks.is_empty()
                        && spacetimedsl_table.hooks.before_update.is_none()
                        && spacetimedsl_table.hooks.after_update.is_none()
                    {
                        TokenStream::default()
                    } else {
                        quote! {
                            let mut #field_name_for_found_value: Option<#struct_name> = None;
                        }
                    };

                    let index_name = match is_multi_column_index {
                        true => &format_ident!("{primary_key_column_name}"),
                        false => index_name,
                    };

                    let before_update_hook = match &spacetimedsl_table.hooks.before_update {
                        None => TokenStream::default(),
                        Some(before_update_hook) => {
                            let hook_trait_name = &before_update_hook.trait_name;
                            let hook_function_name = &before_update_hook.function_name;

                            quote! {
                                if #field_name_for_found_value.is_none() {
                                    #field_name_for_found_value = Some(
                                        self.db.#singular_table_name().#primary_key_column_name()
                                            .find(#singular_table_name.#primary_key_column_name)
                                            .expect("Row should exist for update")
                                    )
                                }

                                use self::#hook_trait_name;
                                let #singular_table_name = crate::spacetimedsl::DSLMethodHooks::#hook_function_name(
                                    self,
                                    #field_name_for_found_value.as_ref().unwrap(),
                                    #singular_table_name
                                )?;
                            }
                        }
                    };

                    let after_update_hook = match &spacetimedsl_table.hooks.after_update {
                        None => TokenStream::default(),
                        Some(after_update_hook) => {
                            let hook_trait_name = &after_update_hook.trait_name;
                            let hook_function_name = &after_update_hook.function_name;

                            quote! {
                                use self::#hook_trait_name;
                                crate::spacetimedsl::DSLMethodHooks::#hook_function_name(
                                    self,
                                    #field_name_for_found_value.as_ref().unwrap(),
                                    &#singular_table_name
                                )?;
                            }
                        }
                    };

                    let set_singleton_id_to_zero = if is_singleton_pk {
                        quote! { #singular_table_name.id = 0u8; }
                    } else {
                        TokenStream::default()
                    };

                    method_impl = quote! {
                        #use_itertools

                        let mut #singular_table_name = #singular_table_name;
                        #set_singleton_id_to_zero

                        #let_field_name_for_found_value

                        #(#multi_column_index_checks)*

                        #(#row_value_getters)*
                        #(#reference_integrity_checks)*

                        #on_update_set_current_timestamp

                        #before_update_hook

                        // FIXME: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/60 try_update instead of update and on error return Err(crate::spacetimedsl::error::SpacetimeDSLError);
                        let #singular_table_name = self
                            .db
                            .#singular_table_name()
                            .#index_name()
                            .update(#singular_table_name);

                        #after_update_hook

                        Ok(#singular_table_name)
                    };
                }
                dsl_method => {
                    let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
                    let mut row_value_getters = vec![];

                    for column_name in &index_columns {
                        let column = internal_columns
                            .iter()
                            .find(|c| c.rust_field_name == *column_name)
                            .expect("Column should exist in internal columns");

                        let column_is_string = column
                            .rust_field_type_name_or_path
                            .to_token_stream()
                            .to_string()
                            .eq(&"String");

                        let wrapper_type_option_to_wrapped_type_option_mapper;
                        let method_arg;
                        let row_value_getter;

                        match &column.spacetimedsl_column_wrapper_type {
                            Some(wrapper_type) => {
                                let wrapper_type_ty = &WrapperType::map(wrapper_type);

                                if column_is_string {
                                    wrapper_type_option_to_wrapped_type_option_mapper =
                                        TokenStream::default();

                                    match &dsl_method {
                                        DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                            method_arg = SpacetimeDSLArg {
                                                is_option: false,
                                                arg_name: column_name.clone(),
                                                arg_type: SpacetimeDSLArgType::Normal(
                                                    quote! { &str },
                                                ),
                                            };
                                            row_value_getter = quote! { #column_name };
                                        }
                                        DSLMethod::GetOne(_) | DSLMethod::DeleteOne(_) => {
                                            method_arg = SpacetimeDSLArg {
                                                is_option: false,
                                                arg_name: column_name.clone(),
                                                arg_type: SpacetimeDSLArgType::Normal(
                                                    quote! { &str },
                                                ),
                                            };
                                            if is_multi_column_index {
                                                row_value_getter = quote! { #column_name };
                                            } else {
                                                row_value_getter =
                                                    quote! { #column_name.to_string() };
                                            }
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
                                            Some(#column_name) => Some(Into::<#wrapper_type_ty>::into(#column_name).value());
                                        }
                                    };

                                    method_arg = SpacetimeDSLArg {
                                        is_option: true,
                                        arg_name: column_name.clone(),
                                        arg_type: SpacetimeDSLArgType::Wrapped {
                                            wrapped_type: WrapperType::map_to_wrapped_type(
                                                wrapper_type,
                                            )
                                            .to_token_stream(),
                                            actual_type: quote! { &impl Into<Option<#wrapper_type_ty>> },
                                        },
                                    };

                                    row_value_getter = quote! { #column_name };
                                } else {
                                    wrapper_type_option_to_wrapped_type_option_mapper =
                                        TokenStream::default();

                                    match &dsl_method {
                                        DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                            method_arg = SpacetimeDSLArg {
                                                is_option: false,
                                                arg_name: column_name.clone(),
                                                arg_type: SpacetimeDSLArgType::Wrapped {
                                                    wrapped_type: WrapperType::map_to_wrapped_type(
                                                        wrapper_type,
                                                    )
                                                    .to_token_stream(),
                                                    actual_type: quote! { impl Into<#wrapper_type_ty> },
                                                },
                                            };
                                            row_value_getter =
                                                quote! { #column_name.into().value() };
                                        }
                                        DSLMethod::GetOne(_) | DSLMethod::DeleteOne(_) => {
                                            method_arg = SpacetimeDSLArg {
                                                is_option: false,
                                                arg_name: column_name.clone(),
                                                arg_type: SpacetimeDSLArgType::Wrapped {
                                                    wrapped_type: WrapperType::map_to_wrapped_type(
                                                        wrapper_type,
                                                    )
                                                    .to_token_stream(),
                                                    actual_type: quote! { impl Into<#wrapper_type_ty> + Clone },
                                                },
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

                                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                                let column_type = if column_is_string {
                                    parse_str("str").expect("parsing should have worked")
                                } else {
                                    column.rust_field_type_name_or_path.clone()
                                };

                                match dsl_method {
                                    DSLMethod::GetMany(_) | DSLMethod::DeleteMany(_) => {
                                        method_arg = SpacetimeDSLArg {
                                            is_option: column.spacetimedsl_column_is_option,
                                            arg_name: column_name.clone(),
                                            arg_type: SpacetimeDSLArgType::Normal(
                                                quote! { &'a #column_type },
                                            ),
                                        };

                                        row_value_getter = quote! { #column_name };
                                    }
                                    DSLMethod::GetOne(_) | DSLMethod::DeleteOne(_) => {
                                        method_arg = SpacetimeDSLArg {
                                            is_option: column.spacetimedsl_column_is_option,
                                            arg_name: column_name.clone(),
                                            arg_type: SpacetimeDSLArgType::Normal(
                                                quote! { &#column_type },
                                            ),
                                        };

                                        if is_multi_column_index {
                                            row_value_getter = quote! { #column_name };
                                        } else if column_is_string {
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
                        if !is_singleton_pk {
                            method_args.push(method_arg);
                            row_value_getters.push(row_value_getter);
                        }
                    }

                    let method_impl_prefix = quote! {
                        self
                            .db
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
                                use ::spacetimedsl::itertools::Itertools;

                                #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                #let_index_name

                                let rows_to_delete: Vec<#struct_name> = #method_impl_prefix
                                    .filter(#index_name)
                                    .collect();

                                if rows_to_delete.is_empty() {
                                    return Ok(crate::spacetimedsl::delete::DeletionResult {
                                        table_name: #singular_table_name_as_string.into(),
                                        one_or_multiple: #multiple,
                                        entries: vec![],
                                    });
                                }
                            };

                            let wrapper_type_struct_name_or_path = match primary_key_column
                                .spacetimedsl_column_wrapper_type
                                .as_ref()
                                .expect("Should have a wrapper type")
                            {
                                WrapperType::Created(wrap) => {
                                    wrap.wrapper_struct_name.to_token_stream()
                                }
                                WrapperType::Used(wrapped) => {
                                    wrapped.wrapper_struct_name_or_path.to_token_stream()
                                }
                            };

                            let map_rows_to_delete_to_deletion_result_entries = quote! {
                                let mut deletion_result_entries = std::collections::HashMap::new();

                                for row_to_delete in &rows_to_delete {
                                    deletion_result_entries.insert(
                                        &row_to_delete.#primary_key_column_name,
                                        crate::spacetimedsl::delete::DeletionResultEntry {
                                            table_name: #singular_table_name_as_string.into(),
                                            column_name: #primary_key_column_name_as_string.into(),
                                            strategy: crate::spacetimedsl::delete::OnDeleteStrategy::Delete,
                                            row_value: format!("{}", #wrapper_type_struct_name_or_path::new(row_to_delete.#primary_key_column_name.clone())).into(),
                                            child_entries: vec![],
                                        }
                                    );
                                }
                            };

                            let before_delete_hook = match &spacetimedsl_table.hooks.before_delete {
                                None => TokenStream::default(),
                                Some(before_delete_hook) => {
                                    let hook_trait_name = &before_delete_hook.trait_name;
                                    let hook_function_name = &before_delete_hook.function_name;

                                    quote! {
                                        use self::#hook_trait_name;
                                        for row_to_delete in &rows_to_delete {
                                            crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &row_to_delete)?;
                                        }
                                    }
                                }
                            };

                            let after_delete_hook = match &spacetimedsl_table.hooks.after_delete {
                                None => TokenStream::default(),
                                Some(after_delete_hook) => {
                                    let hook_trait_name = &after_delete_hook.trait_name;
                                    let hook_function_name = &after_delete_hook.function_name;

                                    quote! {
                                        use self::#hook_trait_name;
                                        for row_to_delete in &rows_to_delete {
                                            crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &row_to_delete)?;
                                        }
                                    }
                                }
                            };

                            let delete_many_impl = quote! {
                                let count_of_rows_to_delete: u64 = rows_to_delete
                                    .len()
                                    .try_into()
                                    .unwrap_or(u64::MAX);

                                let count_of_deleted_rows = #method_impl_prefix.delete(#index_name);

                                if count_of_rows_to_delete.ne(&count_of_deleted_rows) {
                                    return Err(
                                        crate::spacetimedsl::error::SpacetimeDSLError::Error(
                                            format!(
                                                "Delete Many Error: `count_of_rows_to_delete ( {} ) != ( {} ) count_of_deleted_rows`!",
                                                &count_of_rows_to_delete,
                                                &count_of_deleted_rows
                                            )
                                        )
                                    );
                                }
                            };

                            let return_result_impl = quote! {
                                return Ok(crate::spacetimedsl::delete::DeletionResult {
                                    table_name: #singular_table_name_as_string.into(),
                                    one_or_multiple: #multiple,
                                    entries: deletion_result_entries.into_values().collect_vec(),
                                });
                            };

                            if spacetimedsl_table.referencing_tables.is_empty() {
                                method_impl = quote! {
                                    #impl_until_return_ok_on_is_empty

                                    #map_rows_to_delete_to_deletion_result_entries

                                    #before_delete_hook

                                    #delete_many_impl

                                    #after_delete_hook

                                    #return_result_impl
                                };
                            } else {
                                let on_error_handler = quote! {
                                    let error = crate::spacetimedsl::delete::DeletionResult {
                                        table_name: #singular_table_name_as_string.into(),
                                        one_or_multiple: #multiple,
                                        entries: deletion_result_entries.into_values().collect_vec(),
                                    };

                                    return Err(
                                        crate::spacetimedsl::error::SpacetimeDSLError::Error(
                                            format!("Delete Many Error: An unknown error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}")
                                        )
                                    );
                                };

                                let error_strategy =
                                    get_referenced_table_function_call_for_dsl_method(
                                        singular_table_name,
                                        primary_key_column_name,
                                        OnDeleteStrategy::Error,
                                        OneOrMultiple::Multiple,
                                        &quote! {
                                            let error = crate::spacetimedsl::delete::DeletionResult {
                                                table_name: #singular_table_name_as_string.into(),
                                                one_or_multiple: #multiple,
                                                entries: deletion_result_entries.into_values().collect_vec(),
                                            };

                                            return Err(
                                                crate::spacetimedsl::error::SpacetimeDSLError::ReferenceIntegrityViolation(
                                                    crate::spacetimedsl::error::ReferenceIntegrityViolationError::OnDelete(error)
                                                )
                                            );
                                        },
                                    );

                                let delete_strategy =
                                    get_referenced_table_function_call_for_dsl_method(
                                        singular_table_name,
                                        primary_key_column_name,
                                        OnDeleteStrategy::Delete,
                                        OneOrMultiple::Multiple,
                                        &on_error_handler,
                                    );

                                /* TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32
                                let set_none_strategy =
                                    get_referenced_table_function_call_for_dsl_method(
                                        singular_table_name,
                                        primary_key_column_name,
                                        OnDeleteStrategy::SetNone,
                                        OneOrMultiple::Multiple,
                                        &on_error_handler,
                                    );
                                */

                                let set_zero_strategy =
                                    get_referenced_table_function_call_for_dsl_method(
                                        singular_table_name,
                                        primary_key_column_name,
                                        OnDeleteStrategy::SetZero,
                                        OneOrMultiple::Multiple,
                                        &on_error_handler,
                                    );

                                let ignore_strategy =
                                    get_referenced_table_function_call_for_dsl_method(
                                        singular_table_name,
                                        primary_key_column_name,
                                        OnDeleteStrategy::Ignore,
                                        OneOrMultiple::Multiple,
                                        &on_error_handler,
                                    );

                                method_impl = quote! {
                                    #impl_until_return_ok_on_is_empty

                                    #map_rows_to_delete_to_deletion_result_entries

                                    #error_strategy

                                    #before_delete_hook

                                    #delete_many_impl

                                    #after_delete_hook

                                    #delete_strategy

                                    //TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 #set_none_strategy

                                    #set_zero_strategy

                                    #ignore_strategy

                                    #return_result_impl
                                };
                            }
                        }
                        DSLMethod::GetOne(_) => match is_multi_column_index {
                            true => {
                                // FIXME: Row Value Getters of Wrapper Types shouldn't be `id.clone().into().value()`, they should be `let id = id.into();` at the method beginning and then `id.value()` anywhere else
                                let multi_column_index_check = get_unique_multi_column_index_check(
                                    &Action::Get,
                                    singular_table_name,
                                    index_name,
                                    &column_names_and_row_values,
                                    &row_value_getters,
                                );

                                method_impl = quote! {
                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    use ::spacetimedsl::itertools::Itertools;

                                    let mut #field_name_for_found_value: Option<#struct_name> = None;

                                    #multi_column_index_check

                                    match #field_name_for_found_value {
                                        Some(#singular_table_name) => Ok(#singular_table_name),
                                        None => {
                                            return Err(
                                                crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                                    table_name: #singular_table_name_as_string.into(),
                                                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*).into()
                                                }
                                            );
                                        }
                                    }
                                };
                            }
                            false => {
                                if is_singleton_pk {
                                    method_impl = quote! {
                                        match self.db.#singular_table_name().id().find(&0u8) {
                                            Some(#singular_table_name) => Ok(#singular_table_name),
                                            None => return Err(
                                                crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                                    table_name: #singular_table_name_as_string.into(),
                                                    column_names_and_row_values: "{ id : 0 }".into()
                                                }
                                            )
                                        }
                                    };
                                } else {
                                    method_impl = quote! {
                                        #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                        match #method_impl_prefix.find(#(#row_value_getters),*) {
                                            Some(#singular_table_name) => Ok(#singular_table_name),
                                            None => return Err(
                                                crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                                    table_name: #singular_table_name_as_string.into(),
                                                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*).into()
                                                }
                                            )
                                        }
                                    };
                                }
                            }
                        },
                        DSLMethod::DeleteOne(_) => {
                            if is_singleton_pk {
                                let before_delete_hook = match &spacetimedsl_table
                                    .hooks
                                    .before_delete
                                {
                                    None => TokenStream::default(),
                                    Some(before_delete_hook) => {
                                        let hook_trait_name = &before_delete_hook.trait_name;
                                        let hook_function_name = &before_delete_hook.function_name;
                                        quote! {
                                            use self::#hook_trait_name;
                                            crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &row_to_delete)?;
                                        }
                                    }
                                };
                                let after_delete_hook = match &spacetimedsl_table.hooks.after_delete
                                {
                                    None => TokenStream::default(),
                                    Some(after_delete_hook) => {
                                        let hook_trait_name = &after_delete_hook.trait_name;
                                        let hook_function_name = &after_delete_hook.function_name;
                                        quote! {
                                            use self::#hook_trait_name;
                                            crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &row_to_delete)?;
                                        }
                                    }
                                };
                                method_impl = quote! {
                                    use ::spacetimedsl::itertools::Itertools;

                                    let row_to_delete = match self.db.#singular_table_name().id().find(&0u8) {
                                        None => return Err(
                                            crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                                table_name: #singular_table_name_as_string.into(),
                                                column_names_and_row_values: "{ id : 0 }".into()
                                            }
                                        ),
                                        Some(row_to_delete) => row_to_delete,
                                    };

                                    let mut deletion_result_entry = crate::spacetimedsl::delete::DeletionResultEntry {
                                        table_name: #singular_table_name_as_string.into(),
                                        column_name: "id".into(),
                                        strategy: crate::spacetimedsl::delete::OnDeleteStrategy::Delete,
                                        row_value: "0".into(),
                                        child_entries: vec![],
                                    };

                                    #before_delete_hook

                                    match self.db.#singular_table_name().id().delete(&0u8) {
                                        false => {
                                            return Err(
                                                crate::spacetimedsl::error::SpacetimeDSLError::Error(
                                                    "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string(),
                                                )
                                            );
                                        },
                                        true => {},
                                    };

                                    #after_delete_hook

                                    return Ok(crate::spacetimedsl::delete::DeletionResult {
                                        table_name: #singular_table_name_as_string.into(),
                                        one_or_multiple: #one,
                                        entries: vec![deletion_result_entry],
                                    });
                                };
                            } else {
                                let get_row_to_delete;
                                let return_error_on_is_none;

                                match is_multi_column_index {
                                    true => {
                                        let multi_column_index_check =
                                            get_unique_multi_column_index_check(
                                                &Action::Delete,
                                                singular_table_name,
                                                index_name,
                                                &column_names_and_row_values,
                                                &row_value_getters,
                                            );

                                        get_row_to_delete = quote! {
                                            let mut #field_name_for_found_value: Option<#struct_name> = None;

                                            #multi_column_index_check

                                            let row_to_delete = #field_name_for_found_value;
                                        };

                                        return_error_on_is_none = quote! {
                                            let row_to_delete = match row_to_delete {
                                                None => return Err(
                                                    crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                                        table_name: #singular_table_name_as_string.into(),
                                                        column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*).into()
                                                    }
                                                ),
                                                Some(row_to_delete) => row_to_delete,
                                            };
                                        };
                                    }
                                    false => {
                                        let column_name = &index_columns[0];
                                        let column_type = &internal_columns.iter().find(|c| c.rust_field_name.eq(column_name)).expect("The index should have a column in the internal columns").rust_field_type_name_or_path;
                                        if column_type.to_token_stream().to_string().eq(&"String") {
                                            get_row_to_delete = quote! {
                                                let #index_name = #(#row_value_getters),*;

                                                let row_to_delete = #method_impl_prefix.find(&#index_name);
                                            }
                                        } else {
                                            get_row_to_delete = quote! {
                                                let #index_name = #(#row_value_getters),*;

                                                let row_to_delete = #method_impl_prefix.find(#index_name);
                                            }
                                        }

                                        return_error_on_is_none = quote! {
                                            let row_to_delete = match row_to_delete {
                                                None => return Err(
                                                    crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                                        table_name: #singular_table_name_as_string.into(),
                                                        column_names_and_row_values: format!(#column_names_and_row_values, &#index_name).into()
                                                    }
                                                ),
                                                Some(row_to_delete) => row_to_delete,
                                            };
                                        };
                                    }
                                };

                                let impl_until_return_err_on_is_none = quote! {
                                    use ::spacetimedsl::itertools::Itertools;

                                    #(#wrapper_type_option_to_wrapped_type_option_mappers)*

                                    #get_row_to_delete

                                    #return_error_on_is_none
                                };

                                let wrapper_type_struct_name_or_path = match primary_key_column
                                    .spacetimedsl_column_wrapper_type
                                    .as_ref()
                                    .expect("should have a wrapper type")
                                {
                                    WrapperType::Created(wrap) => {
                                        wrap.wrapper_struct_name.to_token_stream()
                                    }
                                    WrapperType::Used(wrapped) => {
                                        wrapped.wrapper_struct_name_or_path.to_token_stream()
                                    }
                                };

                                let map_row_to_delete_to_deletion_result_entry = quote! {
                                    let mut deletion_result_entry = crate::spacetimedsl::delete::DeletionResultEntry {
                                        table_name: #singular_table_name_as_string.into(),
                                        column_name: #primary_key_column_name_as_string.into(),
                                        strategy: crate::spacetimedsl::delete::OnDeleteStrategy::Delete,
                                        row_value: format!("{}", #wrapper_type_struct_name_or_path::new(row_to_delete.#primary_key_column_name.clone())).into(),
                                        child_entries: vec![],
                                    };
                                };

                                let delete_one_impl = quote! {
                                    match self
                                            .db
                                            .#singular_table_name()
                                            .#primary_key_column_name()
                                            .delete(&row_to_delete.#primary_key_column_name) {
                                        false => {
                                            return Err(
                                                crate::spacetimedsl::error::SpacetimeDSLError::Error(
                                                    "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string(),
                                                )
                                            );
                                        },
                                        true => {},
                                    };
                                };

                                let before_delete_hook = match &spacetimedsl_table
                                    .hooks
                                    .before_delete
                                {
                                    None => TokenStream::default(),
                                    Some(before_delete_hook) => {
                                        let hook_trait_name = &before_delete_hook.trait_name;
                                        let hook_function_name = &before_delete_hook.function_name;

                                        quote! {
                                            use self::#hook_trait_name;
                                            crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &row_to_delete)?;
                                        }
                                    }
                                };

                                let after_delete_hook = match &spacetimedsl_table.hooks.after_delete
                                {
                                    None => TokenStream::default(),
                                    Some(after_delete_hook) => {
                                        let hook_trait_name = &after_delete_hook.trait_name;
                                        let hook_function_name = &after_delete_hook.function_name;

                                        quote! {
                                            use self::#hook_trait_name;
                                            crate::spacetimedsl::DSLMethodHooks::#hook_function_name(self, &row_to_delete)?;
                                        }
                                    }
                                };

                                let return_result_impl = quote! {
                                    return Ok(crate::spacetimedsl::delete::DeletionResult {
                                        table_name: #singular_table_name_as_string.into(),
                                        one_or_multiple: #one,
                                        entries: vec![deletion_result_entry],
                                    });
                                };

                                if spacetimedsl_table.referencing_tables.is_empty() {
                                    method_impl = quote! {
                                        #impl_until_return_err_on_is_none

                                        #map_row_to_delete_to_deletion_result_entry

                                        #before_delete_hook

                                        #delete_one_impl

                                        #after_delete_hook

                                        #return_result_impl
                                    };
                                } else {
                                    let on_error_handler = quote! {
                                        let error = crate::spacetimedsl::delete::DeletionResult {
                                            table_name: #singular_table_name_as_string.into(),
                                            one_or_multiple: #one,
                                            entries: vec![deletion_result_entry],
                                        };

                                        return Err(
                                            crate::spacetimedsl::error::SpacetimeDSLError::Error(
                                                format!("Delete One Error: An unknown error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}")
                                            )
                                        );
                                    };

                                    let error_strategy =
                                        get_referenced_table_function_call_for_dsl_method(
                                            singular_table_name,
                                            primary_key_column_name,
                                            OnDeleteStrategy::Error,
                                            OneOrMultiple::One,
                                            &quote! {
                                                let error = crate::spacetimedsl::delete::DeletionResult {
                                                    table_name: #singular_table_name_as_string.into(),
                                                    one_or_multiple: #one,
                                                    entries: vec![deletion_result_entry],
                                                };

                                                return Err(
                                                    crate::spacetimedsl::error::SpacetimeDSLError::ReferenceIntegrityViolation(
                                                        crate::spacetimedsl::error::ReferenceIntegrityViolationError::OnDelete(error)
                                                    )
                                                );
                                            },
                                        );

                                    let delete_strategy =
                                        get_referenced_table_function_call_for_dsl_method(
                                            singular_table_name,
                                            primary_key_column_name,
                                            OnDeleteStrategy::Delete,
                                            OneOrMultiple::One,
                                            &on_error_handler,
                                        );

                                    /* TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32
                                    let set_none_strategy =
                                        get_referenced_table_function_call_for_dsl_method(
                                            singular_table_name,
                                            OnDeleteStrategy::SetNone,
                                            OneOrMultiple::One,
                                            &on_error_handler,
                                        );
                                    */

                                    let set_zero_strategy =
                                        get_referenced_table_function_call_for_dsl_method(
                                            singular_table_name,
                                            primary_key_column_name,
                                            OnDeleteStrategy::SetZero,
                                            OneOrMultiple::One,
                                            &on_error_handler,
                                        );

                                    let ignore_strategy =
                                        get_referenced_table_function_call_for_dsl_method(
                                            singular_table_name,
                                            primary_key_column_name,
                                            OnDeleteStrategy::Ignore,
                                            OneOrMultiple::One,
                                            &on_error_handler,
                                        );

                                    method_impl = quote! {
                                        #impl_until_return_err_on_is_none

                                        #map_row_to_delete_to_deletion_result_entry

                                        #error_strategy

                                        #before_delete_hook

                                        #delete_one_impl

                                        #after_delete_hook

                                        #delete_strategy

                                        //TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 #set_none_strategy

                                        #set_zero_strategy

                                        #ignore_strategy

                                        #return_result_impl
                                    };
                                }
                            } // closes else (non-singleton) block
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
        additional_paths_to_use,
        method_name,
        method_args,
        return_type,
        method_impl,
        read_context_compatible,
    }
}

fn get_referenced_table_function_call_for_dsl_method(
    singular_table_name: &Ident,
    primary_key_column_name: &Ident,
    on_delete_strategy: OnDeleteStrategy,
    one_or_multiple: OneOrMultiple,
    on_error_handler: &TokenStream,
) -> TokenStream {
    match one_or_multiple {
        OneOrMultiple::One => {
            let referenced_table_function_name =
                get_referenced_table_function_name(&OneOrMultiple::One, singular_table_name);

            quote! {
                match crate::spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self, #on_delete_strategy, &row_to_delete.#primary_key_column_name) {
                    Err(mut child_entries) => {
                        deletion_result_entry.child_entries.append(&mut child_entries);

                        #on_error_handler
                    },
                    Ok(mut child_entries) => {
                        deletion_result_entry.child_entries.append(&mut child_entries);
                    }
                };
            }
        }
        OneOrMultiple::Multiple => {
            let referenced_table_function_name =
                get_referenced_table_function_name(&OneOrMultiple::Multiple, singular_table_name);

            quote! {
                match crate::spacetimedsl::internal::DSLInternals::#referenced_table_function_name(self, #on_delete_strategy, &rows_to_delete.iter().map(|row| row.#primary_key_column_name).collect_vec()[..]) {
                    Err(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            deletion_result_entries.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in deletion_result_entries.")).child_entries.append(&mut child_entries);
                        }

                        #on_error_handler
                    },
                    Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            deletion_result_entries.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in deletion_result_entries.")).child_entries.append(&mut child_entries);
                        }
                    }
                };
            }
        }
    }
}

fn reference_integrity_checks_on_create_or_update(
    create_or_update_dsl_method: CreateOrUpdate,
    spacetimedb_table: &SpacetimeDBTable,
    columns: &Vec<InternalColumn>,
    additional_paths_to_use: Vec<Path>,
    column_names_and_row_values_and_column_names: Option<(&String, &Vec<Ident>)>,
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> (Vec<Path>, Vec<TokenStream>) {
    let mut reference_integrity_checks = vec![];

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

        let foreign_key = match &column.spacetimedsl_column_foreign_key {
            Some(fk) => fk,
            None => continue,
        };

        let referenced_table_name = &foreign_key.table_name;
        let referenced_table_name_pascal_case = format_ident!(
            "{}",
            RenameRule::PascalCase.apply_to_field(referenced_table_name.to_string())
        );

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let primary_key_column_name_of_referenced_table_pascal_case = format_ident!(
            "{}",
            RenameRule::PascalCase
                .apply_to_field(primary_key_column_name_of_referenced_table.to_string())
        );
        let get_row_of_referenced_table_by_primary_key_trait_name = format_ident!(
            "Get{referenced_table_name_pascal_case}RowOptionBy{primary_key_column_name_of_referenced_table_pascal_case}"
        );
        let get_row_of_referenced_table_by_primary_key_method_name = format_ident!(
            "get_{referenced_table_name}_by_{primary_key_column_name_of_referenced_table}"
        );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_name_as_string = referencing_table_column_name.to_string();
        let primary_key_column_name_of_referencing_table = &primary_key_column.rust_field_name;
        let referencing_table_column_getter_name =
            format_ident!("get_{referencing_table_column_name}");

        let referencing_table_column_type = column
            .rust_field_type_name_or_path
            .to_token_stream()
            .to_string();

        // With inherent impl methods (no longer trait-based), no need to import the old
        // "Get{Table}RowOptionBy{PK}" trait — the method is now a direct inherent method
        // on `crate::spacetimedsl::DSL<T>`.
        let _ = get_row_of_referenced_table_by_primary_key_trait_name;

        let field_name_for_found_value =
            format_ident!("the_same_or_another_{referencing_table_name}");

        let check = match &create_or_update_dsl_method {
            CreateOrUpdate::Create => {
                quote! {
                    match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                        Ok(_) => {},
                        Err(_) => {
                            return Err(
                                crate::spacetimedsl::error::SpacetimeDSLError::ReferenceIntegrityViolation(
                                    crate::spacetimedsl::error::ReferenceIntegrityViolationError::OnCreateOrUpdate {
                                        table_name: #referencing_table_name_as_string.into(),
                                        create_or_update: crate::spacetimedsl::error::Action::Create,
                                        column_names_and_row_values: format!("{{ {} : {} }}", #referencing_table_column_name, #referencing_table_name.#referencing_table_column_getter_name()).into()
                                    }
                                )
                            );
                        }
                    };
                }
            }
            CreateOrUpdate::Update => {
                let column_names_and_row_value_getters =
                    column_names_and_row_values_and_column_names.expect(
                        "DSLMethod::Update should have column names and row value getters!",
                    );
                let column_names_and_row_values = column_names_and_row_value_getters.0;
                let column_names = column_names_and_row_value_getters.1;
                let row_value_getters = column_names
                    .iter()
                    .map(|cn| {
                        quote! {
                            #referencing_table_name.#cn
                        }
                    })
                    .collect_vec();

                let format_for_not_found_error = match one_or_multiple {
                    OneOrMultiple::One => quote! {
                        format!(#column_names_and_row_values, #referencing_table_column_name)
                    },
                    OneOrMultiple::Multiple => quote! {
                        format!(#column_names_and_row_values, #(#row_value_getters),*)
                    },
                };

                let getter_name =
                    format_ident!("get_{primary_key_column_name_of_referencing_table}");

                quote! {
                    if #field_name_for_found_value.is_none() {
                        #field_name_for_found_value = match self.db.#referencing_table_name().#primary_key_column_name_of_referencing_table().find(#referencing_table_name.#getter_name().value()) {
                            Some(#referencing_table_name) => Some(#referencing_table_name),
                            None => {
                                return Err(
                                    crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
                                        table_name: #referencing_table_name_as_string.into(),
                                        column_names_and_row_values: #format_for_not_found_error.into()
                                    }
                                );
                            }
                        };
                    }
                    if #field_name_for_found_value.as_ref().expect("field_name_for_found_value should be Some(_)").#referencing_table_column_getter_name().ne(&#referencing_table_name.#referencing_table_column_getter_name()) {
                        match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                            Ok(_) => {},
                            Err(_) => return Err(
                                crate::spacetimedsl::error::SpacetimeDSLError::ReferenceIntegrityViolation(
                                    crate::spacetimedsl::error::ReferenceIntegrityViolationError::OnCreateOrUpdate {
                                        table_name: #referencing_table_name_as_string.into(),
                                        create_or_update: crate::spacetimedsl::error::Action::Update,
                                        column_names_and_row_values: format!("{{ {} : {} }}", #referencing_table_column_name_as_string, #referencing_table_column_name).into()
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
            column_type => {
                if column_type.starts_with("Option") {
                    quote! {
                        if #referencing_table_column_name.is_some() {
                            #check
                        }
                    }
                } else {
                    quote! {
                        #check
                    }
                }
            }
        });
    }

    (additional_paths_to_use, reference_integrity_checks)
}

fn multi_column_index_checks(
    action: Action,
    singular_table_name: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
    internal_columns: &Vec<InternalColumn>,
    primary_key_column_name: &Ident,
) -> Vec<TokenStream> {
    let mut multi_column_index_checks = vec![];
    let singular_table_name_as_string = singular_table_name.to_string();

    for multi_column_index in &spacetimedb_table.multi_column_indices {
        let mut index_column_names: VecDeque<Ident> = match &multi_column_index.index_type {
            IndexType::BTreeMultiColumn { columns } => columns.clone().into(),
            _ => {
                continue;
            }
        };

        if !multi_column_index.is_unique {
            continue;
        }

        let mut column_type_by_name = HashMap::new();

        for column in internal_columns {
            column_type_by_name.insert(
                column.rust_field_name.to_string(),
                column
                    .rust_field_type_name_or_path
                    .to_token_stream()
                    .to_string(),
            );
        }

        let index_name = &multi_column_index.name;

        let mut row_value_getters = vec![];
        let mut column_names_and_row_values = String::new();

        let first_column_name = index_column_names
            .pop_front()
            .expect("There should be a first column in Vec<Ident> of BTreeMultiColumn.");

        let last_column_name = index_column_names
            .pop_back()
            .expect("There should be a last column in Vec<Ident> of BTreeMultiColumn.");

        let any_other_column_name = index_column_names;

        column_names_and_row_values.push_str(&format!("{first_column_name} : "));
        column_names_and_row_values.push_str("{} ");
        row_value_getters.push(get_row_value_getter(
            &column_type_by_name,
            singular_table_name,
            &first_column_name,
        ));

        for any_other_column_name in any_other_column_name {
            column_names_and_row_values.push_str(&format!(", {any_other_column_name} : "));
            column_names_and_row_values.push_str("{} ");
            row_value_getters.push(get_row_value_getter(
                &column_type_by_name,
                singular_table_name,
                &any_other_column_name,
            ));
        }

        column_names_and_row_values.push_str(&format!(", {last_column_name} : "));
        column_names_and_row_values.push_str("{}");
        row_value_getters.push(get_row_value_getter(
            &column_type_by_name,
            singular_table_name,
            &last_column_name,
        ));

        let mut multi_column_index_check = get_unique_multi_column_index_check(
            &action,
            singular_table_name,
            index_name,
            &column_names_and_row_values,
            &row_value_getters,
        );

        let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

        let action_as_ident = format_ident!("{action}");

        let multiple = OneOrMultiple::Multiple;

        let return_unique_constraint_violation_error = quote! {
            return Err(
                crate::spacetimedsl::error::SpacetimeDSLError::UniqueConstraintViolation {
                    table_name: #singular_table_name_as_string.into(),
                    action: crate::spacetimedsl::error::Action::#action_as_ident,
                    error_from: crate::spacetimedsl::error::ErrorFrom::SpacetimeDSL,
                    one_or_multiple: #multiple,
                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*).into()
                }
            );
        };

        let on_some = match action {
            Action::Create | Action::Get | Action::Delete => {
                return_unique_constraint_violation_error
            }
            Action::Update => {
                quote! {
                    if #field_name_for_found_value.#primary_key_column_name.ne(&#singular_table_name.#primary_key_column_name) {
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

fn get_row_value_getter(
    column_type_by_name: &HashMap<String, String>,
    singular_table_name: &Ident,
    column_name: &Ident,
) -> TokenStream {
    if column_type_by_name
        .get(&column_name.to_string())
        .expect("Column should exist")
        .eq("String")
    {
        quote! { &#singular_table_name.#column_name }
    } else {
        quote! { #singular_table_name.#column_name }
    }
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
        #field_name_for_found_value = match self.db.#singular_table_name().#index_name().filter((#(#row_value_getters),*)).at_most_one() {
            Ok(#singular_table_name) => #singular_table_name,
            Err(_) => return Err(
                crate::spacetimedsl::error::SpacetimeDSLError::UniqueConstraintViolation {
                    table_name: #singular_table_name_as_string.into(),
                    action: crate::spacetimedsl::error::Action::#action,
                    error_from: crate::spacetimedsl::error::ErrorFrom::SpacetimeDSL,
                    one_or_multiple: #multiple,
                    column_names_and_row_values: format!(#column_names_and_row_values, #(#row_value_getters),*).into()
                }
            ),
        };
    }
}

fn for_referenced_by(
    one_or_multiple: &OneOrMultiple,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &mut SpacetimeDSLTable,
    primary_key_column: &InternalColumn,
) -> SpacetimeDSLMethod {
    let singular_table_name = &spacetimedb_table.singular_name;
    let singular_table_name_pascal_case = format_ident!(
        "{}",
        RenameRule::PascalCase.apply_to_field(spacetimedb_table.singular_name.to_string())
    );

    let primary_key_column_type = &primary_key_column.rust_field_type_name_or_path;

    let doc_comment;
    let trait_name =
        get_referenced_table_trait_name(one_or_multiple, &singular_table_name_pascal_case);
    let function_name = get_referenced_table_function_name(one_or_multiple, singular_table_name);

    let additional_paths_to_use = vec![];
    let mut function_args = vec![
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("dsl"),
            arg_type: SpacetimeDSLArgType::Normal(quote! { &crate::spacetimedsl::DSL<'_, T> }),
        },
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("strategy"),
            arg_type: SpacetimeDSLArgType::Normal(quote! { crate::spacetimedsl::delete::OnDeleteStrategy }),
        },
    ];

    let return_type;

    let arg_name;

    match one_or_multiple {
        OneOrMultiple::One => {
            doc_comment = format!(
                "Execute On Delete Strategies of all referencing tables after one row of the referenced table `{singular_table_name}` was deleted."
            );
            arg_name = format_ident!("primary_key_value_of_a_row_to_delete");
            function_args.push(SpacetimeDSLArg {
                is_option: false,
                arg_name: arg_name.clone(),
                arg_type: SpacetimeDSLArgType::Normal(quote! { &#primary_key_column_type }),
            });
            return_type = quote! {
                Result<Vec<crate::spacetimedsl::delete::DeletionResultEntry>, Vec<crate::spacetimedsl::delete::DeletionResultEntry>>
            };
        }
        OneOrMultiple::Multiple => {
            doc_comment = format!(
                "Execute On Delete Strategies of all referencing tables after multiple rows of the referenced table `{singular_table_name}` were deleted."
            );
            arg_name = format_ident!("primary_key_values_of_rows_to_delete");
            function_args.push(SpacetimeDSLArg {
                is_option: false,
                arg_name: arg_name.clone(),
                arg_type: SpacetimeDSLArgType::Normal(quote! {
                    &'a [#primary_key_column_type]
                }),
            });
            return_type = quote! {
                Result<
                    std::collections::HashMap<&'a #primary_key_column_type, Vec<crate::spacetimedsl::delete::DeletionResultEntry>>,
                    std::collections::HashMap<&'a #primary_key_column_type, Vec<crate::spacetimedsl::delete::DeletionResultEntry>>
                >
            };
        }
    };

    let create_entries = match one_or_multiple {
        OneOrMultiple::One => {
            quote! {
                let mut entries = vec![];
            }
        }
        OneOrMultiple::Multiple => {
            quote! {
                let mut entries = std::collections::HashMap::new();
                for primary_key_value_of_a_row_to_delete in primary_key_values_of_rows_to_delete {
                    entries.insert(primary_key_value_of_a_row_to_delete, vec![]);
                }
            }
        }
    };

    let mut compile_error_check_usages = vec![];

    let mut strategy_calls = vec![];

    for referencing_table in &spacetimedsl_table.referencing_tables {
        let referencing_table_name = &referencing_table.table_name;

        let referencing_table_name_pascal_case = format_ident!(
            "{}",
            RenameRule::PascalCase.apply_to_field(referencing_table_name.to_string())
        );

        let referencing_table_path = &referencing_table.path;

        let compile_error_check =
            get_referenced_table_compile_error_check(singular_table_name, referencing_table_name);
        spacetimedsl_table
            .compile_error_checks
            .insert(compile_error_check.clone());

        let compile_error_check =
            get_referencing_table_compile_error_check(referencing_table_name, singular_table_name);
        compile_error_check_usages.push(quote! {
            use #referencing_table_path::#compile_error_check;
        });

        let referencing_table_trait_name = get_referencing_table_trait_name(
            one_or_multiple,
            &referencing_table_name_pascal_case,
            &singular_table_name_pascal_case,
        );

        let referencing_table_function_name = get_referencing_table_function_name(
            one_or_multiple,
            referencing_table_name,
            singular_table_name,
        );

        let _ = &referencing_table_trait_name; // trait no longer generated; inherent impl used instead
        strategy_calls.push(
            match one_or_multiple {
                OneOrMultiple::One => {
                    quote! {
                        match crate::spacetimedsl::internal::DSLInternals::#referencing_table_function_name(dsl, &strategy, #arg_name) {
                            Err(mut child_entries) => {
                                entries.append(&mut child_entries);

                                error = true;
                            },
                            Ok(mut child_entries) => {
                                entries.append(&mut child_entries);
                            },
                        };
                    }
                },
                OneOrMultiple::Multiple => {
                    quote! {
                        match crate::spacetimedsl::internal::DSLInternals::#referencing_table_function_name(dsl, &strategy, #arg_name) {
                            Err(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                                    entries.get_mut(&primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in entries.")).append(&mut child_entries);
                                }

                                error = true;
                            },
                            Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                                    entries.get_mut(&primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in entries.")).append(&mut child_entries);
                                }
                            },
                        };
                    }
                },
            }
        );
    }

    let function_impl = quote! {
        #(#compile_error_check_usages)*

        #create_entries

        let mut error = false;

        #(#strategy_calls)*

        match error {
            false => Ok(entries),
            true => Err(entries),
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        additional_paths_to_use,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
        read_context_compatible: false,
    }
}

fn for_foreign_key(
    one_or_multiple: &OneOrMultiple,
    has_referenced_bys: bool,
    spacetimedb_table: &SpacetimeDBTable,
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &Vec<&&Column>,
    primary_key_column: &InternalColumn,
    spacetimedsl_table: &mut SpacetimeDSLTable,
) -> SpacetimeDSLMethod {
    let first_foreign_key_column = columns_with_foreign_key
        .first()
        .expect("there should be a column with foreign key");

    let referenced_table_path = first_foreign_key_column
        .spacetimedsl_column
        .foreign_key
        .as_ref()
        .expect("Should have foreign key")
        .path
        .to_token_stream();

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
            // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 If Option is supported, the type of the primary key values needs to be without option and it's allowed to have both, option and non-option columns. There is already a function to remove option from the type representation, search for `Option <`` in the code.
            panic!(
                "All foreign key columns which reference the same primary key of another table should have the same type"
            );
        }

        if column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .expect("should have a foreign key")
            .path
            .to_token_stream()
            .to_string()
            .ne(&referenced_table_path.to_string())
        {
            panic!(
                "All foreign key columns which reference the same primary key of another table should have the same path"
            );
        }

        let on_delete_strategy = &column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .unwrap_or_else(|| {
                panic!(
                    "the column {} should have a foreign key",
                    column_with_foreign_key.rust_field.name
                )
            })
            .on_delete_strategy;

        if !columns_by_on_delete_strategies.contains_key(on_delete_strategy) {
            columns_by_on_delete_strategies.insert(on_delete_strategy, vec![]);
        }

        columns_by_on_delete_strategies
            .get_mut(on_delete_strategy)
            .expect("The key OnDeleteStrategy should exist!")
            .push(column_with_foreign_key);
    }

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
        one_or_multiple,
        &singular_table_name_pascal_case,
        &referenced_table_name_pascal_case,
    );

    let function_name = get_referencing_table_function_name(
        one_or_multiple,
        singular_table_name,
        &referenced_table_name,
    );

    let additional_paths_to_use = vec![];
    let mut function_args = vec![
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("dsl"),
            arg_type: SpacetimeDSLArgType::Normal(quote! { &crate::spacetimedsl::DSL<'_, T> }),
        },
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("strategy"),
            arg_type: SpacetimeDSLArgType::Normal(quote! { &crate::spacetimedsl::delete::OnDeleteStrategy }),
        },
    ];

    let return_type;

    let arg_name;

    match one_or_multiple {
        OneOrMultiple::One => {
            doc_comment = format!(
                "Execute On Delete Strategies of the referencing table `{singular_table_name}` after one row of the referenced table `{referenced_table_name}` was deleted."
            );
            arg_name = format_ident!("primary_key_value_of_a_row_of_another_table_to_delete");
            function_args.push(SpacetimeDSLArg {
                is_option: false,
                arg_name: arg_name.clone(),
                arg_type: SpacetimeDSLArgType::Normal(
                    quote! { &#referenced_table_primary_key_column_type },
                ),
            });
            return_type = quote! {
                Result<Vec<crate::spacetimedsl::delete::DeletionResultEntry>, Vec<crate::spacetimedsl::delete::DeletionResultEntry>>
            };
        }
        OneOrMultiple::Multiple => {
            doc_comment = format!(
                "Execute On Delete Strategies of the referencing table `{singular_table_name}` after multiple rows of the referenced table `{referenced_table_name}` were deleted."
            );
            arg_name = format_ident!("primary_key_values_of_rows_of_another_table_to_delete");
            function_args.push(SpacetimeDSLArg {
                is_option: false,
                arg_name: arg_name.clone(),
                arg_type: SpacetimeDSLArgType::Normal(quote! {
                    &'a [#referenced_table_primary_key_column_type]
                }),
            });
            return_type = quote! {
                Result<
                    std::collections::HashMap<&'a #referenced_table_primary_key_column_type, Vec<crate::spacetimedsl::delete::DeletionResultEntry>>,
                    std::collections::HashMap<&'a #referenced_table_primary_key_column_type, Vec<crate::spacetimedsl::delete::DeletionResultEntry>>
                >
            };
        }
    };

    let create_data_structure_for_child_entries = match one_or_multiple {
        OneOrMultiple::One => {
            quote! {
                let mut entries = vec![];
            }
        }
        OneOrMultiple::Multiple => {
            quote! {
                let mut entries = std::collections::HashMap::new();
                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    entries.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
                }
            }
        }
    };

    let mut strategy_implementations = HashMap::new();

    for on_delete_strategy in OnDeleteStrategy::iter() {
        strategy_implementations.insert(on_delete_strategy.clone(), TokenStream::default());
    }

    for (on_delete_strategy, columns_by_on_delete_strategy) in columns_by_on_delete_strategies {
        strategy_implementations.insert(
            on_delete_strategy.clone(),
            get_on_delete_strategy_implementation(
                spacetimedsl_table,
                has_referenced_bys,
                singular_table_name,
                on_delete_strategy,
                columns_by_on_delete_strategy,
                one_or_multiple,
                primary_key_column,
            ),
        );
    }

    let strategy_implementations = strategy_implementations
        .iter()
        .map(|(on_delete_strategy, implementation)| {
            quote! {
                #on_delete_strategy => {
                    #implementation
                },
            }
        })
        .collect_vec();

    let compile_error_check =
        get_referencing_table_compile_error_check(singular_table_name, &referenced_table_name);

    spacetimedsl_table
        .compile_error_checks
        .insert(compile_error_check.clone());

    let compile_error_check =
        get_referenced_table_compile_error_check(&referenced_table_name, singular_table_name);

    let compile_error_check_usage = quote! {
        use #referenced_table_path::#compile_error_check;
    };

    let function_impl = quote! {
        #compile_error_check_usage

        use ::spacetimedsl::itertools::Itertools;
        #create_data_structure_for_child_entries

        let mut error = false;

        match &strategy {
            #(#strategy_implementations)*
        };

        match error {
            false => Ok(entries),
            true => Err(entries),
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        additional_paths_to_use,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
        read_context_compatible: false,
    }
}

fn get_on_delete_strategy_implementation(
    spacetimedsl_table: &SpacetimeDSLTable,
    has_referenced_bys: bool,
    singular_table_name: &Ident,
    on_delete_strategy: &OnDeleteStrategy,
    columns_by_on_delete_strategy: Vec<&&&Column>,
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> TokenStream {
    let spacetimedb_call_prefix = quote! {
        dsl
            .db
            .#singular_table_name()
    };

    let primary_key_column_name = &primary_key_column.rust_field_name;

    let singular_table_name_as_string = singular_table_name.to_string();

    let strategy_before_all = quote! {};
    let mut strategy_for_before_hook = TokenStream::default();
    let mut strategy_for_after_hook = TokenStream::default();

    let mut strategy_for_referenced_by = TokenStream::default();

    let mut strategy_by_column = vec![];
    let mut strategy_after_all = TokenStream::default();

    let is_singleton = spacetimedsl_table.is_singleton;

    for column in &columns_by_on_delete_strategy {
        let column_name = &column.rust_field.name;
        let column_name_as_string = column_name.to_string();

        // Singletons don't have indices on FK columns; use .id().find(&0u8) instead
        let is_unique_index = if is_singleton {
            true // Singleton has at most 1 row, treat as unique
        } else {
            column
                .spacetimedb_column
                .single_column_index
                .as_ref()
                .expect("Index should exist")
                .is_unique
        };

        let row_finder = if is_singleton {
            // For singletons, find the single row by PK and check FK column manually
            quote! {
                #spacetimedb_call_prefix.id().find(&0u8).filter(|row| row.#column_name == *primary_key_value_of_a_row_of_another_table_to_delete)
            }
        } else {
            match is_unique_index {
                true => {
                    quote! {
                        #spacetimedb_call_prefix.#column_name().find(primary_key_value_of_a_row_of_another_table_to_delete)
                    }
                }
                false => {
                    quote! {
                        #spacetimedb_call_prefix.#column_name().filter(primary_key_value_of_a_row_of_another_table_to_delete)
                    }
                }
            }
        };

        let row_value_format = if is_singleton {
            // Singleton PK is u8(0) with no wrapper type
            quote! { "0".to_string() }
        } else {
            let wrapper_type_struct_name_or_path = match primary_key_column
                .spacetimedsl_column_wrapper_type
                .as_ref()
                .expect("Wrapper Type should exist")
            {
                WrapperType::Created(wrap) => wrap.wrapper_struct_name.to_token_stream(),
                WrapperType::Used(wrapped) => wrapped.wrapper_struct_name_or_path.to_token_stream(),
            };
            quote! { format!("{}", #wrapper_type_struct_name_or_path::new(#primary_key_column_name.clone())) }
        };

        let create_entry = quote! {
            crate::spacetimedsl::delete::DeletionResultEntry {
                table_name: #singular_table_name_as_string.into(),
                column_name: #column_name_as_string.into(),
                strategy: #on_delete_strategy,
                row_value: #row_value_format.into(),
                child_entries,
            }
        };

        let create_entry_and_add_it_to_entries = match one_or_multiple {
            OneOrMultiple::One => {
                quote! {
                    entries.push(
                        #create_entry
                    );
                }
            }
            OneOrMultiple::Multiple => {
                quote! {
                    entries.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).expect(&format!("{primary_key_value_of_a_row_of_another_table_to_delete} should exist in entries.")).push(#create_entry);
                }
            }
        };

        match on_delete_strategy {
            OnDeleteStrategy::Error => {
                strategy_by_column.push(strategy_by_row(
                    false,
                    is_unique_index,
                    &row_finder,
                    quote! {
                        error = true;

                        let child_entries = vec![];
                        let #primary_key_column_name = &row.#primary_key_column_name;
                        #create_entry_and_add_it_to_entries
                    },
                ));
            }
            OnDeleteStrategy::Delete => {
                let before_delete_hook = match &spacetimedsl_table.hooks.before_delete {
                    None => TokenStream::default(),
                    Some(before_delete_hook) => {
                        let hook_trait_name = &before_delete_hook.trait_name;
                        let hook_function_name = &before_delete_hook.function_name;

                        strategy_for_before_hook = quote! {
                            use self::#hook_trait_name;
                        };

                        quote! {
                            if crate::spacetimedsl::DSLMethodHooks::#hook_function_name(&dsl, &row).is_err() {
                                error = true;
                                // FIXME: This results in the error supplied by the hook being ignored, we should propagate it back to the caller but that requires changing the function signature.
                                break;
                            }
                        }
                    }
                };

                let after_delete_hook = match &spacetimedsl_table.hooks.after_delete {
                    None => TokenStream::default(),
                    Some(after_delete_hook) => {
                        let hook_trait_name = &after_delete_hook.trait_name;
                        let hook_function_name = &after_delete_hook.function_name;

                        strategy_for_after_hook = quote! {
                            use self::#hook_trait_name;
                        };

                        quote! {
                            if crate::spacetimedsl::DSLMethodHooks::#hook_function_name(&dsl, &row).is_err() {
                                error = true;
                                // FIXME: This results in the error supplied by the hook being ignored, we should propagate it back to the caller but that requires changing the function signature.
                                break;
                            }
                        }
                    }
                };

                match has_referenced_bys {
                    false => strategy_by_column.push(strategy_by_row(
                        false,
                        is_unique_index,
                        &row_finder,
                        quote! {
                            let child_entries = vec![];
                            let #primary_key_column_name = &row.#primary_key_column_name;
                            #create_entry_and_add_it_to_entries

                            #before_delete_hook

                            #spacetimedb_call_prefix
                                .#primary_key_column_name()
                                .delete(row.#primary_key_column_name);

                            #after_delete_hook
                        },
                    )),
                    true => {
                        let format_str = format!(
                            "{primary_key_column_name} should exist in child_entries_by_primary_key_value_of_row_to_delete."
                        );
                        let create_entries_and_add_them_to_entries = quote! {
                            for (primary_key_value_of_a_row_of_another_table_to_delete, primary_key_values_of_rows_to_delete) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                                for #primary_key_column_name in &primary_key_values_of_rows_to_delete {
                                    let child_entries = child_entries_by_primary_key_value_of_row_to_delete.remove(&#primary_key_column_name).expect(&#format_str);
                                    #create_entry_and_add_it_to_entries
                                }
                            }
                        };

                        let on_error_handler = quote! {
                            #create_entries_and_add_them_to_entries
                            return Err(entries);
                        };

                        let error_strategy =
                            get_referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::Error,
                                &on_error_handler,
                            );

                        let delete_strategy =
                            get_referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::Delete,
                                &on_error_handler,
                            );

                        /*
                        let set_none_strategy =
                            get_referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                &singular_table_name_as_string,
                                OnDeleteStrategy::SetNone,
                                &on_error_handler
                            );
                        */

                        let set_zero_strategy =
                            get_referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::SetZero,
                                &on_error_handler,
                            );

                        let ignore_strategy =
                            get_referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::Ignore,
                                &on_error_handler,
                            );

                        strategy_for_referenced_by = quote! {
                            let mut child_entries_by_primary_key_value_of_row_to_delete = std::collections::HashMap::new();
                            let mut row_to_delete_by_primary_key_value = std::collections::HashMap::new();
                            let mut primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete = std::collections::HashMap::new();
                        };

                        match one_or_multiple {
                            OneOrMultiple::One => strategy_for_referenced_by.append_all(quote! {
                                primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
                            }),
                            OneOrMultiple::Multiple => strategy_for_referenced_by.append_all(quote! {
                                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                                    primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
                                }
                            }),
                        };

                        let strategy_for_each_row = quote! {
                            if !child_entries_by_primary_key_value_of_row_to_delete.contains_key(&row.#primary_key_column_name) {
                                primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).expect(&format!("{primary_key_value_of_a_row_of_another_table_to_delete} should exist in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.")).push(row.#primary_key_column_name);
                                child_entries_by_primary_key_value_of_row_to_delete.insert(row.#primary_key_column_name, vec![]);
                            row_to_delete_by_primary_key_value.insert(row.#primary_key_column_name, row);
                            }
                        };

                        let delete_many_impl = quote! {
                            for #primary_key_column_name in &primary_key_values_of_rows_to_delete {
                                let row = row_to_delete_by_primary_key_value
                                    .get(#primary_key_column_name)
                                    .expect("Should exist");

                                #before_delete_hook

                                if !#spacetimedb_call_prefix
                                    .#primary_key_column_name()
                                    .delete(#primary_key_column_name) {
                                        #on_error_handler
                                    }

                                #after_delete_hook
                            }
                        };

                        strategy_after_all = quote! {
                            let primary_key_values_of_rows_to_delete = child_entries_by_primary_key_value_of_row_to_delete.keys().cloned().collect_vec();

                            #error_strategy

                            #delete_many_impl

                            #delete_strategy

                            //TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 #set_none_strategy

                            #set_zero_strategy

                            #ignore_strategy

                            #create_entries_and_add_them_to_entries
                        };

                        strategy_by_column.push(strategy_by_row(
                            false,
                            is_unique_index,
                            &row_finder,
                            strategy_for_each_row,
                        ));
                    }
                };
            }
            OnDeleteStrategy::SetZero => {
                strategy_by_column.push(strategy_by_row(
                    true,
                    is_unique_index,
                    &row_finder,
                    quote! {
                        row.#column_name = 0;

                        let child_entries = vec![];
                        let #primary_key_column_name = &row.#primary_key_column_name;
                        #create_entry_and_add_it_to_entries

                        // FIXME: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/60 try_update instead of update and on error return Err(crate::spacetimedsl::error::SpacetimeDSLError);
                        #spacetimedb_call_prefix.#primary_key_column_name().update(row);
                    },
                ));
            }
            OnDeleteStrategy::Ignore => {
                strategy_by_column.push(strategy_by_row(
                    false,
                    is_unique_index,
                    &row_finder,
                    quote! {
                        let child_entries = vec![];
                        let #primary_key_column_name = &row.#primary_key_column_name;
                        #create_entry_and_add_it_to_entries
                    },
                ));
            }
        };
    }

    match one_or_multiple {
        OneOrMultiple::One => quote! {
            #strategy_before_all
            #strategy_for_before_hook
            #strategy_for_after_hook
            #strategy_for_referenced_by

            #(#strategy_by_column)*

            #strategy_after_all
        },
        OneOrMultiple::Multiple => quote! {
            #strategy_before_all
            #strategy_for_before_hook
            #strategy_for_after_hook
            #strategy_for_referenced_by

            for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                #(#strategy_by_column)*
            }

            #strategy_after_all
        },
    }
}

fn strategy_by_row(
    mut_row: bool,
    is_unique_index: bool,
    row_finder: &TokenStream,
    strategy_by_row: TokenStream,
) -> TokenStream {
    let row_or_mut_row = match mut_row {
        true => quote! {
            mut row
        },
        false => quote! {
            row
        },
    };

    match is_unique_index {
        true => quote! {
            match #row_finder {
                None => {}
                Some(#row_or_mut_row) => {
                    #strategy_by_row
                }
            };
        },
        false => quote! {
            for #row_or_mut_row in #row_finder {
                #strategy_by_row
            }
        },
    }
}

fn get_referenced_table_function_call_for_strategy_implementation(
    singular_table_name: &Ident,
    on_delete_strategy: OnDeleteStrategy,
    on_error_handler: &TokenStream,
) -> TokenStream {
    let referenced_table_function_name =
        get_referenced_table_function_name(&OneOrMultiple::Multiple, singular_table_name);

    quote! {
        match crate::spacetimedsl::internal::DSLInternals::#referenced_table_function_name(dsl, #on_delete_strategy, &primary_key_values_of_rows_to_delete[..]) {
            Err(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                    child_entries_by_primary_key_value_of_row_to_delete.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in child_entries_by_primary_key_value_of_row_to_delete.")).append(&mut child_entries);
                }

                #on_error_handler
            },
            Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                    child_entries_by_primary_key_value_of_row_to_delete.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in child_entries_by_primary_key_value_of_row_to_delete.")).append(&mut child_entries);
                }
            }
        };
    }
}

fn get_referenced_table_compile_error_check(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

fn get_referencing_table_compile_error_check(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_referencing_the_{referenced_table_name}_table"
    )
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
