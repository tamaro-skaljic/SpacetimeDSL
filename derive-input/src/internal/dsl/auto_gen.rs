use super::auto_gen;
use crate::api::{
    dsl::{auto_gen::UUIDVersion, wrapper::WrapperType},
    rust::{column::RustField, visibility::RustVisibility},
};
use crate::internal::column::ColumnTypeKind;
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::sats::SatsField;
use syn::{Attribute, Error, Ident};

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
            return Err(Error::new_spanned(
                repeated_attribute,
                "Only one `#[auto_gen]` is allowed per column!",
            ));
        }

        let uuid_version = parse_uuid_version(auto_gen_attribute)?;

        if singleton_has_default {
            return Err(Error::new_spanned(
                auto_gen_attribute,
                "`#[auto_gen]` is not allowed on a `singleton(with_default)` table, because it has no create method!",
            ));
        }

        if ColumnTypeKind::of(&rust_field.type_name_or_path) != ColumnTypeKind::UUID {
            return Err(Error::new_spanned(
                field.ty,
                format!(
                    "A column with `#[auto_gen]` should have the type `spacetimedb::Uuid`! Found: {}",
                    field.ty.to_token_stream()
                ),
            ));
        }

        if !matches!(rust_field.visibility, RustVisibility::Private) {
            return Err(Error::new_spanned(
                field.vis,
                format!(
                    "A column with `#[auto_gen]` should be private, because its value is generated! Found: `{}`",
                    field.vis.to_token_stream()
                ),
            ));
        }

        if !matches!(wrapper_type, Some(WrapperType::Created(_))) {
            return Err(Error::new_spanned(
                auto_gen_attribute,
                "A column with `#[auto_gen]` must be accompanied by `#[create_wrapper]`!",
            ));
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
    let invalid_version = || {
        Error::new_spanned(
            auto_gen_attribute,
            "Expected `#[auto_gen(v4)]` or `#[auto_gen(v7)]`!",
        )
    };

    let version: Ident = auto_gen_attribute
        .meta
        .require_list()
        .and_then(|list| list.parse_args())
        .map_err(|_| invalid_version())?;

    match version.to_string().as_str() {
        "v4" => Ok(UUIDVersion::V4),
        "v7" => Ok(UUIDVersion::V7),
        _ => Err(invalid_version()),
    }
}
