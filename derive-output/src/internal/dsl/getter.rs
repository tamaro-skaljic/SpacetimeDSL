use crate::api::dsl::{getter::Getter, wrapper::WrapperType};
use quote::quote;
use spacetime_bindings_macro_input::sats::SatsField;

pub(in crate::internal) fn get_getter(
    field: &SatsField<'_>,
    is_option: bool,
    wrapper_type: &Option<WrapperType>,
) -> Getter {
    let column_name = field.name.as_ref().unwrap();

    let method_name = get_getter_method_name(column_name);
    let return_type;
    let method_impl;

    match wrapper_type {
        Some(wrapper_type) => {
            let wrapper_type_name_or_path = match wrapper_type {
                WrapperType::Wrap(wrap) => &wrap.wrapper_struct_name,
                WrapperType::Wrapped(wrapped) => &wrapped.wrapper_struct_name_or_path,
            };

            if is_option {
                return_type = quote! {
                    Option<#wrapper_type_name_or_path>
                };

                method_impl = quote! {
                    if self.#column_name.is_none() {
                        None
                    } else {
                        Some(#wrapper_type_name_or_path::new(self.#column_name.unwrap()))
                    }
                };
            } else {
                return_type = quote! {
                    #wrapper_type_name_or_path
                };

                method_impl = quote! {
                    #wrapper_type_name_or_path::new(self.#column_name.clone())
                };
            }
        }
        None => {
            let column_type = field.ty;
            return_type = quote! {
                &#column_type
            };
            method_impl = quote! {
                &self.#column_name
            };
        }
    };

    let return_type = return_type.to_string().into();
    let method_impl = method_impl.to_string().into();

    Getter {
        method_name,
        return_type,
        method_impl,
    }
}

pub(in crate::internal) fn get_getter_method_name(column_name: &String) -> Box<str> {
    format!("get_{column_name}").into()
}
