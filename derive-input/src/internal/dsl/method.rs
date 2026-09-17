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
            table::{
                OnDeleteStrategiesOfReferencingTables, OnDeleteStrategiesOfTheReferencedTable,
                SpacetimeDSLTable, SpacetimeDSLTableMethods,
            },
        },
        runtime,
    },
    internal::{
        column::InternalColumn,
        dsl::{one_or_multiple::OneOrMultiple, singleton},
    },
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use std::collections::BTreeMap;
use strum::IntoEnumIterator;
use syn::Ident;

mod context;
mod create;
mod delete;
mod get;
mod hook_call;
mod index;
mod reference_integrity;
mod update;

pub(in crate::internal) use context::{MethodGenerationContext, TableContributions};

use create::for_create;
use delete::{for_delete_many, for_delete_one};
use get::{for_get_all, for_get_count, for_get_many, for_get_one};
use hook_call::{hook_tokens, hook_use_and_call};
use index::IndexShape;
use update::for_update;

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

/// `get_<table>`: the one row of a singleton table.
///
/// A singleton is found by its injected primary key rather than by an index the caller
/// supplies a value for, so this takes no arguments and renders no row values.
fn for_singleton_get(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    let primary_key = singleton::primary_key_ident();
    let primary_key_value = singleton::primary_key_value();

    let not_found_error = runtime::not_found_error(
        singular_table_name_as_string,
        &singleton::rendered_primary_key(),
    );

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Try to get the `{struct_name}` from the singleton `{singular_table_name}` table."
        ),
        method_name: format_ident!("get_{singular_table_name}"),
        method_args: vec![],
        return_type: runtime::error_result_type(struct_name),
        method_impl: quote! {
            match self.db().#singular_table_name().#primary_key().find(&#primary_key_value) {
                Some(#singular_table_name) => Ok(#singular_table_name),
                None => return Err(#not_found_error)
            }
        },
        read_context_compatible: true,
    }
}

/// `delete_<table>`: delete the one row of a singleton table.
fn for_singleton_delete(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    let primary_key = singleton::primary_key_ident();
    let primary_key_value = singleton::primary_key_value();

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

    let not_found_error = runtime::not_found_error(
        singular_table_name_as_string,
        &singleton::rendered_primary_key(),
    );

    let deletion_result_entry = runtime::deletion_result_entry(
        singular_table_name_as_string,
        &singleton::PRIMARY_KEY_NAME,
        &runtime::on_delete_strategy(&quote! { Delete }),
        &singleton::rendered_primary_key_value(),
        &quote! { child_entries: vec![], },
    );

    let count_mismatch_error = runtime::generic_error(&quote! {
        "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string()
    });

    let single_entry_deletion_result = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::One,
        &quote! { vec![deletion_result_entry] },
        &quote! { None },
    );

    let itertools_import = runtime::itertools_import();

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Try to delete the `{struct_name}` row from the singleton `{singular_table_name}` table."
        ),
        method_name: format_ident!("delete_{singular_table_name}"),
        method_args: vec![],
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl: quote! {
            #itertools_import

            let row_to_delete = match self.db().#singular_table_name().#primary_key().find(&#primary_key_value) {
                None => return Err(#not_found_error),
                Some(row_to_delete) => row_to_delete,
            };

            let mut deletion_result_entry = #deletion_result_entry;

            #before_delete_hook

            match self.db().#singular_table_name().#primary_key().delete(&#primary_key_value) {
                false => {
                    return Err(#count_mismatch_error);
                },
                true => {},
            };

            #after_delete_hook

            return Ok(#single_entry_deletion_result);
        },
        read_context_compatible: false,
    }
}

/// The update method an index earns, if any.
///
/// SpacetimeDB's `update` lives on the primary key index, and no other index implements
/// `PrimaryKey`, so no other index can carry one. `method(update = false)` suppresses it
/// on top of that.
fn update_method_for(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> Option<SpacetimeDSLMethod> {
    match context.spacetimedsl_table.has_update_method && shape.is_primary_key {
        false => None,
        true => Some(for_update(shape, context)),
    }
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
    let spacetimedsl_table = context.spacetimedsl_table;

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
            SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                get_one_option: for_get_one(&shape, context),
                update: update_method_for(&shape, context),
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

        let spacetimedsl_table = context.spacetimedsl_table;

        // `internal/db/column.rs` rejects `#[index]` and `#[unique]` on a singleton's own
        // columns, so the only index a singleton reaches here with is its injected primary
        // key. Getting and deleting its row take no arguments and look it up by that key,
        // which is a different method body rather than a branch inside one. Updating it is
        // the ordinary update, with one statement added.
        let methods = match spacetimedsl_table.is_singleton {
            true => {
                SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                    get_one_option: for_singleton_get(context),
                    update: update_method_for(&IndexShape::of(index, context), context),
                    delete_one: match spacetimedsl_table.has_delete_method {
                        false => None,
                        true => Some(for_singleton_delete(context)),
                    },
                })
            }
            false => column_methods_for(index, context),
        };

        Some(methods)
    }
}

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn generate(
        context: &MethodGenerationContext,
        columns: &[Column],
    ) -> syn::Result<(SpacetimeDSLTableMethods, TableContributions)> {
        let MethodGenerationContext {
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column,
            ..
        } = context;

        let is_singleton = spacetimedsl_table.is_singleton;

        let mut contributions = TableContributions::default();

        let (create, create_contributions) = for_create(context);
        contributions.merge(create_contributions);

        // A singleton holds one row, so iterating and counting have nothing to say.
        let get_all = match is_singleton {
            true => None,
            false => Some(for_get_all(context)),
        };

        let get_count = match is_singleton {
            true => None,
            false => Some(for_get_count(context)),
        };

        let on_delete_strategies_of_referencing_tables =
            match spacetimedsl_table.referencing_tables.is_empty() {
                true => None,
                false => {
                    let (after_one_row, after_one_row_contributions) = for_referenced_by(
                        &OneOrMultiple::One,
                        spacetimedb_table,
                        spacetimedsl_table,
                        primary_key_column,
                    );
                    contributions.merge(after_one_row_contributions);

                    let (after_multiple_rows, after_multiple_rows_contributions) =
                        for_referenced_by(
                            &OneOrMultiple::Multiple,
                            spacetimedb_table,
                            spacetimedsl_table,
                            primary_key_column,
                        );
                    contributions.merge(after_multiple_rows_contributions);

                    Some(OnDeleteStrategiesOfReferencingTables {
                        after_one_row_of_this_table_was_deleted: after_one_row,
                        after_multiple_rows_of_this_table_were_deleted: after_multiple_rows,
                    })
                }
            };

        let mut on_delete_strategies_of_this_table = vec![];

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

                let (after_one_row, after_one_row_contributions) = for_foreign_key(
                    &OneOrMultiple::One,
                    referencing_tables,
                    spacetimedb_table,
                    referenced_table_name,
                    &columns_with_foreign_key,
                    primary_key_column,
                    spacetimedsl_table,
                )?;
                contributions.merge(after_one_row_contributions);

                let (after_multiple_rows, after_multiple_rows_contributions) = for_foreign_key(
                    &OneOrMultiple::Multiple,
                    referencing_tables,
                    spacetimedb_table,
                    referenced_table_name,
                    &columns_with_foreign_key,
                    primary_key_column,
                    spacetimedsl_table,
                )?;
                contributions.merge(after_multiple_rows_contributions);

                on_delete_strategies_of_this_table.push(OnDeleteStrategiesOfTheReferencedTable {
                    after_one_row_was_deleted: after_one_row,
                    after_multiple_rows_were_deleted: after_multiple_rows,
                });
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
            on_delete_strategies_of_referencing_tables,
            on_delete_strategies_of_this_table,
            multi_column_indices,
        };

        Ok((methods, contributions))
    }
}

pub(in crate::internal) fn get_referenced_table_function_call_for_dsl_method(
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
                    Err(failure) => {
                        let mut child_entries = failure.entries;
                        deletion_result_entry.child_entries.append(&mut child_entries);

                        let error_from_hook = failure.error_from_hook;

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
                    Err(failure) => {
                        for (primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                            deletion_result_entries.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in deletion_result_entries.")).child_entries.append(&mut child_entries);
                        }

                        let error_from_hook = failure.error_from_hook;

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

fn for_referenced_by(
    one_or_multiple: &OneOrMultiple,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    primary_key_column: &InternalColumn,
) -> (SpacetimeDSLMethod, TableContributions) {
    let mut contributions = TableContributions::default();

    let singular_table_name = &spacetimedb_table.singular_name;
    let primary_key_column_type = &primary_key_column.rust_field_type_name_or_path;

    let doc_comment;
    let function_name = get_referenced_table_function_name(one_or_multiple, singular_table_name);

    let mut function_args = vec![
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("dsl"),
            arg_type: SpacetimeDSLArgType::Normal(runtime::dsl_reference_type()),
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
            let entries_type = quote! { Vec<#deletion_result_entry_type> };
            let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);
            return_type = quote! {
                Result<#entries_type, #failure_type>
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
            let entries_type = quote! {
                std::collections::HashMap<&'a #primary_key_column_type, Vec<#deletion_result_entry_type>>
            };
            let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);
            return_type = quote! {
                Result<#entries_type, #failure_type>
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
        contributions
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
                            Err(failure) => {
                                let mut child_entries = failure.entries;
                                entries.append(&mut child_entries);

                                if error_from_hook.is_none() {
                                    error_from_hook = failure.error_from_hook;
                                }

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
                            Err(failure) => {
                                for (primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                                    entries.get_mut(&primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in entries.")).append(&mut child_entries);
                                }

                                if error_from_hook.is_none() {
                                    error_from_hook = failure.error_from_hook;
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

    let error_from_hook_declaration = runtime::error_from_hook_declaration();
    let failure =
        runtime::on_delete_strategy_failure(&quote! { entries }, &quote! { error_from_hook });

    let function_impl = quote! {
        #(#compile_error_check_usages)*

        #create_entries

        #error_from_hook_declaration
        let mut error = false;

        #(#strategy_calls)*

        match error {
            false => Ok(entries),
            true => Err(#failure),
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

    (method, contributions)
}

fn for_foreign_key(
    one_or_multiple: &OneOrMultiple,
    referencing_tables: ReferencingTables,
    spacetimedb_table: &SpacetimeDBTable,
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &[&Column],
    primary_key_column: &InternalColumn,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> syn::Result<(SpacetimeDSLMethod, TableContributions)> {
    let mut contributions = TableContributions::default();

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
            arg_type: SpacetimeDSLArgType::Normal(runtime::dsl_reference_type()),
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
            let entries_type = quote! { Vec<#deletion_result_entry_type> };
            let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);
            return_type = quote! {
                Result<#entries_type, #failure_type>
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
            let entries_type = quote! {
                std::collections::HashMap<&'a #referenced_table_primary_key_column_type, Vec<#deletion_result_entry_type>>
            };
            let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);
            return_type = quote! {
                Result<#entries_type, #failure_type>
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

    contributions
        .compile_error_checks
        .insert(compile_error_check.clone());

    let compile_error_check =
        get_referenced_table_compile_error_check(&referenced_table_name, singular_table_name);

    let compile_error_check_usage = quote! {
        use #referenced_table_path::#compile_error_check;
    };

    let error_from_hook_declaration = runtime::error_from_hook_declaration();
    let failure =
        runtime::on_delete_strategy_failure(&quote! { entries }, &quote! { error_from_hook });

    let itertools_import = runtime::itertools_import();

    let function_impl = quote! {
        #compile_error_check_usage

        #itertools_import
        #create_data_structure_for_child_entries

        #error_from_hook_declaration
        let mut error = false;

        'outer: {
            match &strategy {
                #(#strategy_implementations)*
            };
        }

        match error {
            false => Ok(entries),
            true => Err(#failure),
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

    Ok((method, contributions))
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
            let wrapper_type_struct_name_or_path =
                context::primary_key_wrapper_type(primary_key_column);
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
                            if let Err(error_raised_by_the_hook) = #hook_call {
                                error = true;
                                error_from_hook = Some(Box::new(error_raised_by_the_hook));
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
                            if let Err(error_raised_by_the_hook) = #hook_call {
                                error = true;
                                error_from_hook = Some(Box::new(error_raised_by_the_hook));
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

                        let failure = runtime::on_delete_strategy_failure(
                            &quote! { entries },
                            &quote! { error_from_hook },
                        );

                        let on_error_handler = quote! {
                            #create_entries_and_add_them_to_entries
                            return Err(#failure);
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
            Err(failure) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                    child_entries_by_primary_key_value_of_row_to_delete.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in child_entries_by_primary_key_value_of_row_to_delete.")).append(&mut child_entries);
                }

                if error_from_hook.is_none() {
                    error_from_hook = failure.error_from_hook;
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
