use crate::api::{
    db::{column::SpacetimeDBColumn, index::Index},
    rust::column::RustField,
};
use crate::internal::error;
use syn::Ident;

impl SpacetimeDBColumn {
    /// `single_column_index` is the index `internal::table` assigned to this column: the
    /// first single-column index declared on it, if any.
    pub(crate) fn map(
        rust_field: &RustField,
        single_column_index: Option<Index>,
        singular_table_name: &Ident,
        auto_inc_column_names: &[Ident],
        primary_key_column_name: &Ident,
        is_singleton: bool,
    ) -> syn::Result<SpacetimeDBColumn> {
        let column_name = &rust_field.name;

        let is_primary_key = column_name.eq(primary_key_column_name);

        if is_primary_key
            && column_name
                .to_string()
                .starts_with(&singular_table_name.to_string())
        {
            return Err(error::primary_key_prefixed_with_table_name(
                column_name,
                singular_table_name,
            ));
        }

        // Singleton validation: user-defined columns must not have #[primary_key], #[index], or #[unique]
        // (the injected `id: u8` pk is allowed)
        if is_singleton && !is_primary_key && single_column_index.is_some() {
            return Err(error::single_column_index_on_singleton(column_name));
        }

        let is_auto_inc = auto_inc_column_names
            .iter()
            .any(|c| c.to_string().eq(&column_name.to_string()));

        Ok(SpacetimeDBColumn {
            is_primary_key,
            single_column_index,
            is_auto_inc,
        })
    }
}
