use crate::internal::dsl::method::naming;
use crate::{
    api::{
        dsl::{setter::Setter, wrapper::WrapperType},
        rust::{column::RustField, visibility::RustVisibility},
    },
    internal::dsl::wrapper::map_wrapper_type_option_to_wrapped_type_option,
};
use proc_macro2::TokenStream;
use quote::quote;

/// The parts of a setter that depend on the column's wrapper, built together per shape.
struct SetterParts {
    argument: TokenStream,
    return_type: TokenStream,
    body: TokenStream,
}

impl Setter {
    pub(crate) fn map(
        rust_field: &RustField,
        is_option: bool,
        wrapper_type: &Option<WrapperType>,
    ) -> Option<Setter> {
        if let RustVisibility::Private = rust_field.visibility {
            return None;
        };

        let column_name = &rust_field.name;

        let replace_old_value = quote! {
            let old_value = std::mem::replace(&mut self.#column_name, #column_name);
        };

        let SetterParts {
            argument,
            return_type,
            body,
        } = match wrapper_type {
            Some(wrapper_type @ WrapperType::Created(_)) => {
                let wrapper = wrapper_type.wrapper_path();
                let wrapped_type = wrapper_type.wrapped_type();

                SetterParts {
                    argument: quote! { #column_name: #wrapped_type },
                    return_type: quote! { #wrapper },
                    body: quote! {
                        #replace_old_value
                        #wrapper::new(old_value)
                    },
                }
            }
            Some(wrapper_type @ WrapperType::Used(_)) if is_option => {
                let wrapper = wrapper_type.wrapper_path();
                let into_option =
                    map_wrapper_type_option_to_wrapped_type_option(column_name, &wrapper);

                SetterParts {
                    argument: quote! { #column_name: impl Into<Option<#wrapper>> },
                    return_type: quote! { Option<#wrapper> },
                    body: quote! {
                        let #column_name = #column_name.into();
                        #into_option
                        #replace_old_value
                        old_value.map(#wrapper::new)
                    },
                }
            }
            Some(wrapper_type @ WrapperType::Used(_)) => {
                let wrapper = wrapper_type.wrapper_path();

                SetterParts {
                    argument: quote! { #column_name: impl Into<#wrapper> },
                    return_type: quote! { #wrapper },
                    body: quote! {
                        let old_value = std::mem::replace(&mut self.#column_name, #column_name.into().value());
                        #wrapper::new(old_value)
                    },
                }
            }
            None => {
                let column_type = &rust_field.type_name_or_path;

                SetterParts {
                    argument: quote! { #column_name: #column_type },
                    return_type: quote! { #column_type },
                    body: quote! {
                        #replace_old_value
                        old_value
                    },
                }
            }
        };

        Some(Setter {
            method_visibility: rust_field.visibility.clone(),
            method_name: naming::setter_name(column_name),
            method_arg: argument,
            return_type,
            method_impl: body,
        })
    }
}
