use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::hook::SpacetimeDSLMethodHook;

use crate::output::map_args;

pub fn build(hook: &Option<SpacetimeDSLMethodHook>) -> syn::Result<TokenStream> {
    if hook.is_none() {
        return Ok(TokenStream::default());
    }

    let hook = hook.as_ref().unwrap();

    let trait_name = &hook.trait_name;
    let function_name = &hook.function_name;
    let function_args = map_args(&hook.function_args);
    let return_type = &hook.return_type;

    let method = quote! {
        pub trait #trait_name<T: crate::spacetimedsl::WriteContext> {
            fn #function_name(
                #(#function_args),*
            ) -> #return_type;
        }
    };

    Ok(method)
}
