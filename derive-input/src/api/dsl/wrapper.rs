use {
    proc_macro2::TokenStream,
    syn::{Ident, Path, Type},
};

/// The wrapper type of a column, which the DSL uses instead of the column type.
#[derive(Clone)]
pub enum WrapperType {
    /// `#[create_wrapper]`: SpacetimeDSL generates the wrapper type.
    Created(CreatedWrapper),
    /// `#[use_wrapper(...)]`: the column uses a wrapper type another column created.
    Used(UsedWrapper),
}

/// A wrapper type `#[create_wrapper]` makes SpacetimeDSL generate.
#[derive(Clone)]
pub struct CreatedWrapper {
    /// The name of the wrapper type: the one given in `#[create_wrapper(...)]`, or the
    /// struct name followed by the column name in PascalCase, such as `EntityObjId` for the
    /// column `obj_id` of `Entity`.
    pub wrapper_struct_name: Ident,
    /// The column type the wrapper type wraps.
    pub wrapped_type_name_or_path: Path,
    /// The definition of the wrapper type and its trait implementations.
    pub wrapper_impl: TokenStream,
}

/// A wrapper type `#[use_wrapper(...)]` names.
#[derive(Clone)]
pub struct UsedWrapper {
    /// The wrapper type as given in `#[use_wrapper(...)]`, such as
    /// `crate::entity::EntityId`.
    pub wrapper_struct_name_or_path: Path,
    /// The column type the wrapper type wraps.
    pub wrapped_type_name_or_path: Path,
}

/// A method a table adds to a wrapper type, which looks rows up through a foreign key: the
/// rows which reference one value of the wrapper type of a foreign key column, like
/// `entity_id.get_position(&dsl)`, or the row a foreign key column of the row with one value
/// of the primary key's wrapper type references, like `position_id.get_entity(&dsl)`.
///
/// `method_impl` reads the DSL from the argument `dsl`, which the code that renders this
/// method declares.
#[derive(Clone)]
pub struct WrapperMethod {
    /// The wrapper type the method is added to.
    pub wrapper_type: Type,
    /// The documentation of the method, without the `///`.
    pub doc_comment: String,
    /// The name of the method.
    pub method_name: Ident,
    /// What the method returns.
    pub return_type: TokenStream,
    /// The body of the method.
    pub method_impl: TokenStream,
}
