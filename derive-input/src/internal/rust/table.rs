use crate::api::rust::{table::RustStruct, visibility::RustVisibility};
use syn::DeriveInput;

pub(in crate::internal) fn map_struct(input: &DeriveInput) -> RustStruct {
    let visibility = RustVisibility::map(&input.vis);
    let name = input.ident.clone();

    RustStruct { visibility, name }
}
