use crate::api::db::{IndexType, SpacetimeDBTable};
use crate::api::dsl::table::{OnDeleteHook, SpacetimeDSLTable};
use crate::internal::dsl::{on_delete, path, plural_name, table, unique_index};
use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::sym::column;
use spacetime_bindings_macro_input::table::ColumnArgs;
use spacetime_bindings_macro_input::{match_meta, sym, util::check_duplicate};
use syn::{
    Ident,
    meta::{ParseNestedMeta, parser},
    parse::Parser,
};

impl SpacetimeDSLTable {
    pub(in crate::internal) fn try_parse(
        args: proc_macro2::TokenStream,
        column_args: &ColumnArgs<'_>,
        mut spacetimedb_table: SpacetimeDBTable,
    ) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable)> {
        let mut name_plural: Option<Ident> = None;
        let mut unique_indices = vec![];
        let mut on_delete_hooks = vec![];

        parser(|meta| {
            match_meta!(match meta {
                plural_name => {
                    check_duplicate(&name_plural, &meta)?;
                    let value = meta.value()?;
                    name_plural = Some(value.parse()?);
                }
                unique_index => unique_indices.push(parse_unique_index(meta)?),
                on_delete => on_delete_hooks.push(parse_on_delete(meta, &spacetimedb_table)?),
            });
            Ok(())
        })
        .parse2(args)?;

        let name_plural = name_plural.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                format_args!("PluralName must be set in `#[dsl(plural_name = PluralName)]`, e.g. `plural_name = {}s`.", spacetimedb_table.singular_name),
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

        let mut is_mutable = false;
        let mut has_created_at_column = false;
        let mut has_modified_at_column = false;
        for field in &column_args.fields {
            if !is_mutable {
                is_mutable = match field.vis {
                    syn::Visibility::Public(_) => true,
                    syn::Visibility::Restricted(_) => true,
                    _ => false,
                };
            }
            if !has_created_at_column {
                has_created_at_column = field.name.as_ref().unwrap().eq("created_at")
                    && field.ty.to_token_stream().to_string().eq("Timestamp");
            }
            if !has_modified_at_column {
                has_modified_at_column = field.name.as_ref().unwrap().eq("modified_at")
                    && field.ty.to_token_stream().to_string().eq("Timestamp");
            }
        }

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
        .ok_or_else(|| meta.error("IndexName must be set in `#[dsl(unique_index(name = IndexName))]`, e.g. `name = my_index`."))?
        .to_token_stream()
        .to_string()
        .into();

    Ok(name)
}

fn parse_on_delete(
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
        .ok_or_else(|| meta.error("ModulePath must be set in `#[dsl(on_delete(path = ModulePath))]`, e.g. `path = path::to::my::module`."))?
        .to_token_stream()
        .to_string();

    let table_value: String = table_value
        .ok_or_else(|| meta.error("TableName must be set in `#[dsl(on_delete(table = TableName))]`, e.g. `table = my_table`."))?
        .to_token_stream()
        .to_string();

    let column_value: String = column_value
        .ok_or_else(|| meta.error("ColumnName must be set in `#[dsl(on_delete(column = ColumnName))]`, e.g. `column = my_column`."))?
        .to_token_stream()
        .to_string();

    // TODO: Implement deletion hooks
    let function_name = format!(
        "{path_value}::delete_{}_hook_for_{table_value}_{column_value}",
        spacetimedb_table.singular_name
    )
    .into();

    Ok(OnDeleteHook {
        function_path: function_name,
    })
}
