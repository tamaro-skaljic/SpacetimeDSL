//! The referencing side of a foreign key: the function a table generates for each table it
//! references, which the referenced side calls when one of its rows is deleted.
//!
//! One function per referenced table, holding a match arm per on-delete strategy. The
//! referenced side is [`super::referenced_by`].

use {
    super::{
        context::MethodGenerationContext,
        naming::{cascade_binding, referencing_table_function_name},
        on_delete_strategy::{ReferencingTables, on_delete_strategy_implementation},
        removal::{Removal, dispatcher_signature},
    },
    crate::{
        api::{
            Column,
            dsl::{
                foreign_key::{ForeignKey, OnDeleteStrategy},
                method::SpacetimeDSLMethod,
            },
            runtime,
        },
        internal::dsl::one_or_multiple::OneOrMultiple,
    },
    itertools::Itertools,
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    std::collections::BTreeMap,
    strum::IntoEnumIterator,
};

/// The foreign key of a column which `foreign_key_columns_by_referenced_table` grouped by it.
pub fn foreign_key_of(column: &Column) -> &ForeignKey {
    column
        .spacetimedsl_column
        .foreign_key
        .as_ref()
        .expect("columns are grouped by their foreign key, so every one carries it")
}

pub fn for_foreign_key(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    referencing_tables: ReferencingTables,
    context: &MethodGenerationContext,
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &[&Column],
) -> SpacetimeDSLMethod {
    let first_foreign_key_column = columns_with_foreign_key
        .first()
        .expect("A table grouped by referenced table must have at least one foreign key column");

    let referenced_table_primary_key_column_type =
        &first_foreign_key_column.rust_field.type_name_or_path;

    let mut columns_by_on_delete_strategies: BTreeMap<_, Vec<&Column>> = BTreeMap::new();

    for column_with_foreign_key in columns_with_foreign_key {
        // A foreign key sets `on_delete`, `on_soft_delete` or both, so a column
        // contributes to the grouping for one kind of removal and not necessarily the
        // other.
        let Some(on_delete_strategy) =
            removal.strategy_declared_by(foreign_key_of(column_with_foreign_key))
        else {
            continue;
        };

        columns_by_on_delete_strategies
            .entry(on_delete_strategy)
            .or_default()
            .push(*column_with_foreign_key);
    }

    let singular_table_name = &context.singular_table_name;
    let referenced_table_name = format_ident!("{}", *referenced_table_name);

    let function_name = referencing_table_function_name(
        removal,
        one_or_multiple,
        singular_table_name,
        &referenced_table_name,
    );

    let past_tense = removal.past_tense(one_or_multiple);

    let (doc_comment, arg_name) = match one_or_multiple {
        OneOrMultiple::One => (
            format!(
                "Execute On Delete Strategies of the referencing table `{singular_table_name}` after one row of the referenced table `{referenced_table_name}` {past_tense}."
            ),
            cascade_binding::primary_key_value_of_a_row_of_another_table_to_delete(),
        ),
        OneOrMultiple::Multiple => (
            format!(
                "Execute On Delete Strategies of the referencing table `{singular_table_name}` after multiple rows of the referenced table `{referenced_table_name}` {past_tense}."
            ),
            cascade_binding::primary_key_values_of_rows_of_another_table_to_delete(),
        ),
    };

    let entries = cascade_binding::entries();
    let error = cascade_binding::error();
    let error_from_hook = cascade_binding::error_from_hook();
    let outer = cascade_binding::outer();
    let primary_key_value_of_a_row_of_another_table_to_delete =
        cascade_binding::primary_key_value_of_a_row_of_another_table_to_delete();
    let primary_key_values_of_rows_of_another_table_to_delete =
        cascade_binding::primary_key_values_of_rows_of_another_table_to_delete();

    let on_delete_strategy_type = runtime::on_delete_strategy_type();

    let (function_args, return_type) = dispatcher_signature(
        one_or_multiple,
        quote! { &#on_delete_strategy_type },
        &arg_name,
        referenced_table_primary_key_column_type,
    );

    let create_data_structure_for_child_entries = match one_or_multiple {
        OneOrMultiple::One => {
            quote! {
                let mut #entries = vec![];
            }
        }
        OneOrMultiple::Multiple => {
            quote! {
                let mut #entries = std::collections::HashMap::new();
                for #primary_key_value_of_a_row_of_another_table_to_delete in #primary_key_values_of_rows_of_another_table_to_delete {
                    #entries.insert(#primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
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
                    context,
                    referencing_tables,
                    &on_delete_strategy,
                    columns_by_on_delete_strategy,
                    one_or_multiple,
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

    let error_from_hook_declaration = runtime::error_from_hook_declaration(&error_from_hook);
    let failure = runtime::on_delete_strategy_failure(&entries, &error_from_hook);

    let itertools_import = runtime::itertools_import();

    let function_impl = quote! {
        #itertools_import
        #create_data_structure_for_child_entries

        #error_from_hook_declaration
        let mut #error = false;

        #outer: {
            match &strategy {
                #(#strategy_implementations)*
            };
        }

        match #error {
            false => Ok(#entries),
            true => Err(#failure),
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        method_name: function_name,
        method_args: function_args,
        return_type,
        method_impl: function_impl,
        // A strategy writes the referencing rows.
        read_context_compatible: false,
    }
}
