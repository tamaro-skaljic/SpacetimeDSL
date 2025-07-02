use crate::api::{
    dsl::{getter::Getter, wrapper::WrapperType},
    rust::column::RustField,
};
use quote::{format_ident, quote};
use syn::Ident;

impl Getter {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        is_option: bool,
        wrapper_type: &Option<WrapperType>,
    ) -> Getter {
        let column_name = &rust_field.name;

        let method_name = get_getter_method_name(&column_name);
        let return_type;
        let method_impl;

        match wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if is_option {
                    return_type = quote! {
                        Option<#wrapper_type_name_or_path>
                    };

                    match wrapper_type {
                        WrapperType::Created(_) => {
                            method_impl = quote! {
                                match &self.#column_name {
                                    None => None,
                                    Some(value) => Some(#wrapper_type_name_or_path::new(self.#column_name.clone())),
                                }
                            };
                        }
                        WrapperType::Used(_) => {
                            method_impl = quote! {
                                match &self.#column_name {
                                    None => None,
                                    Some(value) => Some(#wrapper_type_name_or_path::new(self.#column_name.unwrap())),
                                }
                            };
                        }
                    }
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
                let rt = &rust_field.type_name_or_path;
                return_type = quote! {
                    &#rt
                };

                method_impl = quote! {
                    &self.#column_name
                };
            }
        };

        Getter {
            method_name,
            return_type,
            method_impl,
        }
    }
}

pub(in crate::internal) fn get_getter_method_name(column_name: &Ident) -> Ident {
    format_ident!("get_{column_name}")
}
