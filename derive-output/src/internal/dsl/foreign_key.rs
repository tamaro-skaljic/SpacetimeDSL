use crate::api::dsl::column::{ForeignKey, OnDeleteAction};
use quote::ToTokens;
use spacetime_bindings_macro_input::match_meta;
use spacetime_bindings_macro_input::sym::Symbol;
use spacetime_bindings_macro_input::symbol;
use spacetime_bindings_macro_input::util::check_duplicate;
use syn::Ident;

pub(in crate::internal) fn parse_foreign_key(attr: &syn::Attribute) -> syn::Result<ForeignKey> {
    let mut table_value: Option<Ident> = None;
    let mut column_value: Option<Ident> = None;
    let mut action_value = None;

    attr.parse_nested_meta(|meta| {
        match_meta!(match meta {
            table => {
                check_duplicate(&table_value, &meta)?;
                table_value = Some(meta.value()?.parse()?);
            }
            column => {
                check_duplicate(&column_value, &meta)?;
                column_value = Some(meta.value()?.parse()?);
            }
            action => {
                check_duplicate(&action_value, &meta)?;

                let action_variant: Ident = meta.value()?.parse()?;
                let action_variant: &str = &action_variant.to_string();

                match action_variant {
                    "SetNone" => {
                        action_value = Some(OnDeleteAction::SetNone);
                    }
                    "SetZero" => {
                        action_value = Some(OnDeleteAction::SetZero);
                    }
                    "Delete" => {
                        action_value = Some(OnDeleteAction::Delete);
                    }
                    _ => {}
                };
            }
        });
        Ok(())
    })?;

    let table_value = table_value
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &attr.meta,
                "missing foreign_key table, e.g. table = my_table",
            )
        })?
        .to_token_stream()
        .to_string()
        .into();

    let column_value = column_value
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &attr.meta,
                "missing foreign_key column, e.g. column = my_column",
            )
        })?
        .to_token_stream()
        .to_string()
        .into();

    let action_value = action_value.ok_or_else(|| {
        syn::Error::new_spanned(
            &attr.meta,
            "missing foreign_key action, e.g. action = Delete",
        )
    })?;

    Ok(ForeignKey {
        table_name: table_value,
        column_name: column_value,
        on_delete: action_value,
    })
}

symbol!(foreign_key);
symbol!(table);
symbol!(column);
symbol!(action);
