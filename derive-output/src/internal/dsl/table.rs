use spacetime_bindings_macro_input::table::ColumnArgs;

use crate::api::{db::SpacetimeDBTable, dsl::table::DSLTable, rust::RustStruct};

impl DSLTable {
    pub(in crate::internal) fn try_parse(
        args: &syn::Attribute,
        item: &syn::DeriveInput,
        column_args: &ColumnArgs<'_>,
        rust: &RustStruct,
        spacetimedb_table: &SpacetimeDBTable,
    ) -> syn::Result<Option<DSLTable>> {
        todo!()
    }
}
