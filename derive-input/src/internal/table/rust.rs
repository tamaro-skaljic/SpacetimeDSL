use crate::api::rust::{RustStruct, RustVisibility};
use syn::DeriveInput;

pub(in crate::internal) fn map_struct(input: &DeriveInput) -> RustStruct {
    let visibility = RustVisibility::map(&input.vis);
    let name = input.ident.to_string().into();

    RustStruct { visibility, name }
}
