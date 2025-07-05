use crate::{
    api::rust::{table::RustStruct, visibility::RustVisibility},
    internal::table::rm_rsharp,
};
use syn::DeriveInput;

pub(in crate::internal) fn map_struct(input: &DeriveInput) -> RustStruct {
    let visibility = RustVisibility::map(&input.vis);
    let name = rm_rsharp(input.ident.clone());

    RustStruct { visibility, name }
}
