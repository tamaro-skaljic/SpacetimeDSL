use crate::api::{
    dsl::{mut_getter::MutGetter, wrapper::WrapperType},
    rust::{column::RustField, visibility::RustVisibility},
};
use quote::{format_ident, quote};
use syn::Ident;

impl MutGetter {
    pub(in crate::internal) fn map(
        rust_field: &RustField,
        wrapper_type: &Option<WrapperType>,
    ) -> Option<MutGetter> {
        if let RustVisibility::Private = rust_field.visibility {
            return None;
        };

        let column_name = &rust_field.name;

        let method_visibility = rust_field.visibility.clone();
        let method_name = get_mut_getter_method_name(column_name);
        let return_type;
        let method_impl;

        match wrapper_type {
            Some(_) => {
                return None;
            }
            None => {
                let rt = &rust_field.type_name_or_path;
                return_type = quote! {
                    &mut #rt
                };

                method_impl = quote! {
                    &mut self.#column_name
                };
            }
        };

        Some(MutGetter {
            method_visibility,
            method_name,
            return_type,
            method_impl,
        })
    }
}

pub(in crate::internal) fn get_mut_getter_method_name(column_name: &Ident) -> Ident {
    format_ident!("get_{column_name}_mut")
}
