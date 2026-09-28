//! Recognises the attributes SpacetimeDSL reads by their path, whichever way the user spells
//! it: the bare name a `use` brings into scope, the name prefixed by its crate, or that path
//! with a leading `::`.

use syn::{Attribute, Path};

use crate::internal::dsl;

/// Whether `attribute` is `#[spacetimedsl::dsl]`, spelled `dsl`, `spacetimedsl::dsl` or
/// `::spacetimedsl::dsl`.
pub fn is_dsl_attribute(attribute: &Attribute) -> bool {
    names(attribute.path(), "spacetimedsl", "dsl")
}

/// Whether `attribute` is `#[spacetimedb::table]`, spelled `table`, `spacetimedb::table` or
/// `::spacetimedb::table`.
pub(crate) fn is_table_attribute(attribute: &Attribute) -> bool {
    names(attribute.path(), "spacetimedb", "table")
}

/// Whether `path` is `name`, `krate::name` or `::krate::name`.
fn names(path: &Path, krate: &str, name: &str) -> bool {
    if path
        .segments
        .iter()
        .any(|segment| !segment.arguments.is_none())
    {
        return false;
    }

    let segments: Vec<_> = path.segments.iter().map(|segment| &segment.ident).collect();

    match segments.as_slice() {
        [last] => path.leading_colon.is_none() && *last == name,
        [first, last] => *first == krate && *last == name,
        _ => false,
    }
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
