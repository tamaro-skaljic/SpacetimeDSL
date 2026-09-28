//! What retiring a row and removing it have in common.
//!
//! `delete_<tables>_by_<index>` and `soft_delete_<tables>_by_<index>` differ in four
//! places: the statement that writes, the hooks they call, the dispatcher they cascade
//! through and the strategy they report. Everything around those four — finding the rows,
//! building the entries, refusing before writing, running the remaining strategy passes —
//! is stated once here, because two copies of that order would drift.

use {
    super::{
        context::{self, MethodGenerationContext},
        hook_call::hook_tokens,
        index::{IndexColumnArguments, IndexShape, index_accessor, index_column_arguments},
        message,
        reference_integrity::{Action, unique_multi_column_index_check},
        referenced_by::referenced_table_function_call_for_dsl_method,
        soft_delete,
        upsert::rebind_row_as_mutable_after_hook,
    },
    crate::{
        api::{
            dsl::{
                foreign_key::OnDeleteStrategy,
                hook::HookKind,
                method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
                soft_delete::SoftDeleteMarker,
                table::SpacetimeDSLTable,
            },
            runtime,
        },
        internal::{column::ColumnTypeKind, dsl::one_or_multiple::OneOrMultiple},
    },
    itertools::Itertools,
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
};

/// Whether a generated method removes the rows it matched or retires them.
#[derive(Clone, Copy, PartialEq)]
pub enum Removal {
    Hard,
    Soft,
}

impl Removal {
    /// How a doc comment says the rows were removed: "was deleted", "were soft-deleted", and
    /// so on. `naming` derives the dispatcher names from the same words.
    pub fn past_tense(self, one_or_multiple: &OneOrMultiple) -> &'static str {
        match (self, one_or_multiple) {
            (Removal::Hard, OneOrMultiple::One) => "was deleted",
            (Removal::Hard, OneOrMultiple::Multiple) => "were deleted",
            (Removal::Soft, OneOrMultiple::One) => "was soft-deleted",
            (Removal::Soft, OneOrMultiple::Multiple) => "were soft-deleted",
        }
    }
}

/// The arguments and the return type every cascade dispatcher has: the DSL, the strategy,
/// then one primary key value or a slice of them under `key_arg_name`; it returns the
/// entries it built, in a `Vec` for one row or in a `HashMap` keyed by primary key value
/// for several.
pub fn dispatcher_signature(
    one_or_multiple: &OneOrMultiple,
    strategy_type: TokenStream,
    key_arg_name: &syn::Ident,
    primary_key_column_type: &impl quote::ToTokens,
) -> (Vec<SpacetimeDSLArg>, TokenStream) {
    let deletion_result_entry_type = runtime::deletion_result_entry_type();

    let (key_type, entries_type) = match one_or_multiple {
        OneOrMultiple::One => (
            quote! { &#primary_key_column_type },
            quote! { Vec<#deletion_result_entry_type> },
        ),
        OneOrMultiple::Multiple => (
            quote! { &'a [#primary_key_column_type] },
            quote! {
                std::collections::HashMap<&'a #primary_key_column_type, Vec<#deletion_result_entry_type>>
            },
        ),
    };

    let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);

    let arguments = vec![
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("dsl"),
            arg_type: SpacetimeDSLArgType::Normal(runtime::dsl_reference_type()),
        },
        SpacetimeDSLArg {
            is_option: false,
            arg_name: format_ident!("strategy"),
            arg_type: SpacetimeDSLArgType::Normal(strategy_type),
        },
        SpacetimeDSLArg {
            is_option: false,
            arg_name: key_arg_name.clone(),
            arg_type: SpacetimeDSLArgType::Normal(key_type),
        },
    ];

    (
        arguments,
        quote! {
            Result<#entries_type, #failure_type>
        },
    )
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

/// The statements that retire the row `row` points at.
///
/// SpacetimeDB has no bulk update, so retiring many rows is retiring one row repeatedly,
/// and both row counts emit this. The before hook hands back the row to write, so it runs
/// between the clone and the marker rather than around the whole statement; the `mut` then
/// moves to a rebinding after it, because the hook's own binding is not `mut` — the shape
/// `update_<table>_by_<key>` uses, for the same reason.
///
/// `updated_at` is deliberately left alone: the marker records the retirement, and
/// `updated_at` keeps meaning the last ordinary edit.
fn retire_row(
    row: &syn::Ident,
    spacetimedsl_table: &SpacetimeDSLTable,
    singular_table_name: &syn::Ident,
    primary_key_column_name: &syn::Ident,
) -> TokenStream {
    let new_row = format_ident!("new_row");
    let set_marker =
        soft_delete::set_marker(marker_of(spacetimedsl_table), &quote! { self }, &new_row);

    let before_hook = hook_tokens(
        spacetimedsl_table.hooks.get(HookKind::BEFORE_SOFT_DELETE),
        |hook_function_name| {
            let hook_call =
                runtime::dsl_method_hooks_call(hook_function_name, &quote! { self, #row, new_row });

            quote! {
                let new_row = #hook_call?;
            }
        },
    );

    let after_hook = hook_tokens(
        spacetimedsl_table.hooks.get(HookKind::AFTER_SOFT_DELETE),
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, #row, &new_row },
            );

            quote! {
                #hook_call?;
            }
        },
    );

    let clone_row = match before_hook.is_empty() {
        true => quote! { let mut new_row = #row.clone(); },
        false => quote! { let new_row = #row.clone(); },
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
pub fn for_removal_many(
    removal: Removal,
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    let index_name = &shape.index_name;

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

    let find_rows_to_delete = quote! {
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

    removal_method(
        removal,
        &OneOrMultiple::Multiple,
        shape,
        context,
        method_args,
        find_rows_to_delete,
    )
}

/// `delete_<table>_by_<index>` and `soft_delete_<table>_by_<index>`: retire or remove the
/// one row a unique index finds.
pub fn for_removal_one(
    removal: Removal,
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        internal_columns,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        field_name_for_found_value,
        ..
    } = context;

    let index_name = &shape.index_name;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::One, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    let get_row_to_delete;
    let return_error_on_is_none;

    match shape.is_multi_column {
        true => {
            let multi_column_index_check = unique_multi_column_index_check(
                &Action::Delete,
                singular_table_name,
                field_name_for_found_value,
                index_name,
                &shape.index_columns,
                &row_value_getters,
            );

            get_row_to_delete = quote! {
                let mut #field_name_for_found_value: Option<#struct_name> = None;

                #multi_column_index_check

                let row_to_delete = #field_name_for_found_value;
            };

            let not_found_error = runtime::not_found_error(
                singular_table_name_as_string,
                &message::column_names_and_row_values(&shape.index_columns, &row_value_getters),
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
                &message::column_names_and_row_values(
                    &shape.index_columns,
                    &[quote! { &#index_name }],
                ),
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

    let find_row_to_delete = quote! {
        #itertools_import

        #(#wrapper_option_mappers)*

        #get_row_to_delete

        #return_error_on_is_none

        #return_ok_on_already_retired
    };

    removal_method(
        removal,
        &OneOrMultiple::One,
        shape,
        context,
        method_args,
        find_row_to_delete,
    )
}

/// Everything a removal method does once `find_rows` has bound the rows to remove —
/// `row_to_delete` for one row, `rows_to_delete` for many: build the entries, refuse
/// through the `Error` strategy, run the hooks around the write, write, run the remaining
/// strategies, and report. The kind of removal and the row count are the only variables.
fn removal_method(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    shape: &IndexShape,
    context: &MethodGenerationContext,
    method_args: Vec<SpacetimeDSLArg>,
    find_rows: TokenStream,
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
        &quote! { vec![] },
    );

    // Variation point: one entry, or one per row keyed by its primary key value.
    let (build_entries, entries) = match one_or_multiple {
        OneOrMultiple::One => (
            quote! {
                let mut deletion_result_entry = #deletion_result_entry_for_row;
            },
            quote! { vec![deletion_result_entry] },
        ),
        OneOrMultiple::Multiple => (
            quote! {
                let mut deletion_result_entries = std::collections::HashMap::new();

                for row_to_delete in &rows_to_delete {
                    deletion_result_entries.insert(
                        &row_to_delete.#primary_key_column_name,
                        #deletion_result_entry_for_row
                    );
                }
            },
            quote! { deletion_result_entries.into_values().collect_vec() },
        ),
    };

    let deletion_result = runtime::deletion_result(
        singular_table_name_as_string,
        one_or_multiple,
        &entries,
        &quote! { None },
    );

    let deletion_result_with_error_from_hook = runtime::deletion_result(
        singular_table_name_as_string,
        one_or_multiple,
        &entries,
        &quote! { error_from_hook },
    );

    // Variation point: which pair of hooks runs, and where.
    //
    // A hard deletion's hooks bracket the write, so they are blocks of their own, looping
    // over the rows when there are many. A soft deletion writes each row separately and its
    // before hook hands back the row to write, so its hooks belong inside `retire_row`
    // rather than around it; the two outer slots are then empty.
    let deletion_hook = |hook_kind| match removal {
        Removal::Hard => hard_deletion_hook(spacetimedsl_table, hook_kind, one_or_multiple),
        Removal::Soft => TokenStream::default(),
    };

    let before_delete_hook = deletion_hook(HookKind::BEFORE_DELETE);
    let after_delete_hook = deletion_hook(HookKind::AFTER_DELETE);

    // Variation point: the statement that writes, and the check around it.
    let write = match (removal, one_or_multiple) {
        (Removal::Hard, OneOrMultiple::One) => {
            let count_mismatch_error = delete_one_count_mismatch_error();

            quote! {
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
            }
        }
        (Removal::Hard, OneOrMultiple::Multiple) => {
            let index_accessor = index_accessor(singular_table_name, index_name);

            let count_mismatch_error = runtime::generic_error(&quote! {
                format!(
                    "Delete Many Error: `count_of_rows_to_delete ( {} ) != ( {} ) count_of_deleted_rows`!",
                    &count_of_rows_to_delete,
                    &count_of_deleted_rows
                )
            });

            quote! {
                let count_of_rows_to_delete: u64 = rows_to_delete
                    .len()
                    .try_into()
                    .unwrap_or(u64::MAX);

                let count_of_deleted_rows = #index_accessor.delete(#index_name);

                if count_of_rows_to_delete.ne(&count_of_deleted_rows) {
                    return Err(#count_mismatch_error);
                }
            }
        }
        (Removal::Soft, _) => {
            let old_row = format_ident!("old_row");
            let retire_row = retire_row(
                &old_row,
                spacetimedsl_table,
                singular_table_name,
                primary_key_column_name,
            );

            match one_or_multiple {
                OneOrMultiple::One => quote! {
                    let #old_row = &row_to_delete;

                    #retire_row
                },
                OneOrMultiple::Multiple => quote! {
                    for #old_row in &rows_to_delete {
                        #retire_row
                    }
                },
            }
        }
    };

    let return_result = quote! {
        return Ok(#deletion_result);
    };

    let method_impl = if spacetimedsl_table.referencing_tables.is_empty() {
        quote! {
            #find_rows

            #build_entries

            #before_delete_hook

            #write

            #after_delete_hook

            #return_result
        }
    } else {
        let error_after_state_change_message = format!(
            "{}: An error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {{error}}",
            removal_error_name(removal, one_or_multiple)
        );

        let error_after_state_change = runtime::generic_error(&quote! {
            format!(#error_after_state_change_message)
        });

        let on_error_handler = quote! {
            let error = #deletion_result_with_error_from_hook;

            return Err(#error_after_state_change);
        };

        let reference_integrity_violation_on_delete_error =
            runtime::reference_integrity_violation_on_delete(&quote! { error });

        let error_strategy = referenced_table_function_call_for_dsl_method(
            removal,
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Error,
            *one_or_multiple,
            &quote! {
                let error = #deletion_result_with_error_from_hook;

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
                    *one_or_multiple,
                    &on_error_handler,
                )
            })
            .collect_vec();

        quote! {
            #find_rows

            #build_entries

            #error_strategy

            #before_delete_hook

            #write

            #after_delete_hook

            #(#strategies_after_the_write)*

            #return_result
        }
    };

    // Variation point: what the method is called and what it says it does.
    let (verb, method_prefix) = match removal {
        Removal::Hard => ("delete", "delete"),
        Removal::Soft => ("soft-delete", "soft_delete"),
    };

    let (doc_comment, method_name) = match one_or_multiple {
        OneOrMultiple::One => (
            format!(
                "{}\n\nTry to {verb} a `{struct_name}` row in the `{singular_table_name}` table {described_as}.",
                shape.unique_multi_column_hint
            ),
            format_ident!("{method_prefix}_{singular_table_name}_by_{index_name}"),
        ),
        OneOrMultiple::Multiple => (
            format!(
                "Try to {verb} all `{struct_name}` rows in the `{singular_table_name}` table {described_as}."
            ),
            format_ident!("{method_prefix}_{plural_table_name}_by_{index_name}"),
        ),
    };

    SpacetimeDSLMethod {
        doc_comment,
        method_name,
        method_args,
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl,
        // A removal writes.
        read_context_compatible: false,
    }
}

/// How an error raised after the database changed names the method it came from:
/// "Delete One Error", "Soft Delete Many Error", and so on.
fn removal_error_name(removal: Removal, one_or_multiple: &OneOrMultiple) -> &'static str {
    match (removal, one_or_multiple) {
        (Removal::Hard, OneOrMultiple::One) => "Delete One Error",
        (Removal::Hard, OneOrMultiple::Multiple) => "Delete Many Error",
        (Removal::Soft, OneOrMultiple::One) => "Soft Delete One Error",
        (Removal::Soft, OneOrMultiple::Multiple) => "Soft Delete Many Error",
    }
}

/// The `before_delete` or `after_delete` hook of a hard deletion: one call on
/// `row_to_delete`, or one per row in `rows_to_delete`.
pub(super) fn hard_deletion_hook(
    spacetimedsl_table: &SpacetimeDSLTable,
    hook_kind: HookKind,
    one_or_multiple: &OneOrMultiple,
) -> TokenStream {
    hook_tokens(
        spacetimedsl_table.hooks.get(hook_kind),
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! { self, &row_to_delete },
            );

            match one_or_multiple {
                OneOrMultiple::One => quote! {
                    #hook_call?;
                },
                OneOrMultiple::Multiple => quote! {
                    for row_to_delete in &rows_to_delete {
                        #hook_call?;
                    }
                },
            }
        },
    )
}

/// The error a one-row hard deletion returns when SpacetimeDB deleted nothing.
pub(super) fn delete_one_count_mismatch_error() -> TokenStream {
    runtime::generic_error(&quote! {
        "Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string()
    })
}
