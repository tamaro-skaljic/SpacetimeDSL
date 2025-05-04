use super::quote::{
    get_return_column_type_reference, get_return_wrapper_type, get_return_wrapper_type_option,
};
use crate::api::{
    dsl::{getter::Getter, wrapper::WrapperType},
    rust::RustField,
};
use quote::{format_ident, quote};
use syn::{parse_str, Ident, Type};

impl Getter {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        is_option: bool,
        wrapper_type: &Option<WrapperType>,
    ) -> Getter {
        let column_name = format_ident!("{}", *rust_field.name);

        let method_name = get_getter_method_name(&column_name);
        let return_type;
        let method_impl;

        match wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

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
                let rt: Type = parse_str(&rust_field.type_name_or_path).expect("getter");
                return_type = get_return_column_type_reference(&rt);

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
}

pub(in crate::internal) fn get_getter_method_name(column_name: &Ident) -> Box<str> {
    format!("get_{column_name}").into()
}
