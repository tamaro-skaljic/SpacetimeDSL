use crate::api::rust::{column::RustField, visibility::RustVisibility};
use quote::ToTokens;
use spacetime_bindings_macro_input::sats::SatsField;

impl RustField {
    pub(in crate::internal) fn map(field: &SatsField<'_>) -> RustField {
        let visibility = RustVisibility::map(field.vis);
        let name = field.ident.as_ref().expect("should have a name").to_string().into();
        let type_name_or_path = field.ty.to_token_stream().to_string().into();

        RustField {
            visibility,
            name,
            type_name_or_path,
        }
    }
}
