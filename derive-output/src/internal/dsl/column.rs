use super::{foreign_key, getter, method, setter};
use crate::api::{
    db::SpacetimeDBColumn,
    dsl::{
        column::{SpacetimeDSLColumn, WrapperType},
        table::SpacetimeDSLTable,
    },
};
use quote::ToTokens;
use spacetime_bindings_macro_input::sats::SatsField;

impl SpacetimeDSLColumn {
    pub(in crate::internal) fn try_parse(
        item: &syn::DeriveInput,
        field: &SatsField<'_>,
        spacetimedb_column: &SpacetimeDBColumn,
        mut spacetimedsl_table: SpacetimeDSLTable,
    ) -> syn::Result<(SpacetimeDSLTable, SpacetimeDSLColumn)> {
        let is_option = field
            .ty
            .to_token_stream()
            .to_string()
            .starts_with("Option<");

        let wrapper_type = WrapperType::try_parse(item, field)?;

        let foreign_key = foreign_key::try_parse(field)?;

        let getter = getter::get_getter(field, is_option, &wrapper_type);

        let setter = setter::get_setter(field, is_option, &wrapper_type);

        if setter.is_some() {
            spacetimedsl_table.is_mutable = true;
        }

        let dsl_methods = method::get_column_dsl_methods(item, field, spacetimedb_column);

        Ok((
            spacetimedsl_table,
            SpacetimeDSLColumn {
                is_option,
                wrapper_type,
                foreign_key,
                getter,
                setter,
                dsl_methods,
            },
        ))
    }
}
