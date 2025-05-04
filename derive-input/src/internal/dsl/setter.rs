use super::quote::{
    get_method_arg_column_type, get_method_arg_into_wrapper_type,
    get_method_arg_into_wrapper_type_option, get_return_column_type, get_return_wrapper_type,
    get_return_wrapper_type_option,
};
use crate::{
    api::{
        dsl::{setter::Setter, wrapper::WrapperType},
        rust::{RustField, RustVisibility},
    },
    internal::utils::wrapper_type_into_option,
};
use quote::{format_ident, quote};
use syn::{Type, parse_str};

impl Setter {
    pub(in crate::internal) fn map(
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

        let column_name = format_ident!("{}", *rust_field.name);

        let method_visibility = rust_field.visibility.clone();
        let method_name = format!("set_{column_name}").into();
        let method_arg;
        let return_type;
        let return_expr;
        let method_impl;

        match wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if is_option {
                    let ma = get_method_arg_into_wrapper_type_option(wrapper_type_name_or_path);
                    method_arg = quote! { #column_name: #ma };

                    return_type = get_return_wrapper_type_option(wrapper_type_name_or_path);
                    return_expr = quote! {
                        match old_value {
                            Some(old_value) => {
                                Some(#wrapper_type_name_or_path::new(old_value))
                            }
                            None => {
                                None
                            }
                        }
                    };

                    let into_option =
                        wrapper_type_into_option(&column_name, wrapper_type_name_or_path);
                    method_impl = quote! {
                        #into_option
                        self.#column_name = #column_name;
                    };
                } else {
                    let ma = get_method_arg_into_wrapper_type(wrapper_type_name_or_path);
                    method_arg = quote! { #column_name: #ma };

                    return_type = get_return_wrapper_type(wrapper_type_name_or_path);
                    return_expr = quote! {
                        #wrapper_type_name_or_path::new(old_value)
                    };

                    method_impl = quote! {
                        self.#column_name = #column_name.into().value();
                    };
                }
            }
            None => {
                let rt: Type = parse_str(&rust_field.type_name_or_path).expect("setter");
                let ma = get_method_arg_column_type(&rt);
                method_arg = quote! { #column_name: #ma };

                return_type = get_return_column_type(&rt);
                return_expr = quote! {
                    old_value
                };

                method_impl = quote! {
                    self.#column_name = #column_name;
                };
            }
        };

        let method_arg = method_arg.to_string().into();
        let method_impl = quote! {
            let old_value = self.#column_name.clone();
            #method_impl
            #return_expr
        };
        let method_impl = method_impl.to_string().into();

        Some(Setter {
            method_visibility,
            method_name,
            method_arg,
            return_type,
            method_impl,
        })
    }
}
