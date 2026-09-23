//! The referencing side of a foreign key: the function a table generates for each table it
//! references, which the referenced side calls when one of its rows is deleted.
//!
//! One function per referenced table, holding a match arm per on-delete strategy. The
//! referenced side is [`super::referenced_by`].

use super::{
    context::TableContributions,
    naming::{
        referenced_table_compile_error_check_for_deletions,
        referenced_table_compile_error_check_for_soft_deletions,
        referencing_table_compile_error_check_for_deletions,
        referencing_table_compile_error_check_for_soft_deletions, referencing_table_function_name,
    },
    on_delete_strategy::{ReferencingTables, on_delete_strategy_implementation},
    removal::Removal,
};
use crate::{
    api::{
        Column,
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
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::BTreeMap;
use strum::IntoEnumIterator;

#[allow(clippy::too_many_arguments)]
pub(in crate::internal) fn for_foreign_key(
    removal: Removal,
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

        // A foreign key sets `on_delete`, `on_soft_delete` or both, so a column
        // contributes to the grouping for one kind of removal and not necessarily the
        // other.
        let foreign_key = column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .unwrap_or_else(|| {
                panic!(
                    "the column {} is in a foreign key group, so it carries a foreign key",
                    column_with_foreign_key.rust_field.name
                )
            });

        let on_delete_strategy = match removal {
            Removal::Hard => &foreign_key.on_delete_strategy,
            Removal::Soft => &foreign_key.on_soft_delete_strategy,
        };

        let on_delete_strategy = match on_delete_strategy {
            None => continue,
            Some(on_delete_strategy) => on_delete_strategy,
        };

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

    let function_name = referencing_table_function_name(
        removal,
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

    let past_tense = match (removal, one_or_multiple) {
        (Removal::Hard, OneOrMultiple::One) => "was deleted",
        (Removal::Hard, OneOrMultiple::Multiple) => "were deleted",
        (Removal::Soft, OneOrMultiple::One) => "was soft-deleted",
        (Removal::Soft, OneOrMultiple::Multiple) => "were soft-deleted",
    };

    let return_type;

    let arg_name;

    match one_or_multiple {
        OneOrMultiple::One => {
            doc_comment = format!(
                "Execute On Delete Strategies of the referencing table `{singular_table_name}` after one row of the referenced table `{referenced_table_name}` {past_tense}."
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
                "Execute On Delete Strategies of the referencing table `{singular_table_name}` after multiple rows of the referenced table `{referenced_table_name}` {past_tense}."
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
                Some(columns_by_on_delete_strategy) => on_delete_strategy_implementation(
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

    // This table emits the half it declares a strategy for, and imports from the
    // referenced table the half that table must be able to perform.
    let compile_error_check = match removal {
        Removal::Hard => referencing_table_compile_error_check_for_deletions(
            singular_table_name,
            &referenced_table_name,
        ),
        Removal::Soft => referencing_table_compile_error_check_for_soft_deletions(
            singular_table_name,
            &referenced_table_name,
        ),
    };

    contributions
        .compile_error_checks
        .insert(compile_error_check.clone());

    let compile_error_check = match removal {
        Removal::Hard => referenced_table_compile_error_check_for_deletions(
            &referenced_table_name,
            singular_table_name,
        ),
        Removal::Soft => referenced_table_compile_error_check_for_soft_deletions(
            &referenced_table_name,
            singular_table_name,
        ),
    };

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
