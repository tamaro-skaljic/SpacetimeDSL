use {
    crate::{
        api::rust::{column::RustField, visibility::RustVisibility},
        internal::error,
    },
    spacetime_bindings_macro_input::sats::SatsField,
    syn::{Path, Type},
};

impl RustField {
    pub(crate) fn map(field: &SatsField<'_>) -> syn::Result<RustField> {
        let visibility = RustVisibility::map(field.vis);
        let name = field.ident.expect("should have a name").clone();
        let type_name_or_path = column_type_path(field.ty)?.clone();

        Ok(RustField {
            visibility,
            name,
            type_name_or_path,
        })
    }
}

/// The path a column's type is written as, looking through invisible groups and
/// parentheses. SpacetimeDSL supports only path types as column types, so every other type
/// is rejected with a diagnostic that names its kind.
pub fn column_type_path(column_type: &Type) -> syn::Result<&Path> {
    match column_type {
        Type::Group(group) => column_type_path(&group.elem),
        Type::Paren(paren) => column_type_path(&paren.elem),
        Type::Path(type_path) if type_path.qself.is_none() => Ok(&type_path.path),
        _ => Err(error::unsupported_column_type(column_type)),
    }
}
