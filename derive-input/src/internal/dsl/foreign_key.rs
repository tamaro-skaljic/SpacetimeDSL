use super::foreign_key;
use crate::api::dsl::foreign_key::{ForeignKey, OnDeleteStrategy};
use crate::internal::dsl::{on_delete, on_soft_delete, path, table};
use quote::ToTokens;
use spacetime_bindings_macro_input::match_meta;
use spacetime_bindings_macro_input::sats::SatsField;
use spacetime_bindings_macro_input::sym::{column, index, primary_key, unique};
use spacetime_bindings_macro_input::util::check_duplicate;
use syn::meta::ParseNestedMeta;
use syn::{Ident, Meta, Path};

impl ForeignKey {
    pub(in crate::internal) fn try_parse(
        has_delete_method: &bool,
        is_soft_deletable: bool,
        is_singleton: bool,
        field: &SatsField<'_>,
    ) -> syn::Result<Option<ForeignKey>> {
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

            // Singletons don't require an index on FK columns
            if !has_index && !is_singleton {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`#[foreign_key]` is only allowed in combination with `#[primary_key]`, `#[unique]` or `#[index]`!",
                ));
            }

            if foreign_key_value.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`#[foreign_key]` is only allowed once per column!",
                ));
            }

            let mut path_value: Option<Path> = None;
            let mut table_name: Option<Ident> = None;
            let mut primary_key_column_name: Option<Ident> = None;
            let mut on_delete_strategy = None;
            let mut on_soft_delete_strategy = None;

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
                        check_duplicate(&primary_key_column_name, &meta)?;
                        primary_key_column_name = Some(meta.value()?.parse()?);
                    }
                    on_delete => {
                        check_duplicate(&on_delete_strategy, &meta)?;
                        on_delete_strategy = Some(OnDeleteStrategy::try_parse_for_on_delete(
                            &meta, &attr.meta,
                        )?);
                    }
                    on_soft_delete => {
                        check_duplicate(&on_soft_delete_strategy, &meta)?;
                        on_soft_delete_strategy = Some(
                            OnDeleteStrategy::try_parse_for_on_soft_delete(&meta, &attr.meta)?,
                        );
                    }
                });
                Ok(())
            })?;

            let path_value = path_value
                .ok_or_else(|| syn::Error::new_spanned(
                    &attr.meta,
                    "PathToTable must be set in `#[foreign_key(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`. Supply the path to the referenced table.",
                ))?;

            let table_name = table_name
                .ok_or_else(|| syn::Error::new_spanned(
                    &attr.meta,
                    "TableName must be set in `#[foreign_key(table = TableName)]`, e.g. `table = my_table`. Supply the name of the referenced table.",
                ))?;

            let primary_key_column_name = primary_key_column_name
                .ok_or_else(|| syn::Error::new_spanned(
                    &attr.meta,
                    "PrimaryKeyColumnName must be set in `#[foreign_key(column = PrimaryKeyColumnName)]`, e.g. `column = id`. Supply the name of the primary key column in the referenced table.",
                ))?;

            if on_delete_strategy.is_none() && on_soft_delete_strategy.is_none() {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "A `#[foreign_key]` must set `on_delete`, `on_soft_delete`, or both, e.g. `on_delete = Delete`.\nSet `on_delete` when the referenced table has a delete method, set `on_soft_delete` when it is soft-deletable, and set both when it is both. The referenced table's own `#[referenced_by]` decides which of them is required; leaving out a required one is an unresolved import naming the field to add.",
                ));
            }

            if on_delete_strategy.as_ref() == Some(&OnDeleteStrategy::SetZero)
                && field
                    .vis
                    .to_token_stream()
                    .to_string()
                    .eq(&syn::Visibility::Inherited.to_token_stream().to_string())
            {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "`OnDeleteStrategy::SetZero` is only allowed on non-private columns, because setters are only generated for non-private columns (column-level mutability constraints)!",
                ));
            }

            if !has_delete_method && on_delete_strategy.as_ref() == Some(&OnDeleteStrategy::Delete)
            {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "`OnDeleteStrategy::Delete` is only allowed when the table has a delete method (`#[dsl(method(delete = true))]`)!",
                ));
            }

            let uses_soft_delete = on_delete_strategy.as_ref()
                == Some(&OnDeleteStrategy::SoftDelete)
                || on_soft_delete_strategy.as_ref() == Some(&OnDeleteStrategy::SoftDelete);

            if uses_soft_delete && !is_soft_deletable {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "`OnDeleteStrategy::SoftDelete` is only allowed when this table is soft-deletable (`#[dsl(method(soft_delete = true))]`)!\nThe strategy retires the rows of this table, which needs a marker column for the DSL to write.",
                ));
            }

            foreign_key_value = Some(ForeignKey {
                path: path_value,
                table_name,
                primary_key_column_name,
                on_delete_strategy,
                on_soft_delete_strategy,
            });
        }

        Ok(foreign_key_value)
    }
}

impl OnDeleteStrategy {
    // TODO: Add Checks (https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 Option for SetNone, Numeric for SetZero (SpacetimeDB has a is_numeric function), ...)
    fn try_parse_for_on_delete(
        meta: &ParseNestedMeta<'_>,
        tokens: &Meta,
    ) -> syn::Result<OnDeleteStrategy> {
        let action_variant: Ident = meta.value()?.parse()?;

        match action_variant.to_string().as_str() {
            "Error" => Ok(OnDeleteStrategy::Error),
            "Delete" => Ok(OnDeleteStrategy::Delete),
            "SoftDelete" => Ok(OnDeleteStrategy::SoftDelete),
            "SetNone" => Err(syn::Error::new_spanned(
                tokens,
                "Because Option is currently not allowed on primary_key and unique/btree indices, `OnDeleteStrategy::SetNone` isn't implemented yet. `OnDeleteStrategy` must be one of `Error`, `Delete`, `SoftDelete`, `SetZero` or `Ignore` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.".to_string(),
            )),
            "SetZero" => Ok(OnDeleteStrategy::SetZero),
            "Ignore" => Ok(OnDeleteStrategy::Ignore),
            _ => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy` must be one of `Error`, `Delete`, `SoftDelete`, `SetNone`, `SetZero` or `Ignore` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.".to_string(),
            )),
        }
    }

    /// `on_soft_delete` accepts three of the six strategies. The other three each undo
    /// what a soft deletion is for, so they are rejected by name rather than lumped into
    /// the catch-all: the message can then say which one it is and why.
    fn try_parse_for_on_soft_delete(
        meta: &ParseNestedMeta<'_>,
        tokens: &Meta,
    ) -> syn::Result<OnDeleteStrategy> {
        let action_variant: Ident = meta.value()?.parse()?;

        match action_variant.to_string().as_str() {
            "Error" => Ok(OnDeleteStrategy::Error),
            "SoftDelete" => Ok(OnDeleteStrategy::SoftDelete),
            "Ignore" => Ok(OnDeleteStrategy::Ignore),
            "Delete" => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy::Delete` is not allowed in `on_soft_delete`! Soft-deleting a row must not physically remove the rows which reference it. `on_soft_delete` must be one of `Error`, `SoftDelete` or `Ignore`.".to_string(),
            )),
            "SetZero" => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy::SetZero` is not allowed in `on_soft_delete`! Soft deletion preserves the row, so clearing the foreign key column would destroy what it preserved. `on_soft_delete` must be one of `Error`, `SoftDelete` or `Ignore`.".to_string(),
            )),
            _ => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy` must be one of `Error`, `SoftDelete` or `Ignore` in `#[foreign_key(on_soft_delete = OnDeleteStrategy)]`, e.g. `on_soft_delete = SoftDelete`.".to_string(),
            )),
        }
    }
}
