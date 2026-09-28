//! Recognises the attributes SpacetimeDSL reads by their path, whichever way the user spells
//! it: the bare name a `use` brings into scope, the name prefixed by its crate, or that path
//! with a leading `::`.

use syn::{Attribute, Path};

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
