use crate::internal::dsl::{hook_on, plural_name, unique_index};
use spacetime_bindings_macro_input::{match_meta, sym, util::check_duplicate};
use syn::{
    Ident,
    meta::{ParseNestedMeta, parser},
    parse::Parser,
};

pub(crate) mod integration;

mod table;

mod column;

mod rust;

mod db;

mod dsl;

pub(crate) fn try_parse(
    args: proc_macro2::TokenStream,
    input: &syn::DeriveInput,
) -> syn::Result<crate::api::Table> {
    // Parse plural_name from DSL arguments - it's required
    let (plural_name_value, unique_indices, hook_on_insert, hook_on_update, hook_on_delete) = try_parse_dsl(args)?;

    // Pass plural_name to integration for intelligent table selection
    let (table_args, column_args) =
        integration::spacetime_bindings_macro_input(input, &plural_name_value)?;

    // Pass the parsed plural_name to avoid re-parsing
    table::try_parse(
        input,
        &table_args,
        &column_args,
        plural_name_value,
        unique_indices,
        hook_on_insert,
        hook_on_update,
        hook_on_delete,
    )
}

// Parse plural_name from DSL arguments
fn try_parse_dsl(args: proc_macro2::TokenStream) -> syn::Result<(Ident, Vec<Ident>, bool, bool, bool)> {
    let mut name_plural: Option<Ident> = None;
    let mut unique_indices = vec![];
    let mut hook_on_insert = false;
    let mut hook_on_update = false;
    let mut hook_on_delete = false;

    parser(|meta| {
        match_meta!(match meta {
            plural_name => {
                check_duplicate(&name_plural, &meta)?;
                let value = meta.value()?;
                name_plural = Some(value.parse()?);
            }
            unique_index => unique_indices.push(try_parse_unique_index(meta)?),
            hook_on => {
                meta.parse_nested_meta(|nested_meta| {
                    let hook_type: Ident = nested_meta.path.get_ident()
                        .ok_or_else(|| nested_meta.error("Expected hook type (insert, update, or delete)"))?
                        .clone();
                    
                    match hook_type.to_string().as_str() {
                        "insert" => {
                            if hook_on_insert {
                                return Err(nested_meta.error("Duplicate hook_on(insert)"));
                            }
                            hook_on_insert = true;
                        }
                        "update" => {
                            if hook_on_update {
                                return Err(nested_meta.error("Duplicate hook_on(update)"));
                            }
                            hook_on_update = true;
                        }
                        "delete" => {
                            if hook_on_delete {
                                return Err(nested_meta.error("Duplicate hook_on(delete)"));
                            }
                            hook_on_delete = true;
                        }
                        _ => {
                            return Err(nested_meta.error("Invalid hook type. Expected insert, update, or delete"));
                        }
                    }
                    Ok(())
                })?;
            }
        });
        Ok(())
    })
    .parse2(args)?;

    Ok((
        name_plural.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "PluralName must be set in `#[dsl(plural_name = PluralName)]`",
            )
        })?,
        unique_indices,
        hook_on_insert,
        hook_on_update,
        hook_on_delete,
    ))
}

// Parse unique index from meta
fn try_parse_unique_index(meta: ParseNestedMeta<'_>) -> syn::Result<Ident> {
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
        .ok_or_else(|| meta.error("IndexName must be set in `#[dsl(unique_index(name = IndexName))]`, e.g. `name = my_index`."))?;

    Ok(name)
}
