//! `#[creation_default(<expression>)]`: the value `create_<table>` fills a column with,
//! instead of asking the caller for it in `Create<Table>`.

use {
    super::creation_default,
    crate::internal::error,
    spacetime_bindings_macro_input::sats::SatsField,
    syn::{Attribute, Expr},
};

/// Reads `#[creation_default(<expression>)]` from a column. A column may have one.
pub fn try_parse(field: &SatsField<'_>) -> syn::Result<Option<Expr>> {
    let mut creation_default_attributes = field
        .original_attrs
        .iter()
        .filter(|attribute| attribute.path() == creation_default);

    let Some(creation_default_attribute) = creation_default_attributes.next() else {
        return Ok(None);
    };

    if let Some(repeated_attribute) = creation_default_attributes.next() {
        return Err(error::multiple_creation_default_attributes(
            repeated_attribute,
        ));
    }

    parse_expression(creation_default_attribute).map(Some)
}

fn parse_expression(creation_default_attribute: &Attribute) -> syn::Result<Expr> {
    creation_default_attribute
        .meta
        .require_list()
        .and_then(|list| list.parse_args())
        .map_err(|_| error::creation_default_without_expression(creation_default_attribute))
}
