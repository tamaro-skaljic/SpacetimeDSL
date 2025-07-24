use syn::Ident;

use crate::api::rust::visibility::RustVisibility;

#[derive(Clone)]
pub struct RustStruct {
    pub visibility: RustVisibility,
    pub name: Ident,
}
