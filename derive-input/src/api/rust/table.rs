use syn::Ident;

use crate::api::rust::visibility::RustVisibility;


pub struct RustStruct {
    pub visibility: RustVisibility,
    pub name: Ident,
}