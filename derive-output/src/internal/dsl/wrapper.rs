use spacetime_bindings_macro_input::symbol;
use spacetime_bindings_macro_input::{sats::SatsField, sym::Symbol};

use crate::api::db::DBColumn;
use crate::api::dsl::column::WrapperType;
use crate::api::dsl::table::DSLTable;

use super::column::ColumnAttr;

symbol!(wrapper);
symbol!(wrapped);

impl WrapperType {
    pub(in crate::internal) fn try_parse(
        item: &syn::DeriveInput,
        field: &SatsField<'_>,
        spacetimedb_column: &DBColumn,
        spacetimedsl_table: &DSLTable,
    ) -> syn::Result<Option<WrapperType>> {
        let wrapper_type;


        Ok(wrapper_type)
    }
}
