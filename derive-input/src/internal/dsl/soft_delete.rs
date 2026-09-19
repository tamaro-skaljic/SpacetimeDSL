//! Which column a soft deletion writes, and the rules that column obeys.
//!
//! A column claims the marker role by its name or by carrying `#[set_on_soft_delete]`, and
//! the claim fixes its type. The role and the `method(soft_delete)` flag imply each other,
//! so this module rejects either one without the other and every generator afterwards
//! reads one `Option`.

use crate::api::dsl::{
    soft_delete::{SoftDeleteMarker, SoftDeleteMarkerKind},
    table::SingletonKind,
};
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::{sats::SatsField, table::ColumnArgs};
use syn::Ident;

const FLAG_COLUMN_NAMES: [&str; 2] = ["deleted", "removed"];
const TIMESTAMP_COLUMN_NAMES: [&str; 2] = ["deleted_at", "removed_at"];

pub(in crate::internal) fn try_parse(
    has_soft_delete_method: Option<bool>,
    singleton: Option<SingletonKind>,
    column_args: &ColumnArgs<'_>,
    original_struct_name: &Ident,
) -> syn::Result<Option<SoftDeleteMarker>> {
    let is_soft_deletable = has_soft_delete_method == Some(true);

    if is_soft_deletable && singleton.is_some() {
        return Err(syn::Error::new_spanned(
            original_struct_name,
            "`#[dsl(method(soft_delete = true))]` is not allowed on a singleton table!\nA singleton holds one row which the DSL looks up by its injected primary key, so retiring that row would leave the table with a row no method can reach.",
        ));
    }

    let mut marker: Option<SoftDeleteMarker> = None;

    for field in &column_args.fields {
        let kind = match claimed_kind(field)? {
            None => continue,
            Some(kind) => kind,
        };

        let column_name = field.name.as_ref().expect("should have a name");
        let field_identifier = field.ident.expect("a named field has an identifier");

        // Rejected here rather than after the loop, so the message can underline the
        // column. A `SoftDeleteMarker` carries a synthesized `Ident` whose span is the
        // call site, which would point at the `#[dsl(..)]` attribute instead.
        if !is_soft_deletable {
            return Err(syn::Error::new_spanned(
                field_identifier,
                "This column claims the soft-delete marker role, but the table is not soft-deletable!\nAdd `#[dsl(method(soft_delete = true))]` to the table, or rename the column and remove `#[set_on_soft_delete]` from it.",
            ));
        }

        let field_type = field.ty.to_token_stream().to_string();

        if !type_fits(kind, &field_type) {
            return Err(syn::Error::new_spanned(
                field.ty,
                format!(
                    "A column with the soft-delete marker role should have the type `{}`! Found: {field_type}",
                    match kind {
                        SoftDeleteMarkerKind::Flag => "bool",
                        SoftDeleteMarkerKind::Timestamp => "Option<spacetimedb::Timestamp>",
                    }
                ),
            ));
        }

        if marker.is_some() {
            return Err(syn::Error::new_spanned(
                field_identifier,
                "Multiple columns claim the soft-delete marker role! Only one column is allowed.",
            ));
        }

        if !matches!(field.vis, syn::Visibility::Inherited) {
            return Err(syn::Error::new_spanned(
                field.vis,
                "A column with the soft-delete marker role should have `Visibility::Inherited`!\n`soft_delete_<table>_by_<index>` is its only writer, so it has a getter but no setter.",
            ));
        }

        marker = Some(SoftDeleteMarker {
            column_name: format_ident!("{column_name}"),
            kind,
        });
    }

    if is_soft_deletable && marker.is_none() {
        return Err(syn::Error::new_spanned(
            original_struct_name,
            "`#[dsl(method(soft_delete = true))]` requires a column which the soft deletion writes!\nName a column `deleted` or `removed` and give it the type `bool`, name a column `deleted_at` or `removed_at` and give it the type `Option<spacetimedb::Timestamp>`, or put `#[set_on_soft_delete]` on a column of either type.",
        ));
    }

    Ok(marker)
}

/// Which role a column's name or attribute claims, if any.
fn claimed_kind(field: &SatsField<'_>) -> syn::Result<Option<SoftDeleteMarkerKind>> {
    let column_name = field.name.as_ref().expect("should have a name");

    let has_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("set_on_soft_delete"));

    let claims_flag = FLAG_COLUMN_NAMES.contains(&column_name.as_ref());
    let claims_timestamp = TIMESTAMP_COLUMN_NAMES.contains(&column_name.as_ref());

    if claims_flag {
        return Ok(Some(SoftDeleteMarkerKind::Flag));
    }

    if claims_timestamp {
        return Ok(Some(SoftDeleteMarkerKind::Timestamp));
    }

    if !has_attribute {
        return Ok(None);
    }

    // The attribute names no shape, so the column's type picks one.
    let field_type = field.ty.to_token_stream().to_string();

    if type_fits(SoftDeleteMarkerKind::Flag, &field_type) {
        return Ok(Some(SoftDeleteMarkerKind::Flag));
    }

    if type_fits(SoftDeleteMarkerKind::Timestamp, &field_type) {
        return Ok(Some(SoftDeleteMarkerKind::Timestamp));
    }

    Err(syn::Error::new_spanned(
        field.ty,
        format!(
            "A column with `#[set_on_soft_delete]` should have the type `bool` or `Option<spacetimedb::Timestamp>`! Found: {field_type}"
        ),
    ))
}

fn type_fits(kind: SoftDeleteMarkerKind, field_type: &str) -> bool {
    match kind {
        SoftDeleteMarkerKind::Flag => field_type.eq("bool"),
        SoftDeleteMarkerKind::Timestamp => {
            field_type.eq("Option < Timestamp >")
                || field_type.eq("Option < spacetimedb :: Timestamp >")
        }
    }
}
