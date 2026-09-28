//! The attributes `#[spacetimedsl::dsl]` reads: the attribute itself, in whichever way the
//! user spells it, and the attributes it reads on the fields.

use {
    crate::internal::{attribute::is_spelling_of, dsl},
    syn::Attribute,
};

/// Whether `attribute` is `#[spacetimedsl::dsl]`, spelled `dsl`, `spacetimedsl::dsl` or
/// `::spacetimedsl::dsl`.
pub fn is_dsl_attribute(attribute: &Attribute) -> bool {
    is_spelling_of(attribute.path(), "spacetimedsl", "dsl")
}

/// Every field attribute `#[spacetimedsl::dsl]` reads, built from the symbols its parsers
/// match against.
///
/// The `SpacetimeDSL` derive declares the same names as its helper attributes, so the
/// compiler accepts them on the fields; a test in `spacetimedsl_derive` keeps the two lists
/// equal.
pub const FIELD_ATTRIBUTE_NAMES: [&str; 8] = [
    dsl::create_wrapper.0,
    dsl::use_wrapper.0,
    dsl::foreign_key.0,
    dsl::referenced_by.0,
    dsl::set_on_create.0,
    dsl::set_on_update.0,
    dsl::set_on_soft_delete.0,
    dsl::auto_gen.0,
];
