use super::{column, path, referenced_by, table};
use crate::api::dsl::reference::ReferencingTable;
use quote::ToTokens;
use spacetime_bindings_macro_input::{
    match_meta, sats::SatsField, sym::primary_key, util::check_duplicate,
};
use syn::{Ident, Path};

impl ReferencingTable {
    // TODO: There should be a proper error message if the column which references the primary_key column has not a valid type (This column: T | Option<T>, the other column: T). But this probably won't work from inside rust macros, more likely in a build.rs. Currently it's a compilation error.
    pub(in crate::internal) fn try_parse(
        field: &SatsField<'_>,
    ) -> syn::Result<Vec<ReferencingTable>> {
        let mut referencing_tables: Vec<ReferencingTable> = vec![];

        let mut is_primary_key = false;
        for attr in field.original_attrs {
            if attr.meta.path().eq(&primary_key) {
                is_primary_key = true;
                break;
            }
        }

        for attr in field.original_attrs {
            if attr.meta.path().ne(&referenced_by) {
                continue;
            }

            if !is_primary_key {
                return Err(syn::Error::new_spanned(
                    &attr,
                    "`#[referenced_by]` is only allowed in combination with `#[primary_key]`!",
                ));
            }

            let mut path_value: Option<Path> = None;
            let mut table_name: Option<Ident> = None;
            let mut column_name: Option<Ident> = None;

            attr.parse_nested_meta(|meta| {
                match_meta!(match meta {
                    path => {
                        check_duplicate(&path_value, &meta)?;
                        path_value = Some(meta.value()?.parse()?);
                    }
                    table => {
                        check_duplicate(&table_name, &meta)?;
                        table_name = Some(meta.value()?.parse()?);
                    }
                    column => {
                        check_duplicate(&column_name, &meta)?;
                        column_name = Some(meta.value()?.parse()?);
                    }
                });

                Ok(())
            })?;

            let path_value = path_value
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "PathToTable must be set in `#[referenced_by(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`.",
            ))?
            .to_token_stream()
            .to_string()
            .into();

            let table_name = table_name
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "TableName must be set in `#[referenced_by(table = TableName)]`, e.g. `table = my_table`.",
            ))?
            .to_token_stream()
            .to_string()
            .into();

            let column_name = column_name
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "ColumnName must be set in `#[referenced_by(column = ColumnName)]`, e.g. `column = id`.",
            ))?
            .to_token_stream()
            .to_string()
            .into();

            referencing_tables.push(ReferencingTable {
                path: path_value,
                table_name,
                column_name,
            });
        }

        Ok(referencing_tables)
    }
}
