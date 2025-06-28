use proc_macro2::TokenStream;
use syn::{Ident, Path};

#[derive(Clone, Debug)]
pub enum WrapperType {
    Wrap(Wrap),
    Wrapped(Wrapped),
}

#[derive(Clone, Debug)]
pub struct Wrap {
    pub wrapper_struct_name: Ident,
    pub wrapped_type_name_or_path: Path,
    pub wrapper_impl: TokenStream,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Wrapped {
    pub wrapper_struct_name_or_path: Path,
    pub wrapped_type_name_or_path: Path,
}
