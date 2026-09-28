use {
    super::{
        context::{MethodGenerationContext, TableContributions},
        hook_call::hook_tokens,
        message, naming,
        reference_integrity::{
            Action, multi_column_index_checks, reference_integrity_checks_on_create,
        },
    },
    crate::{
        api::{
            dsl::{
                auto_gen::UUIDVersion,
                hook::HookKind,
                method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
                soft_delete::SoftDeleteMarkerKind,
                table::{CreateDSLMethodArg, SpacetimeDSLTable},
                wrapper::WrapperType,
            },
            runtime, spacetimedb,
        },
        internal::{
            column::{ColumnTypeKind, InternalColumn},
            dsl::{
                one_or_multiple::OneOrMultiple, singleton,
                wrapper::map_wrapper_type_option_to_wrapped_type_option,
            },
        },
    },
    itertools::Itertools,
    proc_macro2::TokenStream,
    quote::{ToTokens, format_ident, quote},
};

/// The pieces the Create method needs from one column: the argument it contributes to the
/// `Create<Table>` struct, the mapper that unwraps an optional wrapper, the `let` binding
/// that feeds the row constructor, and the name that binding introduces.
struct CreateMethodColumnParts {
    arg: Option<SpacetimeDSLArg>,
    wrapper_option_mapper: Option<TokenStream>,
    constructor_arg: Option<TokenStream>,
    constructor_arg_name: TokenStream,
}

/// What a column is to `create_<table>`: filled in by the DSL, or asked of the caller, and
/// in which shape. `upsert_<singleton>` reads the timestamp roles from it too.
#[derive(Clone, Copy)]
pub(super) enum CreateColumnRole {
    /// The primary key the `derive` crate injects into a singleton.
    SingletonPrimaryKey,
    /// An `#[auto_gen]` column, generated through its wrapper.
    GeneratedUuid(UUIDVersion),
    /// An `#[auto_inc]` column, which SpacetimeDB fills in.
    AutoIncrement,
    /// The column with the `set_on_create` role.
    SetOnCreate { optional: bool },
    /// The column with the `set_on_update` role.
    SetOnUpdate { optional: bool },
    /// The soft-delete marker, which starts out unmarked.
    SoftDeleteMarker(SoftDeleteMarkerKind),
    /// A `#[create_wrapper]` column the caller supplies as the wrapped type.
    CreatedWrapper,
    /// A `#[use_wrapper(...)]` column the caller supplies as the wrapper.
    UsedWrapper { optional: bool },
    /// Any other column, supplied as its own type.
    Plain,
}

impl CreateColumnRole {
    /// The first of the roles, in the order they are declared, that the column has.
    pub(super) fn of(
        spacetimedsl_table: &SpacetimeDSLTable,
        internal_column: &InternalColumn,
    ) -> CreateColumnRole {
        let column_name = &internal_column.rust_field_name;
        let optional = internal_column.rust_field_type_kind == ColumnTypeKind::Optional;
        let names_this_column =
            |role_column_name: &Option<syn::Ident>| role_column_name.as_ref() == Some(column_name);

        if spacetimedsl_table.is_singleton()
            && singleton::is_primary_key_column(
                column_name,
                &internal_column.rust_field_type_name_or_path,
            )
        {
            return CreateColumnRole::SingletonPrimaryKey;
        }

        if let Some(uuid_version) = internal_column.spacetimedsl_column_auto_generated_uuid_version
        {
            return CreateColumnRole::GeneratedUuid(uuid_version);
        }

        if internal_column.spacetimedb_column_is_auto_inc {
            return CreateColumnRole::AutoIncrement;
        }

        if names_this_column(&spacetimedsl_table.on_insert_set_current_timestamp_column_name) {
            return CreateColumnRole::SetOnCreate { optional };
        }

        if names_this_column(&spacetimedsl_table.on_update_set_current_timestamp_column_name) {
            return CreateColumnRole::SetOnUpdate { optional };
        }

        if let Some(marker) = &spacetimedsl_table.soft_delete_marker
            && marker.column_name == *column_name
        {
            return CreateColumnRole::SoftDeleteMarker(marker.kind);
        }

        match &internal_column.spacetimedsl_column_wrapper_type {
            Some(WrapperType::Created(_)) => CreateColumnRole::CreatedWrapper,
            Some(WrapperType::Used(_)) => CreateColumnRole::UsedWrapper {
                optional: internal_column.spacetimedsl_column_is_option,
            },
            None => CreateColumnRole::Plain,
        }
    }
}

fn create_method_column_parts(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_column: &InternalColumn,
) -> CreateMethodColumnParts {
    let singular_table_name = &internal_column.spacetimedb_table_singular_name;
    let column_name = &internal_column.rust_field_name;
    let column_type = &internal_column.rust_field_type_name_or_path;

    // A column the DSL fills in: no argument, only the binding.
    let filled_in = |value: TokenStream| CreateMethodColumnParts {
        arg: None,
        wrapper_option_mapper: None,
        constructor_arg: Some(quote! { let #column_name = #value; }),
        constructor_arg_name: quote! { #column_name },
    };

    // A column the caller supplies: its argument, read back into the binding as `value`.
    let supplied = |is_option: bool, arg_type: SpacetimeDSLArgType, value: TokenStream| {
        CreateMethodColumnParts {
            arg: Some(SpacetimeDSLArg {
                is_option,
                arg_name: column_name.clone(),
                arg_type,
            }),
            wrapper_option_mapper: None,
            constructor_arg: Some(quote! { let #column_name = #value; }),
            constructor_arg_name: quote! { #column_name },
        }
    };

    let wrapper_type = || {
        internal_column
            .spacetimedsl_column_wrapper_type
            .as_ref()
            .expect("a wrapper role is only given to a column with a wrapper")
    };

    let current_timestamp = || runtime::current_timestamp(&quote! { self });

    match CreateColumnRole::of(spacetimedsl_table, internal_column) {
        CreateColumnRole::SingletonPrimaryKey => {
            filled_in(singleton::primary_key_value().to_token_stream())
        }
        CreateColumnRole::GeneratedUuid(uuid_version) => {
            let wrapper_path = wrapper_type().wrapper_path();
            let wrapper_constructor_name = uuid_version.wrapper_constructor_name();

            filled_in(quote! { #wrapper_path::#wrapper_constructor_name(self)?.value() })
        }
        CreateColumnRole::AutoIncrement => filled_in(quote! { #column_type::default() }),
        CreateColumnRole::SetOnCreate { .. } => filled_in(current_timestamp()),
        CreateColumnRole::SetOnUpdate { optional: true } => filled_in(quote! { None }),
        CreateColumnRole::SetOnUpdate { optional: false } => filled_in(current_timestamp()),
        CreateColumnRole::SoftDeleteMarker(SoftDeleteMarkerKind::Flag) => {
            filled_in(quote! { false })
        }
        CreateColumnRole::SoftDeleteMarker(SoftDeleteMarkerKind::Timestamp) => {
            filled_in(quote! { None })
        }
        CreateColumnRole::CreatedWrapper => supplied(
            false,
            SpacetimeDSLArgType::Normal(wrapper_type().wrapped_type().to_token_stream()),
            quote! { #singular_table_name.#column_name },
        ),
        CreateColumnRole::UsedWrapper { optional: true } => {
            let wrapper_type = wrapper_type();
            let wrapper_path = wrapper_type.wrapper_path();

            CreateMethodColumnParts {
                wrapper_option_mapper: Some(map_wrapper_type_option_to_wrapped_type_option(
                    column_name,
                    &wrapper_path,
                )),
                ..supplied(
                    true,
                    SpacetimeDSLArgType::Wrapped {
                        wrapped_type: wrapper_type.wrapped_type().to_token_stream(),
                        actual_type: quote! { Option<#wrapper_path> },
                    },
                    quote! { #singular_table_name.#column_name },
                )
            }
        }
        CreateColumnRole::UsedWrapper { optional: false } => {
            let wrapper_type = wrapper_type();
            let wrapper_path = wrapper_type.wrapper_path();

            supplied(
                false,
                SpacetimeDSLArgType::Wrapped {
                    wrapped_type: wrapper_type.wrapped_type().to_token_stream(),
                    actual_type: quote! { #wrapper_path },
                },
                quote! { #singular_table_name.#column_name.value() },
            )
        }
        CreateColumnRole::Plain => supplied(
            internal_column.spacetimedsl_column_is_option,
            SpacetimeDSLArgType::Normal(quote! { #column_type }),
            quote! { #singular_table_name.#column_name },
        ),
    }
}

/// `create_<table>`: insert one row, built from the columns the caller has to supply.
///
/// This is the only generator that contributes something on the table: the argument struct it
/// invents when the table has more than zero columns to ask for.
pub fn for_create(context: &MethodGenerationContext) -> (SpacetimeDSLMethod, TableContributions) {
    let MethodGenerationContext {
        spacetimedb_table,
        spacetimedsl_table,
        internal_columns,
        struct_name,
        singular_table_name,
        singular_table_name_as_string: _,
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
        let method_arg_name = naming::create_request_struct_name(singular_table_name);

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
                let member_type = member.arg_type.actual_type();
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
        spacetimedsl_table.hooks.get(HookKind::BEFORE_INSERT),
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
        spacetimedsl_table.hooks.get(HookKind::AFTER_INSERT),
        |hook_function_name| {
            let hook_call =
                runtime::dsl_method_hooks_call(hook_function_name, &quote! { self, &entity });

            quote! {
                #hook_call?;
            }
        },
    );

    let insert = insert_and_map_errors(context, &after_insert_hook);

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

            #insert
        },
        // Inserting writes a row.
        read_context_compatible: false,
    };

    (method, contributions)
}

/// The `try_insert` of the row bound to the table's singular name, with SpacetimeDB's insert
/// errors mapped to `SpacetimeDSLError` and `after_insert_hook` run on success, shared by
/// `create_<table>` and the insert path of `upsert_<singleton>`.
pub(super) fn insert_and_map_errors(
    context: &MethodGenerationContext,
    after_insert_hook: &TokenStream,
) -> TokenStream {
    let MethodGenerationContext {
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    // The row does not exist yet, so the message renders the whole struct rather than
    // naming the columns a lookup was made on.
    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        singular_table_name_as_string,
        &quote! { Create },
        &quote! { SpacetimeDB },
        &OneOrMultiple::One,
        &message::whole_row(singular_table_name),
    );
    let auto_inc_overflow_error = runtime::auto_inc_overflow(singular_table_name_as_string);
    let unique_constraint_violation =
        spacetimedb::try_insert_error(&quote! { UniqueConstraintViolation });
    let auto_inc_overflow = spacetimedb::try_insert_error(&quote! { AutoIncOverflow });

    quote! {
        match self
            .db()
            .#singular_table_name()
            .try_insert(#singular_table_name.clone()) {
            Ok(entity) => {
                #after_insert_hook

                Ok(entity)
            },
            Err(error) => match error {
                #unique_constraint_violation(_) => {
                    Err(#unique_constraint_violation_error)
                }
                #auto_inc_overflow(_) => {
                    Err(#auto_inc_overflow_error)
                }
            },
        }
    }
}
