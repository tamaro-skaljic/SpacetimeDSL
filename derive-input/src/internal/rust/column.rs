use crate::api::rust::{column::RustField, visibility::RustVisibility};
use quote::ToTokens;
use spacetime_bindings_macro_input::sats::SatsField;
use syn::parse2;

impl RustField {
    pub(in crate::internal) fn map(field: &SatsField<'_>) -> RustField {
        let visibility = RustVisibility::map(field.vis);
        let name = field.ident.expect("should have a name").clone();
        let type_name_or_path =
            parse2(field.ty.to_token_stream()).expect("should be parseable as Path");

        RustField {
            visibility,
            name,
            type_name_or_path,
        }
    }
}
