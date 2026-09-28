use crate::api::{
    dsl::{getter::Getter, wrapper::WrapperType},
    rust::column::RustField,
};
use crate::internal::dsl::method::naming;
use quote::quote;

impl Getter {
    pub(crate) fn map(
        rust_field: &RustField,
        is_option: bool,
        wrapper_type: &Option<WrapperType>,
    ) -> Getter {
        let column_name = &rust_field.name;

        let (return_type, method_impl) = match wrapper_type {
            Some(wrapper_type) if is_option => {
                let wrapper = wrapper_type.wrapper_path();

                // A created wrapper wraps the whole `Option`, a used one only its content.
                let wrapped_value = match wrapper_type {
                    WrapperType::Created(_) => quote! { Some(value.clone()) },
                    WrapperType::Used(_) => quote! { value.clone() },
                };

                (
                    quote! { Option<#wrapper> },
                    quote! {
                        match &self.#column_name {
                            None => None,
                            Some(value) => Some(#wrapper::new(#wrapped_value)),
                        }
                    },
                )
            }
            Some(wrapper_type) => {
                let wrapper = wrapper_type.wrapper_path();

                (
                    quote! { #wrapper },
                    quote! { #wrapper::new(self.#column_name.clone()) },
                )
            }
            None => {
                let column_type = &rust_field.type_name_or_path;

                (quote! { &#column_type }, quote! { &self.#column_name })
            }
        };

        Getter {
            method_name: naming::getter_name(column_name),
            return_type,
            method_impl,
        }
    }
}
