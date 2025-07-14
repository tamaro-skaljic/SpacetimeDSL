use syn::{Ident, Path};

use crate::api::rust::visibility::RustVisibility;

pub struct RustField {
    pub visibility: RustVisibility,
    pub name: Ident,
    pub type_name_or_path: Path,
}
