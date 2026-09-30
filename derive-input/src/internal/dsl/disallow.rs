//! `#[disallow(...)]`: what the value of a column must not be, and the columns a rule is not
//! allowed on.

use {
    super::{disallow, zero},
    crate::{
        api::{
            dsl::{
                disallow::Disallowed,
                foreign_key::{ForeignKey, OnDeleteStrategy},
            },
            rust::column::RustField,
        },
        internal::{column::ColumnTypeKind, dsl::method::doc, error},
    },
    quote::ToTokens,
    spacetime_bindings_macro_input::{match_meta, sats::SatsField},
    std::collections::BTreeSet,
    syn::{Attribute, Expr, ExprLit, Lit},
};

impl Disallowed {
    /// The word naming the rule inside `#[disallow(...)]`.
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Disallowed::Zero => zero.0,
        }
    }
}

/// How a message or the documentation writes the value `#[disallow(zero)]` forbids in a
/// column of this kind: `0`, or `Uuid::NIL`. `None` for a kind the rule is not allowed on.
pub fn forbidden_zero(column_type_kind: ColumnTypeKind) -> Option<&'static str> {
    match column_type_kind {
        ColumnTypeKind::UnsignedInteger => Some("0"),
        ColumnTypeKind::UUID => Some("Uuid::NIL"),
        _ => None,
    }
}

/// Reads `#[disallow(...)]` from a column, and rejects a rule the column cannot keep.
pub fn try_parse(
    field: &SatsField<'_>,
    rust_field: &RustField,
    foreign_key: Option<&ForeignKey>,
    creation_default: Option<&Expr>,
) -> syn::Result<BTreeSet<Disallowed>> {
    let mut disallow_attributes = field
        .original_attrs
        .iter()
        .filter(|attribute| attribute.path() == disallow);

    let Some(disallow_attribute) = disallow_attributes.next() else {
        return Ok(BTreeSet::new());
    };

    if let Some(repeated_attribute) = disallow_attributes.next() {
        return Err(error::multiple_disallow_attributes(repeated_attribute));
    }

    let disallowed = parse_rules(disallow_attribute)?;

    if disallowed.contains(&Disallowed::Zero) {
        reject_zero_the_column_cannot_keep(
            field,
            rust_field,
            disallow_attribute,
            foreign_key,
            creation_default,
        )?;
    }

    Ok(disallowed)
}

fn parse_rules(disallow_attribute: &Attribute) -> syn::Result<BTreeSet<Disallowed>> {
    if disallow_attribute.meta.require_list().is_err() {
        return Err(error::disallow_without_rules(disallow_attribute));
    }

    let mut disallowed = BTreeSet::new();

    disallow_attribute.parse_nested_meta(|meta| {
        let rule = match_meta!(match meta {
            zero => Disallowed::Zero,
        });

        if !disallowed.insert(rule) {
            return Err(error::repeated_disallow_rule(&meta.path));
        }

        Ok(())
    })?;

    if disallowed.is_empty() {
        return Err(error::disallow_without_rules(disallow_attribute));
    }

    Ok(disallowed)
}

/// `zero` needs a type with a value that references nothing, and a column nothing else
/// writes that value into.
fn reject_zero_the_column_cannot_keep(
    field: &SatsField<'_>,
    rust_field: &RustField,
    disallow_attribute: &Attribute,
    foreign_key: Option<&ForeignKey>,
    creation_default: Option<&Expr>,
) -> syn::Result<()> {
    let Some(zero_value) = forbidden_zero(ColumnTypeKind::of(&rust_field.type_name_or_path)) else {
        return Err(error::disallow_zero_on_unsupported_type(field.ty));
    };

    let is_cleared_by_set_zero = foreign_key
        .and_then(|foreign_key| foreign_key.on_delete_strategy.as_ref())
        == Some(&OnDeleteStrategy::SetZero);

    if is_cleared_by_set_zero {
        return Err(error::disallow_zero_with_set_zero_strategy(
            disallow_attribute,
        ));
    }

    if let Some(creation_default) = creation_default
        && writes_zero(creation_default)
    {
        return Err(error::disallow_zero_with_zero_creation_default(
            creation_default,
            &doc::written_tokens(creation_default.to_token_stream()),
            zero_value,
        ));
    }

    Ok(())
}

/// Whether `expression` is written as the value `#[disallow(zero)]` forbids: an integer
/// literal `0`, with or without a suffix, or a path ending in `Uuid::NIL`. Any other
/// expression is left to the check that runs when the row is written.
fn writes_zero(expression: &Expr) -> bool {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Int(integer),
            ..
        }) => integer.base10_digits() == "0",
        Expr::Path(path) => {
            let segments: Vec<String> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();

            matches!(
                segments.as_slice(),
                [.., type_name, constant] if type_name == "Uuid" && constant == "NIL"
            )
        }
        Expr::Group(group) => writes_zero(&group.expr),
        Expr::Paren(parenthesized) => writes_zero(&parenthesized.expr),
        _ => false,
    }
}
