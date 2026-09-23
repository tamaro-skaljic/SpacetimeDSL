//! `delete_<table>_by_<index>` and `delete_<tables>_by_<index>`: remove the rows an index
//! matches.
//!
//! The body they produce is [`super::removal`]'s, with `Removal::Hard`. Retiring rows
//! instead of removing them is [`super::soft_delete`].

use super::{
    context::MethodGenerationContext,
    index::IndexShape,
    removal::{Removal, for_removal_many, for_removal_one},
};
use crate::api::dsl::method::SpacetimeDSLMethod;

/// `delete_<tables>_by_<index>`: delete every row an index matches.
pub(in crate::internal) fn for_delete_many(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_many(Removal::Hard, shape, context)
}

/// `delete_<table>_by_<index>`: delete the one row a unique index finds.
pub(in crate::internal) fn for_delete_one(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_one(Removal::Hard, shape, context)
}
