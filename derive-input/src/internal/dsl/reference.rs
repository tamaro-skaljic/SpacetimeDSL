use super::{path, referenced_by, table};
use crate::api::dsl::reference::ReferencingTable;
use crate::internal::dsl::error;
use spacetime_bindings_macro_input::{
    match_meta, sats::SatsField, sym::primary_key, util::check_duplicate,
};
use syn::{Ident, Path};

impl ReferencingTable {
    pub(in crate::internal) fn try_parse(
        has_delete_method: &bool,
        is_soft_deletable: bool,
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
                return Err(error::referenced_by_without_primary_key(attr));
            }

            if !has_delete_method && !is_soft_deletable {
                return Err(error::referenced_by_without_delete_or_soft_delete_method(
                    attr,
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

            let path_value =
                path_value.ok_or_else(|| error::missing_referenced_by_path(&attr.meta))?;

            let table_name =
                table_name.ok_or_else(|| error::missing_referenced_by_table(&attr.meta))?;

            referencing_tables.push(ReferencingTable {
                path: path_value,
                table_name,
            });
        }

        Ok(referencing_tables)
    }
}
