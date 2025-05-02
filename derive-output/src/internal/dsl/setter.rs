use crate::{
    api::{
        dsl::column::{Setter, WrapperType},
        rust::RustVisibility,
    },
    internal::wrapper_type_into_option,
};
use quote::quote;
use spacetime_bindings_macro_input::sats::SatsField;

pub(in crate::internal) fn get_setter(
    field: &SatsField<'_>,
    is_option: bool,
    wrapper_type: &Option<WrapperType>,
) -> Option<Setter> {
    match field.vis {
        syn::Visibility::Inherited => {
            return None;
        }
        _ => {}
    };

    let column_name = field.name.as_ref().unwrap().to_string().into();

    let method_visibility = RustVisibility::map(field.vis);
    let method_name = format!("set_{column_name}").into();
    let method_arg;
    let return_type;
    let method_impl;

    match wrapper_type {
        Some(wrapper_type) => {
            let wrapper_type_name_or_path = match wrapper_type {
                WrapperType::Wrap(wrap) => &wrap.wrapper_struct_name,
                WrapperType::Wrapped(wrapped) => &wrapped.wrapper_struct_name_or_path,
            };

            if is_option {
                method_arg = quote! {
                    #column_name: impl Into<Option<#wrapper_type_name_or_path>>
                };

                return_type = quote! {
                    Option<#wrapper_type_name_or_path>
                };

                let column_option_name = format!("{column_name}_option").into();
                let into_option = wrapper_type_into_option(
                    &column_name,
                    &column_option_name,
                    wrapper_type_name_or_path,
                );
                method_impl = quote! {
                    #into_option
                    self.#column_name = #column_option_name;
                };
            } else {
                method_arg = quote! {
                    #column_name: impl Into<#wrapper_type_name_or_path>
                };

                return_type = quote! {
                    #wrapper_type_name_or_path
                };

                method_impl = quote! {
                    self.#column_name = #column_name.into().value();
                };
            }
        }
        None => {
            let column_type = field.ty;

            method_arg = quote! {
                #column_name: #column_type
            };

            return_type = quote! {
                #column_type
            };

            method_impl = quote! {
                self.#column_name = #column_name;
            };
        }
    };

    let method_arg = method_arg.to_string().into();
    let return_type = return_type.to_string().into();
    let method_impl = method_impl.to_string().into();

    Some(Setter {
        method_visibility,
        method_name,
        method_arg,
        return_type,
        method_impl,
    })
}
