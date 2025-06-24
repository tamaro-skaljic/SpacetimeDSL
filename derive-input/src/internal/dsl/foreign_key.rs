use super::foreign_key;
use crate::api::dsl::foreign_key::{ForeignKey, OnDeleteStrategy};
use crate::internal::dsl::{column, on_delete, path, table};
use quote::ToTokens;
use spacetime_bindings_macro_input::match_meta;
use spacetime_bindings_macro_input::sats::SatsField;
use spacetime_bindings_macro_input::sym::{index, primary_key, unique};
use spacetime_bindings_macro_input::util::check_duplicate;
use syn::meta::ParseNestedMeta;
use syn::{Ident, Meta, Path};

impl ForeignKey {
    // TODO: There should be a proper error message if the primary_key column which is referenced by this column has not a valid type (This column: T, the other column: T | Option<T>). But this probably won't work from inside rust macros, more likely in a build.rs. Currently it's a compilation error.
    pub(in crate::internal) fn try_parse(field: &SatsField<'_>) -> syn::Result<Option<ForeignKey>> {
        let mut foreign_key_value = None;

        let mut has_index = false;
        for attr in field.original_attrs {
            if attr.meta.path().eq(&primary_key)
                || attr.meta.path().eq(&unique)
                || attr.meta.path().eq(&index)
            {
                has_index = true;
                break;
            }
        }

        for attr in field.original_attrs {
            if attr.meta.path().ne(&foreign_key) {
                continue;
            }

            if !has_index {
                return Err(syn::Error::new_spanned(
                    &attr,
                    "`#[foreign_key]` is only allowed in combination with `#[primary_key]`, `#[unique]` or `#[index]`!",
                ));
            }

            if foreign_key_value.is_some() {
                return Err(syn::Error::new_spanned(
                    &attr,
                    "`#[foreign_key]` is only allowed once per column!",
                ));
            }

            let mut path_value: Option<Path> = None;
            let mut table_name: Option<Ident> = None;
            let mut column_name: Option<Ident> = None;
            let mut on_delete_strategy = None;

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
                    on_delete => {
                        check_duplicate(&on_delete_strategy, &meta)?;
                        on_delete_strategy = Some(OnDeleteStrategy::try_parse(&meta, &attr.meta)?);
                    }
                });
                Ok(())
            })?;

            let path_value = path_value
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "PathToTable must be set in `#[foreign_key(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`.",
            ))?
            .to_token_stream()
            .to_string().into();

            let table_name = table_name
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "TableName must be set in `#[foreign_key(table = TableName)]`, e.g. `table = my_table`.",
            ))?
            .to_token_stream()
            .to_string()
            .into();

            let column_name = match column_name {
                Some(column_name) => column_name.to_token_stream().to_string().into(),
                // TODO: Document this default
                None => "id".into(),
            };

            let on_delete_strategy = on_delete_strategy.ok_or_else(|| {
            syn::Error::new_spanned(
                &attr.meta,
                "OnDeleteStrategy must be set in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete` (or Error, SetNone or SetZero).",
            )
        })?;

            foreign_key_value = Some(ForeignKey {
                path: path_value,
                table_name,
                column_name,
                on_delete_strategy,
            });
        }

        Ok(foreign_key_value)
    }
}

impl OnDeleteStrategy {
    fn try_parse(meta: &ParseNestedMeta<'_>, tokens: &Meta) -> syn::Result<OnDeleteStrategy> {
        let action_variant: Ident = meta.value()?.parse()?;
        let action_variant: &str = &action_variant.to_string();

        // TODO: Add Checks (Option for SetNone, Numeric for SetZero (SpacetimeDB has a is_numeric function), ...)
        match action_variant {
            "Error" => Ok(OnDeleteStrategy::Error),
            "Delete" => Ok(OnDeleteStrategy::Delete),
            "SetNone" => Err(syn::Error::new_spanned(
                &tokens,
                "Because Option is currently not allowed on primary_key and unique/btree indices, `OnDeleteStrategy::SetNone` isn't implemented yet. `OnDeleteStrategy` must be one of `Error`, `Delete` or `SetZero` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.".to_string(),
            )),
            "SetZero" => Ok(OnDeleteStrategy::SetZero),
            _ => Err(syn::Error::new_spanned(
                &tokens,
                "`OnDeleteStrategy` must be one of `Error`, `Delete`, `SetNone` or `SetZero` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.".to_string(),
            )),
        }
    }
}
