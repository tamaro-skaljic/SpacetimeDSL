use crate::{
    api::{
        dsl::{setter::Setter, wrapper::WrapperType},
        rust::{column::RustField, visibility::RustVisibility},
    },
    internal::dsl::wrapper::map_wrapper_type_option_to_wrapped_type_option,
};
use quote::{ToTokens, format_ident, quote};

impl Setter {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        is_option: bool,
        wrapper_type: &Option<WrapperType>,
    ) -> Option<Setter> {
        if let RustVisibility::Private = rust_field.visibility {
            return None;
        };

        let column_name = &rust_field.name;

        let method_visibility = rust_field.visibility.clone();
        let method_name = format_ident!("set_{column_name}");
        let method_arg;
        let return_type;
        let return_expr;
        let method_impl;

        match wrapper_type {
            Some(wrapper_type) => match wrapper_type {
                WrapperType::Created(wrap) => {
                    let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                    if rust_field
                        .type_name_or_path
                        .to_token_stream()
                        .to_string()
                        .eq(&"String")
                    {
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
                WrapperType::Used(_) => {
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

                        let into_option = map_wrapper_type_option_to_wrapped_type_option(
                            column_name,
                            wrapper_type_name_or_path,
                        );
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
                let rt = &rust_field.type_name_or_path;

                if rust_field
                    .type_name_or_path
                    .to_token_stream()
                    .to_string()
                    .eq(&"String")
                {
                    method_arg = quote! {
                        #column_name: &str
                    };

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

        let method_impl = quote! {
            let old_value = self.#column_name.clone();
            #method_impl
            #return_expr
        };
        let method_impl = method_impl;

        Some(Setter {
            method_visibility,
            method_name,
            method_arg,
            return_type,
            method_impl,
        })
    }
}
