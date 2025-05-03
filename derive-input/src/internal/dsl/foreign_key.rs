use crate::api::dsl::foreign_key::{ForeignKey, OnDeleteStrategy};
use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::match_meta;
use spacetime_bindings_macro_input::sats::SatsField;
use spacetime_bindings_macro_input::sym::Symbol;
use spacetime_bindings_macro_input::symbol;
use spacetime_bindings_macro_input::util::check_duplicate;
use syn::meta::ParseNestedMeta;
use syn::{Error, Ident};

/// TODOs: Check that the referenced field has a valid type (This field: T | Option<T> | Vec<T>, the other field: T)
pub(in crate::internal) fn try_parse(field: &SatsField<'_>) -> syn::Result<Option<ForeignKey>> {
    let mut foreign_key_value = None;

    for attr in field.original_attrs {
        if attr.meta.path().ne(&foreign_key) {
            continue;
        }

        if foreign_key_value.is_some() {
            return Err(syn::Error::new_spanned(
                &attr,
                "`#[foreign_key]` is only allowed once per column!",
            ));
        }

        let mut table_name: Option<Ident> = None;
        let mut column_name: Option<Ident> = None;
        let mut on_delete_strategy = None;

        attr.parse_nested_meta(|meta| {
            match_meta!(match meta {
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
                    on_delete_strategy = Some(OnDeleteStrategy::try_parse(&meta)?);
                }
            });
            Ok(())
        })?;

        let table_name = table_name
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    &attr.meta,
                    "TableName must be set in `#[foreign_key(table = TableName)]`, e.g. `table = my_table`.",
                )
            })?
            .to_token_stream()
            .to_string()
            .into();

        let column_name = column_name
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    &attr.meta,
                    "ColumnName must be set in `#[foreign_key(column = ColumnName)]`, e.g. `column = my_column`.",
                )
            })?
            .to_token_stream()
            .to_string()
            .into();

        let on_delete_strategy = on_delete_strategy.ok_or_else(|| {
            syn::Error::new_spanned(
                &attr.meta,
                "OnDeleteStrategy must be set in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Cascade`.",
            )
        })?;

        foreign_key_value = Some(ForeignKey {
            table_name,
            column_name,
            on_delete_strategy,
        });
    }

    Ok(foreign_key_value)
}

symbol!(foreign_key);
symbol!(table);
symbol!(column);
symbol!(on_delete);

impl OnDeleteStrategy {
    fn try_parse(meta: &ParseNestedMeta<'_>) -> syn::Result<OnDeleteStrategy> {
        let action_variant: Ident = meta.value()?.parse()?;
        let action_variant: &str = &action_variant.to_string();

        match action_variant {
            "Cascade" => Ok(OnDeleteStrategy::Cascade),
            "SetNone" => Ok(OnDeleteStrategy::SetNone),
            "SetZero" => Ok(OnDeleteStrategy::SetZero),
            _ => Err(Error::new(
                Span::call_site(),
                "`OnDeleteStrategy` must be one of `Cascade`, `SetNone` or `SetZero` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Cascade`.".to_string(),
            )),
        }
    }
}
