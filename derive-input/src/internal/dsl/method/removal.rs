//! What retiring a row and removing it have in common.
//!
//! `delete_<tables>_by_<index>` and `soft_delete_<tables>_by_<index>` differ in four
//! places: the statement that writes, the hooks they call, the dispatcher they cascade
//! through and the strategy they report. Everything around those four — finding the rows,
//! building the entries, refusing before writing, running the remaining strategy passes —
//! is stated once here, because two copies of that order would drift.

use super::referenced_by::referenced_table_function_call_for_dsl_method;
use super::{
    context::{self, MethodGenerationContext},
    hook_call::hook_tokens,
    index::{IndexColumnArguments, IndexShape, index_accessor, index_column_arguments},
    reference_integrity::{Action, unique_multi_column_index_check},
    soft_delete,
    upsert::rebind_row_as_mutable_after_hook,
};
use crate::{
    api::{
        dsl::{
            foreign_key::OnDeleteStrategy, method::SpacetimeDSLMethod,
            soft_delete::SoftDeleteMarker, table::SpacetimeDSLTable,
        },
        runtime,
    },
    internal::{column::ColumnTypeKind, dsl::one_or_multiple::OneOrMultiple},
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Whether a generated method removes the rows it matched or retires them.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::internal) enum Removal {
    Hard,
    Soft,
}

/// The strategies a removal fans out to after it has written, in the order the generated
/// body runs them.
///
/// `Error` is not here: it runs before the write, so that a refusal leaves the database
/// untouched. A hard deletion can reach every strategy a `#[foreign_key]` accepts in
/// `on_delete`, a soft one only the two `on_soft_delete` accepts besides `Error`.
///
/// TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 `SetNone` joins the hard
/// list once `Option` is allowed on an indexed column.
fn strategies_after_the_write(removal: Removal) -> &'static [OnDeleteStrategy] {
    match removal {
        Removal::Hard => &[
            OnDeleteStrategy::Delete,
            OnDeleteStrategy::SoftDelete,
            OnDeleteStrategy::SetZero,
            OnDeleteStrategy::Ignore,
        ],
        Removal::Soft => &[OnDeleteStrategy::SoftDelete, OnDeleteStrategy::Ignore],
    }
}

/// The marker column a soft removal writes.
///
/// `internal/dsl/soft_delete.rs` rejects `method(soft_delete = true)` without a marker
/// column and a marker column without the flag, so a table reaching here with
/// `Removal::Soft` always has one.
fn marker_of(spacetimedsl_table: &SpacetimeDSLTable) -> &SoftDeleteMarker {
    spacetimedsl_table
        .soft_delete_marker
        .as_ref()
        .expect("a soft removal is only generated for a soft-deletable table")
}

/// The statements that retire the row a surrounding binding called `old_row` points at.
///
/// SpacetimeDB has no bulk update, so retiring many rows is retiring one row repeatedly,
/// and both generators emit this. The before hook hands back the row to write, so it runs
/// between the clone and the marker rather than around the whole statement; the `mut` then
/// moves to a rebinding after it, because the hook's own binding is not `mut` — the shape
/// `update_<table>_by_<key>` uses, for the same reason.
///
/// `updated_at` is deliberately left alone: the marker records the retirement, and
/// `updated_at` keeps meaning the last ordinary edit.
fn retire_row_named_old_row(
    spacetimedsl_table: &SpacetimeDSLTable,
    singular_table_name: &syn::Ident,
    primary_key_column_name: &syn::Ident,
) -> TokenStream {
    let new_row = format_ident!("new_row");
    let set_marker = soft_delete::set_marker(
        marker_of(spacetimedsl_table),
        &quote! { self.ctx().timestamp()? },
        &new_row,
    );

    let before_hook = hook_tokens(
        &spacetimedsl_table.hooks.before_soft_delete,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, old_row, new_row },
            );

            quote! {
                let new_row = #hook_call?;
            }
        },
    );

    let after_hook = hook_tokens(
        &spacetimedsl_table.hooks.after_soft_delete,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, old_row, &new_row },
            );

            quote! {
                #hook_call?;
            }
        },
    );

    let clone_row = match before_hook.is_empty() {
        true => quote! { let mut new_row = old_row.clone(); },
        false => quote! { let new_row = old_row.clone(); },
    };

    let rebind_row = rebind_row_as_mutable_after_hook(&new_row, &before_hook, &[&set_marker]);

    // Only the after hook reads the stored row, so only then is it worth naming.
    let store_row = match after_hook.is_empty() {
        true => quote! {
            self.db().#singular_table_name().#primary_key_column_name().update(new_row);
        },
        false => quote! {
            let new_row = self.db().#singular_table_name().#primary_key_column_name().update(new_row);
        },
    };

    quote! {
        #clone_row

        #before_hook

        #rebind_row

        #set_marker

        #store_row

        #after_hook
    }
}

/// `delete_<tables>_by_<index>` and `soft_delete_<tables>_by_<index>`: retire or remove
/// every row an index matches.
pub(in crate::internal) fn for_removal_many(
    removal: Removal,
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
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
        &quote! { None },
    );

    let itertools_import = runtime::itertools_import();

    // A soft removal is idempotent: a row already retired is not matched again, so the
    // hooks do not run twice and the result reports only the rows this call retired.
    let skip_already_retired = match removal {
        Removal::Hard => TokenStream::default(),
        Removal::Soft => {
            let is_marked = soft_delete::is_marked(marker_of(spacetimedsl_table), &quote! { row });
            quote! { .filter(|row| !(#is_marked)) }
        }
    };

    let impl_until_return_ok_on_is_empty = quote! {
        #itertools_import

        #(#wrapper_option_mappers)*

        #let_index_name

        let rows_to_delete: Vec<#struct_name> = #index_accessor
            .filter(#index_name)
            #skip_already_retired
            .collect();

        if rows_to_delete.is_empty() {
            return Ok(#empty_deletion_result);
        }
    };

    let wrapper_type_struct_name_or_path = context::primary_key_wrapper_type(primary_key_column);

    // Variation point: the strategy the entry reports.
    let reported_strategy = match removal {
        Removal::Hard => runtime::on_delete_strategy(&quote! { Delete }),
        Removal::Soft => runtime::on_delete_strategy(&quote! { SoftDelete }),
    };

    let deletion_result_entry_per_row = runtime::deletion_result_entry(
        singular_table_name_as_string,
        primary_key_column_name_as_string,
        &reported_strategy,
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

    // Variation point: which pair of hooks runs, and where.
    //
    // A hard deletion's hooks bracket the one bulk `delete`, so they are blocks of their
    // own. A soft deletion writes each row separately and its before hook hands back the
    // row to write, so its hooks belong inside that loop rather than around it; the two
    // outer slots are then empty and the loop below carries the calls.
    let before_delete_hook = match removal {
        Removal::Hard => hook_tokens(
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
        ),
        Removal::Soft => TokenStream::default(),
    };

    let after_delete_hook = match removal {
        Removal::Hard => hook_tokens(
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
        ),
        Removal::Soft => TokenStream::default(),
    };

    let count_mismatch_error = runtime::generic_error(&quote! {
        format!(
            "Delete Many Error: `count_of_rows_to_delete ( {} ) != ( {} ) count_of_deleted_rows`!",
            &count_of_rows_to_delete,
            &count_of_deleted_rows
        )
    });

    // Variation point: the statement that writes, and the check around it.
    let delete_many_impl = match removal {
        Removal::Hard => quote! {
            let count_of_rows_to_delete: u64 = rows_to_delete
                .len()
                .try_into()
                .unwrap_or(u64::MAX);

            let count_of_deleted_rows = #index_accessor.delete(#index_name);

            if count_of_rows_to_delete.ne(&count_of_deleted_rows) {
                return Err(#count_mismatch_error);
            }
        },
        Removal::Soft => {
            let retire_row = retire_row_named_old_row(
                spacetimedsl_table,
                singular_table_name,
                primary_key_column_name,
            );

            quote! {
                for old_row in &rows_to_delete {
                    #retire_row
                }
            }
        }
    };

    let deletion_result_from_entries = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::Multiple,
        &quote! { deletion_result_entries.into_values().collect_vec() },
        &quote! { None },
    );

    let deletion_result_from_entries_with_error_from_hook = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::Multiple,
        &quote! { deletion_result_entries.into_values().collect_vec() },
        &quote! { error_from_hook },
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
        let error_after_state_change = runtime::generic_error(&quote! {
            format!("Delete Many Error: An error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}")
        });

        let on_error_handler = quote! {
            let error = #deletion_result_from_entries_with_error_from_hook;

            return Err(#error_after_state_change);
        };

        let reference_integrity_violation_on_delete_error =
            runtime::reference_integrity_violation_on_delete(&quote! { error });

        let error_strategy = referenced_table_function_call_for_dsl_method(
            removal,
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Error,
            OneOrMultiple::Multiple,
            &quote! {
                let error = #deletion_result_from_entries_with_error_from_hook;

                return Err(#reference_integrity_violation_on_delete_error);
            },
        );

        let strategies_after_the_write = strategies_after_the_write(removal)
            .iter()
            .map(|strategy| {
                referenced_table_function_call_for_dsl_method(
                    removal,
                    singular_table_name,
                    primary_key_column_name,
                    strategy.clone(),
                    OneOrMultiple::Multiple,
                    &on_error_handler,
                )
            })
            .collect_vec();

        quote! {
            #impl_until_return_ok_on_is_empty

            #map_rows_to_delete_to_deletion_result_entries

            #error_strategy

            #before_delete_hook

            #delete_many_impl

            #after_delete_hook

            #(#strategies_after_the_write)*

            #return_result_impl
        }
    };

    // Variation point: what the method is called and what it says it does.
    let (doc_comment, method_name) = match removal {
        Removal::Hard => (
            format!(
                "Try to delete all `{struct_name}` rows in the `{singular_table_name}` table {described_as}."
            ),
            format_ident!("delete_{plural_table_name}_by_{index_name}"),
        ),
        Removal::Soft => (
            format!(
                "Try to soft-delete all `{struct_name}` rows in the `{singular_table_name}` table {described_as}."
            ),
            format_ident!("soft_delete_{plural_table_name}_by_{index_name}"),
        ),
    };

    SpacetimeDSLMethod {
        doc_comment,
        method_name,
        method_args,
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl,
        read_context_compatible: false,
    }
}

/// `delete_<table>_by_<index>` and `soft_delete_<table>_by_<index>`: retire or remove the
/// one row a unique index finds.
pub(in crate::internal) fn for_removal_one(
    removal: Removal,
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
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

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::One, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    // Variation point: which pair of hooks runs, and where. A soft deletion's hooks sit
    // inside the write, for the reason given in `for_removal_many`.
    let before_delete_hook = match removal {
        Removal::Hard => hook_tokens(
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
        ),
        Removal::Soft => TokenStream::default(),
    };

    let after_delete_hook = match removal {
        Removal::Hard => hook_tokens(
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
        ),
        Removal::Soft => TokenStream::default(),
    };

    let count_mismatch_error = runtime::generic_error(&quote! {
        "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string()
    });

    let single_entry_deletion_result = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::One,
        &quote! { vec![deletion_result_entry] },
        &quote! { None },
    );

    let single_entry_deletion_result_with_error_from_hook = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::One,
        &quote! { vec![deletion_result_entry] },
        &quote! { error_from_hook },
    );

    let get_row_to_delete;
    let return_error_on_is_none;

    match shape.is_multi_column {
        true => {
            let multi_column_index_check = unique_multi_column_index_check(
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

    let itertools_import = runtime::itertools_import();

    // A soft removal is idempotent: retiring an already retired row is not an error, it is
    // nothing to do, so it reports an empty result without running a hook.
    let return_ok_on_already_retired = match removal {
        Removal::Hard => TokenStream::default(),
        Removal::Soft => {
            let is_marked =
                soft_delete::is_marked(marker_of(spacetimedsl_table), &quote! { row_to_delete });

            let empty_deletion_result = runtime::deletion_result(
                singular_table_name_as_string,
                &OneOrMultiple::One,
                &quote! { vec![] },
                &quote! { None },
            );

            quote! {
                if #is_marked {
                    return Ok(#empty_deletion_result);
                }
            }
        }
    };

    let impl_until_return_err_on_is_none = quote! {
        #itertools_import

        #(#wrapper_option_mappers)*

        #get_row_to_delete

        #return_error_on_is_none

        #return_ok_on_already_retired
    };

    let wrapper_type_struct_name_or_path = context::primary_key_wrapper_type(primary_key_column);

    // Variation point: the strategy the entry reports.
    let reported_strategy = match removal {
        Removal::Hard => runtime::on_delete_strategy(&quote! { Delete }),
        Removal::Soft => runtime::on_delete_strategy(&quote! { SoftDelete }),
    };

    let deletion_result_entry_for_row = runtime::deletion_result_entry(
        singular_table_name_as_string,
        primary_key_column_name_as_string,
        &reported_strategy,
        &quote! {
            format!("{}", #wrapper_type_struct_name_or_path::new(row_to_delete.#primary_key_column_name.clone()))
        },
        &quote! { child_entries: vec![], },
    );

    let map_row_to_delete_to_deletion_result_entry = quote! {
        let mut deletion_result_entry = #deletion_result_entry_for_row;
    };

    // Variation point: the statement that writes, and the check around it.
    let delete_one_impl = match removal {
        Removal::Hard => quote! {
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
        },
        Removal::Soft => {
            let retire_row = retire_row_named_old_row(
                spacetimedsl_table,
                singular_table_name,
                primary_key_column_name,
            );

            quote! {
                let old_row = &row_to_delete;

                #retire_row
            }
        }
    };

    let return_result_impl = quote! {
        return Ok(#single_entry_deletion_result);
    };

    let method_impl = if spacetimedsl_table.referencing_tables.is_empty() {
        quote! {
            #impl_until_return_err_on_is_none

            #map_row_to_delete_to_deletion_result_entry

            #before_delete_hook

            #delete_one_impl

            #after_delete_hook

            #return_result_impl
        }
    } else {
        let error_after_state_change = runtime::generic_error(&quote! {
            format!("Delete One Error: An error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}")
        });

        let on_error_handler = quote! {
            let error = #single_entry_deletion_result_with_error_from_hook;

            return Err(#error_after_state_change);
        };

        let reference_integrity_violation_on_delete_error =
            runtime::reference_integrity_violation_on_delete(&quote! { error });

        let error_strategy = referenced_table_function_call_for_dsl_method(
            removal,
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Error,
            OneOrMultiple::One,
            &quote! {
                let error = #single_entry_deletion_result_with_error_from_hook;

                return Err(#reference_integrity_violation_on_delete_error);
            },
        );

        let strategies_after_the_write = strategies_after_the_write(removal)
            .iter()
            .map(|strategy| {
                referenced_table_function_call_for_dsl_method(
                    removal,
                    singular_table_name,
                    primary_key_column_name,
                    strategy.clone(),
                    OneOrMultiple::One,
                    &on_error_handler,
                )
            })
            .collect_vec();

        quote! {
            #impl_until_return_err_on_is_none

            #map_row_to_delete_to_deletion_result_entry

            #error_strategy

            #before_delete_hook

            #delete_one_impl

            #after_delete_hook

            #(#strategies_after_the_write)*

            #return_result_impl
        }
    };

    // Variation point: what the method is called and what it says it does.
    let (doc_comment, method_name) = match removal {
        Removal::Hard => (
            format!(
                "{unique_multi_column_index_hint}\n\nTry to delete a `{struct_name}` row in the `{singular_table_name}` table {described_as}."
            ),
            format_ident!("delete_{singular_table_name}_by_{index_name}"),
        ),
        Removal::Soft => (
            format!(
                "{unique_multi_column_index_hint}\n\nTry to soft-delete a `{struct_name}` row in the `{singular_table_name}` table {described_as}."
            ),
            format_ident!("soft_delete_{singular_table_name}_by_{index_name}"),
        ),
    };

    SpacetimeDSLMethod {
        doc_comment,
        method_name,
        method_args,
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl,
        read_context_compatible: false,
    }
}
