use crate::api::db::{IndexType, SpacetimeDBTable};
use crate::api::dsl::reference::ReferencingTable;
use crate::api::dsl::table::SpacetimeDSLTable;
use crate::internal::dsl::{plural_name, unique_index};
use proc_macro2::Span;
use quote::ToTokens;
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

        parser(|meta| {
            match_meta!(match meta {
                plural_name => {
                    check_duplicate(&name_plural, &meta)?;
                    let value = meta.value()?;
                    name_plural = Some(value.parse()?);
                }
                unique_index => unique_indices.push(parse_unique_index(meta)?),
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
        let mut referencing_tables = vec![];
        for field in &column_args.fields {
            let refs = ReferencingTable::try_parse(field)?;
            if referencing_tables.is_empty() {
                referencing_tables = refs;
            }

            if !is_mutable {
                is_mutable = match field.vis {
                    syn::Visibility::Public(_) => true,
                    syn::Visibility::Restricted(_) => true,
                    _ => false,
                };
            }
            if !has_created_at_column {
                if field.name.as_ref().unwrap().eq("created_at") {
                    let field_type = field.ty.to_token_stream().to_string();
                    if !field_type.eq("Timestamp") {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!(
                                "A column with name `created_at` should have the type `spacetimedb::Timestamp`! Found: {field_type}"
                            ),
                        ));
                    }

                    match field.vis {
                        syn::Visibility::Public(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `created_at` should have `Visibility::Inherited`! Found: Visibility::Public",
                            ));
                        }
                        syn::Visibility::Restricted(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `created_at` should have `Visibility::Inherited`! Found: Visibility::Restricted",
                            ));
                        }
                        syn::Visibility::Inherited => {
                            has_created_at_column = true;
                        }
                    }
                }
            }
            if !has_modified_at_column {
                if field.name.as_ref().unwrap().eq("modified_at") {
                    let field_type = field.ty.to_token_stream().to_string();
                    if !field_type.eq("Timestamp") {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!(
                                "A column with name `modified_at` should have the type `spacetimedb::Timestamp`! Found: {field_type}"
                            ),
                        ));
                    }

                    match field.vis {
                        syn::Visibility::Public(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `modified_at` should have `Visibility::Inherited`! Found: Visibility::Public",
                            ));
                        }
                        syn::Visibility::Restricted(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `modified_at` should have `Visibility::Inherited`! Found: Visibility::Restricted",
                            ));
                        }
                        syn::Visibility::Inherited => {
                            has_modified_at_column = true;
                        }
                    }
                }
            }
        }

        Ok((
            spacetimedb_table,
            SpacetimeDSLTable {
                plural_name: name_plural,
                is_mutable,
                has_created_at_column,
                has_modified_at_column,
                referencing_tables,
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
