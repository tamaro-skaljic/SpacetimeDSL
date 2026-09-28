use crate::api::rust::{table::RustStruct, visibility::RustVisibility};
use syn::{DeriveInput, ext::IdentExt};

pub fn map_struct(input: &DeriveInput) -> RustStruct {
    let visibility = RustVisibility::map(&input.vis);
    let name = input.ident.unraw();

    RustStruct { visibility, name }
}
