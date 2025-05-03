use crate::api::rust::{RustStruct, RustVisibility};

pub(in crate::internal) fn map(input: &syn::DeriveInput) -> RustStruct {
    let visibility = RustVisibility::map(&input.vis);
    let name = input.ident.to_string().into();

    RustStruct { visibility, name }
}
