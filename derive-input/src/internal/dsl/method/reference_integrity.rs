//! The checks a generated method runs before it writes: that every foreign key still points
//! at a row that exists, and that no unique multi-column index is about to be violated.
//!
//! SpacetimeDB enforces neither. A unique multi-column index is not a SpacetimeDB feature at
//! all, so its uniqueness is checked in generated code, and referential integrity is checked
//! on create and on update because the delete side is handled by the on-delete strategies.

use {
    super::{message, naming},
    crate::{
        api::{
            db::{index::IndexType, table::SpacetimeDBTable},
            dsl::foreign_key::ForeignKey,
            runtime,
        },
        internal::{
            column::{ColumnTypeKind, InternalColumn},
            dsl::{one_or_multiple::OneOrMultiple, singleton},
            spacetimedb,
        },
    },
    itertools::Itertools,
    proc_macro2::TokenStream,
    quote::{TokenStreamExt, format_ident, quote},
    syn::Ident,
};

#[derive(PartialEq, strum::Display)]
pub enum Action {
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
            && matches!(
                column.rust_field_visibility,
                crate::api::rust::visibility::RustVisibility::Private
            )
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
            ColumnTypeKind::UUID => {
                let nil = spacetimedb::uuid_nil();

                quote! {
                    if #referencing_table_column_name.ne(&#nil) {
                        #check
                    }
                }
            }
            ColumnTypeKind::String
            | ColumnTypeKind::Bool
            | ColumnTypeKind::Timestamp
            | ColumnTypeKind::Other => quote! {
                #check
            },
        });
    }

    reference_integrity_checks
}

pub fn reference_integrity_checks_on_create(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, false, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name =
            naming::get_by_index_method_name(
                referenced_table_name,
                primary_key_column_name_of_referenced_table,
            );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_getter_name =
            naming::getter_name(referencing_table_column_name);

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Create },
                &message::single_column_and_value(
                    referencing_table_column_name,
                    &quote! { #referencing_table_name.#referencing_table_column_getter_name().value() },
                ),
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

/// Why the generated update may unwrap the stored row: the first foreign key check looks it up
/// by its primary key and returns when it finds none.
const STORED_ROW_LOOKED_UP: &str =
    "the stored row is looked up by its primary key before its foreign key columns are compared";

/// `is_singleton` decides how the check finds the row it compares against. Every other table
/// reads its primary key off the row through the key's wrapper, but a singleton's injected
/// `id: u8` has neither a getter nor a wrapper, so the check names its only legal value
/// instead.
pub fn reference_integrity_checks_on_update(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
    field_name_for_found_value: &Ident,
    primary_key_column: &InternalColumn,
    is_singleton: bool,
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, true, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name =
            naming::get_by_index_method_name(
                referenced_table_name,
                primary_key_column_name_of_referenced_table,
            );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let primary_key_column_name_of_referencing_table = &primary_key_column.rust_field_name;
        let referencing_table_column_getter_name =
            naming::getter_name(referencing_table_column_name);

        // The stored row is looked up by the primary key value of the row to write, so a
        // missing row is reported with that value.
        let (primary_key_value_of_referencing_table, missing_row) = match is_singleton {
            true => {
                let primary_key_value = singleton::primary_key_value();

                (
                    quote! { &#primary_key_value },
                    message::singleton_primary_key(),
                )
            }
            false => {
                let getter_name = naming::getter_name(primary_key_column_name_of_referencing_table);
                let primary_key_value = quote! { #referencing_table_name.#getter_name().value() };
                let missing_row = message::single_column_and_value(
                    primary_key_column_name_of_referencing_table,
                    &primary_key_value,
                );

                (primary_key_value, missing_row)
            }
        };

        let not_found_error =
            runtime::not_found_error(&referencing_table_name_as_string, &missing_row);

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Update },
                &message::single_column_and_value(
                    referencing_table_column_name,
                    referencing_table_column_name,
                ),
            );

        quote! {
            if #field_name_for_found_value.is_none() {
                #field_name_for_found_value = match self.db().#referencing_table_name().#primary_key_column_name_of_referencing_table().find(#primary_key_value_of_referencing_table) {
                    Some(#referencing_table_name) => Some(#referencing_table_name),
                    None => {
                        return Err(#not_found_error);
                    }
                };
            }
            if #field_name_for_found_value.as_ref().expect(#STORED_ROW_LOOKED_UP).#referencing_table_column_getter_name().ne(&#referencing_table_name.#referencing_table_column_getter_name()) {
                match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                    Ok(_) => {},
                    Err(_) => return Err(#reference_integrity_violation_error)
                };
            }
        }
    })
}

pub fn multi_column_index_checks(
    action: Action,
    singular_table_name: &Ident,
    field_name_for_found_value: &Ident,
    spacetimedb_table: &SpacetimeDBTable,
    internal_columns: &[InternalColumn],
    primary_key_column_name: &Ident,
) -> Vec<TokenStream> {
    let mut multi_column_index_checks = vec![];

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

        // Built from the same ordered column list as the message, so the placeholder count
        // and the getter count cannot drift apart.
        let row_value_getters = index_column_names
            .iter()
            .map(|column_name| {
                row_value_getter(internal_column_named(column_name), singular_table_name)
            })
            .collect_vec();

        let mut multi_column_index_check = unique_multi_column_index_check(
            &action,
            singular_table_name,
            field_name_for_found_value,
            index_name,
            index_column_names,
            &row_value_getters,
        );

        let unique_constraint_violation_error = unique_multi_column_index_violation(
            &action,
            singular_table_name,
            index_column_names,
            &row_value_getters,
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

fn row_value_getter(internal_column: &InternalColumn, singular_table_name: &Ident) -> TokenStream {
    let column_name = &internal_column.rust_field_name;

    if internal_column.rust_field_type_kind == ColumnTypeKind::String {
        quote! { &#singular_table_name.#column_name }
    } else {
        quote! { #singular_table_name.#column_name }
    }
}

pub fn unique_multi_column_index_check(
    action: &Action,
    singular_table_name: &Ident,
    field_name_for_found_value: &Ident,
    index_name: &Ident,
    index_column_names: &[Ident],
    row_value_getters: &[TokenStream],
) -> TokenStream {
    let unique_constraint_violation_error = unique_multi_column_index_violation(
        action,
        singular_table_name,
        index_column_names,
        row_value_getters,
    );

    quote! {
        #field_name_for_found_value = match self.db().#singular_table_name().#index_name().filter((#(#row_value_getters),*)).at_most_one() {
            Ok(#singular_table_name) => #singular_table_name,
            Err(_) => return Err(#unique_constraint_violation_error),
        };
    }
}

/// The unique-constraint violation of a unique multi-column index that already holds a row
/// with these values, which SpacetimeDSL rather than SpacetimeDB detects.
fn unique_multi_column_index_violation(
    action: &Action,
    singular_table_name: &Ident,
    index_column_names: &[Ident],
    row_value_getters: &[TokenStream],
) -> TokenStream {
    runtime::unique_constraint_violation(
        &singular_table_name.to_string(),
        &format_ident!("{action}"),
        &quote! { SpacetimeDSL },
        &OneOrMultiple::Multiple,
        &message::column_names_and_row_values(index_column_names, row_value_getters),
    )
}
