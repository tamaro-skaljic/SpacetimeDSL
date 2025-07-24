use syn::{Ident, Path};

#[derive(Debug, Clone)]
pub struct ReferencingTable {
    pub path: Path,
    pub table_name: Ident,
}
