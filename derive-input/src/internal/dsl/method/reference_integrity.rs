//! The checks a generated method runs before it writes: that every foreign key still points
//! at a row that exists, and that no unique multi-column index is about to be violated.
//!
//! SpacetimeDB enforces neither. A unique multi-column index is not a SpacetimeDB feature at
//! all, so its uniqueness is checked in generated code, and referential integrity is checked
//! on create and on update because the delete side is handled by the on-delete strategies.

use super::index::column_names_and_row_values;
use crate::{
    api::{
        db::{index::IndexType, table::SpacetimeDBTable},
        dsl::foreign_key::ForeignKey,
        runtime,
    },
    internal::{
        column::{ColumnTypeKind, InternalColumn},
        dsl::one_or_multiple::OneOrMultiple,
    },
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, format_ident, quote};
use syn::Ident;

#[derive(PartialEq, strum::Display)]
pub(in crate::internal) enum Action {
    Create,
    Get,
    Update,
    Delete,
}

/// The scaffolding both reference-integrity builders share: skip the columns the mode does
/// not check, skip the columns without a foreign key, and wrap the mode's own check in the
/// guard that keeps it from running on a column that holds no reference yet.
fn reference_integrity_checks(
    columns: &[InternalColumn],
    skip_private_columns: bool,
    build_check: impl Fn(&InternalColumn, &ForeignKey) -> TokenStream,
) -> Vec<TokenStream> {
    let mut reference_integrity_checks = vec![];

    for column in columns {
        if skip_private_columns
            && column
                .rust_field_visibility
                .to_string()
                .eq(&crate::api::rust::visibility::RustVisibility::Private.to_string())
        {
            continue;
        }

        let foreign_key = match &column.spacetimedsl_column_foreign_key {
            Some(foreign_key) => foreign_key,
            None => continue,
        };

        let check = build_check(column, foreign_key);

        let referencing_table_column_name = &column.rust_field_name;

        reference_integrity_checks.push(match column.rust_field_type_kind {
            ColumnTypeKind::UnsignedInteger => quote! {
                if #referencing_table_column_name.ne(&0) {
                    #check
                }
            },
            ColumnTypeKind::Optional => quote! {
                if #referencing_table_column_name.is_some() {
                    #check
                }
            },
            ColumnTypeKind::String | ColumnTypeKind::Other => quote! {
                #check
            },
        });
    }

    reference_integrity_checks
}

pub(in crate::internal) fn reference_integrity_checks_on_create(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, false, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name = format_ident!(
            "get_{referenced_table_name}_by_{primary_key_column_name_of_referenced_table}"
        );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_getter_name =
            format_ident!("get_{referencing_table_column_name}");

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Create },
                &quote! {
                    format!("{{ {} : {} }}", #referencing_table_column_name, #referencing_table_name.#referencing_table_column_getter_name())
                },
            );

        quote! {
            match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                Ok(_) => {},
                Err(_) => {
                    return Err(#reference_integrity_violation_error);
                }
            };
        }
    })
}

pub(in crate::internal) fn reference_integrity_checks_on_update(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
    column_names_and_row_values: &str,
    index_columns: &[Ident],
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, true, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name = format_ident!(
            "get_{referenced_table_name}_by_{primary_key_column_name_of_referenced_table}"
        );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_name_as_string = referencing_table_column_name.to_string();
        let primary_key_column_name_of_referencing_table = &primary_key_column.rust_field_name;
        let referencing_table_column_getter_name =
            format_ident!("get_{referencing_table_column_name}");

        let field_name_for_found_value =
            format_ident!("the_same_or_another_{referencing_table_name}");

        let row_value_getters = index_columns
            .iter()
            .map(|cn| {
                quote! {
                    #referencing_table_name.#cn
                }
            })
            .collect_vec();

        let format_for_not_found_error = match one_or_multiple {
            OneOrMultiple::One => quote! {
                format!(#column_names_and_row_values, #referencing_table_column_name)
            },
            OneOrMultiple::Multiple => quote! {
                format!(#column_names_and_row_values, #(#row_value_getters),*)
            },
        };

        let getter_name = format_ident!("get_{primary_key_column_name_of_referencing_table}");

        let not_found_error = runtime::not_found_error(
            &referencing_table_name_as_string,
            &format_for_not_found_error,
        );

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Update },
                &quote! {
                    format!("{{ {} : {} }}", #referencing_table_column_name_as_string, #referencing_table_column_name)
                },
            );

        quote! {
            if #field_name_for_found_value.is_none() {
                #field_name_for_found_value = match self.db().#referencing_table_name().#primary_key_column_name_of_referencing_table().find(#referencing_table_name.#getter_name().value()) {
                    Some(#referencing_table_name) => Some(#referencing_table_name),
                    None => {
                        return Err(#not_found_error);
                    }
                };
            }
            if #field_name_for_found_value.as_ref().expect("field_name_for_found_value should be Some(_)").#referencing_table_column_getter_name().ne(&#referencing_table_name.#referencing_table_column_getter_name()) {
                match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                    Ok(_) => {},
                    Err(_) => return Err(#reference_integrity_violation_error)
                };
            }
        }
    })
}

pub(in crate::internal) fn multi_column_index_checks(
    action: Action,
    singular_table_name: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
    internal_columns: &[InternalColumn],
    primary_key_column_name: &Ident,
) -> Vec<TokenStream> {
    let mut multi_column_index_checks = vec![];
    let singular_table_name_as_string = singular_table_name.to_string();

    for multi_column_index in &spacetimedb_table.multi_column_indices {
        let index_column_names: &[Ident] = match &multi_column_index.index_type {
            IndexType::BTreeMultiColumn { columns } => columns,
            _ => {
                continue;
            }
        };

        if !multi_column_index.is_unique {
            continue;
        }

        let internal_column_named = |column_name: &Ident| {
            internal_columns
                .iter()
                .find(|c| c.rust_field_name.eq(column_name))
                .expect("A multi-column index column should exist in the internal columns")
        };

        let index_name = &multi_column_index.name;

        // Built from the same ordered column list, so the placeholder count and the
        // getter count cannot drift apart.
        let column_names_and_row_values = column_names_and_row_values(index_column_names);
        let row_value_getters = index_column_names
            .iter()
            .map(|column_name| {
                get_row_value_getter(internal_column_named(column_name), singular_table_name)
            })
            .collect_vec();

        let mut multi_column_index_check = get_unique_multi_column_index_check(
            &action,
            singular_table_name,
            index_name,
            &column_names_and_row_values,
            &row_value_getters,
        );

        let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

        let action_as_ident = format_ident!("{action}");

        let multiple = OneOrMultiple::Multiple;

        let unique_constraint_violation_error = runtime::unique_constraint_violation(
            &singular_table_name_as_string,
            &action_as_ident,
            &quote! { SpacetimeDSL },
            &multiple,
            &quote! { format!(#column_names_and_row_values, #(#row_value_getters),*) },
        );

        let return_unique_constraint_violation_error = quote! {
            return Err(#unique_constraint_violation_error);
        };

        let on_some = match action {
            Action::Create | Action::Get | Action::Delete => {
                return_unique_constraint_violation_error
            }
            Action::Update => {
                quote! {
                    if #field_name_for_found_value.#primary_key_column_name.ne(&#singular_table_name.#primary_key_column_name) {
                        #return_unique_constraint_violation_error
                    }
                }
            }
        };

        multi_column_index_check.append_all(quote! {
            match &#field_name_for_found_value {
                Some(#field_name_for_found_value) => {
                    #on_some
                },
                _ => {},
            };
        });

        multi_column_index_checks.push(multi_column_index_check);
    }

    multi_column_index_checks
}

fn get_row_value_getter(
    internal_column: &InternalColumn,
    singular_table_name: &Ident,
) -> TokenStream {
    let column_name = &internal_column.rust_field_name;

    if internal_column.rust_field_type_kind == ColumnTypeKind::String {
        quote! { &#singular_table_name.#column_name }
    } else {
        quote! { #singular_table_name.#column_name }
    }
}

pub(in crate::internal) fn get_unique_multi_column_index_check(
    action: &Action,
    singular_table_name: &Ident,
    index_name: &Ident,
    column_names_and_row_values: &str,
    row_value_getters: &[TokenStream],
) -> TokenStream {
    let field_name_for_found_value = format_ident!("the_same_or_another_{singular_table_name}");

    let singular_table_name_as_string = singular_table_name.to_string();

    let action = format_ident!("{action}");

    let multiple = OneOrMultiple::Multiple;

    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        &singular_table_name_as_string,
        &action,
        &quote! { SpacetimeDSL },
        &multiple,
        &quote! { format!(#column_names_and_row_values, #(#row_value_getters),*) },
    );

    quote! {
        #field_name_for_found_value = match self.db().#singular_table_name().#index_name().filter((#(#row_value_getters),*)).at_most_one() {
            Ok(#singular_table_name) => #singular_table_name,
            Err(_) => return Err(#unique_constraint_violation_error),
        };
    }
}
