use super::foreign_key;
use crate::api::dsl::foreign_key::{ForeignKey, OnDeleteStrategy};
use crate::internal::dsl::{error, on_delete, on_soft_delete, path, table};
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
                return Err(error::foreign_key_without_index(attr));
            }

            if foreign_key_value.is_some() {
                return Err(error::multiple_foreign_keys(attr));
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

            let path_value =
                path_value.ok_or_else(|| error::missing_foreign_key_path(&attr.meta))?;

            let table_name =
                table_name.ok_or_else(|| error::missing_foreign_key_table(&attr.meta))?;

            let primary_key_column_name = primary_key_column_name
                .ok_or_else(|| error::missing_foreign_key_column(&attr.meta))?;

            if on_delete_strategy.is_none() && on_soft_delete_strategy.is_none() {
                return Err(error::foreign_key_without_on_delete_strategy(&attr.meta));
            }

            if on_delete_strategy.as_ref() == Some(&OnDeleteStrategy::SetZero)
                && field
                    .vis
                    .to_token_stream()
                    .to_string()
                    .eq(&syn::Visibility::Inherited.to_token_stream().to_string())
            {
                return Err(error::set_zero_strategy_on_private_column(&attr.meta));
            }

            if !has_delete_method && on_delete_strategy.as_ref() == Some(&OnDeleteStrategy::Delete)
            {
                return Err(error::delete_strategy_without_delete_method(&attr.meta));
            }

            let uses_soft_delete = on_delete_strategy.as_ref()
                == Some(&OnDeleteStrategy::SoftDelete)
                || on_soft_delete_strategy.as_ref() == Some(&OnDeleteStrategy::SoftDelete);

            if uses_soft_delete && !is_soft_deletable {
                return Err(error::soft_delete_strategy_on_table_not_soft_deletable(
                    &attr.meta,
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
            "SetNone" => Err(error::set_none_strategy_not_implemented(tokens)),
            "SetZero" => Ok(OnDeleteStrategy::SetZero),
            "Ignore" => Ok(OnDeleteStrategy::Ignore),
            _ => Err(error::unknown_on_delete_strategy(tokens)),
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
            "Delete" => Err(error::delete_strategy_in_on_soft_delete(tokens)),
            "SetZero" => Err(error::set_zero_strategy_in_on_soft_delete(tokens)),
            _ => Err(error::unknown_on_soft_delete_strategy(tokens)),
        }
    }
}
