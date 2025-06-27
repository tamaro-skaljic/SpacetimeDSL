use crate::api::{
    db::column::SpacetimeDBColumn,
    dsl::{
        column::SpacetimeDSLColumn, foreign_key::ForeignKey, getter::Getter, setter::Setter,
        wrapper::WrapperType,
    },
    rust::{column::RustField, table::RustStruct},
};
use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::sats::SatsField;
use syn::Error;

impl SpacetimeDSLColumn {
    pub(in crate::internal) fn try_parse(
        field: &SatsField<'_>,
        rust_struct: &RustStruct,
        rust_field: &RustField,
        spacetimedb_column: &SpacetimeDBColumn,
    ) -> syn::Result<SpacetimeDSLColumn> {
        let is_option = field
            .ty
            .to_token_stream()
            .to_string()
            .starts_with("Option <");

        let wrapper_type = WrapperType::try_parse(rust_struct, rust_field, field)?;

        if spacetimedb_column.is_primary_key && wrapper_type.is_none() {
            return Err(Error::new(
                Span::call_site(),
                "A #[primary_key] column must have `#[wrap]` or `#[wrapped]`!",
            ));
        }

        let foreign_key = ForeignKey::try_parse(field)?;

        if foreign_key.is_some() {
            if wrapper_type.is_none() {
                return Err(Error::new(
                    Span::call_site(),
                    "A #[foreign_key] column must have `#[wrapped]`!",
                ));
            } else {
                match wrapper_type.as_ref().unwrap() {
                    WrapperType::Wrap(_) => {
                        return Err(Error::new(
                            Span::call_site(),
                            "A #[foreign_key] column must have `#[wrapped]`, not `#[wrap]`!",
                        ));
                    }
                    WrapperType::Wrapped(_) => {}
                }
            }
        }

        let getter = Getter::map(rust_field, is_option, &wrapper_type);

        let setter = Setter::map(rust_field, is_option, &wrapper_type);

        Ok(SpacetimeDSLColumn {
            is_option,
            wrapper_type,
            foreign_key,
            getter,
            setter,
        })
    }
}
