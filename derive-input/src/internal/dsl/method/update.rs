use super::{
    context::MethodGenerationContext,
    hook_call::{hook_tokens, hook_use_and_call},
    index::IndexShape,
    reference_integrity::{
        Action, multi_column_index_checks, reference_integrity_checks_on_update,
    },
};
use crate::{
    api::{
        dsl::{
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            wrapper::WrapperType,
        },
        runtime,
        rust::visibility::RustVisibility,
    },
    internal::{
        column::{ColumnTypeKind, InternalColumn},
        dsl::{one_or_multiple::OneOrMultiple, singleton},
    },
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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
