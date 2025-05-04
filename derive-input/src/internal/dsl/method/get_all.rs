use quote::{format_ident, quote};

use crate::api::{
    db::SpacetimeDBTable,
    dsl::{method::SpacetimeDSLMethod, table::SpacetimeDSLTable},
    rust::RustStruct,
};

pub(in crate::internal) fn build(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    spacetimedsl_table: &SpacetimeDSLTable,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);

    let doc_comment = format!(
        "Get all {} rows inside the {} table.",
        struct_name, table_name
    )
    .into();

    let trait_name = format!("GetAll{}Rows", struct_name).into();

    let method_name = format!("get_all_{}", &spacetimedsl_table.plural_name).into();

    let method_args = vec![];

    let row_type = format_ident!("{struct_name}");
    let return_type = quote! {
        impl Iterator<Item = #row_type>
    }
    .to_string()
    .into();

    let method_impl = quote! {
        return self
                    .ctx()
                    .db()
                    .#table_name()
                    .iter();
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
