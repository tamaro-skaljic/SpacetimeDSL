//! The contract between the code SpacetimeDSL generates and the `spacetimedsl` runtime
//! crate it runs against.
//!
//! Every `crate::spacetimedsl::…` path any generator emits is written here exactly once, so
//! renaming or relocating a runtime item is a change to this file rather than a text hunt
//! through `quote!` bodies that no compiler checks. A crate building on
//! [`crate::api::Table`] emits the same paths by calling these, instead of spelling them
//! out and drifting from them.
//!
//! These are plain token constructors: they splice already-built token streams and take no
//! decisions. They must not grow branching, or they become a second generator.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// `Result<#ok_type, SpacetimeDSLError>`, the return type of every fallible DSL method.
pub fn error_result_type(ok_type: &impl ToTokens) -> TokenStream {
    quote! {
        Result<#ok_type, crate::spacetimedsl::error::SpacetimeDSLError>
    }
}

/// `error::SpacetimeDSLError`, as a type rather than a value.
pub fn spacetimedsl_error_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError
    }
}

/// `<dsl>.ctx().timestamp()`, unwrapped.
///
/// `GetTimestamp` only fails for the view contexts, and a `DSL` only ever wraps a
/// `WriteContext`, so generated write code never reaches the `expect`.
pub fn current_timestamp(dsl: &impl ToTokens) -> TokenStream {
    quote! {
        #dsl.ctx().timestamp().expect("a `WriteContext` always has a timestamp")
    }
}

/// `SpacetimeDSLError::NotFoundError { .. }`
pub fn not_found_error(
    table_name: &impl ToTokens,
    column_names_and_row_values: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError::NotFoundError {
            table_name: #table_name.into(),
            column_names_and_row_values: #column_names_and_row_values.into()
        }
    }
}

/// `SpacetimeDSLError::UniqueConstraintViolation { .. }`. `action` and `error_from` name
/// a variant of `error::Action` and `error::ErrorFrom` respectively.
pub fn unique_constraint_violation(
    table_name: &impl ToTokens,
    action: &impl ToTokens,
    error_from: &impl ToTokens,
    one_or_multiple: &impl ToTokens,
    column_names_and_row_values: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError::UniqueConstraintViolation {
            table_name: #table_name.into(),
            action: crate::spacetimedsl::error::Action::#action,
            error_from: crate::spacetimedsl::error::ErrorFrom::#error_from,
            one_or_multiple: #one_or_multiple,
            column_names_and_row_values: #column_names_and_row_values.into()
        }
    }
}

/// `SpacetimeDSLError::Error(#message)`, the catch-all variant carrying a prose message.
pub fn generic_error(message: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError::Error(#message)
    }
}

/// `SpacetimeDSLError::ReferenceIntegrityViolation(ReferenceIntegrityViolationError::OnDelete(..))`
pub fn reference_integrity_violation_on_delete(deletion_result: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError::ReferenceIntegrityViolation(
            crate::spacetimedsl::error::ReferenceIntegrityViolationError::OnDelete(#deletion_result)
        )
    }
}

/// `SpacetimeDSLError::ReferenceIntegrityViolation(ReferenceIntegrityViolationError::OnCreateOrUpdate { .. })`.
/// `action` names a variant of `error::Action`.
pub fn reference_integrity_violation_on_create_or_update(
    table_name: &impl ToTokens,
    action: &impl ToTokens,
    column_names_and_row_values: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError::ReferenceIntegrityViolation(
            crate::spacetimedsl::error::ReferenceIntegrityViolationError::OnCreateOrUpdate {
                table_name: #table_name.into(),
                create_or_update: crate::spacetimedsl::error::Action::#action,
                column_names_and_row_values: #column_names_and_row_values.into()
            }
        )
    }
}

/// `SpacetimeDSLError::AutoIncOverflow { .. }`
pub fn auto_inc_overflow(table_name: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError::AutoIncOverflow {
            table_name: #table_name.into(),
        }
    }
}

/// `delete::DeletionResultEntry`, as a type rather than a value.
pub fn deletion_result_entry_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::DeletionResultEntry
    }
}

/// `delete::OnDeleteStrategy`, as a type rather than a value.
pub fn on_delete_strategy_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::OnDeleteStrategy
    }
}

/// `delete::DeletionResult`, as a type rather than a value.
pub fn deletion_result_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::DeletionResult
    }
}

/// `delete::DeletionResult { .. }`
pub fn deletion_result(
    table_name: &impl ToTokens,
    one_or_multiple: &impl ToTokens,
    entries: &impl ToTokens,
    error_from_hook: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::DeletionResult {
            table_name: #table_name.into(),
            one_or_multiple: #one_or_multiple,
            entries: #entries,
            error_from_hook: #error_from_hook,
        }
    }
}

/// `delete::OnDeleteStrategyFailure { entries, error_from_hook }`, the `Err` payload of
/// every generated cascade function.
pub fn on_delete_strategy_failure(
    entries: &impl ToTokens,
    error_from_hook: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::OnDeleteStrategyFailure {
            entries: #entries,
            error_from_hook: #error_from_hook,
        }
    }
}

/// `delete::OnDeleteStrategyFailure<#entries_type>`, as a type rather than a value.
pub fn on_delete_strategy_failure_type(entries_type: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::OnDeleteStrategyFailure<#entries_type>
    }
}

/// `let mut error_from_hook: Option<Box<SpacetimeDSLError>> = None;`
///
/// Annotated rather than inferred: a table whose strategies never assign to it would
/// otherwise leave the type ambiguous.
pub fn error_from_hook_declaration() -> TokenStream {
    let error_type = spacetimedsl_error_type();

    quote! {
        let mut error_from_hook: Option<Box<#error_type>> = None;
    }
}

/// `delete::DeletionResultEntry { .. }`. `child_entries_field` is spliced in as the whole
/// final field, trailing comma included, because one call site emits it in shorthand form
/// (`child_entries,`) and the others give it a value (`child_entries: vec![],`).
pub fn deletion_result_entry(
    table_name: &impl ToTokens,
    column_name: &impl ToTokens,
    strategy: &impl ToTokens,
    row_value: &impl ToTokens,
    child_entries_field: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::DeletionResultEntry {
            table_name: #table_name.into(),
            column_name: #column_name.into(),
            strategy: #strategy,
            row_value: #row_value.into(),
            #child_entries_field
        }
    }
}

/// `delete::OnDeleteStrategy::#variant`
pub fn on_delete_strategy(variant: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::OnDeleteStrategy::#variant
    }
}

/// `internal::DSLInternals::#function_name(#args)`, the inherent call the referencing and
/// referenced table methods reach each other through.
pub fn dsl_internals_call(function_name: &impl ToTokens, args: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::internal::DSLInternals::#function_name(#args)
    }
}

/// `DSLMethodHooks::#function_name(#args)`, the call every emitted hook wraps.
pub fn dsl_method_hooks_call(function_name: &impl ToTokens, args: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::DSLMethodHooks::#function_name(#args)
    }
}

/// `error::OneOrMultiple::#variant`
pub fn one_or_multiple(variant: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::error::OneOrMultiple::#variant
    }
}

/// `WriteContext`, the bound on every DSL method that writes.
pub fn write_context() -> TokenStream {
    quote! {
        crate::spacetimedsl::WriteContext
    }
}

/// `ReadContext`, the bound on every DSL method that only reads.
pub fn read_context() -> TokenStream {
    quote! {
        crate::spacetimedsl::ReadContext
    }
}

/// `DSL<'_, T>`, the receiver of every public DSL method.
pub fn dsl_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::DSL<'_, T>
    }
}

/// `DSL<'_, T>`, behind a reference, as a cascade function's first argument takes it.
pub fn dsl_reference_type() -> TokenStream {
    let dsl_type = dsl_type();

    quote! {
        &#dsl_type
    }
}

/// `&DSL<'_, impl WriteContext>`, the argument of the `v4` and `v7` constructors of a
/// `Uuid` wrapper, which are not generic over a `T` of their own.
pub fn dsl_reference_type_with_any_write_context() -> TokenStream {
    let write_context = write_context();

    quote! {
        &crate::spacetimedsl::DSL<'_, impl #write_context>
    }
}

/// `NewUUID`, the context trait `v4` and `v7` generate their value through.
pub fn new_uuid_trait() -> TokenStream {
    quote! {
        crate::spacetimedsl::NewUUID
    }
}

/// `ReadOnlyDSL<'_, T>`, the second receiver a read-compatible method is emitted on.
pub fn read_only_dsl_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::ReadOnlyDSL<'_, T>
    }
}

/// `ReadOnlyDSL<#lifetime, T>`, the DSL a wrapper method reads through, for a lifetime the
/// method declares itself.
pub fn read_only_dsl_type_with_lifetime(lifetime: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::ReadOnlyDSL<#lifetime, T>
    }
}

/// `read_only_dsl(#context)`, the read-only DSL a singleton's default is asked for through.
pub fn read_only_dsl_call(context: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::read_only_dsl(#context)
    }
}

/// `DefaultSingleton`, the trait a `#[dsl(singleton(with_default))]` table's struct implements.
pub fn default_singleton_trait() -> TokenStream {
    quote! {
        crate::spacetimedsl::DefaultSingleton
    }
}

/// `internal::DSLInternals`, as a type rather than a call.
pub fn dsl_internals_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::internal::DSLInternals
    }
}

/// `DSLMethodHooks`, as a type rather than a call.
pub fn dsl_method_hooks_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::DSLMethodHooks
    }
}

/// `Wrapper<#wrapped_type, #wrapper_type>`, the trait a generated wrapper implements.
pub fn wrapper_trait(wrapped_type: &impl ToTokens, wrapper_type: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::Wrapper<#wrapped_type, #wrapper_type>
    }
}

/// `::spacetimedsl::Wrapper`, the `use` every method body opens with.
pub fn wrapper_trait_path() -> TokenStream {
    quote! {
        ::spacetimedsl::Wrapper
    }
}

/// `use ::spacetimedsl::itertools::Itertools;`, the import every body that calls
/// `at_most_one`, `collect_vec` or `into_values().collect_vec()` opens with.
pub fn itertools_import() -> TokenStream {
    quote! {
        use ::spacetimedsl::itertools::Itertools;
    }
}
