use syn::{Ident, Path};

use crate::api::rust::visibility::RustVisibility;

/// A field of the struct carrying `#[spacetimedb::table]`, as Rust sees it.
#[derive(Clone)]
pub struct RustField {
    /// The visibility the field is declared with. A private field gets no setter and no
    /// mut getter, so its value changes only through the DSL.
    pub visibility: RustVisibility,
    /// The name of the field, which is also the name of the column.
    pub name: Ident,
    /// The type of the field as written, such as `u64`, `Option<Timestamp>` or
    /// `spacetimedb::Identity`.
    pub type_name_or_path: Path,
}
