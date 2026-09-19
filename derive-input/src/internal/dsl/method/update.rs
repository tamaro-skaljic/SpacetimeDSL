use super::{
    context::MethodGenerationContext,
    index::IndexShape,
    reference_integrity::{
        Action, multi_column_index_checks, reference_integrity_checks_on_update,
    },
    upsert::{
        ForeignKeyColumnScope, after_update_hook, before_update_hook_use_and_call,
        rebind_row_as_mutable_after_hook, row_value_getters_for_foreign_key_columns,
        set_singleton_primary_key, set_updated_at_on_update,
    },
};
use crate::{
    api::{
        dsl::method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
        runtime,
    },
    internal::dsl::one_or_multiple::OneOrMultiple,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// `update_<table>_by_<index>`: write a row back over the one the index finds.
///
/// Only the primary key gets here. SpacetimeDB puts `update` on the primary key index
/// alone, so the index this takes is always single-column.
pub(in crate::internal) fn for_update(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
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

    let row_value_getters = row_value_getters_for_foreign_key_columns(
        internal_columns,
        ForeignKeyColumnScope::CheckedOnUpdate,
    );

    let on_update_set_current_timestamp =
        set_updated_at_on_update(spacetimedsl_table, internal_columns, singular_table_name);

    let use_itertools = if !multi_column_index_checks.is_empty() {
        runtime::itertools_import()
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
        is_singleton_pk,
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
    let (use_before_update_hook_trait, before_update_hook_call) = before_update_hook_use_and_call(
        spacetimedsl_table,
        singular_table_name,
        field_name_for_found_value,
    );

    let rebind_row_as_mutable = rebind_row_as_mutable_after_hook(
        singular_table_name,
        &before_update_hook_call,
        &[&on_update_set_current_timestamp],
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

    let after_update_hook = after_update_hook(
        spacetimedsl_table,
        singular_table_name,
        field_name_for_found_value,
    );

    let set_singleton_id_to_zero = match is_singleton_pk {
        false => TokenStream::default(),
        true => set_singleton_primary_key(singular_table_name),
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

            #before_update_hook

            #rebind_row_as_mutable
            #on_update_set_current_timestamp

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
