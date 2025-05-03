use crate::api::db::{IndexType, SpacetimeDBTable};
use crate::api::dsl::table::{OnDeleteHook, SpacetimeDSLTable};
use crate::internal::dsl::foreign_key::column;
use crate::internal::dsl::foreign_key::table;
use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::sym::Symbol;
use spacetime_bindings_macro_input::symbol;
use spacetime_bindings_macro_input::util::check_duplicate;
use spacetime_bindings_macro_input::{match_meta, sym};
use syn::Ident;
use syn::meta::ParseNestedMeta;
use syn::parse::Parser;

impl SpacetimeDSLTable {
    pub(in crate::internal) fn try_parse(
        args: &syn::Attribute,
        mut spacetimedb_table: SpacetimeDBTable,
    ) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable)> {
        let mut name_plural: Option<Ident> = None;
        let mut unique_indices = vec![];
        let mut on_delete_hooks = vec![];

        syn::meta::parser(|meta| {
            match_meta!(match meta {
                plural_name => {
                    check_duplicate(&name_plural, &meta)?;
                    let value = meta.value()?;
                    name_plural = Some(value.parse()?);
                }
                unique_index => unique_indices.push(parse_unique_index(meta)?),
                on_delete => on_delete_hooks.push(parse_on_delete_hook(meta, &spacetimedb_table)?),
            });
            Ok(())
        })
        .parse2(args.to_token_stream())?;

        let name_plural = name_plural.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                format_args!("must specify table plural_name, e.g. `#[spacetimedsl::table(plural_name = {}s)]", spacetimedb_table.singular_name),
            )
        })?.to_token_stream().to_string().into();

        for unique_index_name in unique_indices {
            for multi_column_index in &mut spacetimedb_table.multi_column_indices {
                match &multi_column_index.index_type {
                    IndexType::BTreeMultiColumn { columns: _ } => {
                        if multi_column_index.name.eq(&unique_index_name) {
                            multi_column_index.is_unique = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Is set to true later if a column is mutable
        let is_mutable = false;
        // Is set to true later if the column exists
        let has_created_at_column = false;
        // Is set to true later if the column exists
        let has_modified_at_column = false;

        Ok((
            spacetimedb_table,
            SpacetimeDSLTable {
                plural_name: name_plural,
                on_delete_hooks,
                is_mutable,
                has_created_at_column,
                has_modified_at_column,
            },
        ))
    }
}

fn parse_unique_index(meta: ParseNestedMeta<'_>) -> syn::Result<Box<str>> {
    let mut name: Option<Ident> = None;

    meta.parse_nested_meta(|meta| {
        match_meta!(match meta {
            sym::name => {
                check_duplicate(&name, &meta)?;
                name = Some(meta.value()?.parse()?);
            }
        });
        Ok(())
    })?;

    let name = name
        .ok_or_else(|| meta.error("missing unique_index name, e.g. name = my_index"))?
        .to_token_stream()
        .to_string()
        .into();

    Ok(name)
}

fn parse_on_delete_hook(
    meta: ParseNestedMeta<'_>,
    spacetimedb_table: &SpacetimeDBTable,
) -> syn::Result<OnDeleteHook> {
    let mut path_value: Option<Ident> = None;
    let mut table_value: Option<Ident> = None;
    let mut column_value: Option<Ident> = None;

    meta.parse_nested_meta(|meta| {
        match_meta!(match meta {
            path => {
                check_duplicate(&path_value, &meta)?;
                path_value = Some(meta.value()?.parse()?);
            }
            table => {
                check_duplicate(&table_value, &meta)?;
                table_value = Some(meta.value()?.parse()?);
            }
            column => {
                check_duplicate(&column_value, &meta)?;
                column_value = Some(meta.value()?.parse()?);
            }
        });
        Ok(())
    })?;

    let path_value: String = path_value
        .ok_or_else(|| meta.error("missing on_delete path, e.g. path = path::to::table"))?
        .to_token_stream()
        .to_string();

    let table_value: String = table_value
        .ok_or_else(|| meta.error("missing on_delete table, e.g. table = my_table"))?
        .to_token_stream()
        .to_string();

    let column_value: String = column_value
        .ok_or_else(|| meta.error("missing on_delete column, e.g. column = my_column"))?
        .to_token_stream()
        .to_string();

    let function_name = format!(
        "{path_value}::delete_{}_hook_for_{table_value}_{column_value}",
        spacetimedb_table.singular_name
    )
    .into();

    Ok(OnDeleteHook {
        function_path: function_name,
    })
}

symbol!(plural_name);

symbol!(unique_index);

symbol!(on_delete);
symbol!(path);
