//! Recognises the attributes SpacetimeDSL reads by their path, whichever way the user spells
//! it: the bare name a `use` brings into scope, the name prefixed by its crate, or that path
//! with a leading `::`.

use syn::{Attribute, Path};

/// Whether `attribute` is `#[spacetimedb::table]`, spelled `table`, `spacetimedb::table` or
/// `::spacetimedb::table`.
pub fn is_table_attribute(attribute: &Attribute) -> bool {
    is_spelling_of(attribute.path(), "spacetimedb", "table")
}

/// Whether `path` spells the item `name` of the crate `krate`.
///
/// `true` for exactly three paths: `name`, `krate::name` and `::krate::name`. `false` for
/// every other path, among them `::name`, a path into another crate such as `other::name`,
/// a longer path such as `krate::module::name`, and a path with generic arguments such as
/// `name::<T>`.
pub fn is_spelling_of(path: &Path, krate: &str, name: &str) -> bool {
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
