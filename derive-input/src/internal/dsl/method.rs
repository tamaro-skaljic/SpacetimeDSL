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
            foreign_key::{ForeignKey, OnDeleteStrategy},
            hook::SpacetimeDSLMethodHook,
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            table::{CreateDSLMethodArg, SpacetimeDSLTable, SpacetimeDSLTableMethods},
            wrapper::WrapperType,
        },
        rust::{table::RustStruct, visibility::RustVisibility},
    },
    internal::{
        column::{ColumnTypeKind, InternalColumn},
        dsl::{
            generated_runtime as runtime, singleton,
            wrapper::map_wrapper_type_option_to_wrapped_type_option,
        },
    },
};
use ident_case::RenameRule;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use strum::IntoEnumIterator;
use syn::{Ident, parse_str};

/// The invariant `internal/dsl/column.rs` enforces: every primary key column except a singleton's injected `id: u8` carries a wrapper type.
const PRIMARY_KEY_WRAPPER_TYPE_INVARIANT: &str = "A primary key column must be accompanied by `#[create_wrapper]` or `#[use_wrapper(crate::path::to::MyIdType)]`";

#[derive(Debug)]
pub enum OneOrMultiple {
    One,
    Multiple,
}

impl quote::ToTokens for OneOrMultiple {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            OneOrMultiple::One => quote! { crate::spacetimedsl::error::OneOrMultiple::One },
            OneOrMultiple::Multiple => {
                quote! { crate::spacetimedsl::error::OneOrMultiple::Multiple }
            }
        };
        tokens.extend(variant);
    }
}

/// Everything every method generator needs, under one name.
///
/// The five references used to travel as a positional bundle through the entry points and
/// most helpers. Several are references to different-but-similar table types, so a
/// transposed argument compiled in some call shapes, and adding one more piece of shared
/// context meant editing every signature in the chain.
///
/// The derived names are resolved once here rather than in each generator.
/// `field_name_for_found_value` in particular was built by the same `format_ident!` in
/// four separate functions, which is a rule about a generated identifier that nothing kept
/// in step.
///
/// This is a plain data carrier. It must not grow generation methods, or it becomes a
/// second god object in place of the one this plan removes.
pub(in crate::internal) struct MethodGenerationContext<'a> {
    pub spacetimedb_table: &'a SpacetimeDBTable,
    pub spacetimedsl_table: &'a SpacetimeDSLTable,
    pub internal_columns: &'a [InternalColumn],
    pub primary_key_column: &'a InternalColumn,

    pub struct_name: Ident,
    pub singular_table_name: Ident,
    pub singular_table_name_as_string: String,
    pub singular_table_name_pascal_case: String,
    pub plural_table_name: Ident,
    pub primary_key_column_name: Ident,
    pub primary_key_column_name_as_string: String,
    /// The local the generated code binds the row it looked up to.
    pub field_name_for_found_value: Ident,
}

impl<'a> MethodGenerationContext<'a> {
    pub(in crate::internal) fn new(
        rust_struct: &'a RustStruct,
        spacetimedb_table: &'a SpacetimeDBTable,
        spacetimedsl_table: &'a SpacetimeDSLTable,
        internal_columns: &'a [InternalColumn],
        primary_key_column: &'a InternalColumn,
    ) -> MethodGenerationContext<'a> {
        let singular_table_name = spacetimedb_table.singular_name.clone();
        let primary_key_column_name = primary_key_column.rust_field_name.clone();

        MethodGenerationContext {
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
            primary_key_column,

            struct_name: rust_struct.name.clone(),
            singular_table_name_as_string: singular_table_name.to_string(),
            singular_table_name_pascal_case: RenameRule::PascalCase
                .apply_to_field(singular_table_name.to_string()),
            plural_table_name: spacetimedsl_table.plural_name.clone(),
            primary_key_column_name_as_string: primary_key_column_name.to_string(),
            field_name_for_found_value: format_ident!("the_same_or_another_{singular_table_name}"),
            singular_table_name,
            primary_key_column_name,
        }
    }
}

/// What a generator wants written onto the table, returned instead of written.
///
/// The generators are named for what they produce and they produce a method; the table
/// state they also need is part of their result rather than a side effect, so reordering
/// two generator calls cannot change the table. `SpacetimeDSLTableMethods::generate`
/// collects these and hands them to the one caller that owns the table.
#[derive(Default)]
pub(in crate::internal) struct GeneratedTableRecordings {
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    pub compile_error_checks: BTreeSet<Ident>,
}

impl GeneratedTableRecordings {
    fn merge(&mut self, other: GeneratedTableRecordings) {
        if let Some(create_dsl_method_arg) = other.create_dsl_method_arg {
            self.create_dsl_method_arg = Some(create_dsl_method_arg);
        }

        self.compile_error_checks.extend(other.compile_error_checks);
    }

    pub(in crate::internal) fn apply_to(self, spacetimedsl_table: &mut SpacetimeDSLTable) {
        if let Some(create_dsl_method_arg) = self.create_dsl_method_arg {
            spacetimedsl_table.create_dsl_method_arg = Some(create_dsl_method_arg);
        }

        spacetimedsl_table
            .compile_error_checks
            .extend(self.compile_error_checks);
    }
}

#[derive(PartialEq, strum::Display)]
enum Action {
    Create,
    Get,
    Update,
    Delete,
}

/// How the generated code binds the row it iterates over or matches on.
#[derive(Clone, Copy)]
enum RowBinding {
    Immutable,
    Mutable,
}

/// Whether the index the generated code looks a row up through yields at most one row.
#[derive(Clone, Copy)]
enum IndexUniqueness {
    Unique,
    NonUnique,
}

/// Whether any other table declares a foreign key referencing the table being generated.
#[derive(Clone, Copy)]
enum ReferencingTables {
    Present,
    Absent,
}

/// Which DSL methods an index earns.
///
/// A non-unique index yields many rows, so it earns `get_many` and `delete_many`. A
/// unique index yields at most one, so it earns `get_one_option` and `delete_one`, plus
/// `update` when it is the primary key. The `method(...)` flags suppress the delete and
/// update methods on top of that.
///
/// Single-column and multi-column indices read this rule from here, which is the point:
/// it used to be written out once for each, and the two copies disagreed about which
/// unique index may update a row.
fn column_methods_for(
    index: &Index,
    context: &MethodGenerationContext,
) -> SpacetimeDSLColumnMethods {
    let MethodGenerationContext {
        spacetimedsl_table,
        primary_key_column,
        ..
    } = context;

    let shape = IndexShape::of(index, context);

    match index.is_unique {
        false => SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
            get_many: for_get_many(&shape, context),
            delete_many: match spacetimedsl_table.has_delete_method {
                false => None,
                true => Some(for_delete_many(&shape, context)),
            },
        }),
        true => {
            // Only the primary key can update a row: SpacetimeDB's `update` lives on the
            // primary key index, and no other index implements `PrimaryKey`.
            let is_primary_key_index = match &index.index_type {
                IndexType::BTreeSingleColumn { column }
                | IndexType::HashSingleColumn { column }
                | IndexType::Direct { column } => *column == primary_key_column.rust_field_name,
                // A primary key is one column, so a multi-column index is never it.
                IndexType::BTreeMultiColumn { .. } | IndexType::HashMultiColumn { .. } => false,
            };

            SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                get_one_option: for_get_one(&shape, context),
                update: match spacetimedsl_table.has_update_method && is_primary_key_index {
                    false => None,
                    true => Some(for_update(&shape, context)),
                },
                delete_one: match spacetimedsl_table.has_delete_method {
                    false => None,
                    true => Some(for_delete_one(&shape, context)),
                },
            })
        }
    }
}

impl SpacetimeDSLColumnMethods {
    pub(in crate::internal) fn map(
        context: &MethodGenerationContext,
        spacetimedb_column: &SpacetimeDBColumn,
    ) -> Option<SpacetimeDSLColumnMethods> {
        let index = match &spacetimedb_column.single_column_index {
            None => {
                return None;
            }
            Some(index) => index,
        };

        // `internal/db/column.rs` rejects `#[index]` and `#[unique]` on a singleton's own
        // columns, so the only index a singleton reaches here with is its injected primary
        // key, which is unique.
        if context.spacetimedsl_table.is_singleton && !index.is_unique {
            return None;
        }

        Some(column_methods_for(index, context))
    }
}

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn generate(
        context: &MethodGenerationContext,
        columns: &[Column],
    ) -> syn::Result<(SpacetimeDSLTableMethods, GeneratedTableRecordings)> {
        let MethodGenerationContext {
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column,
            ..
        } = context;

        let is_singleton = spacetimedsl_table.is_singleton;

        let mut recordings = GeneratedTableRecordings::default();

        let (create, create_recordings) = for_create(context);
        recordings.merge(create_recordings);

        // A singleton holds one row, so iterating and counting have nothing to say.
        let get_all = match is_singleton {
            true => None,
            false => Some(for_get_all(context)),
        };

        let get_count = match is_singleton {
            true => None,
            false => Some(for_get_count(context)),
        };

        let execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted;
        let execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted;

        if spacetimedsl_table.referencing_tables.is_empty() {
            execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted =
                None;
            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted = None;
        } else {
            let (after_one_row, after_one_row_recordings) = for_referenced_by(
                &OneOrMultiple::One,
                spacetimedb_table,
                spacetimedsl_table,
                primary_key_column,
            );
            recordings.merge(after_one_row_recordings);
            execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted =
                Some(after_one_row);

            let (after_multiple_rows, after_multiple_rows_recordings) = for_referenced_by(
                &OneOrMultiple::Multiple,
                spacetimedb_table,
                spacetimedsl_table,
                primary_key_column,
            );
            recordings.merge(after_multiple_rows_recordings);
            execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted =
                Some(after_multiple_rows);
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
            let mut columns_with_foreign_keys_by_table = BTreeMap::new();

            columns_with_foreign_keys.iter().for_each(|c| {
                let name_of_another_table = &c
                    .spacetimedsl_column
                    .foreign_key
                    .as_ref()
                    .expect("The columns were just filtered to those that have a foreign key")
                    .table_name;

                if !columns_with_foreign_keys_by_table.contains_key(name_of_another_table) {
                    columns_with_foreign_keys_by_table.insert(name_of_another_table, vec![]);
                }

                columns_with_foreign_keys_by_table
                    .get_mut(name_of_another_table)
                    .expect("The entry was inserted above when it was missing")
                    .push(*c);
            });

            for (referenced_table_name, columns_with_foreign_key) in
                columns_with_foreign_keys_by_table
            {
                let referencing_tables = match spacetimedsl_table.referencing_tables.is_empty() {
                    true => ReferencingTables::Absent,
                    false => ReferencingTables::Present,
                };

                let (after_one_row, after_one_row_recordings) = for_foreign_key(
                    &OneOrMultiple::One,
                    referencing_tables,
                    spacetimedb_table,
                    referenced_table_name,
                    &columns_with_foreign_key,
                    primary_key_column,
                    spacetimedsl_table,
                )?;
                recordings.merge(after_one_row_recordings);
                execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted
                    .push(after_one_row);

                let (after_multiple_rows, after_multiple_rows_recordings) = for_foreign_key(
                    &OneOrMultiple::Multiple,
                    referencing_tables,
                    spacetimedb_table,
                    referenced_table_name,
                    &columns_with_foreign_key,
                    primary_key_column,
                    spacetimedsl_table,
                )?;
                recordings.merge(after_multiple_rows_recordings);
                execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted
                    .push(after_multiple_rows);
            }
        }

        let mut multi_column_indices = vec![];

        for multi_column_index in &spacetimedb_table.multi_column_indices {
            // `internal/db/column.rs` moves every single-column index onto its column, so
            // only genuinely multi-column indices reach here. It stops at the first index
            // per column, though, so a column carrying two single-column indices would
            // leak one into this list. Skip it rather than generate it from the wrong path.
            if !matches!(
                multi_column_index.index_type,
                IndexType::BTreeMultiColumn { .. } | IndexType::HashMultiColumn { .. }
            ) {
                continue;
            }

            multi_column_indices.push(column_methods_for(multi_column_index, context));
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

        Ok((methods, recordings))
    }
}

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
    if spacetimedsl_table.is_singleton
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
        constructor_arg = Some(quote! {
            let #column_name = self.ctx().timestamp()?;
        });
    } else if let Some(column_name) =
        &spacetimedsl_table.on_update_set_current_timestamp_column_name
        && { internal_column.rust_field_name.eq(column_name) }
    {
        let timestamp_value = if internal_column.rust_field_type_kind == ColumnTypeKind::Optional {
            quote! { None }
        } else {
            quote! { self.ctx().timestamp()? }
        };
        constructor_arg = Some(quote! {
            let #column_name = #timestamp_value;
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

/// The `let` binding the Update method's reference-integrity checks read a column's value
/// through. Unlike the Create path there is always one, and the wrapper handling the Create
/// path needs is irrelevant here, because Update reads the row rather than building it.
fn update_method_row_value_getter(internal_column: &InternalColumn) -> TokenStream {
    let singular_table_name = &internal_column.spacetimedb_table_singular_name;
    let column_name = &internal_column.rust_field_name;
    let getter_name = format_ident!("get_{column_name}");

    let is_string = internal_column.rust_field_type_kind == ColumnTypeKind::String;

    match &internal_column.spacetimedsl_column_wrapper_type {
        Some(WrapperType::Used(_)) if !internal_column.spacetimedsl_column_is_option => quote! {
            let #column_name = #singular_table_name.#getter_name().value();
        },
        Some(WrapperType::Created(_)) | None if is_string => quote! {
            let #column_name = #singular_table_name.#getter_name();
        },
        _ => quote! {
            let #column_name = #singular_table_name.#column_name;
        },
    }
}

/// `create_<table>`: insert one row, built from the columns the caller has to supply.
///
/// This is the only generator that records something on the table: the argument struct it
/// invents when the table has more than zero columns to ask for.
fn for_create(context: &MethodGenerationContext) -> (SpacetimeDSLMethod, GeneratedTableRecordings) {
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

    let mut recordings = GeneratedTableRecordings::default();
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

        recordings.create_dsl_method_arg = Some(CreateDSLMethodArg {
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
        quote! {
            use ::spacetimedsl::itertools::Itertools;
        }
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

    (method, recordings)
}

/// `get_all_<tables>`: iterate every row of the table.
fn for_get_all(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    SpacetimeDSLMethod {
        doc_comment: format!("Get all rows inside the `{singular_table_name}` table."),
        method_name: format_ident!("get_all_{}", plural_table_name),
        method_args: vec![],
        return_type: quote! {
            impl Iterator<Item = #struct_name>
        },
        method_impl: quote! {
            self
                .db()
                .#singular_table_name()
                .iter()
        },
        read_context_compatible: false,
    }
}

/// `count_of_all_<tables>`: how many rows the table holds.
fn for_get_count(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    SpacetimeDSLMethod {
        doc_comment: format!("Count all rows inside the `{singular_table_name}` table."),
        method_name: format_ident!("count_of_all_{}", plural_table_name),
        method_args: vec![],
        return_type: quote! {
            u64
        },
        method_impl: quote! {
            self
                .db()
                .#singular_table_name()
                .count()
        },
        read_context_compatible: true,
    }
}

/// `self.db().<table>().<index>()`, where every index-based body starts.
fn index_accessor(singular_table_name: &Ident, index_name: &Ident) -> TokenStream {
    quote! {
        self
            .db()
            .#singular_table_name()
            .#index_name()
    }
}

/// `get_<tables>_by_<index>`: iterate the rows an index matches.
fn for_get_many(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::Multiple, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    // A multi-column index is filtered by the tuple of its columns.
    let method_impl = match shape.is_multi_column {
        true => quote! {
            #(#wrapper_option_mappers)*

            #index_accessor
                .filter((#(#row_value_getters),*))
        },
        false => quote! {
            #(#wrapper_option_mappers)*

            #index_accessor
                .filter(#(#row_value_getters),*)
        },
    };

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Get a `{struct_name}` iterator that contains all rows in the `{singular_table_name}` table {described_as}."
        ),
        method_name: format_ident!("get_{plural_table_name}_by_{index_name}"),
        method_args,
        return_type: quote! {
            impl Iterator<Item = #struct_name>
        },
        method_impl,
        read_context_compatible: true,
    }
}

/// `delete_<tables>_by_<index>`: delete every row an index matches.
fn for_delete_many(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        primary_key_column,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        plural_table_name,
        primary_key_column_name,
        primary_key_column_name_as_string,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::Multiple, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    let let_index_name = match shape.is_multi_column {
        true => quote! {
            let #index_name = (#(#row_value_getters),*);
        },
        false => quote! {
            let #index_name = #(#row_value_getters),*;
        },
    };

    let empty_deletion_result = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::Multiple,
        &quote! { vec![] },
    );

    let impl_until_return_ok_on_is_empty = quote! {
        use ::spacetimedsl::itertools::Itertools;

        #(#wrapper_option_mappers)*

        #let_index_name

        let rows_to_delete: Vec<#struct_name> = #index_accessor
            .filter(#index_name)
            .collect();

        if rows_to_delete.is_empty() {
            return Ok(#empty_deletion_result);
        }
    };

    let wrapper_type_struct_name_or_path = primary_key_column
        .spacetimedsl_column_wrapper_type
        .as_ref()
        .expect(PRIMARY_KEY_WRAPPER_TYPE_INVARIANT)
        .struct_name_or_path_tokens();

    let deletion_result_entry_per_row = runtime::deletion_result_entry(
        singular_table_name_as_string,
        primary_key_column_name_as_string,
        &runtime::on_delete_strategy(&quote! { Delete }),
        &quote! {
            format!("{}", #wrapper_type_struct_name_or_path::new(row_to_delete.#primary_key_column_name.clone()))
        },
        &quote! { child_entries: vec![], },
    );

    let map_rows_to_delete_to_deletion_result_entries = quote! {
        let mut deletion_result_entries = std::collections::HashMap::new();

        for row_to_delete in &rows_to_delete {
            deletion_result_entries.insert(
                &row_to_delete.#primary_key_column_name,
                #deletion_result_entry_per_row
            );
        }
    };

    let before_delete_hook = hook_tokens(
        &spacetimedsl_table.hooks.before_delete,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, &row_to_delete },
            );

            quote! {
                for row_to_delete in &rows_to_delete {
                    #hook_call?;
                }
            }
        },
    );

    let after_delete_hook = hook_tokens(
        &spacetimedsl_table.hooks.after_delete,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, &row_to_delete },
            );

            quote! {
                for row_to_delete in &rows_to_delete {
                    #hook_call?;
                }
            }
        },
    );

    let count_mismatch_error = runtime::generic_error(&quote! {
        format!(
            "Delete Many Error: `count_of_rows_to_delete ( {} ) != ( {} ) count_of_deleted_rows`!",
            &count_of_rows_to_delete,
            &count_of_deleted_rows
        )
    });

    let delete_many_impl = quote! {
        let count_of_rows_to_delete: u64 = rows_to_delete
            .len()
            .try_into()
            .unwrap_or(u64::MAX);

        let count_of_deleted_rows = #index_accessor.delete(#index_name);

        if count_of_rows_to_delete.ne(&count_of_deleted_rows) {
            return Err(#count_mismatch_error);
        }
    };

    let deletion_result_from_entries = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::Multiple,
        &quote! { deletion_result_entries.into_values().collect_vec() },
    );

    let return_result_impl = quote! {
        return Ok(#deletion_result_from_entries);
    };

    let method_impl = if spacetimedsl_table.referencing_tables.is_empty() {
        quote! {
            #impl_until_return_ok_on_is_empty

            #map_rows_to_delete_to_deletion_result_entries

            #before_delete_hook

            #delete_many_impl

            #after_delete_hook

            #return_result_impl
        }
    } else {
        let unknown_error_after_state_change = runtime::generic_error(&quote! {
            format!("Delete Many Error: An unknown error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}")
        });

        let on_error_handler = quote! {
            let error = #deletion_result_from_entries;

            return Err(#unknown_error_after_state_change);
        };

        let reference_integrity_violation_on_delete_error =
            runtime::reference_integrity_violation_on_delete(&quote! { error });

        let error_strategy = get_referenced_table_function_call_for_dsl_method(
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Error,
            OneOrMultiple::Multiple,
            &quote! {
                let error = #deletion_result_from_entries;

                return Err(#reference_integrity_violation_on_delete_error);
            },
        );

        let delete_strategy = get_referenced_table_function_call_for_dsl_method(
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

        let set_zero_strategy = get_referenced_table_function_call_for_dsl_method(
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::SetZero,
            OneOrMultiple::Multiple,
            &on_error_handler,
        );

        let ignore_strategy = get_referenced_table_function_call_for_dsl_method(
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Ignore,
            OneOrMultiple::Multiple,
            &on_error_handler,
        );

        quote! {
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
        }
    };

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Try to delete all `{struct_name}` rows in the `{singular_table_name}` table {described_as}."
        ),
        method_name: format_ident!("delete_{plural_table_name}_by_{index_name}"),
        method_args,
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl,
        read_context_compatible: false,
    }
}

/// `get_<table>_by_<index>`: look one row up by a unique index.
fn for_get_one(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        field_name_for_found_value,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;
    let column_names_and_row_values = &shape.column_names_and_row_values;
    let unique_multi_column_index_hint = shape.unique_multi_column_hint;
    let is_singleton_pk = shape.is_singleton_primary_key;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::One, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    let method_impl = match shape.is_multi_column {
        true => {
            // FIXME: Row Value Getters of Wrapper Types shouldn't be `id.clone().into().value()`, they should be `let id = id.into();` at the method beginning and then `id.value()` anywhere else
            let multi_column_index_check = get_unique_multi_column_index_check(
                &Action::Get,
                singular_table_name,
                index_name,
                column_names_and_row_values,
                &row_value_getters,
            );

            let not_found_error = runtime::not_found_error(
                singular_table_name_as_string,
                &quote! {
                    format!(#column_names_and_row_values, #(#row_value_getters),*)
                },
            );

            quote! {
                #(#wrapper_option_mappers)*

                use ::spacetimedsl::itertools::Itertools;

                let mut #field_name_for_found_value: Option<#struct_name> = None;

                #multi_column_index_check

                match #field_name_for_found_value {
                    Some(#singular_table_name) => Ok(#singular_table_name),
                    None => {
                        return Err(#not_found_error);
                    }
                }
            }
        }
        false => {
            if is_singleton_pk {
                let primary_key = singleton::primary_key_ident();
                let primary_key_value = singleton::primary_key_value();
                let singleton_not_found_error = runtime::not_found_error(
                    singular_table_name_as_string,
                    &singleton::rendered_primary_key(),
                );

                quote! {
                    match self.db().#singular_table_name().#primary_key().find(&#primary_key_value) {
                        Some(#singular_table_name) => Ok(#singular_table_name),
                        None => return Err(#singleton_not_found_error)
                    }
                }
            } else {
                let not_found_error = runtime::not_found_error(
                    singular_table_name_as_string,
                    &quote! {
                        format!(#column_names_and_row_values, #(#row_value_getters),*)
                    },
                );

                quote! {
                    #(#wrapper_option_mappers)*

                    match #index_accessor.find(#(#row_value_getters),*) {
                        Some(#singular_table_name) => Ok(#singular_table_name),
                        None => return Err(#not_found_error)
                    }
                }
            }
        }
    };

    SpacetimeDSLMethod {
        doc_comment: match is_singleton_pk {
            true => format!(
                "Try to get the `{struct_name}` from the singleton `{singular_table_name}` table."
            ),
            false => format!(
                "{unique_multi_column_index_hint}\n\nTry to get a `{struct_name}` from the `{singular_table_name}` table {described_as}."
            ),
        },
        method_name: match is_singleton_pk {
            true => format_ident!("get_{singular_table_name}"),
            false => format_ident!("get_{singular_table_name}_by_{index_name}"),
        },
        method_args,
        return_type: runtime::error_result_type(struct_name),
        method_impl,
        read_context_compatible: true,
    }
}

/// `update_<table>_by_<index>`: write a row back over the one the index finds.
///
/// Only the primary key gets here. SpacetimeDB puts `update` on the primary key index
/// alone, so the index this takes is always single-column.
fn for_update(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedb_table,
        spacetimedsl_table,
        internal_columns,
        primary_key_column,
        struct_name,
        singular_table_name,
        primary_key_column_name,
        field_name_for_found_value,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;
    let unique_multi_column_index_hint = shape.unique_multi_column_hint;
    let is_singleton_pk = shape.is_singleton_primary_key;

    let method_args = vec![SpacetimeDSLArg {
        is_option: false,
        arg_name: singular_table_name.clone(),
        arg_type: SpacetimeDSLArgType::Normal(quote! { #struct_name }),
    }];

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
            row_value_getters.push(update_method_row_value_getter(internal_column));
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
                    panic!("The column {column_name} named by an on_update attribute must be one of this table's columns")
                });

            let timestamp_value = if on_update_set_current_timestamp_column.rust_field_type_kind
                == ColumnTypeKind::Optional
            {
                quote! { Some(self.ctx().timestamp()?) }
            } else {
                quote! { self.ctx().timestamp()? }
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

    let one_or_multiple = match shape.is_multi_column {
        false => OneOrMultiple::One,
        true => OneOrMultiple::Multiple,
    };

    let reference_integrity_checks = reference_integrity_checks_on_update(
        spacetimedb_table,
        internal_columns,
        &shape.column_names_and_row_values,
        &shape.index_columns,
        &one_or_multiple,
        primary_key_column,
    );

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

    // The found-value prelude has to run before the import, so this site
    // places both itself instead of taking them already joined.
    let (use_before_update_hook_trait, before_update_hook_call) = hook_use_and_call(
        &spacetimedsl_table.hooks.before_update,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! {
                    self,
                    #field_name_for_found_value.as_ref().unwrap(),
                    #singular_table_name
                },
            );

            quote! {
                let #singular_table_name = #hook_call?;
            }
        },
    );

    let before_update_hook = if before_update_hook_call.is_empty() {
        TokenStream::default()
    } else {
        quote! {
            if #field_name_for_found_value.is_none() {
                #field_name_for_found_value = Some(
                    self.db().#singular_table_name().#primary_key_column_name()
                        .find(#singular_table_name.#primary_key_column_name)
                        .expect("Row should exist for update")
                )
            }

            #use_before_update_hook_trait
            #before_update_hook_call
        }
    };

    let after_update_hook = hook_tokens(
        &spacetimedsl_table.hooks.after_update,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! {
                    self,
                    #field_name_for_found_value.as_ref().unwrap(),
                    &#singular_table_name
                },
            );

            quote! {
                #hook_call?;
            }
        },
    );

    let set_singleton_id_to_zero = if is_singleton_pk {
        let primary_key = singleton::primary_key_ident();
        let primary_key_value = singleton::primary_key_value();

        quote! { #singular_table_name.#primary_key = #primary_key_value; }
    } else {
        TokenStream::default()
    };

    SpacetimeDSLMethod {
        doc_comment: match is_singleton_pk {
            true => format!(
                "Try to update the `{struct_name}` row of the singleton `{singular_table_name}` table."
            ),
            false => format!(
                "{unique_multi_column_index_hint}\n\nTry to update a `{struct_name}` row of the `{singular_table_name}` table {described_as}."
            ),
        },
        method_name: match is_singleton_pk {
            true => format_ident!("update_{singular_table_name}"),
            false => format_ident!("update_{singular_table_name}_by_{index_name}"),
        },
        method_args,
        return_type: runtime::error_result_type(struct_name),
        method_impl: quote! {
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
                .db()
                .#singular_table_name()
                .#index_name()
                .update(#singular_table_name);

            #after_update_hook

            Ok(#singular_table_name)
        },
        read_context_compatible: false,
    }
}

/// `delete_<table>_by_<index>`: delete the one row a unique index finds.
fn for_delete_one(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        internal_columns,
        primary_key_column,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        primary_key_column_name,
        primary_key_column_name_as_string,
        field_name_for_found_value,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;
    let column_names_and_row_values = &shape.column_names_and_row_values;
    let unique_multi_column_index_hint = shape.unique_multi_column_hint;
    let is_singleton_pk = shape.is_singleton_primary_key;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::One, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    let before_delete_hook = hook_tokens(
        &spacetimedsl_table.hooks.before_delete,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, &row_to_delete },
            );

            quote! {
                #hook_call?;
            }
        },
    );

    let after_delete_hook = hook_tokens(
        &spacetimedsl_table.hooks.after_delete,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, &row_to_delete },
            );

            quote! {
                #hook_call?;
            }
        },
    );

    let count_mismatch_error = runtime::generic_error(&quote! {
        "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string()
    });

    let single_entry_deletion_result = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::One,
        &quote! { vec![deletion_result_entry] },
    );

    let method_impl = if is_singleton_pk {
        let primary_key = singleton::primary_key_ident();
        let primary_key_value = singleton::primary_key_value();

        let singleton_not_found_error = runtime::not_found_error(
            singular_table_name_as_string,
            &singleton::rendered_primary_key(),
        );
        let singleton_deletion_result_entry = runtime::deletion_result_entry(
            singular_table_name_as_string,
            &singleton::PRIMARY_KEY_NAME,
            &runtime::on_delete_strategy(&quote! { Delete }),
            &singleton::rendered_primary_key_value(),
            &quote! { child_entries: vec![], },
        );

        quote! {
            use ::spacetimedsl::itertools::Itertools;

            let row_to_delete = match self.db().#singular_table_name().#primary_key().find(&#primary_key_value) {
                None => return Err(#singleton_not_found_error),
                Some(row_to_delete) => row_to_delete,
            };

            let mut deletion_result_entry = #singleton_deletion_result_entry;

            #before_delete_hook

            match self.db().#singular_table_name().#primary_key().delete(&#primary_key_value) {
                false => {
                    return Err(#count_mismatch_error);
                },
                true => {},
            };

            #after_delete_hook

            return Ok(#single_entry_deletion_result);
        }
    } else {
        let get_row_to_delete;
        let return_error_on_is_none;

        match shape.is_multi_column {
            true => {
                let multi_column_index_check = get_unique_multi_column_index_check(
                    &Action::Delete,
                    singular_table_name,
                    index_name,
                    column_names_and_row_values,
                    &row_value_getters,
                );

                get_row_to_delete = quote! {
                    let mut #field_name_for_found_value: Option<#struct_name> = None;

                    #multi_column_index_check

                    let row_to_delete = #field_name_for_found_value;
                };

                let not_found_error = runtime::not_found_error(
                    singular_table_name_as_string,
                    &quote! {
                        format!(#column_names_and_row_values, #(#row_value_getters),*)
                    },
                );

                return_error_on_is_none = quote! {
                    let row_to_delete = match row_to_delete {
                        None => return Err(#not_found_error),
                        Some(row_to_delete) => row_to_delete,
                    };
                };
            }
            false => {
                let column_name = &shape.index_columns[0];
                let column_type_kind = internal_columns
                    .iter()
                    .find(|c| c.rust_field_name.eq(column_name))
                    .expect("An index column is always one of the table's columns")
                    .rust_field_type_kind;

                if column_type_kind == ColumnTypeKind::String {
                    get_row_to_delete = quote! {
                        let #index_name = #(#row_value_getters),*;

                        let row_to_delete = #index_accessor.find(&#index_name);
                    }
                } else {
                    get_row_to_delete = quote! {
                        let #index_name = #(#row_value_getters),*;

                        let row_to_delete = #index_accessor.find(#index_name);
                    }
                }

                let not_found_error = runtime::not_found_error(
                    singular_table_name_as_string,
                    &quote! { format!(#column_names_and_row_values, &#index_name) },
                );

                return_error_on_is_none = quote! {
                    let row_to_delete = match row_to_delete {
                        None => return Err(#not_found_error),
                        Some(row_to_delete) => row_to_delete,
                    };
                };
            }
        };

        let impl_until_return_err_on_is_none = quote! {
            use ::spacetimedsl::itertools::Itertools;

            #(#wrapper_option_mappers)*

            #get_row_to_delete

            #return_error_on_is_none
        };

        let wrapper_type_struct_name_or_path = primary_key_column
            .spacetimedsl_column_wrapper_type
            .as_ref()
            .expect(PRIMARY_KEY_WRAPPER_TYPE_INVARIANT)
            .struct_name_or_path_tokens();

        let deletion_result_entry_for_row = runtime::deletion_result_entry(
            singular_table_name_as_string,
            primary_key_column_name_as_string,
            &runtime::on_delete_strategy(&quote! { Delete }),
            &quote! {
                format!("{}", #wrapper_type_struct_name_or_path::new(row_to_delete.#primary_key_column_name.clone()))
            },
            &quote! { child_entries: vec![], },
        );

        let map_row_to_delete_to_deletion_result_entry = quote! {
            let mut deletion_result_entry = #deletion_result_entry_for_row;
        };

        let delete_one_impl = quote! {
            match self
                    .db()
                    .#singular_table_name()
                    .#primary_key_column_name()
                    .delete(&row_to_delete.#primary_key_column_name) {
                false => {
                    return Err(#count_mismatch_error);
                },
                true => {},
            };
        };

        let return_result_impl = quote! {
            return Ok(#single_entry_deletion_result);
        };

        if spacetimedsl_table.referencing_tables.is_empty() {
            quote! {
                #impl_until_return_err_on_is_none

                #map_row_to_delete_to_deletion_result_entry

                #before_delete_hook

                #delete_one_impl

                #after_delete_hook

                #return_result_impl
            }
        } else {
            let unknown_error_after_state_change = runtime::generic_error(&quote! {
                format!("Delete One Error: An unknown error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}")
            });

            let on_error_handler = quote! {
                let error = #single_entry_deletion_result;

                return Err(#unknown_error_after_state_change);
            };

            let reference_integrity_violation_on_delete_error =
                runtime::reference_integrity_violation_on_delete(&quote! { error });

            let error_strategy = get_referenced_table_function_call_for_dsl_method(
                singular_table_name,
                primary_key_column_name,
                OnDeleteStrategy::Error,
                OneOrMultiple::One,
                &quote! {
                    let error = #single_entry_deletion_result;

                    return Err(#reference_integrity_violation_on_delete_error);
                },
            );

            let delete_strategy = get_referenced_table_function_call_for_dsl_method(
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

            let set_zero_strategy = get_referenced_table_function_call_for_dsl_method(
                singular_table_name,
                primary_key_column_name,
                OnDeleteStrategy::SetZero,
                OneOrMultiple::One,
                &on_error_handler,
            );

            let ignore_strategy = get_referenced_table_function_call_for_dsl_method(
                singular_table_name,
                primary_key_column_name,
                OnDeleteStrategy::Ignore,
                OneOrMultiple::One,
                &on_error_handler,
            );

            quote! {
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
            }
        }
    };

    SpacetimeDSLMethod {
        doc_comment: match is_singleton_pk {
            true => format!(
                "Try to delete the `{struct_name}` row from the singleton `{singular_table_name}` table."
            ),
            false => format!(
                "{unique_multi_column_index_hint}\n\nTry to delete a `{struct_name}` row in the `{singular_table_name}` table {described_as}."
            ),
        },
        method_name: match is_singleton_pk {
            true => format_ident!("delete_{singular_table_name}"),
            false => format_ident!("delete_{singular_table_name}_by_{index_name}"),
        },
        method_args,
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl,
        read_context_compatible: false,
    }
}

/// Everything the five index-based generators derive from the index they are given.
///
/// This used to sit inside `for_method`, which is why no generator could be lifted out of
/// it. The four prose fragments the doc comments were built from are assembled here into
/// the one phrase all five of them built identically.
struct IndexShape {
    index_name: Ident,
    index_columns: Vec<Ident>,
    is_multi_column: bool,
    /// Whether this is the injected primary key of a singleton table.
    is_singleton_primary_key: bool,
    /// `{{ a : {}, b : {} }}`, with one placeholder per index column.
    column_names_and_row_values: String,
    /// "whose value matches the value from the unique single-column btree index on the
    /// `x` column", the tail every generated doc comment ends with.
    described_as: String,
    /// The experimental-feature warning a unique multi-column index carries, or empty.
    unique_multi_column_hint: &'static str,
}

impl IndexShape {
    fn of(index: &Index, context: &MethodGenerationContext) -> IndexShape {
        let index_name = index.name.clone();

        let (index_columns, is_multi_column, value_matches, single_or_multi, on_the_columns) =
            match &index.index_type {
                IndexType::BTreeSingleColumn { column }
                | IndexType::HashSingleColumn { column } => (
                    vec![column.clone()],
                    false,
                    "value matches the value from",
                    "single",
                    format!("`{column}` column"),
                ),
                IndexType::BTreeMultiColumn { columns }
                | IndexType::HashMultiColumn { columns } => (
                    columns.clone(),
                    true,
                    "values match the values from",
                    "multi",
                    documentation_on_columns(columns),
                ),
                IndexType::Direct { column } => (
                    vec![column.clone()],
                    false,
                    "value matches",
                    "single",
                    format!("`{column}` column"),
                ),
            };

        let index_documentation = match is_multi_column {
            false => format!("{} index", index_kind(&index.index_type)),
            true => format!("{} index `{index_name}`", index_kind(&index.index_type)),
        };

        // Only a unique index reaches the one-row generators, and only they say "unique".
        let unique = match index.is_unique {
            false => "",
            true => "unique ",
        };

        IndexShape {
            is_singleton_primary_key: context.spacetimedsl_table.is_singleton
                && !is_multi_column
                && index_columns
                    .first()
                    .is_some_and(|c| context.primary_key_column.rust_field_name == *c),
            column_names_and_row_values: column_names_and_row_values(&index_columns),
            described_as: format!(
                "whose {value_matches} the {unique}{single_or_multi}-column {index_documentation} on the {on_the_columns}"
            ),
            unique_multi_column_hint: match index.is_unique && is_multi_column {
                false => "",
                true => {
                    "Warning: The unique multi-column index feature of SpacetimeDSL is experimental.\n- It will be removed if unique multi-column indices are implemented in SpacetimeDB.\n- SpacetimeDSL is only able to enforce referential integrity if you never use the (mutating) `insert`, `update` and `delete` methods of `spacetimedb::ReducerContext` yourself."
                }
            },
            index_name,
            index_columns,
            is_multi_column,
        }
    }
}

/// What an index's columns contribute to the method that looks rows up by them.
///
/// The three lists are built from one walk over the index's columns, in index order, so
/// the n-th argument, the n-th row value and the n-th mapper all belong to the same
/// column.
struct IndexColumnArguments {
    method_args: Vec<SpacetimeDSLArg>,
    /// How the body reads each argument back out when it renders a not-found message.
    row_value_getters: Vec<TokenStream>,
    /// The `let` that unwraps an optional wrapper argument, or empty for a column that
    /// needs no unwrapping.
    wrapper_option_mappers: Vec<TokenStream>,
}

/// The arguments the four index-based lookup generators take.
///
/// `for_get_many`, `for_delete_many`, `for_get_one` and `for_delete_one` differ here in
/// one thing: whether the method returns many rows or one. A many-row method borrows its
/// arguments for the iterator that outlives the call, while a one-row method also renders
/// its arguments into a not-found message, so a wrapped argument has to be cloned before
/// it is consumed and a single-column string has to be owned.
///
/// A singleton's primary key takes no arguments at all: there is only one row to find.
fn index_column_arguments(
    shape: &IndexShape,
    one_or_multiple: &OneOrMultiple,
    context: &MethodGenerationContext,
) -> IndexColumnArguments {
    let mut arguments = IndexColumnArguments {
        method_args: vec![],
        row_value_getters: vec![],
        wrapper_option_mappers: vec![],
    };

    for column_name in &shape.index_columns {
        let column = context
            .internal_columns
            .iter()
            .find(|c| c.rust_field_name == *column_name)
            .expect("An index column is always one of the table's columns");

        let column_is_string = column.rust_field_type_kind == ColumnTypeKind::String;

        let wrapper_option_mapper;
        let method_arg;
        let row_value_getter;

        match &column.spacetimedsl_column_wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_ty = &WrapperType::map(wrapper_type);

                if column_is_string {
                    wrapper_option_mapper = TokenStream::default();

                    method_arg = SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(quote! { &str }),
                    };

                    row_value_getter = match one_or_multiple {
                        OneOrMultiple::Multiple => quote! { #column_name },
                        // A multi-column message renders the whole tuple with `{:?}`,
                        // which a `&str` already satisfies.
                        OneOrMultiple::One => match shape.is_multi_column {
                            true => quote! { #column_name },
                            false => quote! { #column_name.to_string() },
                        },
                    };
                } else if column.spacetimedsl_column_is_option {
                    wrapper_option_mapper = match wrapper_type {
                        // A created wrapper wraps the whole Option, so
                        // value() already yields it.
                        WrapperType::Created(_) => quote! {
                            let #column_name = match #column_name.into() {
                                None => None,
                                Some(#column_name) => Into::<#wrapper_type_ty>::into(#column_name).value(),
                            };
                        },
                        // A used wrapper wraps the inner type, so the
                        // Option has to be rebuilt around value().
                        WrapperType::Used(_) => quote! {
                            let #column_name = match #column_name.into() {
                                None => None,
                                Some(#column_name) => Some(Into::<#wrapper_type_ty>::into(#column_name).value()),
                            };
                        },
                    };

                    method_arg = SpacetimeDSLArg {
                        is_option: true,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Wrapped {
                            wrapped_type: WrapperType::map_to_wrapped_type(wrapper_type)
                                .to_token_stream(),
                            actual_type: quote! { &impl Into<Option<#wrapper_type_ty>> },
                        },
                    };

                    row_value_getter = quote! { #column_name };
                } else {
                    wrapper_option_mapper = TokenStream::default();

                    let wrapped_type =
                        WrapperType::map_to_wrapped_type(wrapper_type).to_token_stream();

                    match one_or_multiple {
                        OneOrMultiple::Multiple => {
                            method_arg = SpacetimeDSLArg {
                                is_option: false,
                                arg_name: column_name.clone(),
                                arg_type: SpacetimeDSLArgType::Wrapped {
                                    wrapped_type,
                                    actual_type: quote! { impl Into<#wrapper_type_ty> },
                                },
                            };

                            row_value_getter = quote! { #column_name.into().value() };
                        }
                        // `into()` consumes the argument, and the not-found message needs
                        // it afterwards.
                        OneOrMultiple::One => {
                            method_arg = SpacetimeDSLArg {
                                is_option: false,
                                arg_name: column_name.clone(),
                                arg_type: SpacetimeDSLArgType::Wrapped {
                                    wrapped_type,
                                    actual_type: quote! { impl Into<#wrapper_type_ty> + Clone },
                                },
                            };

                            row_value_getter = quote! { #column_name.clone().into().value() };
                        }
                    }
                }
            }
            None => {
                wrapper_option_mapper = TokenStream::default();

                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                let column_type = if column_is_string {
                    parse_str("str").expect("`str` is a valid type path")
                } else {
                    column.rust_field_type_name_or_path.clone()
                };

                match one_or_multiple {
                    OneOrMultiple::Multiple => {
                        method_arg = SpacetimeDSLArg {
                            is_option: column.spacetimedsl_column_is_option,
                            arg_name: column_name.clone(),
                            arg_type: SpacetimeDSLArgType::Normal(quote! { &'a #column_type }),
                        };

                        row_value_getter = quote! { #column_name };
                    }
                    OneOrMultiple::One => {
                        method_arg = SpacetimeDSLArg {
                            is_option: column.spacetimedsl_column_is_option,
                            arg_name: column_name.clone(),
                            arg_type: SpacetimeDSLArgType::Normal(quote! { &#column_type }),
                        };

                        row_value_getter = match column_is_string && !shape.is_multi_column {
                            true => quote! { #column_name.to_string() },
                            false => quote! { #column_name },
                        };
                    }
                }
            }
        }

        arguments.wrapper_option_mappers.push(wrapper_option_mapper);

        if !shape.is_singleton_primary_key {
            arguments.method_args.push(method_arg);
            arguments.row_value_getters.push(row_value_getter);
        }
    }

    arguments
}

/// "`a`, `b` and `c`", as the doc comment of a multi-column index names its columns.
fn documentation_on_columns(columns: &[Ident]) -> String {
    let mut columns: VecDeque<&Ident> = columns.iter().collect();

    let first_column = columns.pop_front().expect(
        "A multi-column index is only built from two or more columns, so it has a first one",
    );
    let last_column = columns.pop_back().expect(
        "A multi-column index is only built from two or more columns, so it has a last one",
    );

    let mut documentation = format!("columns `{first_column}`");

    for any_other_column in columns {
        documentation.push_str(&format!(", `{any_other_column}`"));
    }

    documentation.push_str(&format!(" and `{last_column}`"));

    documentation
}

/// The format string behind every "these columns had these values" message:
/// `{{ id : {} }}` for one column, `{{ a : {}, b : {} }}` for several.
///
/// One placeholder per column, in the order given, so a caller that builds its row-value
/// getters from the same list cannot get the two out of step.
fn column_names_and_row_values(column_names: &[Ident]) -> String {
    let placeholders = column_names
        .iter()
        .map(|column_name| format!("{column_name} : {{}}"))
        .collect_vec()
        .join(", ");

    format!("{{{{ {placeholders} }}}}")
}

/// The kind of index, as the doc comments of the generated methods name it.
fn index_kind(index_type: &IndexType) -> &'static str {
    match index_type {
        IndexType::BTreeSingleColumn { .. } | IndexType::BTreeMultiColumn { .. } => "btree",
        IndexType::HashSingleColumn { .. } | IndexType::HashMultiColumn { .. } => "hash",
        IndexType::Direct { .. } => "direct",
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
            let referenced_table_call = runtime::dsl_internals_call(
                &referenced_table_function_name,
                &quote! { self, #on_delete_strategy, &row_to_delete.#primary_key_column_name },
            );

            quote! {
                match #referenced_table_call {
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
            let referenced_table_call = runtime::dsl_internals_call(
                &referenced_table_function_name,
                &quote! {
                    self,
                    #on_delete_strategy,
                    &rows_to_delete.iter().map(|row| row.#primary_key_column_name).collect_vec()[..]
                },
            );

            quote! {
                match #referenced_table_call {
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

/// The `use self::<trait>;` import and the call `build_call` produces, kept apart so a
/// caller can place the import itself - before a prelude that has to run first, or outside
/// the loop the call sits in. Both are empty when the table declares no such hook.
fn hook_use_and_call(
    hook: &Option<SpacetimeDSLMethodHook>,
    build_call: impl FnOnce(&Ident) -> TokenStream,
) -> (TokenStream, TokenStream) {
    match hook {
        None => (TokenStream::default(), TokenStream::default()),
        Some(hook) => {
            let hook_trait_name = &hook.trait_name;

            (
                quote! { use self::#hook_trait_name; },
                build_call(&hook.function_name),
            )
        }
    }
}

/// `use self::<trait>;` followed by the call `build_call` produces, or nothing when the
/// table declares no such hook.
fn hook_tokens(
    hook: &Option<SpacetimeDSLMethodHook>,
    build_call: impl FnOnce(&Ident) -> TokenStream,
) -> TokenStream {
    let (use_hook_trait, hook_call) = hook_use_and_call(hook, build_call);

    quote! {
        #use_hook_trait
        #hook_call
    }
}

/// The scaffolding both reference-integrity builders share: skip the columns the mode does
/// not check, skip the columns without a foreign key, and wrap the mode's own check in the
/// guard that keeps it from running on a column that holds no reference yet.
fn reference_integrity_checks(
    columns: &[InternalColumn],
    skip_private_columns: bool,
    build_check: impl Fn(&InternalColumn, &ForeignKey) -> TokenStream,
) -> Vec<TokenStream> {
    let mut reference_integrity_checks = vec![];

    for column in columns {
        if skip_private_columns
            && column
                .rust_field_visibility
                .to_string()
                .eq(&crate::api::rust::visibility::RustVisibility::Private.to_string())
        {
            continue;
        }

        let foreign_key = match &column.spacetimedsl_column_foreign_key {
            Some(foreign_key) => foreign_key,
            None => continue,
        };

        let check = build_check(column, foreign_key);

        let referencing_table_column_name = &column.rust_field_name;

        reference_integrity_checks.push(match column.rust_field_type_kind {
            ColumnTypeKind::UnsignedInteger => quote! {
                if #referencing_table_column_name.ne(&0) {
                    #check
                }
            },
            ColumnTypeKind::Optional => quote! {
                if #referencing_table_column_name.is_some() {
                    #check
                }
            },
            ColumnTypeKind::String | ColumnTypeKind::Other => quote! {
                #check
            },
        });
    }

    reference_integrity_checks
}

fn reference_integrity_checks_on_create(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, false, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name = format_ident!(
            "get_{referenced_table_name}_by_{primary_key_column_name_of_referenced_table}"
        );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_getter_name =
            format_ident!("get_{referencing_table_column_name}");

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Create },
                &quote! {
                    format!("{{ {} : {} }}", #referencing_table_column_name, #referencing_table_name.#referencing_table_column_getter_name())
                },
            );

        quote! {
            match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                Ok(_) => {},
                Err(_) => {
                    return Err(#reference_integrity_violation_error);
                }
            };
        }
    })
}

fn reference_integrity_checks_on_update(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
    column_names_and_row_values: &str,
    index_columns: &[Ident],
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, true, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
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

        let field_name_for_found_value =
            format_ident!("the_same_or_another_{referencing_table_name}");

        let row_value_getters = index_columns
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

        let getter_name = format_ident!("get_{primary_key_column_name_of_referencing_table}");

        let not_found_error = runtime::not_found_error(
            &referencing_table_name_as_string,
            &format_for_not_found_error,
        );

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Update },
                &quote! {
                    format!("{{ {} : {} }}", #referencing_table_column_name_as_string, #referencing_table_column_name)
                },
            );

        quote! {
            if #field_name_for_found_value.is_none() {
                #field_name_for_found_value = match self.db().#referencing_table_name().#primary_key_column_name_of_referencing_table().find(#referencing_table_name.#getter_name().value()) {
                    Some(#referencing_table_name) => Some(#referencing_table_name),
                    None => {
                        return Err(#not_found_error);
                    }
                };
            }
            if #field_name_for_found_value.as_ref().expect("field_name_for_found_value should be Some(_)").#referencing_table_column_getter_name().ne(&#referencing_table_name.#referencing_table_column_getter_name()) {
                match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                    Ok(_) => {},
                    Err(_) => return Err(#reference_integrity_violation_error)
                };
            }
        }
    })
}

fn multi_column_index_checks(
    action: Action,
    singular_table_name: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
    internal_columns: &[InternalColumn],
    primary_key_column_name: &Ident,
) -> Vec<TokenStream> {
    let mut multi_column_index_checks = vec![];
    let singular_table_name_as_string = singular_table_name.to_string();

    for multi_column_index in &spacetimedb_table.multi_column_indices {
        let index_column_names: &[Ident] = match &multi_column_index.index_type {
            IndexType::BTreeMultiColumn { columns } => columns,
            _ => {
                continue;
            }
        };

        if !multi_column_index.is_unique {
            continue;
        }

        let internal_column_named = |column_name: &Ident| {
            internal_columns
                .iter()
                .find(|c| c.rust_field_name.eq(column_name))
                .expect("A multi-column index column should exist in the internal columns")
        };

        let index_name = &multi_column_index.name;

        // Built from the same ordered column list, so the placeholder count and the
        // getter count cannot drift apart.
        let column_names_and_row_values = column_names_and_row_values(index_column_names);
        let row_value_getters = index_column_names
            .iter()
            .map(|column_name| {
                get_row_value_getter(internal_column_named(column_name), singular_table_name)
            })
            .collect_vec();

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

        let unique_constraint_violation_error = runtime::unique_constraint_violation(
            &singular_table_name_as_string,
            &action_as_ident,
            &quote! { SpacetimeDSL },
            &multiple,
            &quote! { format!(#column_names_and_row_values, #(#row_value_getters),*) },
        );

        let return_unique_constraint_violation_error = quote! {
            return Err(#unique_constraint_violation_error);
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
    internal_column: &InternalColumn,
    singular_table_name: &Ident,
) -> TokenStream {
    let column_name = &internal_column.rust_field_name;

    if internal_column.rust_field_type_kind == ColumnTypeKind::String {
        quote! { &#singular_table_name.#column_name }
    } else {
        quote! { #singular_table_name.#column_name }
    }
}

fn get_unique_multi_column_index_check(
    action: &Action,
    singular_table_name: &Ident,
    index_name: &Ident,
    column_names_and_row_values: &str,
    row_value_getters: &[TokenStream],
) -> TokenStream {
    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let singular_table_name_as_string = singular_table_name.to_string();

    let action = format_ident!("{action}");

    let multiple = OneOrMultiple::Multiple;

    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        &singular_table_name_as_string,
        &action,
        &quote! { SpacetimeDSL },
        &multiple,
        &quote! { format!(#column_names_and_row_values, #(#row_value_getters),*) },
    );

    quote! {
        #field_name_for_found_value = match self.db().#singular_table_name().#index_name().filter((#(#row_value_getters),*)).at_most_one() {
            Ok(#singular_table_name) => #singular_table_name,
            Err(_) => return Err(#unique_constraint_violation_error),
        };
    }
}

fn for_referenced_by(
    one_or_multiple: &OneOrMultiple,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    primary_key_column: &InternalColumn,
) -> (SpacetimeDSLMethod, GeneratedTableRecordings) {
    let mut recordings = GeneratedTableRecordings::default();

    let singular_table_name = &spacetimedb_table.singular_name;
    let primary_key_column_type = &primary_key_column.rust_field_type_name_or_path;

    let doc_comment;
    let function_name = get_referenced_table_function_name(one_or_multiple, singular_table_name);

    let mut function_args = vec![
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("dsl"),
            arg_type: SpacetimeDSLArgType::Normal(quote! { &crate::spacetimedsl::DSL<'_, T> }),
        },
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("strategy"),
            arg_type: SpacetimeDSLArgType::Normal(runtime::on_delete_strategy_type()),
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
            let deletion_result_entry_type = runtime::deletion_result_entry_type();
            return_type = quote! {
                Result<Vec<#deletion_result_entry_type>, Vec<#deletion_result_entry_type>>
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
            let deletion_result_entry_type = runtime::deletion_result_entry_type();
            return_type = quote! {
                Result<
                    std::collections::HashMap<&'a #primary_key_column_type, Vec<#deletion_result_entry_type>>,
                    std::collections::HashMap<&'a #primary_key_column_type, Vec<#deletion_result_entry_type>>
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

        let referencing_table_path = &referencing_table.path;

        let compile_error_check =
            get_referenced_table_compile_error_check(singular_table_name, referencing_table_name);
        recordings
            .compile_error_checks
            .insert(compile_error_check.clone());

        let compile_error_check =
            get_referencing_table_compile_error_check(referencing_table_name, singular_table_name);
        compile_error_check_usages.push(quote! {
            use #referencing_table_path::#compile_error_check;
        });

        let referencing_table_function_name = get_referencing_table_function_name(
            one_or_multiple,
            referencing_table_name,
            singular_table_name,
        );

        let referencing_table_call = runtime::dsl_internals_call(
            &referencing_table_function_name,
            &quote! { dsl, &strategy, #arg_name },
        );

        strategy_calls.push(
            match one_or_multiple {
                OneOrMultiple::One => {
                    quote! {
                        match #referencing_table_call {
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
                        match #referencing_table_call {
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

    let method = SpacetimeDSLMethod {
        doc_comment,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
        read_context_compatible: false,
    };

    (method, recordings)
}

fn for_foreign_key(
    one_or_multiple: &OneOrMultiple,
    referencing_tables: ReferencingTables,
    spacetimedb_table: &SpacetimeDBTable,
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &[&Column],
    primary_key_column: &InternalColumn,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> syn::Result<(SpacetimeDSLMethod, GeneratedTableRecordings)> {
    let mut recordings = GeneratedTableRecordings::default();

    let first_foreign_key_column = columns_with_foreign_key
        .first()
        .expect("A table grouped by referenced table must have at least one foreign key column");

    let referenced_table_path = first_foreign_key_column
        .spacetimedsl_column
        .foreign_key
        .as_ref()
        .expect("The first column of a foreign key group carries the foreign key that grouped it")
        .path
        .to_token_stream();

    let referenced_table_primary_key_column_type =
        &first_foreign_key_column.rust_field.type_name_or_path;

    let mut columns_by_on_delete_strategies = BTreeMap::new();

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
            return Err(syn::Error::new_spanned(
                &column_with_foreign_key.rust_field.name,
                "All foreign key columns which reference the same primary key of another table should have the same type",
            ));
        }

        if column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .expect("Every column of a foreign key group carries a foreign key")
            .path
            .to_token_stream()
            .to_string()
            .ne(&referenced_table_path.to_string())
        {
            return Err(syn::Error::new_spanned(
                &column_with_foreign_key.rust_field.name,
                "All foreign key columns which reference the same primary key of another table should have the same path",
            ));
        }

        let on_delete_strategy = &column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .unwrap_or_else(|| {
                panic!(
                    "the column {} is in a foreign key group, so it carries a foreign key",
                    column_with_foreign_key.rust_field.name
                )
            })
            .on_delete_strategy;

        if !columns_by_on_delete_strategies.contains_key(on_delete_strategy) {
            columns_by_on_delete_strategies.insert(on_delete_strategy, vec![]);
        }

        columns_by_on_delete_strategies
            .get_mut(on_delete_strategy)
            .expect("The entry was inserted above when it was missing")
            .push(*column_with_foreign_key);
    }

    let singular_table_name = &spacetimedb_table.singular_name;
    let referenced_table_name = format_ident!("{}", *referenced_table_name);

    let doc_comment;

    let function_name = get_referencing_table_function_name(
        one_or_multiple,
        singular_table_name,
        &referenced_table_name,
    );

    let mut function_args = vec![
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("dsl"),
            arg_type: SpacetimeDSLArgType::Normal(quote! { &crate::spacetimedsl::DSL<'_, T> }),
        },
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("strategy"),
            arg_type: SpacetimeDSLArgType::Normal({
                let on_delete_strategy_type = runtime::on_delete_strategy_type();
                quote! { &#on_delete_strategy_type }
            }),
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
            let deletion_result_entry_type = runtime::deletion_result_entry_type();
            return_type = quote! {
                Result<Vec<#deletion_result_entry_type>, Vec<#deletion_result_entry_type>>
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
            let deletion_result_entry_type = runtime::deletion_result_entry_type();
            return_type = quote! {
                Result<
                    std::collections::HashMap<&'a #referenced_table_primary_key_column_type, Vec<#deletion_result_entry_type>>,
                    std::collections::HashMap<&'a #referenced_table_primary_key_column_type, Vec<#deletion_result_entry_type>>
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

    // `OnDeleteStrategy::iter()` yields the strategies in declaration order,
    // so the generated match arms are ordered independently of the input order.
    let strategy_implementations = OnDeleteStrategy::iter()
        .map(|on_delete_strategy| {
            let implementation = match columns_by_on_delete_strategies.remove(&on_delete_strategy) {
                Some(columns_by_on_delete_strategy) => get_on_delete_strategy_implementation(
                    spacetimedsl_table,
                    referencing_tables,
                    singular_table_name,
                    &on_delete_strategy,
                    columns_by_on_delete_strategy,
                    one_or_multiple,
                    primary_key_column,
                ),
                None => TokenStream::default(),
            };

            quote! {
                #on_delete_strategy => {
                    #implementation
                },
            }
        })
        .collect_vec();

    let compile_error_check =
        get_referencing_table_compile_error_check(singular_table_name, &referenced_table_name);

    recordings
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

        'outer: {
            match &strategy {
                #(#strategy_implementations)*
            };
        }

        match error {
            false => Ok(entries),
            true => Err(entries),
        }
    };

    let method = SpacetimeDSLMethod {
        doc_comment,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
        read_context_compatible: false,
    };

    Ok((method, recordings))
}

fn get_on_delete_strategy_implementation(
    spacetimedsl_table: &SpacetimeDSLTable,
    referencing_tables: ReferencingTables,
    singular_table_name: &Ident,
    on_delete_strategy: &OnDeleteStrategy,
    columns_by_on_delete_strategy: Vec<&Column>,
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> TokenStream {
    let spacetimedb_call_prefix = quote! {
        dsl
            .db()
            .#singular_table_name()
    };

    let primary_key_column_name = &primary_key_column.rust_field_name;

    let singular_table_name_as_string = singular_table_name.to_string();

    // Deliberate empty slot, symmetric with strategy_after_all.
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

        // A singleton has no index on a foreign key column, so find its one row by the
        // injected primary key and check the column afterwards.
        let index_uniqueness = if is_singleton {
            // Singleton has at most 1 row, treat as unique
            IndexUniqueness::Unique
        } else {
            match column
                .spacetimedb_column
                .single_column_index
                .as_ref()
                .expect("A foreign key column always has a single-column index, except in singleton-tables")
                .is_unique
            {
                true => IndexUniqueness::Unique,
                false => IndexUniqueness::NonUnique,
            }
        };

        let row_finder = if is_singleton {
            // For singletons, find the single row by PK and check FK column manually
            let primary_key = singleton::primary_key_ident();
            let primary_key_value = singleton::primary_key_value();

            quote! {
                #spacetimedb_call_prefix.#primary_key().find(&#primary_key_value).filter(|row| row.#column_name == *primary_key_value_of_a_row_of_another_table_to_delete)
            }
        } else {
            match index_uniqueness {
                IndexUniqueness::Unique => {
                    quote! {
                        #spacetimedb_call_prefix.#column_name().find(primary_key_value_of_a_row_of_another_table_to_delete)
                    }
                }
                IndexUniqueness::NonUnique => {
                    quote! {
                        #spacetimedb_call_prefix.#column_name().filter(primary_key_value_of_a_row_of_another_table_to_delete)
                    }
                }
            }
        };

        let row_value_format = if is_singleton {
            // The injected primary key has no wrapper type to render it.
            let rendered_primary_key_value = singleton::rendered_primary_key_value();

            quote! { #rendered_primary_key_value.to_string() }
        } else {
            let wrapper_type_struct_name_or_path = primary_key_column
                .spacetimedsl_column_wrapper_type
                .as_ref()
                .expect(PRIMARY_KEY_WRAPPER_TYPE_INVARIANT)
                .struct_name_or_path_tokens();
            quote! { format!("{}", #wrapper_type_struct_name_or_path::new(#primary_key_column_name.clone())) }
        };

        let create_entry = runtime::deletion_result_entry(
            &singular_table_name_as_string,
            &column_name_as_string,
            on_delete_strategy,
            &row_value_format,
            &quote! { child_entries, },
        );

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
                    RowBinding::Immutable,
                    index_uniqueness,
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
                // These two imports have to escape the per-row loop their guard sits in,
                // so they are hoisted into strategy_for_before_hook / _after_hook instead
                // of being emitted next to the call.
                let (use_before_delete_hook_trait, before_delete_hook) = hook_use_and_call(
                    &spacetimedsl_table.hooks.before_delete,
                    |hook_function_name| {
                        let hook_call = runtime::dsl_method_hooks_call(
                            hook_function_name,
                            &quote! { &dsl, &row },
                        );

                        quote! {
                            if #hook_call.is_err() {
                                error = true;
                                // FIXME: This results in the error supplied by the hook being ignored, we should propagate it back to the caller but that requires changing the function signature.
                                break 'outer;
                            }
                        }
                    },
                );
                strategy_for_before_hook = use_before_delete_hook_trait;

                let (use_after_delete_hook_trait, after_delete_hook) = hook_use_and_call(
                    &spacetimedsl_table.hooks.after_delete,
                    |hook_function_name| {
                        let hook_call = runtime::dsl_method_hooks_call(
                            hook_function_name,
                            &quote! { &dsl, &row },
                        );

                        quote! {
                            if #hook_call.is_err() {
                                error = true;
                                // FIXME: This results in the error supplied by the hook being ignored, we should propagate it back to the caller but that requires changing the function signature.
                                break 'outer;
                            }
                        }
                    },
                );
                strategy_for_after_hook = use_after_delete_hook_trait;

                match referencing_tables {
                    ReferencingTables::Absent => strategy_by_column.push(strategy_by_row(
                        RowBinding::Immutable,
                        index_uniqueness,
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
                    ReferencingTables::Present => {
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
                            RowBinding::Immutable,
                            index_uniqueness,
                            &row_finder,
                            strategy_for_each_row,
                        ));
                    }
                };
            }
            OnDeleteStrategy::SetZero => {
                strategy_by_column.push(strategy_by_row(
                    RowBinding::Mutable,
                    index_uniqueness,
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
                    RowBinding::Immutable,
                    index_uniqueness,
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
    row_binding: RowBinding,
    index_uniqueness: IndexUniqueness,
    row_finder: &TokenStream,
    strategy_for_each_row: TokenStream,
) -> TokenStream {
    let row_or_mut_row = match row_binding {
        RowBinding::Mutable => quote! {
            mut row
        },
        RowBinding::Immutable => quote! {
            row
        },
    };

    match index_uniqueness {
        IndexUniqueness::Unique => quote! {
            match #row_finder {
                None => {}
                Some(#row_or_mut_row) => {
                    #strategy_for_each_row
                }
            };
        },
        IndexUniqueness::NonUnique => quote! {
            for #row_or_mut_row in #row_finder {
                #strategy_for_each_row
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
    let referenced_table_call = runtime::dsl_internals_call(
        &referenced_table_function_name,
        &quote! { dsl, #on_delete_strategy, &primary_key_values_of_rows_to_delete[..] },
    );

    quote! {
        match #referenced_table_call {
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
