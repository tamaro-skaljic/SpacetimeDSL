use crate::api::{
    dsl::{
        column::SpacetimeDSLColumn, foreign_key::ForeignKey, getter::Getter, setter::Setter, wrapper::WrapperType
    },
    rust::{column::RustField, table::RustStruct},
};
use quote::ToTokens;
use spacetime_bindings_macro_input::sats::SatsField;

impl SpacetimeDSLColumn {
    pub(in crate::internal) fn try_parse(
        field: &SatsField<'_>,
        rust_struct: &RustStruct,
        rust_field: &RustField,
    ) -> syn::Result<SpacetimeDSLColumn> {
        let is_option = field
            .ty
            .to_token_stream()
            .to_string()
            .starts_with("Option <");

        let wrapper_type = WrapperType::try_parse(rust_struct, rust_field, field)?;

        let foreign_key = ForeignKey::try_parse(field)?;

        let getter = Getter::map(rust_field, is_option, &wrapper_type);

        let setter = Setter::map(rust_field, is_option, &wrapper_type);

        Ok(
            SpacetimeDSLColumn {
                is_option,
                wrapper_type,
                foreign_key,
                getter,
                setter,
            },
        )
    }
}
