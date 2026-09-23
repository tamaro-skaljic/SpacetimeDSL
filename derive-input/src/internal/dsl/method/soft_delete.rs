//! `soft_delete_<table>_by_<index>` and `soft_delete_<tables>_by_<index>`: retire the rows
//! an index matches by writing the table's marker column.
//!
//! The body they produce is [`super::removal`]'s, with `Removal::Soft`. Removing rows
//! instead of retiring them is [`super::delete`].

use super::{
    context::MethodGenerationContext,
    index::IndexShape,
    removal::{Removal, for_removal_many, for_removal_one},
};
use crate::api::{
    dsl::{
        method::SpacetimeDSLMethod,
        soft_delete::{SoftDeleteMarker, SoftDeleteMarkerKind},
    },
    runtime,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// `soft_delete_<tables>_by_<index>`: retire every row an index matches.
pub(in crate::internal) fn for_soft_delete_many(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_many(Removal::Soft, shape, context)
}

/// `soft_delete_<table>_by_<index>`: retire the one row a unique index finds.
pub(in crate::internal) fn for_soft_delete_one(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_one(Removal::Soft, shape, context)
}

/// The statement that retires one row.
///
/// `dsl` is what the surrounding body calls `ctx()` on for the `Timestamp` shape: `self`
/// inside a DSL method, `dsl` inside a cascade function, which is a free function taking
/// the DSL as an argument.
pub(in crate::internal) fn set_marker(
    marker: &SoftDeleteMarker,
    dsl: &TokenStream,
    row: &Ident,
) -> TokenStream {
    let column_name = &marker.column_name;

    match marker.kind {
        SoftDeleteMarkerKind::Flag => quote! {
            #row.#column_name = true;
        },
        SoftDeleteMarkerKind::Timestamp => {
            let current_timestamp = runtime::current_timestamp(dsl);

            quote! {
                #row.#column_name = Some(#current_timestamp);
            }
        }
    }
}

/// The expression that asks whether a row is already retired.
pub(in crate::internal) fn is_marked(marker: &SoftDeleteMarker, row: &TokenStream) -> TokenStream {
    let column_name = &marker.column_name;

    match marker.kind {
        SoftDeleteMarkerKind::Flag => quote! { #row.#column_name },
        SoftDeleteMarkerKind::Timestamp => quote! { #row.#column_name.is_some() },
    }
}
