use crate::api::dsl::method::{ColumnDSLMethods, TableDSLMethods};

impl TableDSLMethods {
    pub(in crate::internal) fn try_parse() -> syn::Result<TableDSLMethods> {
        todo!()
    }
}

pub(in crate::internal) fn get_column_dsl_methods() -> Option<ColumnDSLMethods> {
    todo!()
}
