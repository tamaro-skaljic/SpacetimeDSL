use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::{dsl::wrapper::WrapperMethod, runtime};

/// One `impl` block per method, because two tables can add methods to the same wrapper type
/// without knowing each other.
pub fn build(wrapper_method: &WrapperMethod) -> TokenStream {
    let WrapperMethod {
        wrapper_type,
        doc_comment,
        method_name,
        return_type,
        method_impl,
    } = wrapper_method;

    let read_context = runtime::read_context();
    let read_only_dsl_type = runtime::read_only_dsl_type_with_lifetime(&quote! { 'a });

    quote! {
        impl #wrapper_type {
            #[doc = #doc_comment]
            pub fn #method_name<'a, T: 'a + #read_context>(
                &self,
                dsl: impl Into<#read_only_dsl_type>,
            ) -> #return_type {
                #method_impl
            }
        }
    }
}
