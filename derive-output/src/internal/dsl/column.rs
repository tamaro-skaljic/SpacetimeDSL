use crate::api::{
    db::DBColumn,
    dsl::{column::DSLColumn, table::DSLTable},
};

impl DSLColumn {
    pub(in crate::internal) fn try_parse(
        item: &syn::DeriveInput,
        spacetimedb_column: &DBColumn,
        spacetimedsl_table: DSLTable,
    ) -> syn::Result<(DSLTable, DSLColumn)> {
        todo!()
    }
}
