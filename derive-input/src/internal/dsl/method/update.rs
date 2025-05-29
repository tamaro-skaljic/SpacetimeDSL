use crate::api::{
    db::{Index, SpacetimeDBTable},
    dsl::{method::SpacetimeDSLMethod, table::SpacetimeDSLTable},
    rust::RustStruct,
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::multi_column_index_checks;

// TODO: Use try_update instead of update
pub(in crate::internal) fn for_multi_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    multi_column_index: &Index,
    spacetimedsl_table: &SpacetimeDSLTable,
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = format_ident!("{}", *rust_struct.name);
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);
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
        &spacetimedb_table.singular_name, index_name
    )
    .into();

    let method_args = vec![quote! { mut #table_name: #struct_name }.to_string().into()];

    let try_insert_error_generic_type = format_ident!("{table_name}__TableHandle");
    let return_type = quote! {
        Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
    }
    .to_string()
    .into();

    let primary_key_column_name = format_ident!("{primary_key_column_name}");
    let multi_column_index_checks = multi_column_index_checks(
        rust_struct,
        spacetimedb_table,
        &primary_key_column_name,
        &table_name,
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
        use spacetimedsl::itertools::Itertools;

        #(#multi_column_index_checks)*

        #modified_at

        return Ok(self
            .ctx()
            .db()
            .#table_name()
            .#primary_key_column_name()
            .update(#table_name));
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
