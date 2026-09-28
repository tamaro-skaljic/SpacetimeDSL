use crate::api::{
    dsl::{mut_getter::MutGetter, wrapper::WrapperType},
    rust::{column::RustField, visibility::RustVisibility},
};
use crate::internal::dsl::method::naming;
use quote::quote;

impl MutGetter {
    pub(crate) fn map(
        rust_field: &RustField,
        wrapper_type: &Option<WrapperType>,
    ) -> Option<MutGetter> {
        if let RustVisibility::Private = rust_field.visibility {
            return None;
        };

        // A wrapped column is changed through its setter, which takes the wrapper.
        if wrapper_type.is_some() {
            return None;
        }

        let column_name = &rust_field.name;
        let column_type = &rust_field.type_name_or_path;

        Some(MutGetter {
            method_visibility: rust_field.visibility.clone(),
            method_name: naming::mut_getter_name(column_name),
            return_type: quote! { &mut #column_type },
            method_impl: quote! { &mut self.#column_name },
        })
    }
}
