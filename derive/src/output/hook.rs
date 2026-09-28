use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::{dsl::hook::SpacetimeDSLMethodHook, runtime};

use crate::output::map_args;

pub fn build(hook: Option<&SpacetimeDSLMethodHook>) -> TokenStream {
    let Some(hook) = hook else {
        return TokenStream::default();
    };

    let trait_name = &hook.trait_name;
    let function_name = &hook.function_name;
    let function_args = map_args(&hook.function_args);
    let return_type = &hook.return_type;

    let write_context = runtime::write_context();

    quote! {
        pub trait #trait_name<T: #write_context> {
            fn #function_name(
                #(#function_args),*
            ) -> #return_type;
        }
    }
}
