use syn::{Ident, Path};


pub struct ReferencingTable {
    pub path: Path,
    pub table_name: Ident,
}
