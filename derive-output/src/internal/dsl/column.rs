use crate::api::{
    db::DBColumn,
    dsl::{
        column::{DSLColumn, WrapperType},
        table::DSLTable,
    },
};
use spacetime_bindings_macro_input::{sats::SatsField, sym::Symbol, symbol};
use syn::Ident;

impl DSLColumn {
    pub(in crate::internal) fn try_parse(
        item: &syn::DeriveInput,
        field: &SatsField<'_>,
        spacetimedb_column: &DBColumn,
        spacetimedsl_table: DSLTable,
    ) -> syn::Result<(DSLTable, DSLColumn)> {
        let wrapper_type =
            WrapperType::try_parse(item, field, spacetimedb_column, &spacetimedsl_table)?;
        let foreign_key;
        let getter;
        let setter;
        let dsl_methods;

        Ok((
            spacetimedsl_table,
            DSLColumn {
                wrapper_type,
                foreign_key,
                getter,
                setter,
                dsl_methods,
            },
        ))
    }
}

pub(in crate::internal) fn parse(
    attr: &syn::Attribute,
    field_ident: &Ident,
) -> syn::Result<Option<Self>> {
    let Some(ident) = attr.path().get_ident() else {
        return Ok(None);
    };
    Ok(if ident == super::wrapper::wrapper {
        attr.meta.require_name_value()?;
        Some(ColumnAttr::Wrapper(index))
    } else if ident == super::wrapper::wrapped {
        attr.meta.require_name_value()?;
        Some(ColumnAttr::Wrapped(ident.span()))
    } else if ident == foreign_key {
        attr.meta.require_name_value()?;
        Some(ColumnAttr::ForeignKey(ident.span()))
    } else {
        None
    })
}
