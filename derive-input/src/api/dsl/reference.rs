use syn::{Ident, Path};

#[derive(Clone)]
pub struct ReferencingTable {
    pub path: Path,
    pub table_name: Ident,
}
