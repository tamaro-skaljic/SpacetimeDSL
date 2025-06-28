use syn::{Ident, Path};

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct ReferencingTable {
    pub path: Path,
    pub table_name: Ident,
}
