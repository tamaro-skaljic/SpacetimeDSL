use crate::{
    api::{
        dsl::{setter::Setter, wrapper::WrapperType},
        rust::{RustField, RustVisibility},
    },
    internal::wrapper_type_into_option,
};
use quote::{TokenStreamExt, quote};

use super::quote::{
    get_method_arg_column_type, get_method_arg_into_wrapper_type,
    get_method_arg_into_wrapper_type_option, get_return_column_type, get_return_wrapper_type,
    get_return_wrapper_type_option,
};

pub(in crate::internal) fn get_setter(
    rust_field: &RustField,
    is_option: bool,
    wrapper_type: &Option<WrapperType>,
) -> Option<Setter> {
    match rust_field.visibility {
        RustVisibility::Private => {
            return None;
        }
        _ => {}
    };

    let column_name = &rust_field.name;

    let method_visibility = rust_field.visibility.clone();
    let method_name = format!("set_{column_name}").into();
    let mut method_arg = quote! { #column_name: };
    let return_type;
    let method_impl;

    match wrapper_type {
        Some(wrapper_type) => {
            let wrapper_type_name_or_path = match wrapper_type {
                WrapperType::Wrap(wrap) => &wrap.wrapper_struct_name,
                WrapperType::Wrapped(wrapped) => &wrapped.wrapper_struct_name_or_path,
            };

            if is_option {
                method_arg.append_all(get_method_arg_into_wrapper_type_option(
                    wrapper_type_name_or_path,
                ));

                return_type = get_return_wrapper_type_option(wrapper_type_name_or_path);

                let into_option = wrapper_type_into_option(column_name, wrapper_type_name_or_path);
                method_impl = quote! {
                    #into_option
                    self.#column_name = #column_name;
                };
            } else {
                method_arg.append_all(get_method_arg_into_wrapper_type(wrapper_type_name_or_path));

                return_type = get_return_wrapper_type(wrapper_type_name_or_path);

                method_impl = quote! {
                    self.#column_name = #column_name.into().value();
                };
            }
        }
        None => {
            method_arg.append_all(get_method_arg_column_type(&rust_field.type_name_or_path));

            return_type = get_return_column_type(&rust_field.type_name_or_path);

            method_impl = quote! {
                self.#column_name = #column_name;
            };
        }
    };

    let method_arg = method_arg.to_string().into();
    let method_impl = method_impl.to_string().into();

    Some(Setter {
        method_visibility,
        method_name,
        method_arg,
        return_type,
        method_impl,
    })
}
