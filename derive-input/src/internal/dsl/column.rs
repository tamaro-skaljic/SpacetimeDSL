use crate::api::{
    db::column::SpacetimeDBColumn,
    dsl::{
        column::SpacetimeDSLColumn, foreign_key::ForeignKey, getter::Getter, mut_getter::MutGetter,
        setter::Setter, wrapper::WrapperType,
    },
    rust::{column::RustField, table::RustStruct},
};
use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::sats::SatsField;
use syn::Error;

impl SpacetimeDSLColumn {
    pub(in crate::internal) fn try_parse(
        has_delete_method: &bool,
        is_singleton: bool,
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

        if !is_singleton && spacetimedb_column.is_primary_key && wrapper_type.is_none() {
            return Err(Error::new(
                Span::call_site(),
                "A #[primary_key] column must have `#[create_wrapper]` or `#[use_wrapper]`!",
            ));
        }

        let foreign_key = ForeignKey::try_parse(has_delete_method, is_singleton, field)?;

        if foreign_key.is_some() {
            match &wrapper_type {
                Some(wrapper_type) => match wrapper_type {
                    WrapperType::Created(_) => {
                        return Err(Error::new(
                            Span::call_site(),
                            "A #[foreign_key] column must have `#[use_wrapper]`, not `#[create_wrapper]`!",
                        ));
                    }
                    WrapperType::Used(_) => {}
                },
                None => {
                    return Err(Error::new(
                        Span::call_site(),
                        "A #[foreign_key] column must have `#[use_wrapper]`!",
                    ));
                }
            }
        }

        // Singleton PK column (id: u8) doesn't need getter/setter/mut_getter
        let is_singleton_pk = is_singleton && spacetimedb_column.is_primary_key;

        let (getter, mut_getter, setter) = if is_singleton_pk {
            (None, None, None)
        } else {
            (
                Some(Getter::map(rust_field, is_option, &wrapper_type)),
                MutGetter::map(rust_field, &wrapper_type),
                Setter::map(rust_field, is_option, &wrapper_type),
            )
        };

        Ok(SpacetimeDSLColumn {
            is_option,
            wrapper_type,
            foreign_key,
            getter,
            mut_getter,
            setter,
        })
    }
}
