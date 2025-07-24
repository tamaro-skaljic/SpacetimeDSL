use proc_macro2::TokenStream;
use syn::{Ident, Path};

#[derive(Clone)]
pub enum WrapperType {
    Created(CreatedWrapper),
    Used(UsedWrapper),
}

#[derive(Clone)]
pub struct CreatedWrapper {
    pub wrapper_struct_name: Ident,
    pub wrapped_type_name_or_path: Path,
    pub wrapper_impl: TokenStream,
}

#[derive(Clone)]
pub struct UsedWrapper {
    pub wrapper_struct_name_or_path: Path,
    pub wrapped_type_name_or_path: Path,
}
