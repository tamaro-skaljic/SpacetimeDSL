use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::{getter::Getter, setter::Setter};

pub fn getter(getter: &Getter) -> syn::Result<TokenStream> {
    let method_name = &getter.method_name;
    let return_type = &getter.return_type;
    let method_impl = &getter.method_impl;

    Ok(quote! {
        pub fn #method_name(&self) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}

pub fn setter(setter: &Setter) -> syn::Result<TokenStream> {
    let method_visibility: Visibility = parse_str(&setter.method_visibility.to_string())?;
    let method_name = &setter.method_name;
    let method_arg = &setter.method_arg;
    let return_type = &setter.return_type;
    let method_impl = &setter.method_impl;

    Ok(quote! {
        #method_visibility fn #method_name(&mut self, #method_arg) -> #return_type {
            use spacetimedsl::Wrapper;
            #method_impl
        }
    })
}
