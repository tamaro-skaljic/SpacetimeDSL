//! The referenced side of a foreign key: the two entry points a table earns when another
//! table declares `#[referenced_by]` on it, and the call its own delete methods make into
//! them.
//!
//! These fan out to every referencing table. The referencing side is
//! [`super::foreign_key`].

use super::{
    context::TableContributions,
    naming::{
        get_referenced_table_compile_error_check, get_referenced_table_function_name,
        get_referencing_table_compile_error_check, get_referencing_table_function_name,
    },
};
use crate::{
    api::{
        db::table::SpacetimeDBTable,
        dsl::{
            foreign_key::OnDeleteStrategy,
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            table::SpacetimeDSLTable,
        },
        runtime,
    },
    internal::{column::InternalColumn, dsl::one_or_multiple::OneOrMultiple},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

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

pub(in crate::internal) fn for_referenced_by(
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
