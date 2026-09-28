use syn::{Ident, Path};

/// A table whose foreign key references the primary key of this table, from
/// `#[referenced_by(...)]` on the primary key column.
#[derive(Clone)]
pub struct ReferencingTable {
    /// The module of the referencing table, from `path = ...`.
    pub path: Path,
    /// The accessor of the referencing table, from `table = ...`.
    pub table_name: Ident,
}
