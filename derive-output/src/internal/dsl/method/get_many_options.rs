use crate::api::Column;

pub(crate) fn for_single_column_index(
    item: &syn::DeriveInput,
    field: &spacetime_bindings_macro_input::sats::SatsField<'_>,
    spacetimedb_column: &crate::api::db::SpacetimeDBColumn,
) -> crate::api::dsl::method::SpacetimeDSLMethod {
    todo!()
}

pub(crate) fn for_multi_column_index(
    spacetimedb_table: &crate::api::db::SpacetimeDBTable,
    spacetimedsl_table: &crate::api::dsl::table::SpacetimeDSLTable,
    multi_column_index: &crate::api::db::Index,
    columns: &[Column],
) -> crate::api::dsl::method::SpacetimeDSLMethod {
    todo!()
}
