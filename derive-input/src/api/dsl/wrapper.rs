use proc_macro2::TokenStream;
use syn::{Ident, Path, Type};

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

/// A method on the wrapper type of a foreign key column which looks up the rows that
/// reference one value of that wrapper, like `entity_id.get_position(&dsl)`.
///
/// `method_impl` reads the DSL from the argument `dsl`, which the code that renders this
/// method declares.
#[derive(Clone)]
pub struct WrapperMethod {
    pub wrapper_type: Type,
    pub doc_comment: String,
    pub method_name: Ident,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}
