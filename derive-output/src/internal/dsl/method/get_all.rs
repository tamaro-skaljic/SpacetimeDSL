use crate::api::Column;

pub(crate) fn build(
    spacetimedb_table: &crate::api::db::SpacetimeDBTable,
    spacetimedsl_table: &crate::api::dsl::table::SpacetimeDSLTable,
    columns: &[Column],
) -> crate::api::dsl::method::SpacetimeDSLMethod {
    todo!()
}
