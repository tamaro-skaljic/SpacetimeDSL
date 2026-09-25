//! `upsert_<table>`: write the one row of a `#[dsl(singleton(with_default))]` table,
//! whether or not that row exists yet - and the pieces this shares with `update_<table>`.
//!
//! A table with a default has no create method, so this is the only method that writes its
//! row. Its body is an update path and an insert path side by side, which is why the parts
//! it has in common with `update.rs` live here instead of being copied: the foreign-key row
//! values, the `updated_at` assignment, the update hooks and the singleton primary key.
//! `update.rs` is the other caller of those, and the `SetZero` cascade in
//! `on_delete_strategy.rs` writes `updated_at` through the same assignment.
//!
//! Both paths call their hook before they set the columns the framework owns, so the
//! framework has the last word on a timestamp. `create_<table>`, `update_<table>_by_<key>`
//! and `soft_delete_<table>_by_<index>` order the two the same way.

use super::{
    context::MethodGenerationContext,
    hook_call::{hook_tokens, hook_use_and_call},
    index::column_names_and_row_values,
    reference_integrity::{
        reference_integrity_checks_on_create, reference_integrity_checks_on_update,
    },
};
use crate::{
    api::{
        dsl::{
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            table::SpacetimeDSLTable,
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
use syn::Ident;

//region Pieces shared with `update.rs`

/// Which foreign-key columns a write path binds a row value for.
///
/// The binding lets a reference-integrity check read the column's current value without
/// reaching through the row again. Which columns need one differs between the two check
/// sets, so each set names its own scope rather than both settling for the wider one and
/// leaving the generated code with a binding nothing reads.
pub(in crate::internal) enum ForeignKeyColumnScope {
    /// An update check skips a private column, because a column without a setter cannot
    /// change, and it renders the value of every other one into its violation message.
    CheckedOnUpdate,
    /// A create check runs on every column, private ones included, but reads the value only
    /// in the guard that skips a reference which is not set yet.
    CheckedOnCreate,
}

impl ForeignKeyColumnScope {
    fn covers(&self, internal_column: &InternalColumn) -> bool {
        if internal_column.spacetimedsl_column_foreign_key.is_none() {
            return false;
        }

        match self {
            ForeignKeyColumnScope::CheckedOnUpdate => internal_column
                .rust_field_visibility
                .to_string()
                .ne(&RustVisibility::Private.to_string()),
            ForeignKeyColumnScope::CheckedOnCreate => matches!(
                internal_column.rust_field_type_kind,
                ColumnTypeKind::UnsignedInteger | ColumnTypeKind::Optional
            ),
        }
    }
}

/// The `let` binding a reference-integrity check reads a column's value through. Unlike the
/// Create path there is always one, and the wrapper handling the Create path needs is
/// irrelevant here, because a write path reads the row rather than building it.
fn row_value_getter(internal_column: &InternalColumn) -> TokenStream {
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

pub(in crate::internal) fn row_value_getters_for_foreign_key_columns(
    internal_columns: &[InternalColumn],
    scope: ForeignKeyColumnScope,
) -> Vec<TokenStream> {
    internal_columns
        .iter()
        .filter(|internal_column| scope.covers(internal_column))
        .map(row_value_getter)
        .collect()
}

/// The column an `updated_at` role named, and whether its type is optional.
fn updated_at_column<'a>(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_columns: &'a [InternalColumn],
) -> Option<(&'a Ident, bool)> {
    let column_name = spacetimedsl_table
        .on_update_set_current_timestamp_column_name
        .as_ref()?;

    let internal_column = internal_columns
        .iter()
        .find(|c| c.rust_field_name.eq(column_name))
        .unwrap_or_else(|| {
            panic!("The column {column_name} named by an on_update attribute must be one of this table's columns")
        });

    Some((
        &internal_column.rust_field_name,
        internal_column.rust_field_type_kind == ColumnTypeKind::Optional,
    ))
}

/// `<row>.<updated_at> = <now>;`, or nothing when the table declares no such column.
///
/// `dsl` is what the timestamp is read through: `self` in a DSL method, the `dsl` argument
/// in a cascade function.
pub(in crate::internal) fn set_updated_at_on_update(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_columns: &[InternalColumn],
    dsl: &TokenStream,
    row: &Ident,
) -> TokenStream {
    match updated_at_column(spacetimedsl_table, internal_columns) {
        None => TokenStream::default(),
        Some((column_name, is_optional)) => {
            let current_timestamp = runtime::current_timestamp(dsl);
            let timestamp_value = match is_optional {
                true => quote! { Some(#current_timestamp) },
                false => current_timestamp,
            };

            quote! {
                #row.#column_name = #timestamp_value;
            }
        }
    }
}

/// `<row>.<updated_at> = <nothing yet>;`, the insert path's counterpart.
///
/// An optional column stays `None`, which is how `create_<table>` says "this row was never
/// updated". A column that cannot hold `None` gets the insert time instead.
fn set_updated_at_on_insert(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_columns: &[InternalColumn],
    row: &Ident,
) -> TokenStream {
    match updated_at_column(spacetimedsl_table, internal_columns) {
        None => TokenStream::default(),
        Some((column_name, is_optional)) => {
            let timestamp_value = match is_optional {
                true => quote! { None },
                false => runtime::current_timestamp(&quote! { self }),
            };

            quote! {
                #row.#column_name = #timestamp_value;
            }
        }
    }
}

/// The `use` and the call of the `before_update` hook, kept apart because `update.rs` has to
/// place a prelude between them.
pub(in crate::internal) fn before_update_hook_use_and_call(
    spacetimedsl_table: &SpacetimeDSLTable,
    row: &Ident,
    found_row: &Ident,
) -> (TokenStream, TokenStream) {
    hook_use_and_call(
        &spacetimedsl_table.hooks.before_update,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! {
                    self,
                    #found_row.as_ref().unwrap(),
                    #row
                },
            );

            quote! {
                let #row = #hook_call?;
            }
        },
    )
}

/// `let mut <row> = <row>;`, needed only when a hook call shadowed the mutable outer binding
/// with a non-`mut` one and a write to a framework-owned column follows it.
///
/// The hook's own `let #row = #hook_call?;` is never `mut`, because most tables have no
/// framework-owned column left to write after it; making that binding `mut` unconditionally
/// would leave `unused_mut` on every one of those. Rebinding once here, gated on a write
/// actually following, keeps both shapes free of warnings, and doing it at the call site
/// (rather than inside `keep_created_at`, `set_updated_at_on_update`, and their kin) means the
/// two writes that can follow a hook cannot each emit their own rebinding.
pub(in crate::internal) fn rebind_row_as_mutable_after_hook(
    row: &Ident,
    hook_call: &TokenStream,
    framework_owned_writes: &[&TokenStream],
) -> TokenStream {
    let hook_shadowed_the_binding = !hook_call.is_empty();
    let a_write_follows = framework_owned_writes.iter().any(|write| !write.is_empty());

    match hook_shadowed_the_binding && a_write_follows {
        true => quote! { let mut #row = #row; },
        false => TokenStream::default(),
    }
}

pub(in crate::internal) fn after_update_hook(
    spacetimedsl_table: &SpacetimeDSLTable,
    row: &Ident,
    found_row: &Ident,
) -> TokenStream {
    hook_tokens(
        &spacetimedsl_table.hooks.after_update,
        |hook_function_name| {
            let hook_call = runtime::dsl_method_hooks_call(
                hook_function_name,
                &quote! {
                    self,
                    #found_row.as_ref().unwrap(),
                    &#row
                },
            );

            quote! {
                #hook_call?;
            }
        },
    )
}

/// `<row>.id = 0;` - a singleton's row is written under its one legal key, whatever the
/// caller left in that field.
pub(in crate::internal) fn set_singleton_primary_key(row: &Ident) -> TokenStream {
    let primary_key = singleton::primary_key_ident();
    let primary_key_value = singleton::primary_key_value();

    quote! {
        #row.#primary_key = #primary_key_value;
    }
}

//endregion Pieces shared with `update.rs`

/// `<row>.<created_at> = <the stored value>;`, so an update cannot move the insert time.
///
/// The caller hands in a whole row, and that row may well have come from
/// `DefaultSingleton::get_default` rather than from the table, so trusting its `created_at`
/// would let a default overwrite a real one.
fn keep_created_at_on_update(
    spacetimedsl_table: &SpacetimeDSLTable,
    row: &Ident,
    found_row: &Ident,
) -> TokenStream {
    match &spacetimedsl_table.on_insert_set_current_timestamp_column_name {
        None => TokenStream::default(),
        Some(column_name) => quote! {
            #row.#column_name = #found_row
                .as_ref()
                .expect("The update path only runs when the row was found")
                .#column_name;
        },
    }
}

/// `<row>.<created_at> = <now>;`, the one moment the insert time is written.
fn set_created_at_on_insert(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_columns: &[InternalColumn],
    row: &Ident,
) -> TokenStream {
    match &spacetimedsl_table.on_insert_set_current_timestamp_column_name {
        None => TokenStream::default(),
        Some(column_name) => {
            let internal_column = internal_columns
                .iter()
                .find(|column| column.rust_field_name.eq(column_name))
                .unwrap_or_else(|| {
                    panic!(
                        "The column {column_name} named by an on_insert attribute must be one of this table's columns"
                    )
                });
            let current_timestamp = runtime::current_timestamp(&quote! { self });
            let timestamp_value = match internal_column.rust_field_type_kind {
                ColumnTypeKind::Optional => quote! { Some(#current_timestamp) },
                _ => current_timestamp,
            };

            quote! {
                #row.#column_name = #timestamp_value;
            }
        }
    }
}

/// `upsert_<table>`: write the one row of a singleton table that has a default.
///
/// It emits no multi-column index check and no `itertools` import, because
/// `internal/db/table.rs` rejects a multi-column index on a singleton.
pub(in crate::internal) fn for_singleton_upsert(
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedb_table,
        spacetimedsl_table,
        internal_columns,
        primary_key_column,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        field_name_for_found_value,
        ..
    } = context;

    let primary_key = singleton::primary_key_ident();
    let primary_key_value = singleton::primary_key_value();

    let method_args = vec![SpacetimeDSLArg {
        is_option: false,
        arg_name: singular_table_name.clone(),
        arg_type: SpacetimeDSLArgType::Normal(quote! { #struct_name }),
    }];

    let index_columns = vec![primary_key.clone()];

    let checks_on_update = reference_integrity_checks_on_update(
        spacetimedb_table,
        internal_columns,
        &column_names_and_row_values(&index_columns),
        &index_columns,
        &OneOrMultiple::One,
        primary_key_column,
        true,
    );

    let checks_on_create =
        reference_integrity_checks_on_create(spacetimedb_table, internal_columns);

    // Only an update check writes the binding back, and only when it has to look the row up
    // itself - which it never does here, because the binding is filled before the branch.
    let found_row_mutability = match checks_on_update.is_empty() {
        true => TokenStream::default(),
        false => quote! { mut },
    };

    let row_values_on_update = row_value_getters_for_foreign_key_columns(
        internal_columns,
        ForeignKeyColumnScope::CheckedOnUpdate,
    );
    let row_values_on_create = row_value_getters_for_foreign_key_columns(
        internal_columns,
        ForeignKeyColumnScope::CheckedOnCreate,
    );

    let set_singleton_primary_key = set_singleton_primary_key(singular_table_name);

    let keep_created_at = keep_created_at_on_update(
        spacetimedsl_table,
        singular_table_name,
        field_name_for_found_value,
    );
    let set_created_at =
        set_created_at_on_insert(spacetimedsl_table, internal_columns, singular_table_name);

    let set_updated_at_on_update = set_updated_at_on_update(
        spacetimedsl_table,
        internal_columns,
        &quote! { self },
        singular_table_name,
    );
    let set_updated_at_on_insert =
        set_updated_at_on_insert(spacetimedsl_table, internal_columns, singular_table_name);

    let (use_before_update_hook_trait, before_update_hook_call) = before_update_hook_use_and_call(
        spacetimedsl_table,
        singular_table_name,
        field_name_for_found_value,
    );
    let rebind_row_as_mutable_on_update = rebind_row_as_mutable_after_hook(
        singular_table_name,
        &before_update_hook_call,
        &[&keep_created_at, &set_updated_at_on_update],
    );
    let after_update_hook = after_update_hook(
        spacetimedsl_table,
        singular_table_name,
        field_name_for_found_value,
    );

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
    let rebind_row_as_mutable_on_insert = rebind_row_as_mutable_after_hook(
        singular_table_name,
        &before_insert_hook,
        &[&set_created_at, &set_updated_at_on_insert],
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

    // The row does not exist yet, so the message renders the whole struct rather than naming
    // the columns a lookup was made on.
    let column_names_and_row_values = format!("{{{{ {singular_table_name} : {{:?}} }}}}");

    // FIXME: Only show unique columns here
    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        singular_table_name_as_string,
        &quote! { Create },
        &quote! { SpacetimeDB },
        &OneOrMultiple::One,
        &quote! { format!(#column_names_and_row_values, #singular_table_name) },
    );
    let auto_inc_overflow_error = runtime::auto_inc_overflow(singular_table_name_as_string);

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Write the `{struct_name}` row of the singleton `{singular_table_name}` table, whether or not it exists yet."
        ),
        method_name: format_ident!("upsert_{singular_table_name}"),
        method_args,
        return_type: runtime::error_result_type(struct_name),
        method_impl: quote! {
            let mut #singular_table_name = #singular_table_name;
            #set_singleton_primary_key

            let #found_row_mutability #field_name_for_found_value: Option<#struct_name> =
                self.db().#singular_table_name().#primary_key().find(&#primary_key_value);

            // An `if` rather than a `match`, because the checks and the hooks of the update
            // path read the binding the row was found in.
            if #field_name_for_found_value.is_some() {
                #(#row_values_on_update)*
                #(#checks_on_update)*

                #use_before_update_hook_trait
                #before_update_hook_call

                #rebind_row_as_mutable_on_update
                #keep_created_at
                #set_updated_at_on_update

                // FIXME: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/60 try_update instead of update and on error return Err(crate::spacetimedsl::error::SpacetimeDSLError);
                let #singular_table_name = self
                    .db()
                    .#singular_table_name()
                    .#primary_key()
                    .update(#singular_table_name);

                #after_update_hook

                Ok(#singular_table_name)
            } else {
                #(#row_values_on_create)*
                #(#checks_on_create)*

                #before_insert_hook

                #rebind_row_as_mutable_on_insert
                #set_created_at
                #set_updated_at_on_insert

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
            }
        },
        read_context_compatible: false,
    }
}
