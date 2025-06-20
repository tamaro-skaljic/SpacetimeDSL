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
            Some(wrapper_type) => match wrapper_type {
                WrapperType::Wrap(wrap) => {
                    let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                    if rust_field.type_name_or_path.eq(&"String".into()) {
                        method_arg = quote! {
                            #column_name: &str
                        };

                        return_expr = quote! {
                            #wrapper_type_name_or_path::new(old_value)
                        };

                        method_impl = quote! {
                            self.#column_name = #column_name.to_string();
                        };
                    } else {
                        let wrapped_type_name_or_path = &WrapperType::map_to_wrapped_type(wrap);
                        method_arg = quote! {
                            #column_name: #wrapped_type_name_or_path
                        };

                        return_expr = quote! {
                            #wrapper_type_name_or_path::new(old_value)
                        };

                        method_impl = quote! {
                            self.#column_name = #column_name;
                        };
                    }

                    return_type = quote! {
                        #wrapper_type_name_or_path
                    };
                }
                WrapperType::Wrapped(_) => {
                    let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                    if is_option {
                        method_arg =
                            quote! { #column_name: impl Into<Option<#wrapper_type_name_or_path>> };

                        return_type = quote! {
                            Option<#wrapper_type_name_or_path>
                        };
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
                        method_arg = quote! { #column_name: impl Into<#wrapper_type_name_or_path> };

                        return_type = quote! {
                            #wrapper_type_name_or_path
                        };
                        return_expr = quote! {
                            #wrapper_type_name_or_path::new(old_value)
                        };

                        method_impl = quote! {
                            self.#column_name = #column_name.into().value();
                        };
                    }
                }
            },
            None => {
                if rust_field.type_name_or_path.eq(&"String".into()) {
                    method_arg = quote! {
                        #column_name: &str
                    };

                    let rt: Type = parse_str(&rust_field.type_name_or_path).expect("setter");
                    return_type = quote! {
                        #rt
                    };
                    return_expr = quote! {
                        old_value.clone()
                    };

                    method_impl = quote! {
                        self.#column_name = #column_name.to_string();
                    };
                } else {
                    let rt: Type = parse_str(&rust_field.type_name_or_path).expect("setter");
                    method_arg = quote! { #column_name: #rt };

                    return_type = quote! {
                        #rt
                    };
                    return_expr = quote! {
                        old_value
                    };

                    method_impl = quote! {
                        self.#column_name = #column_name;
                    };
                }
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
            return_type: return_type.to_string().into(),
            method_impl,
        })
    }
}
