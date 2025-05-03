use crate::api::{
    dsl::{getter::Getter, wrapper::WrapperType},
    rust::RustField,
};
use quote::quote;

use super::quote::{
    get_return_column_type_reference, get_return_wrapper_type, get_return_wrapper_type_option,
};

pub(in crate::internal) fn get_getter(
    rust_field: &RustField,
    is_option: bool,
    wrapper_type: &Option<WrapperType>,
) -> Getter {
    let column_name = &rust_field.name;

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
                return_type = get_return_wrapper_type_option(wrapper_type_name_or_path);

                method_impl = quote! {
                    if self.#column_name.is_none() {
                        None
                    } else {
                        Some(#wrapper_type_name_or_path::new(self.#column_name.unwrap()))
                    }
                };
            } else {
                return_type = get_return_wrapper_type(wrapper_type_name_or_path);

                method_impl = quote! {
                    #wrapper_type_name_or_path::new(self.#column_name.clone())
                };
            }
        }
        None => {
            return_type = get_return_column_type_reference(&rust_field.type_name_or_path);

            method_impl = quote! {
                &self.#column_name
            };
        }
    };

    let method_impl = method_impl.to_string().into();

    Getter {
        method_name,
        return_type,
        method_impl,
    }
}

pub(in crate::internal) fn get_getter_method_name(column_name: &Box<str>) -> Box<str> {
    format!("get_{column_name}").into()
}
