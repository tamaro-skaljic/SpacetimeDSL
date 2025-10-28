use proc_macro2::TokenStream;
use syn::Ident;

use crate::api::dsl::method::SpacetimeDSLArg;

#[derive(Clone)]
pub struct SpacetimeDSLMethodHooks {
    pub before_insert: Option<SpacetimeDSLMethodHook>,
    pub before_update: Option<SpacetimeDSLMethodHook>,
    pub before_delete: Option<SpacetimeDSLMethodHook>,
    pub after_insert: Option<SpacetimeDSLMethodHook>,
    pub after_update: Option<SpacetimeDSLMethodHook>,
    pub after_delete: Option<SpacetimeDSLMethodHook>,
}

#[derive(Clone)]
pub struct SpacetimeDSLMethodHook {
    pub trait_name: Ident,
    pub function_name: Ident,
    pub function_args: Vec<SpacetimeDSLArg>,
    pub return_type: TokenStream,
}
