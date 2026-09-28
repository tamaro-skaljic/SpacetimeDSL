use crate::api::{
    db::{column::SpacetimeDBColumn, index::Index},
    rust::column::RustField,
};
use syn::Ident;

impl SpacetimeDBColumn {
    /// `single_column_index` is the index `internal::table` assigned to this column, if any.
    pub(crate) fn map(
        rust_field: &RustField,
        single_column_index: Option<Index>,
        auto_inc_column_names: &[Ident],
        primary_key_column_name: &Ident,
    ) -> SpacetimeDBColumn {
        let column_name = &rust_field.name;

        let is_primary_key = column_name.eq(primary_key_column_name);

        let is_auto_inc = auto_inc_column_names
            .iter()
            .any(|c| c.to_string().eq(&column_name.to_string()));

        SpacetimeDBColumn {
            is_primary_key,
            single_column_index,
            is_auto_inc,
        }
    }
}
