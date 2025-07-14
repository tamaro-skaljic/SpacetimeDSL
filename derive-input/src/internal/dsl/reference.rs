use super::{path, referenced_by, table};
use crate::api::dsl::reference::ReferencingTable;
use spacetime_bindings_macro_input::{
    match_meta, sats::SatsField, sym::primary_key, util::check_duplicate,
};
use syn::{Ident, Path};

impl ReferencingTable {
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
                    attr,
                    "`#[referenced_by]` is only allowed in combination with `#[primary_key]`!",
                ));
            }

            let mut path_value: Option<Path> = None;
            let mut table_name: Option<Ident> = None;

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
                });

                Ok(())
            })?;

            let path_value = path_value
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "PathToTable must be set in `#[referenced_by(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`.",
            ))?;

            let table_name = table_name
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "TableName must be set in `#[referenced_by(table = TableName)]`, e.g. `table = my_table`.",
            ))?;

            referencing_tables.push(ReferencingTable {
                path: path_value,
                table_name,
            });
        }

        Ok(referencing_tables)
    }
}
