use crate::api::{
    db::{Index, SpacetimeDBTable},
    dsl::{method::SpacetimeDSLMethod, table::SpacetimeDSLTable},
    rust::{RustField, RustStruct},
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::quote;

use super::get_unique_multi_column_index_checks;

// TODO: Use try_update instead of update and create a UniqueConstraintViolation error if a unique multi-column index is violated. (return a Result)
pub(in crate::internal) fn for_single_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    rust_field: &RustField,
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = &spacetimedb_table.singular_name;
    let column_name = &rust_field.name;

    let doc_comment = format!(
        "Update a {} row inside the {} table by the unique single-column index {}.",
        struct_name, table_name, column_name,
    )
    .into();

    let trait_name = format!(
        "Update{}RowBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(column_name)
    )
    .into();

    let method_name = format!(
        "update_{}_by_{}",
        &spacetimedb_table.singular_name, column_name
    )
    .into();

    let method_args = vec![quote! { mut #table_name: #struct_name }.to_string().into()];

    let return_type = struct_name.clone();

    let multi_column_index_checks = get_unique_multi_column_index_checks(
        rust_struct,
        spacetimedb_table,
        primary_key_column_name,
    );

    let modified_at = match spacetimedsl_table.has_modified_at_column {
        false => TokenStream::default(),
        true => {
            quote! {
                #table_name.modified_at = self.ctx().timestamp;
            }
        }
    };

    let method_impl = quote! {
        use itertools::Itertools;

        #(#multi_column_index_checks)*

        #modified_at
        return self
                .ctx()
                .db()
                .#table_name()
                .#column_name()
                .update(#table_name);
    }
    .to_string()
    .into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
}

// TODO: Use try_update instead of update and create a UniqueConstraintViolation error if a unique multi-column index is violated. (return a Result)
pub(in crate::internal) fn for_multi_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    multi_column_index: &Index,
    spacetimedsl_table: &SpacetimeDSLTable,
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = &spacetimedb_table.singular_name;
    let index_name = &multi_column_index.name;

    let doc_comment = format!(
        "Update a {} row inside the {} table by the unique multi-column index {}.\n\nPanics if it finds more than one, because then the unique constraint is violated somewhere.",
        struct_name, table_name, index_name,
    )
    .into();

    let trait_name = format!(
        "Update{}RowBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(index_name)
    )
    .into();

    let method_name = format!(
        "update_{}_by_{}",
        &spacetimedsl_table.plural_name, index_name
    )
    .into();

    let method_args = vec![quote! { mut #table_name: #struct_name }.to_string().into()];

    let return_type = struct_name.clone();

    let multi_column_index_checks = get_unique_multi_column_index_checks(
        rust_struct,
        spacetimedb_table,
        primary_key_column_name,
    );

    let modified_at = match spacetimedsl_table.has_modified_at_column {
        false => TokenStream::default(),
        true => {
            quote! {
                #table_name.modified_at = self.ctx().timestamp;
            }
        }
    };

    let method_impl = quote! {
        use itertools::Itertools;

        #(#multi_column_index_checks)*

        #modified_at

        return self
            .ctx()
            .db()
            .#table_name()
            .#primary_key_column_name()
            .update(#table_name);
    }
    .to_string()
    .into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
}
