use {proc_macro2::TokenStream, syn::Ident};

/// A generated DSL method, such as `create_entity` or `get_entity_by_obj_id`.
#[derive(Clone)]
pub struct SpacetimeDSLMethod {
    /// The documentation of the method, without the `///`.
    pub doc_comment: String,
    /// The name of the method.
    pub method_name: Ident,
    /// The parameters of the method after `&self`, in order.
    pub method_args: Vec<SpacetimeDSLArg>,
    /// What the method returns.
    pub return_type: TokenStream,
    /// The body of the method.
    pub method_impl: TokenStream,
    /// Whether the method is also generated for the read-only DSL, which views use.
    ///
    /// A read-only table handle (a view's, or `LocalReadOnly`) offers `count()` and index
    /// lookups but no full-table `iter()`. So `get_all_<plural_name>` is `false`, while
    /// `count_of_all_<plural_name>` and the getters of an index are `true`. Every method which
    /// writes is `false`.
    pub read_context_compatible: bool,
}

/// A parameter of a generated method or of a hook function, or a field of a generated
/// `Create<Table>` struct.
#[derive(Clone)]
pub struct SpacetimeDSLArg {
    /// Whether the column behind the parameter is an `Option`.
    pub is_option: bool,
    /// The name of the parameter.
    pub arg_name: Ident,
    /// The type of the parameter.
    pub arg_type: SpacetimeDSLArgType,
}

/// The type of a parameter.
#[derive(Clone)]
pub enum SpacetimeDSLArgType {
    /// A parameter of a column without a wrapper type, or of anything that is not a column,
    /// holding the type as written.
    Normal(TokenStream),
    /// A parameter of a column with a wrapper type.
    Wrapped {
        /// The type the wrapper type wraps, such as `u64`.
        wrapped_type: TokenStream,
        /// The type of the parameter as written, such as `impl Into<EntityId>`.
        actual_type: TokenStream,
    },
}

impl SpacetimeDSLArgType {
    /// The type of the parameter as written, whether or not its column has a wrapper type.
    ///
    /// Public because `spacetimedsl_derive` writes every parameter of a generated method and
    /// of a hook function with it, in `map_args`.
    pub fn actual_type(&self) -> &TokenStream {
        match self {
            Self::Normal(actual_type) | Self::Wrapped { actual_type, .. } => actual_type,
        }
    }
}
