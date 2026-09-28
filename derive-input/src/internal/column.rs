use std::collections::BTreeMap;

use crate::api::{
    Column,
    db::{column::SpacetimeDBColumn, index::Index, table::SpacetimeDBTable},
    dsl::{
        auto_gen::UUIDVersion,
        column::{SpacetimeDSLColumn, SpacetimeDSLColumnMethods},
        foreign_key::ForeignKey,
        table::SpacetimeDSLTable,
        wrapper::WrapperType,
    },
    rust::{column::RustField, table::RustStruct, visibility::RustVisibility},
};
use crate::internal::dsl::method::MethodGenerationContext;
use crate::internal::error;
use crate::internal::rust::column::column_type_path;
use itertools::izip;
use quote::ToTokens;
use spacetime_bindings_macro_input::table::ColumnArgs;
use syn::{GenericArgument, Ident, Path, PathArguments, Type};

#[allow(clippy::type_complexity)]
pub fn try_parse(
    column_args: &ColumnArgs,
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    mut single_column_index_by_column: BTreeMap<Ident, Index>,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> syn::Result<(Vec<Column>, Column, Vec<InternalColumn>, InternalColumn)> {
    let primary_key_column_name = match get_primary_key_column_name(column_args) {
        Some(pk) => pk,
        None => {
            return Err(error::missing_primary_key(&rust_struct.name));
        }
    };

    let auto_inc_column_names = get_auto_inc_column_names(column_args);

    let mut rust_fields = vec![];
    let mut spacetimedb_columns = vec![];
    let mut spacetimedsl_columns = vec![];
    let mut columns = vec![];
    let mut internal_columns = vec![];

    for field in &column_args.fields {
        let rust_field = RustField::map(field)?;

        let spacetimedb_column = SpacetimeDBColumn::map(
            &rust_field,
            single_column_index_by_column.remove(&rust_field.name),
            &spacetimedb_table.singular_name,
            &auto_inc_column_names,
            &primary_key_column_name,
            spacetimedsl_table.is_singleton(),
        )?;

        let spacetimedsl_column = SpacetimeDSLColumn::try_parse(
            spacetimedsl_table,
            field,
            rust_struct,
            &rust_field,
            &spacetimedb_column,
        )?;

        let internal_column = InternalColumn {
            spacetimedb_table_singular_name: spacetimedb_table.singular_name.clone(),
            rust_field_visibility: rust_field.visibility.clone(),
            rust_field_name: rust_field.name.clone(),
            rust_field_type_name_or_path: rust_field.type_name_or_path.clone(),
            rust_field_type_kind: ColumnTypeKind::of(&rust_field.type_name_or_path),
            spacetimedsl_column_foreign_key: spacetimedsl_column.foreign_key.clone(),
            spacetimedb_column_is_auto_inc: spacetimedb_column.is_auto_inc,
            spacetimedsl_column_is_option: spacetimedsl_column.is_option,
            spacetimedsl_column_wrapper_type: spacetimedsl_column.wrapper_type.clone(),
            spacetimedsl_column_auto_generated_uuid_version: spacetimedsl_column
                .auto_generated_uuid_version,
        };

        rust_fields.push(rust_field);
        spacetimedb_columns.push(spacetimedb_column);
        spacetimedsl_columns.push(spacetimedsl_column);
        internal_columns.push(internal_column);
    }

    let internal_primary_key_column = internal_columns
        .iter()
        .find(|c| {
            c.rust_field_name
                .to_string()
                .eq(&primary_key_column_name.to_string())
        })
        .expect("PK column should be present")
        .clone();

    let context = MethodGenerationContext::new(
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        &internal_columns,
        &internal_primary_key_column,
    );

    for (rust_field, spacetimedb_column, spacetimedsl_column) in
        izip!(rust_fields, spacetimedb_columns, spacetimedsl_columns)
    {
        let spacetimedsl_methods = SpacetimeDSLColumnMethods::map(&context, &spacetimedb_column);

        columns.push(Column {
            rust_field,
            spacetimedb_column,
            spacetimedsl_column,
            spacetimedsl_methods,
        });
    }

    let primary_key_column = columns
        .iter()
        .find(|c| {
            c.rust_field
                .name
                .to_string()
                .eq(&primary_key_column_name.to_string())
        })
        .expect("PK column should be present")
        .clone();

    Ok((
        columns,
        primary_key_column,
        internal_columns,
        internal_primary_key_column,
    ))
}

/// What the generators and the validation need to know about a column's type.
///
/// The kinds are mutually exclusive because every question is asked of the *whole* type:
/// `Option<String>` is `Optional`, not `String`; [`ColumnTypeKind::of_option_argument`]
/// asks about the `T` in `Option<T>`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColumnTypeKind {
    String,
    UnsignedInteger,
    Bool,
    Timestamp,
    Optional,
    // The project spells the acronym `UUID`, as in `NewUUID` and `UUIDVersion`.
    #[allow(clippy::upper_case_acronyms)]
    UUID,
    Other,
}

impl ColumnTypeKind {
    /// Classifies a column's type by the path's last segment and the path in front of it:
    ///
    /// - `String` and `Option<_>` bare or rooted in the standard library (`std`, `core`,
    ///   `alloc`), such as `std::string::String` or `core::option::Option<_>`;
    /// - `u8`–`u128` and `bool` bare or as `core::primitive::X` / `std::primitive::X`, which
    ///   name the same primitive;
    /// - `Timestamp` and `Uuid` bare or as `spacetimedb::X`.
    ///
    /// Every rooted path may start with `::`. Anything else, such as a user's own
    /// `my_crate::String`, is `Other`.
    pub fn of(type_name_or_path: &Path) -> ColumnTypeKind {
        let segments: Vec<String> = type_name_or_path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();

        let Some(last_segment) = segments.last() else {
            return ColumnTypeKind::Other;
        };

        let is_bare = segments.len() == 1 && type_name_or_path.leading_colon.is_none();
        let root = segments[0].as_str();
        let is_rooted_in_std = matches!(root, "std" | "core" | "alloc");
        let is_primitive_path =
            segments.len() == 3 && matches!(root, "std" | "core") && segments[1] == "primitive";
        let is_spacetimedb_path = segments.len() == 2 && root == "spacetimedb";

        match last_segment.as_str() {
            "Uuid" if is_bare || is_spacetimedb_path => ColumnTypeKind::UUID,
            "Timestamp" if is_bare || is_spacetimedb_path => ColumnTypeKind::Timestamp,
            "u8" | "u16" | "u32" | "u64" | "u128" if is_bare || is_primitive_path => {
                ColumnTypeKind::UnsignedInteger
            }
            "bool" if is_bare || is_primitive_path => ColumnTypeKind::Bool,
            _ if !is_bare && !is_rooted_in_std => ColumnTypeKind::Other,
            "String" if !is_primitive_path => ColumnTypeKind::String,
            "Option" if !is_primitive_path => ColumnTypeKind::Optional,
            _ => ColumnTypeKind::Other,
        }
    }

    /// The kind of `T` when `type_name_or_path` is `Option<T>`, `None` for any other type.
    /// A `T` which is no path, such as a tuple, is `Other`.
    pub fn of_option_argument(type_name_or_path: &Path) -> Option<ColumnTypeKind> {
        if ColumnTypeKind::of(type_name_or_path) != ColumnTypeKind::Optional {
            return None;
        }

        let PathArguments::AngleBracketed(arguments) =
            &type_name_or_path.segments.last()?.arguments
        else {
            return Some(ColumnTypeKind::Other);
        };

        Some(match arguments.args.first() {
            Some(GenericArgument::Type(Type::Path(type_path))) if type_path.qself.is_none() => {
                ColumnTypeKind::of(&type_path.path)
            }
            _ => ColumnTypeKind::Other,
        })
    }

    /// The kind of a field's type, `Other` for a type which is no path.
    pub fn of_type(field_type: &Type) -> ColumnTypeKind {
        column_type_path(field_type).map_or(ColumnTypeKind::Other, ColumnTypeKind::of)
    }

    /// [`ColumnTypeKind::of_option_argument`] of a field's type.
    pub fn of_option_argument_of_type(field_type: &Type) -> Option<ColumnTypeKind> {
        column_type_path(field_type)
            .ok()
            .and_then(ColumnTypeKind::of_option_argument)
    }
}

/// A type as text in one spelling per type, so two spellings of the same type compare
/// equal: a type [`ColumnTypeKind::of`] knows is reduced to its bare name, generic
/// arguments are canonicalised the same way, and every other path is kept as written
/// without a leading `::`. `core::primitive::u8` is `"u8"`, and
/// `std::option::Option<::spacetimedb::Timestamp>` is `"Option<Timestamp>"`.
pub fn canonical_type(type_name_or_path: &Path) -> String {
    let segment_text = |segment: &syn::PathSegment| {
        format!(
            "{}{}",
            segment.ident,
            canonical_arguments(&segment.arguments)
        )
    };

    match (
        ColumnTypeKind::of(type_name_or_path),
        type_name_or_path.segments.last(),
    ) {
        (ColumnTypeKind::Other, _) | (_, None) => type_name_or_path
            .segments
            .iter()
            .map(segment_text)
            .collect::<Vec<_>>()
            .join("::"),
        (_, Some(last_segment)) => segment_text(last_segment),
    }
}

fn canonical_arguments(arguments: &PathArguments) -> String {
    match arguments {
        PathArguments::None => String::new(),
        PathArguments::AngleBracketed(arguments) => {
            let arguments: Vec<String> = arguments
                .args
                .iter()
                .map(|argument| match argument {
                    GenericArgument::Type(Type::Path(type_path)) if type_path.qself.is_none() => {
                        canonical_type(&type_path.path)
                    }
                    other => other.to_token_stream().to_string(),
                })
                .collect();

            format!("<{}>", arguments.join(", "))
        }
        PathArguments::Parenthesized(arguments) => arguments.to_token_stream().to_string(),
    }
}

#[derive(Clone)]
pub struct InternalColumn {
    pub spacetimedb_table_singular_name: Ident,
    pub rust_field_visibility: RustVisibility,
    pub rust_field_name: Ident,
    pub rust_field_type_name_or_path: Path,
    pub rust_field_type_kind: ColumnTypeKind,
    pub spacetimedb_column_is_auto_inc: bool,
    pub spacetimedsl_column_is_option: bool,
    pub spacetimedsl_column_foreign_key: Option<ForeignKey>,
    pub spacetimedsl_column_wrapper_type: Option<WrapperType>,
    pub spacetimedsl_column_auto_generated_uuid_version: Option<UUIDVersion>,
}

fn get_auto_inc_column_names(column_args: &ColumnArgs<'_>) -> Vec<Ident> {
    column_args
        .sequenced_columns
        .iter()
        .map(|c| c.ident.clone())
        .collect()
}

pub fn get_primary_key_column_name(column_args: &ColumnArgs<'_>) -> Option<Ident> {
    column_args
        .primary_key_column
        .as_ref()
        .map(|c| c.ident.clone())
}
