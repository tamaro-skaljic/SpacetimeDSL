use proc_macro2::TokenStream;
use syn::{Ident, Path};

#[derive(Debug, Clone)]
pub enum WrapperType {
    Created(CreatedWrapper),
    Used(UsedWrapper),
}

#[derive(Debug, Clone)]
pub struct CreatedWrapper {
    pub wrapper_struct_name: Ident,
    pub wrapped_type_name_or_path: Path,
    pub wrapper_impl: TokenStream,
}

#[derive(Debug, Clone)]
pub struct UsedWrapper {
    pub wrapper_struct_name_or_path: Path,
    pub wrapped_type_name_or_path: Path,
}
