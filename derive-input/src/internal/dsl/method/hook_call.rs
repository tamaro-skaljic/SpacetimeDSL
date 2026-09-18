//! The `use self::<trait>;` and the call that every emitted hook is made of.
//!
//! [`hook_tokens`] joins them, which is what almost every site wants. [`hook_use_and_call`]
//! keeps them apart for the two sites that have to place the import themselves — before a
//! prelude that has to run first, or outside the loop the call sits in.

use crate::api::dsl::hook::SpacetimeDSLMethodHook;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// The `use self::<trait>;` import and the call `build_call` produces, kept apart so a
/// caller can place the import itself - before a prelude that has to run first, or outside
/// the loop the call sits in. Both are empty when the table declares no such hook.
pub(in crate::internal) fn hook_use_and_call(
    hook: &Option<SpacetimeDSLMethodHook>,
    build_call: impl FnOnce(&Ident) -> TokenStream,
) -> (TokenStream, TokenStream) {
    match hook {
        None => (TokenStream::default(), TokenStream::default()),
        Some(hook) => {
            let hook_trait_name = &hook.trait_name;

            (
                quote! { use self::#hook_trait_name; },
                build_call(&hook.function_name),
            )
        }
    }
}

/// `use self::<trait>;` followed by the call `build_call` produces, or nothing when the
/// table declares no such hook.
pub(in crate::internal) fn hook_tokens(
    hook: &Option<SpacetimeDSLMethodHook>,
    build_call: impl FnOnce(&Ident) -> TokenStream,
) -> TokenStream {
    let (use_hook_trait, hook_call) = hook_use_and_call(hook, build_call);

    quote! {
        #use_hook_trait
        #hook_call
    }
}
