use super::context::TableContributions;
use super::{
    context::MethodGenerationContext,
    hook_call::hook_tokens,
    reference_integrity::{
        Action, multi_column_index_checks, reference_integrity_checks_on_create,
    },
};
use crate::{
    api::{
        dsl::{
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            soft_delete::SoftDeleteMarkerKind,
            table::{CreateDSLMethodArg, SpacetimeDSLTable},
            wrapper::WrapperType,
        },
        runtime,
    },
    internal::{
        column::{ColumnTypeKind, InternalColumn},
        dsl::{
            one_or_multiple::OneOrMultiple, singleton,
            wrapper::map_wrapper_type_option_to_wrapped_type_option,
        },
    },
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

/// The pieces the Create method needs from one column: the argument it contributes to the
/// `Create<Table>` struct, the mapper that unwraps an optional wrapper, the `let` binding
/// that feeds the row constructor, and the name that binding introduces.
struct CreateMethodColumnParts {
    arg: Option<SpacetimeDSLArg>,
    wrapper_option_mapper: Option<TokenStream>,
    constructor_arg: Option<TokenStream>,
    constructor_arg_name: TokenStream,
}

fn create_method_column_parts(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_column: &InternalColumn,
) -> CreateMethodColumnParts {
    let mut arg = None;
    let mut wrapper_option_mapper = None;
    let mut constructor_arg = None;

    let singular_table_name = &internal_column.spacetimedb_table_singular_name;
    let column_name = &internal_column.rust_field_name;
    let constructor_arg_name = quote! { #column_name };

    let column_type = &internal_column.rust_field_type_name_or_path;

    // A singleton table does not ask for its injected primary key, it fills it in.
    if spacetimedsl_table.is_singleton()
        && singleton::is_primary_key_column(
            &internal_column.rust_field_name,
            &internal_column.rust_field_type_name_or_path,
        )
    {
        let primary_key_value = singleton::primary_key_value();

        return CreateMethodColumnParts {
            arg,
            wrapper_option_mapper,
            constructor_arg: Some(quote! {
                let #column_name = #primary_key_value;
            }),
            constructor_arg_name,
        };
    }

    if internal_column.spacetimedb_column_is_auto_inc {
        constructor_arg = Some(quote! {
            let #column_name = #column_type::default();
        });
    } else if let Some(column_name) =
        &spacetimedsl_table.on_insert_set_current_timestamp_column_name
        && { internal_column.rust_field_name.eq(column_name) }
    {
        let current_timestamp = runtime::current_timestamp(&quote! { self });
        constructor_arg = Some(quote! {
            let #column_name = #current_timestamp;
        });
    } else if let Some(column_name) =
        &spacetimedsl_table.on_update_set_current_timestamp_column_name
        && { internal_column.rust_field_name.eq(column_name) }
    {
        let timestamp_value = if internal_column.rust_field_type_kind == ColumnTypeKind::Optional {
            quote! { None }
        } else {
            runtime::current_timestamp(&quote! { self })
        };
        constructor_arg = Some(quote! {
            let #column_name = #timestamp_value;
        });
    } else if let Some(marker) = &spacetimedsl_table.soft_delete_marker
        && { internal_column.rust_field_name.eq(&marker.column_name) }
    {
        let initial_value = match marker.kind {
            SoftDeleteMarkerKind::Flag => quote! { false },
            SoftDeleteMarkerKind::Timestamp => quote! { None },
        };
        constructor_arg = Some(quote! {
            let #column_name = #initial_value;
        });
    }

    if constructor_arg.is_some() {
        return CreateMethodColumnParts {
            arg,
            wrapper_option_mapper,
            constructor_arg,
            constructor_arg_name,
        };
    }

    match &internal_column.spacetimedsl_column_wrapper_type {
        Some(wrapper_type) => match wrapper_type {
            WrapperType::Created(_) => {
                if internal_column.rust_field_type_kind == ColumnTypeKind::String {
                    arg = Some(SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(quote! { String }),
                    });
                } else {
                    arg = Some(SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(
                            WrapperType::map_to_wrapped_type(wrapper_type).to_token_stream(),
                        ),
                    });
                }

                constructor_arg = Some(quote! {
                    let #column_name = #singular_table_name.#column_name;
                });
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
                    wrapper_option_mapper = Some(map_wrapper_type_option_to_wrapped_type_option(
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

                    constructor_arg = Some(quote! {
                        let #column_name = #singular_table_name.#column_name.value();
                    });
                }
            }
        },
        None => {
            if internal_column.rust_field_type_kind == ColumnTypeKind::String {
                arg = Some(SpacetimeDSLArg {
                    is_option: false,
                    arg_name: column_name.clone(),
                    arg_type: SpacetimeDSLArgType::Normal(quote! { String }),
                });
            } else {
                arg = Some(SpacetimeDSLArg {
                    is_option: internal_column.spacetimedsl_column_is_option,
                    arg_name: column_name.clone(),
                    arg_type: SpacetimeDSLArgType::Normal(quote! { #column_type }),
                });
            }

            constructor_arg = Some(quote! {
                let #column_name = #singular_table_name.#column_name;
            });
        }
    };

    CreateMethodColumnParts {
        arg,
        wrapper_option_mapper,
        constructor_arg,
        constructor_arg_name,
    }
}

/// `create_<table>`: insert one row, built from the columns the caller has to supply.
///
/// This is the only generator that contributes something on the table: the argument struct it
/// invents when the table has more than zero columns to ask for.
pub(in crate::internal) fn for_create(
    context: &MethodGenerationContext,
) -> (SpacetimeDSLMethod, TableContributions) {
    let MethodGenerationContext {
        spacetimedb_table,
        spacetimedsl_table,
        internal_columns,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        singular_table_name_pascal_case,
        primary_key_column_name,
        field_name_for_found_value,
        ..
    } = context;

    let mut contributions = TableContributions::default();
    let mut method_args = vec![];

    let mut method_arg_members = vec![];

    let mut wrapper_type_option_to_wrapped_type_option_mappers = vec![];
    let mut constructor_args = vec![];
    let mut constructor_arg_names = vec![];

    for internal_column in *internal_columns {
        let CreateMethodColumnParts {
            arg,
            wrapper_option_mapper,
            constructor_arg,
            constructor_arg_name,
        } = create_method_column_parts(spacetimedsl_table, internal_column);

        if let Some(arg) = arg {
            method_arg_members.push(arg)
        }

        if let Some(wrapper_option_mapper) = wrapper_option_mapper {
            wrapper_type_option_to_wrapped_type_option_mappers.push(wrapper_option_mapper)
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

        contributions.create_dsl_method_arg = Some(CreateDSLMethodArg {
            struct_name: method_arg_name.clone(),
            struct_members: method_arg_members,
            struct_impl: quote! {
                pub struct #method_arg_name {
                    #(#method_arg_member_names_and_types),*
                }
            },
        });
    }

    // The row does not exist yet, so the message renders the whole struct rather than
    // naming the columns a lookup was made on.
    let column_names_and_row_values = format!("{{{{ {singular_table_name} : {{:?}} }}}}");

    let multi_column_index_checks = multi_column_index_checks(
        Action::Create,
        singular_table_name,
        spacetimedb_table,
        internal_columns,
        primary_key_column_name,
    );

    let use_itertools = if !multi_column_index_checks.is_empty() {
        runtime::itertools_import()
    } else {
        TokenStream::default()
    };

    let reference_integrity_checks =
        reference_integrity_checks_on_create(spacetimedb_table, internal_columns);

    let let_field_name_for_found_value =
        if multi_column_index_checks.is_empty() && reference_integrity_checks.is_empty() {
            TokenStream::default()
        } else {
            quote! {
                let mut #field_name_for_found_value: Option<#struct_name> = None;
            }
        };

    let before_insert_hook = hook_tokens(
        &spacetimedsl_table.hooks.before_insert,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, #singular_table_name },
            );

            quote! {
                let #singular_table_name = #hook_call?;
            }
        },
    );

    let after_insert_hook = hook_tokens(
        &spacetimedsl_table.hooks.after_insert,
        |hook_function_name| {
            let hook_call =
                runtime::dsl_method_hooks_call(hook_function_name, &quote! { self, &entity });

            quote! {
                #hook_call?;
            }
        },
    );

    // FIXME: Only show unique columns here
    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        singular_table_name_as_string,
        &quote! { Create },
        &quote! { SpacetimeDB },
        &OneOrMultiple::One,
        &quote! { format!(#column_names_and_row_values, #singular_table_name) },
    );
    let auto_inc_overflow_error = runtime::auto_inc_overflow(singular_table_name_as_string);

    let method = SpacetimeDSLMethod {
        doc_comment: format!("Create a row in the `{singular_table_name}` table."),
        method_name: format_ident!("create_{}", singular_table_name),
        method_args,
        return_type: runtime::error_result_type(struct_name),
        method_impl: quote! {
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
                .db()
                .#singular_table_name()
                .try_insert(#singular_table_name.clone()) { // FIXME: No clone?
                Ok(entity) => {
                    #after_insert_hook

                    Ok(entity)
                },
                Err(error) => match error {
                    spacetimedb::TryInsertError::UniqueConstraintViolation(_) => {
                        Err(#unique_constraint_violation_error)
                    }
                    spacetimedb::TryInsertError::AutoIncOverflow(_) => {
                        Err(#auto_inc_overflow_error)
                    }
                },
            }
        },
        read_context_compatible: false,
    };

    (method, contributions)
}
