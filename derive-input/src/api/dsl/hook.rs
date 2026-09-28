use proc_macro2::TokenStream;
use syn::Ident;

use crate::api::dsl::method::SpacetimeDSLArg;

/// The hooks `#[dsl(hook(before(...), after(...)))]` declares. Each is `Some` when its
/// operation is listed in its timing.
#[derive(Clone)]
pub struct SpacetimeDSLMethodHooks {
    /// `hook(before(insert))`
    pub before_insert: Option<SpacetimeDSLMethodHook>,
    /// `hook(before(update))`
    pub before_update: Option<SpacetimeDSLMethodHook>,
    /// `hook(before(delete))`
    pub before_delete: Option<SpacetimeDSLMethodHook>,
    /// `hook(before(soft_delete))`
    pub before_soft_delete: Option<SpacetimeDSLMethodHook>,
    /// `hook(after(insert))`
    pub after_insert: Option<SpacetimeDSLMethodHook>,
    /// `hook(after(update))`
    pub after_update: Option<SpacetimeDSLMethodHook>,
    /// `hook(after(delete))`
    pub after_delete: Option<SpacetimeDSLMethodHook>,
    /// `hook(after(soft_delete))`
    pub after_soft_delete: Option<SpacetimeDSLMethodHook>,
}

/// One declared hook: a trait the generated code declares and calls, which the user
/// implements with a `#[spacetimedsl::hook]` function.
#[derive(Clone)]
pub struct SpacetimeDSLMethodHook {
    /// `<Timing><Table><Operation>Hook`, such as `BeforeEntityInsertHook`.
    pub trait_name: Ident,
    /// `<timing>_<table>_<operation>`, such as `before_entity_insert`: the name the
    /// `#[spacetimedsl::hook]` function has to have.
    pub function_name: Ident,
    /// The parameters of the hook function: the DSL, then the row or rows it sees.
    pub function_args: Vec<SpacetimeDSLArg>,
    /// What the hook function returns: for a before hook of an insert, update or soft
    /// deletion the value to write, otherwise `()`, in a `Result` with `SpacetimeDSLError`.
    pub return_type: TokenStream,
}
