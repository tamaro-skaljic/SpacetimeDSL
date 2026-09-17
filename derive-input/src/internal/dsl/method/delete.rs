use super::get_referenced_table_function_call_for_dsl_method;
use super::{
    context::{self, MethodGenerationContext},
    hook_call::hook_tokens,
    index::{IndexColumnArguments, IndexShape, index_accessor, index_column_arguments},
    reference_integrity::{Action, get_unique_multi_column_index_check},
};
use crate::{
    api::{
        dsl::{foreign_key::OnDeleteStrategy, method::SpacetimeDSLMethod},
        runtime,
    },
    internal::{column::ColumnTypeKind, dsl::one_or_multiple::OneOrMultiple},
};
use quote::{format_ident, quote};

/// `delete_<tables>_by_<index>`: delete every row an index matches.
pub(in crate::internal) fn for_delete_many(
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

    let impl_until_return_ok_on_is_empty = quote! {
        #itertools_import

        #(#wrapper_option_mappers)*

        #let_index_name

        let rows_to_delete: Vec<#struct_name> = #index_accessor
            .filter(#index_name)
            .collect();

        if rows_to_delete.is_empty() {
            return Ok(#empty_deletion_result);
        }
    };

    let wrapper_type_struct_name_or_path = context::primary_key_wrapper_type(primary_key_column);

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

        let error_strategy = get_referenced_table_function_call_for_dsl_method(
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Error,
            OneOrMultiple::Multiple,
            &quote! {
                let error = #deletion_result_from_entries_with_error_from_hook;

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

/// `delete_<table>_by_<index>`: delete the one row a unique index finds.
pub(in crate::internal) fn for_delete_one(
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

    let itertools_import = runtime::itertools_import();

    let impl_until_return_err_on_is_none = quote! {
        #itertools_import

        #(#wrapper_option_mappers)*

        #get_row_to_delete

        #return_error_on_is_none
    };

    let wrapper_type_struct_name_or_path = context::primary_key_wrapper_type(primary_key_column);

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

        let error_strategy = get_referenced_table_function_call_for_dsl_method(
            singular_table_name,
            primary_key_column_name,
            OnDeleteStrategy::Error,
            OneOrMultiple::One,
            &quote! {
                let error = #single_entry_deletion_result_with_error_from_hook;

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
    };

    SpacetimeDSLMethod {
        doc_comment: format!(
            "{unique_multi_column_index_hint}\n\nTry to delete a `{struct_name}` row in the `{singular_table_name}` table {described_as}."
        ),
        method_name: format_ident!("delete_{singular_table_name}_by_{index_name}"),
        method_args,
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl,
        read_context_compatible: false,
    }
}
