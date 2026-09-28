//! The attributes `#[spacetimedsl::dsl]` reads: the attribute itself, in whichever way the
//! user spells it, and the attributes it reads on the fields.

use {
    crate::internal::{attribute::is_spelling_of, dsl},
    syn::Attribute,
};

/// Whether `attribute` is `#[spacetimedsl::dsl]`, spelled `dsl`, `spacetimedsl::dsl` or
/// `::spacetimedsl::dsl`.
///
/// Public because `spacetimedsl_derive` recognises the `#[dsl]` attributes still on a struct
/// with it: `is_last_dsl_attribute`, and the characterization tests, which expand a fixture
/// one `#[dsl]` at a time.
pub fn is_dsl_attribute(attribute: &Attribute) -> bool {
    is_spelling_of(attribute.path(), "spacetimedsl", "dsl")
}

/// Every field attribute `#[spacetimedsl::dsl]` reads, built from the symbols its parsers
/// match against.
///
/// Public because the `SpacetimeDSL` derive of `spacetimedsl_derive` declares the same names
/// as its helper attributes, so the compiler accepts them on the fields. Its test
/// `helper_attributes_match_field_attributes` keeps the two lists equal.
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
