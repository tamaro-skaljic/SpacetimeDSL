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
    /// A write over a stored row. `row_key` renders the row's primary key as `{ id : 7 }`, and
    /// `stored_row` is the row as it is stored, which `decreasing` and `increasing` compare
    /// with.
    Update {
        row_key: TokenStream,
        stored_row: TokenStream,
    },
}

/// Which of a column's rules a documented method checks.
#[derive(Clone, Copy)]
pub enum CheckedRules {
    /// `create_<table>`, which has no stored row: `zero`, but not on an `#[auto_inc]` column.
    Insert,
    /// A write over a stored row: every rule.
    Overwrite,
    /// `upsert_<table>`: every rule, `decreasing` and `increasing` while the row exists.
    Upsert,
}

/// The change `decreasing` or `increasing` forbids.
#[derive(Clone, Copy)]
enum Change {
    Decrease,
    Increase,
}

/// The change `rule` forbids, `None` for `zero`, which reads the written value alone.
fn forbidden_change(rule: Disallowed) -> Option<Change> {
    match rule {
        Disallowed::Zero => None,
        Disallowed::Decreasing => Some(Change::Decrease),
        Disallowed::Increasing => Some(Change::Increase),
    }
}

/// Whether a write over a stored row has to look that row up for the table's rules: whether
/// a column states `decreasing` or `increasing`.
pub fn compares_with_the_stored_row(internal_columns: &[InternalColumn]) -> bool {
    internal_columns.iter().any(|internal_column| {
        internal_column
            .spacetimedsl_column_disallowed
            .iter()
            .any(|rule| forbidden_change(*rule).is_some())
    })
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

            let check = match (forbidden_change(*rule), write) {
                (None, _) => zero_check(internal_column, row),
                (Some(change), GuardedWrite::Update { stored_row, .. }) => {
                    change_check(internal_column, change, row, stored_row)
                }
                (Some(_), GuardedWrite::Create) => continue,
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
/// never `0`, and a new row has no stored one to compare with.
fn is_checked(rule: Disallowed, internal_column: &InternalColumn, write: &GuardedWrite) -> bool {
    match (rule, write) {
        (Disallowed::Zero, GuardedWrite::Create) => !internal_column.spacetimedb_column_is_auto_inc,
        (Disallowed::Decreasing | Disallowed::Increasing, GuardedWrite::Create) => false,
        (_, GuardedWrite::Update { .. }) => true,
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

/// The condition under which `row` breaks `decreasing` or `increasing` of `internal_column`
/// against `stored_row`. An integer compares with `<` or `>`. A float compares through
/// `partial_cmp`, so a change to or from NaN breaks both rules, while an unchanged value —
/// the same bits, NaN included — breaks neither.
fn change_check(
    internal_column: &InternalColumn,
    change: Change,
    row: &TokenStream,
    stored_row: &TokenStream,
) -> Check {
    let column_name = &internal_column.rust_field_name;
    let written_value = quote! { #row.#column_name };
    let stored_value = quote! { #stored_row.#column_name };

    let (integer_violation, allowed_orderings, verb) = match change {
        Change::Decrease => (
            quote! { #written_value < #stored_value },
            quote! { ::core::cmp::Ordering::Greater | ::core::cmp::Ordering::Equal },
            "decrease",
        ),
        Change::Increase => (
            quote! { #written_value > #stored_value },
            quote! { ::core::cmp::Ordering::Less | ::core::cmp::Ordering::Equal },
            "increase",
        ),
    };

    let violated = match internal_column.rust_field_type_kind {
        ColumnTypeKind::Float => quote! {
            #written_value.to_bits() != #stored_value.to_bits()
                && !matches!(#written_value.partial_cmp(&#stored_value), Some(#allowed_orderings))
        },
        _ => integer_violation,
    };

    Check {
        violated,
        reason: format!("would {verb} from `{{}}` to `{{}}`"),
        reason_arguments: vec![stored_value, written_value],
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
        GuardedWrite::Update { row_key, .. } => ("update the row `{}` in", vec![row_key.clone()]),
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
                    CheckedRules::Insert => {
                        is_checked(**rule, internal_column, &GuardedWrite::Create)
                    }
                    CheckedRules::Overwrite | CheckedRules::Upsert => true,
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

    // The insert path of an upsert has no stored row, which only the change rules need.
    let lead = match checked {
        CheckedRules::Upsert if compares_with_the_stored_row(internal_columns) => {
            "Fails with a *Disallowed Value Error* if a column holds what its `#[disallow]` forbids, a decrease or an increase only while the row exists:"
        }
        CheckedRules::Insert | CheckedRules::Overwrite | CheckedRules::Upsert => {
            "Fails with a *Disallowed Value Error* if a column holds what its `#[disallow]` forbids:"
        }
    };

    doc::section("Disallowed values", Some(lead), &bullets)
}

/// How the documentation names what `rule` forbids.
fn forbidden_value(rule: Disallowed, column_type_kind: ColumnTypeKind) -> String {
    match rule {
        Disallowed::Zero => format!("`{}`", written_zero(column_type_kind)),
        Disallowed::Decreasing => "a decrease".to_string(),
        Disallowed::Increasing => "an increase".to_string(),
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
            Disallowed::Decreasing => "decreases".to_string(),
            Disallowed::Increasing => "increases".to_string(),
        })
        .collect();

    let keywords: Vec<&str> = disallowed.iter().map(|rule| rule.keyword()).collect();

    format!(
        "Writing the row through the DSL fails with a *Disallowed Value Error* if this column {} (`#[disallow({})]`).",
        refused.join(" or "),
        keywords.join(", "),
    )
}
