//! The checks `#[disallow(...)]` adds to every DSL method which writes a row, and what the
//! documentation says about them.
//!
//! A write runs the checks after its before hook, so a hook may repair a value and cannot
//! slip a forbidden one past them, and before the framework writes the columns it owns.

use {
    super::{context::MethodGenerationContext, doc, message},
    crate::{
        api::{dsl::disallow::Disallowed, runtime},
        internal::{
            column::{ColumnTypeKind, InternalColumn},
            dsl::disallow::forbidden_zero,
            spacetimedb,
        },
    },
    proc_macro2::TokenStream,
    quote::quote,
    std::collections::BTreeSet,
};

/// Why a column with `zero` always has a value it forbids.
const ZERO_ONLY_WITH_A_FORBIDDEN_VALUE: &str =
    "`internal/dsl/disallow.rs` allows `zero` only on unsigned integer and `Uuid` columns";

/// The write a check guards, which its message names.
pub enum GuardedWrite {
    /// `create_<table>` and the insert path of `upsert_<table>`. The row has no key yet which
    /// a message could name: an `#[auto_inc]` key is still the placeholder `0`.
    Create,
    /// A write over a stored row. `row_key` renders the row's primary key as `{ id : 7 }`.
    Update { row_key: TokenStream },
}

/// Which of a column's rules a documented method checks.
#[derive(Clone, Copy)]
pub enum CheckedRules {
    /// `create_<table>`: every rule, but none on an `#[auto_inc]` column.
    OfANewRow,
    /// A write over a stored row: every rule.
    OfAWrittenRow,
}

/// `return Err(<error>);`, how a DSL method leaves on a broken rule.
pub fn return_the_error(error: &TokenStream) -> TokenStream {
    quote! {
        return Err(#error);
    }
}

/// The expression which renders the primary key of the row `row` as `{ id : 7 }`, or the
/// `{ id : 0 }` of a singleton, whose injected key has no getter.
pub fn row_key(context: &MethodGenerationContext, row: &TokenStream) -> TokenStream {
    let primary_key_column_name = &context.primary_key_column_name;

    match context.spacetimedsl_table.is_singleton() {
        true => message::singleton_primary_key(),
        false => message::single_column_and_value(
            primary_key_column_name,
            &quote! { #row.#primary_key_column_name },
        ),
    }
}

/// One `if` per rule of a column of the table which `write` checks, reading the row `row`
/// and ending in `on_violation` with the error the broken rule reports. Empty when no column
/// has such a rule.
pub fn checks(
    context: &MethodGenerationContext,
    write: &GuardedWrite,
    row: &TokenStream,
    on_violation: impl Fn(&TokenStream) -> TokenStream,
) -> TokenStream {
    let mut checks = TokenStream::new();

    for internal_column in context.internal_columns {
        for rule in &internal_column.spacetimedsl_column_disallowed {
            if !is_checked(*rule, internal_column, write) {
                continue;
            }

            let check = match rule {
                Disallowed::Zero => zero_check(internal_column, row),
            };

            let error = disallowed_value_error(context, write, internal_column, *rule, &check);
            let leave = on_violation(&error);
            let violated = check.violated;

            checks.extend(quote! {
                if #violated {
                    #leave
                }
            });
        }
    }

    checks
}

/// Whether `write` checks `rule` of `internal_column`. `create_<table>` writes the placeholder
/// `0` into an `#[auto_inc]` column, which SpacetimeDB replaces with a value of its sequence,
/// never `0`.
fn is_checked(rule: Disallowed, internal_column: &InternalColumn, write: &GuardedWrite) -> bool {
    match (rule, write) {
        (Disallowed::Zero, GuardedWrite::Create) => !internal_column.spacetimedb_column_is_auto_inc,
        (Disallowed::Zero, GuardedWrite::Update { .. }) => true,
    }
}

/// A rule's condition, and the part of its message which says what the column is or did,
/// with the values the `{}` placeholders of that part show.
struct Check {
    violated: TokenStream,
    reason: String,
    reason_arguments: Vec<TokenStream>,
}

fn zero_check(internal_column: &InternalColumn, row: &TokenStream) -> Check {
    let column_name = &internal_column.rust_field_name;
    let zero_value = match internal_column.rust_field_type_kind {
        ColumnTypeKind::UUID => spacetimedb::uuid_nil(),
        _ => quote! { 0 },
    };

    Check {
        violated: quote! { #row.#column_name == #zero_value },
        reason: format!(
            "is `{}`",
            written_zero(internal_column.rust_field_type_kind)
        ),
        reason_arguments: vec![],
    }
}

fn written_zero(column_type_kind: ColumnTypeKind) -> &'static str {
    forbidden_zero(column_type_kind).expect(ZERO_ONLY_WITH_A_FORBIDDEN_VALUE)
}

/// `SpacetimeDSLError::Error(<message>)` for `rule` of `internal_column` broken by `write`,
/// such as *Disallowed Value Error while trying to update the row `{ id : 7 }` in the
/// `player` table because `level` is `0`, which `#[disallow(zero)]` forbids!*
fn disallowed_value_error(
    context: &MethodGenerationContext,
    write: &GuardedWrite,
    internal_column: &InternalColumn,
    rule: Disallowed,
    check: &Check,
) -> TokenStream {
    let (attempted, mut arguments) = match write {
        GuardedWrite::Create => ("create a row in", vec![]),
        GuardedWrite::Update { row_key } => ("update the row `{}` in", vec![row_key.clone()]),
    };
    arguments.extend(check.reason_arguments.iter().cloned());

    let message = format!(
        "Disallowed Value Error while trying to {attempted} the `{}` table because `{}` {}, which `#[disallow({})]` forbids!",
        context.singular_table_name,
        internal_column.rust_field_name,
        check.reason,
        rule.keyword(),
    );

    let message = match arguments.is_empty() {
        true => quote! { #message.to_string() },
        false => quote! { format!(#message, #(#arguments),*) },
    };

    runtime::generic_error(&message)
}

/// The `# Disallowed values` section of a method which checks `checked`. Empty when no column
/// of the table has a rule it checks.
pub fn section(internal_columns: &[InternalColumn], checked: CheckedRules) -> String {
    let bullets: Vec<String> = internal_columns
        .iter()
        .filter_map(|internal_column| {
            let forbidden: Vec<String> = internal_column
                .spacetimedsl_column_disallowed
                .iter()
                .filter(|rule| match checked {
                    CheckedRules::OfANewRow => {
                        is_checked(**rule, internal_column, &GuardedWrite::Create)
                    }
                    CheckedRules::OfAWrittenRow => true,
                })
                .map(|rule| forbidden_value(*rule, internal_column.rust_field_type_kind))
                .collect();

            (!forbidden.is_empty()).then(|| {
                format!(
                    "- `{}`: {}",
                    internal_column.rust_field_name,
                    forbidden.join(", ")
                )
            })
        })
        .collect();

    doc::section(
        "Disallowed values",
        Some(
            "Fails with a *Disallowed Value Error* if a column holds what its `#[disallow]` forbids:",
        ),
        &bullets,
    )
}

/// How the documentation names what `rule` forbids.
fn forbidden_value(rule: Disallowed, column_type_kind: ColumnTypeKind) -> String {
    match rule {
        Disallowed::Zero => format!("`{}`", written_zero(column_type_kind)),
    }
}

/// What the setter of a column says about the column's rules: which values a write through
/// the DSL refuses. Empty without rules.
pub fn setter_doc(disallowed: &BTreeSet<Disallowed>, column_type_kind: ColumnTypeKind) -> String {
    if disallowed.is_empty() {
        return String::new();
    }

    let refused: Vec<String> = disallowed
        .iter()
        .map(|rule| match rule {
            Disallowed::Zero => format!("is `{}`", written_zero(column_type_kind)),
        })
        .collect();

    let keywords: Vec<&str> = disallowed.iter().map(|rule| rule.keyword()).collect();

    format!(
        "Writing the row through the DSL fails with a *Disallowed Value Error* if this column {} (`#[disallow({})]`).",
        refused.join(" or "),
        keywords.join(", "),
    )
}
