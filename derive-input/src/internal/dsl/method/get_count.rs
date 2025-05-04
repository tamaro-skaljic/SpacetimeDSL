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
        "Get the count of all {} rows inside the {} table.",
        struct_name, table_name
    )
    .into();

    let trait_name = format!("GetCountOf{}Rows", struct_name).into();

    let method_name = format!("get_count_of_{}", &spacetimedsl_table.plural_name).into();

    let method_args = vec![];

    let return_type = "u64".into();

    let method_impl = quote! {
        return self
                    .ctx()
                    .db()
                    .#table_name()
                    .count();
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
