//! The identifiers that two tables in a foreign key relationship agree on by name.
//!
//! A referencing table calls a function it does not see the definition of, and imports a
//! trait the other table defines, so both sides have to build the same identifier from the
//! same two table names. Both sides build it here.
//!
//! These names are part of the generated API. Changing one breaks every module generated
//! against the previous name until it is regenerated.

use crate::internal::dsl::one_or_multiple::OneOrMultiple;
use quote::format_ident;
use syn::Ident;

pub(in crate::internal) fn referenced_table_compile_error_check(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

pub(in crate::internal) fn referencing_table_compile_error_check(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_referencing_the_{referenced_table_name}_table"
    )
}

pub(in crate::internal) fn referenced_table_function_name(
    one_or_multiple: &OneOrMultiple,
    referenced_table_name: &Ident,
) -> Ident {
    match one_or_multiple {
        OneOrMultiple::One => {
            format_ident!(
                "execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_{referenced_table_name}_table_was_deleted"
            )
        }
        OneOrMultiple::Multiple => {
            format_ident!(
                "execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_{referenced_table_name}_table_were_deleted"
            )
        }
    }
}

pub(in crate::internal) fn referencing_table_function_name(
    one_or_multiple: &OneOrMultiple,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    match one_or_multiple {
        OneOrMultiple::One => {
            format_ident!(
                "execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_one_row_of_the_{referenced_table_name}_table_was_deleted"
            )
        }
        OneOrMultiple::Multiple => {
            format_ident!(
                "execute_on_delete_strategies_of_the_{referencing_table_name}_table_after_multiple_rows_of_the_{referenced_table_name}_table_were_deleted"
            )
        }
    }
}
