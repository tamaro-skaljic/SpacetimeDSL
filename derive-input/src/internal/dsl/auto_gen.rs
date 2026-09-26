use super::auto_gen;
use crate::api::{
    dsl::{auto_gen::UUIDVersion, wrapper::WrapperType},
    rust::{column::RustField, visibility::RustVisibility},
};
use crate::internal::column::ColumnTypeKind;
use crate::internal::dsl::error;
use quote::format_ident;
use spacetime_bindings_macro_input::sats::SatsField;
use syn::{Attribute, Ident};

impl UUIDVersion {
    /// Reads `#[auto_gen(v4)]` or `#[auto_gen(v7)]` from a column, and rejects the column
    /// when the create method could not generate its value.
    pub(in crate::internal) fn try_parse(
        field: &SatsField<'_>,
        rust_field: &RustField,
        wrapper_type: &Option<WrapperType>,
        singleton_has_default: bool,
    ) -> syn::Result<Option<UUIDVersion>> {
        let mut auto_gen_attributes = field
            .original_attrs
            .iter()
            .filter(|attribute| attribute.path() == auto_gen);

        let Some(auto_gen_attribute) = auto_gen_attributes.next() else {
            return Ok(None);
        };

        if let Some(repeated_attribute) = auto_gen_attributes.next() {
            return Err(error::multiple_auto_gen_attributes(repeated_attribute));
        }

        let uuid_version = parse_uuid_version(auto_gen_attribute)?;

        if singleton_has_default {
            return Err(error::auto_gen_on_singleton_with_default(
                auto_gen_attribute,
            ));
        }

        if ColumnTypeKind::of(&rust_field.type_name_or_path) != ColumnTypeKind::UUID {
            return Err(error::auto_gen_column_type_mismatch(field.ty));
        }

        if !matches!(rust_field.visibility, RustVisibility::Private) {
            return Err(error::auto_gen_column_not_private(field.vis));
        }

        match wrapper_type {
            Some(wrapper_type) => match wrapper_type {
                WrapperType::Created(_) => {}
                _ => {
                    return Err(error::auto_gen_with_used_wrapper(auto_gen_attribute));
                }
            },
            None => {
                return Err(error::auto_gen_without_wrapper(auto_gen_attribute));
            }
        }

        Ok(Some(uuid_version))
    }

    /// `v4` or `v7`, the constructor of a `Uuid` wrapper which generates this version.
    pub(in crate::internal) fn wrapper_constructor_name(&self) -> Ident {
        match self {
            UUIDVersion::V4 => format_ident!("v4"),
            UUIDVersion::V7 => format_ident!("v7"),
        }
    }
}

fn parse_uuid_version(auto_gen_attribute: &Attribute) -> syn::Result<UUIDVersion> {
    let version: Ident = auto_gen_attribute
        .meta
        .require_list()
        .and_then(|list| list.parse_args())
        .map_err(|_| error::invalid_uuid_version(auto_gen_attribute))?;

    match version.to_string().as_str() {
        "v4" => Ok(UUIDVersion::V4),
        "v7" => Ok(UUIDVersion::V7),
        _ => Err(error::invalid_uuid_version(auto_gen_attribute)),
    }
}
