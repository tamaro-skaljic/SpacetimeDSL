//! The identifiers that two tables in a foreign key relationship agree on by name.
//!
//! A referencing table calls a function it does not see the definition of, and imports a
//! trait the other table defines, so both sides have to build the same identifier from the
//! same two table names. Both sides build it here.
//!
//! These names are part of the generated API. Changing one breaks every module generated
//! against the previous name until it is regenerated.
//!
//! Each direction of the paired check is split by capability, because a table may be
//! deletable, soft-deletable or both, and the two say different things about what the
//! referencing side must declare. A table emits the half it can perform and imports the
//! half it needs from the other side, so a missing `on_delete` and a missing
//! `on_soft_delete` fail as two different unresolved imports, each naming the field to
//! add.

use super::removal::Removal;
use crate::internal::dsl::one_or_multiple::OneOrMultiple;
use quote::format_ident;
use syn::Ident;

pub(in crate::internal) fn referenced_table_compile_error_check_for_deletions(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

pub(in crate::internal) fn referenced_table_compile_error_check_for_soft_deletions(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_soft_deletable_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

pub(in crate::internal) fn referencing_table_compile_error_check_for_deletions(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_with_on_delete_defined_referencing_the_{referenced_table_name}_table"
    )
}

pub(in crate::internal) fn referencing_table_compile_error_check_for_soft_deletions(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_with_on_soft_delete_defined_referencing_the_{referenced_table_name}_table"
    )
}

/// How a dispatcher name ends, which is the only part the kind of removal changes.
fn removal_suffix(removal: Removal, one_or_multiple: &OneOrMultiple) -> &'static str {
    match (removal, one_or_multiple) {
        (Removal::Hard, OneOrMultiple::One) => "was_deleted",
        (Removal::Hard, OneOrMultiple::Multiple) => "were_deleted",
        (Removal::Soft, OneOrMultiple::One) => "was_soft_deleted",
        (Removal::Soft, OneOrMultiple::Multiple) => "were_soft_deleted",
    }
}

/// How the beginning of a dispatcher name reads, which is the only part the count changes.
fn one_row_or_multiple_rows(one_or_multiple: &OneOrMultiple) -> &'static str {
    match one_or_multiple {
        OneOrMultiple::One => "one_row",
        OneOrMultiple::Multiple => "multiple_rows",
    }
}

pub(in crate::internal) fn referenced_table_function_name(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    referenced_table_name: &Ident,
) -> Ident {
    let count = one_row_or_multiple_rows(one_or_multiple);
    let suffix = removal_suffix(removal, one_or_multiple);

    format_ident!(
        "execute_on_delete_strategies_of_referencing_tables_after_{count}_of_the_{referenced_table_name}_table_{suffix}"
    )
}

pub(in crate::internal) fn referencing_table_function_name(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    let count = one_row_or_multiple_rows(one_or_multiple);
    let suffix = removal_suffix(removal, one_or_multiple);

    format_ident!(
        "execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_{count}_of_the_{referenced_table_name}_table_{suffix}"
    )
}
