use crate::api::{
    Column,
    db::{column::SpacetimeDBColumn, table::SpacetimeDBTable},
    dsl::{
        column::{SpacetimeDSLColumn, SpacetimeDSLColumnMethods},
        foreign_key::ForeignKey,
        table::SpacetimeDSLTable,
        wrapper::WrapperType,
    },
    rust::{column::RustField, table::RustStruct, visibility::RustVisibility},
};
use crate::internal::dsl::method::MethodGenerationContext;
use itertools::izip;
use spacetime_bindings_macro_input::table::ColumnArgs;
use syn::{Ident, Path};

#[allow(clippy::type_complexity)]
pub(in crate::internal) fn try_parse(
    column_args: &ColumnArgs,
    rust_struct: &RustStruct,
    mut spacetimedb_table: SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> syn::Result<(
    SpacetimeDBTable,
    Vec<Column>,
    Column,
    Vec<InternalColumn>,
    InternalColumn,
)> {
    let primary_key_column_name = match get_primary_key_column_name(column_args) {
        Some(pk) => pk,
        None => {
            return Err(syn::Error::new_spanned(
                &rust_struct.name,
                "Your table should have a `#[primary_key]` column!",
            ));
        }
    };

    let auto_inc_column_names = get_auto_inc_column_names(column_args);

    let mut rust_fields = vec![];
    let mut spacetimedb_columns = vec![];
    let mut spacetimedsl_columns = vec![];
    let mut columns = vec![];
    let mut internal_columns = vec![];

    for field in &column_args.fields {
        let rust_field = RustField::map(field);

        let res = SpacetimeDBColumn::map(
            &rust_field,
            spacetimedb_table,
            &auto_inc_column_names,
            &primary_key_column_name,
            spacetimedsl_table.is_singleton(),
        )?;
        spacetimedb_table = res.0;
        let spacetimedb_column = res.1;

        let spacetimedsl_column = SpacetimeDSLColumn::try_parse(
            &spacetimedsl_table.has_delete_method,
            spacetimedsl_table.is_soft_deletable(),
            spacetimedsl_table.is_singleton(),
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
        &spacetimedb_table,
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
        spacetimedb_table,
        columns,
        primary_key_column,
        internal_columns,
        internal_primary_key_column,
    ))
}

/// What the generators need to know about a column's type.
///
/// The kinds are mutually exclusive because every question the generators ask is asked of
/// the *whole* type: `Option<String>` is `Optional`, not `String`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::internal) enum ColumnTypeKind {
    String,
    UnsignedInteger,
    Optional,
    UUID,
    Other,
}

impl ColumnTypeKind {
    /// Classifies a column's type by the path's last segment, accepting it only when the
    /// path is bare or rooted in the standard library. So `String`, `std::string::String`
    /// and `alloc::string::String` all classify as `String`, and `Option<_>`,
    /// `std::option::Option<_>` and `core::option::Option<_>` all as `Optional`, while a
    /// user's own `my_crate::String` stays `Other`.
    ///
    /// Unsigned integers are matched bare only: they are primitives, so a qualified
    /// spelling would not be the same type. `Uuid` is matched bare or as `spacetimedb::Uuid`.
    pub(in crate::internal) fn of(type_name_or_path: &Path) -> ColumnTypeKind {
        let Some(last_segment) = type_name_or_path.segments.last() else {
            return ColumnTypeKind::Other;
        };

        let is_bare = type_name_or_path.segments.len() == 1;
        let root = type_name_or_path.segments[0].ident.to_string();
        let is_rooted_in_std = matches!(root.as_str(), "std" | "core" | "alloc");
        let is_spacetimedb_uuid = type_name_or_path.segments.len() == 2 && root == "spacetimedb";

        match last_segment.ident.to_string().as_str() {
            "Uuid" if is_bare || is_spacetimedb_uuid => ColumnTypeKind::UUID,
            _ if !is_bare && !is_rooted_in_std => ColumnTypeKind::Other,
            "String" => ColumnTypeKind::String,
            "Option" => ColumnTypeKind::Optional,
            "u8" | "u16" | "u32" | "u64" | "u128" if is_bare => ColumnTypeKind::UnsignedInteger,
            _ => ColumnTypeKind::Other,
        }
    }
}

#[derive(Clone)]
pub(in crate::internal) struct InternalColumn {
    pub spacetimedb_table_singular_name: Ident,
    pub rust_field_visibility: RustVisibility,
    pub rust_field_name: Ident,
    pub rust_field_type_name_or_path: Path,
    pub rust_field_type_kind: ColumnTypeKind,
    pub spacetimedb_column_is_auto_inc: bool,
    pub spacetimedsl_column_is_option: bool,
    pub spacetimedsl_column_foreign_key: Option<ForeignKey>,
    pub spacetimedsl_column_wrapper_type: Option<WrapperType>,
}

fn get_auto_inc_column_names(column_args: &ColumnArgs<'_>) -> Vec<Ident> {
    column_args
        .sequenced_columns
        .iter()
        .map(|c| c.ident.clone())
        .collect()
}

pub(in crate::internal) fn get_primary_key_column_name(
    column_args: &ColumnArgs<'_>,
) -> Option<Ident> {
    column_args
        .primary_key_column
        .as_ref()
        .map(|c| c.ident.clone())
}
