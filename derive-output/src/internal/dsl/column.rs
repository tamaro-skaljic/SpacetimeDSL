use crate::api::{
    db::DBColumn,
    dsl::{column::DSLColumn, table::DSLTable},
};

impl DSLColumn {
    pub(in crate::internal) fn try_parse(
        item: &syn::DeriveInput,
        spacetimedsl_table: &Option<DSLTable>,
        spacetimedb_column: &DBColumn,
    ) -> syn::Result<Option<DSLColumn>> {
        todo!()
    }
}
