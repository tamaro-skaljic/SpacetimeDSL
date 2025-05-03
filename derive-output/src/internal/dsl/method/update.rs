use crate::api::{
    Column,
    db::{Index, SpacetimeDBTable},
    dsl::{method::SpacetimeDSLMethod, table::SpacetimeDSLTable},
    rust::{RustField, RustStruct},
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn for_single_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
    rust_field: &RustField,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = &spacetimedb_table.singular_name;
    let column_name = &rust_field.name;

    let doc_comment = format!(
        "Update a {} row inside the {} table by {}.",
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

    let modified_at = match spacetimedsl_table.has_modified_at_column {
        false => TokenStream::default(),
        true => {
            quote! {
                #table_name.modified_at = self.ctx().timestamp;
            }
        }
    };

    let method_impl = quote! {
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

pub(in crate::internal) fn for_multi_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    multi_column_index: &Index,
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &[Column],
) -> SpacetimeDSLMethod {
    todo!()
}
