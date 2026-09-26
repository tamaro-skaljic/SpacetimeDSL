use crate::api::dsl::table::SingletonKind;
use crate::internal::dsl::error;
use crate::internal::dsl::soft_delete::SoftDeleteMethodArgument;
use crate::internal::dsl::{
    after, before, delete, hook, insert, method, plural_name, singleton, soft_delete, unique_index,
    update, with_default,
};
use proc_macro2::Span;
use spacetime_bindings_macro_input::{match_meta, sym, table::TableArgs, util::check_duplicate};
use syn::{
    Ident,
    meta::{ParseNestedMeta, parser},
    parse::Parser,
    spanned::Spanned,
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
    // Parse DSL attribute arguments
    let mut dsl_data = try_parse_dsl(&args)?;

    // Pass plural_name to integration for intelligent table selection
    let (table_args, column_args) = integration::spacetime_bindings_macro_input(
        input,
        &dsl_data.plural_name,
        dsl_data.singleton.is_some(),
    )?;

    if dsl_data.singleton.is_some() {
        reject_unique_index_on_singleton(&dsl_data.unique_indices, &table_args)?;
    }

    // For singletons, set plural_name to the singular name from the table accessor
    // (it's only used for get_all/count_of_all which won't be generated)
    if dsl_data.singleton.is_some() {
        dsl_data.plural_name = crate::internal::table::rm_rsharp(table_args.accessor.clone());
    }

    // Pass the parsed plural_name to avoid re-parsing
    table::try_parse(input, dsl_data, &table_args, &column_args)
}

/// A singleton holds exactly one row, so a declared unique index would be a second way to
/// fetch it.
///
/// When the index it names is declared in `#[table]`, the message names that index as
/// well: removing only `unique_index` would leave a multi-column index, which
/// `SpacetimeDBTable::map` rejects next.
fn reject_unique_index_on_singleton(
    unique_indices: &[Ident],
    table_args: &TableArgs,
) -> syn::Result<()> {
    let Some(unique_index_name) = unique_indices.first() else {
        return Ok(());
    };

    let names_a_declared_index = table_args
        .indices
        .iter()
        .any(|index| index.accessor == *unique_index_name);

    Err(error::unique_index_on_singleton(
        unique_index_name,
        names_a_declared_index,
    ))
}

// Parse plural_name from DSL arguments
fn try_parse_dsl(args: &proc_macro2::TokenStream) -> syn::Result<DSLData> {
    let mut name_plural: Option<Ident> = None;
    let mut is_singleton: Option<()> = None;
    let mut singleton_with_default: Option<Span> = None;

    let mut unique_indices = vec![];

    let mut hooks = None;
    let mut before_hooks = None;
    let mut after_hooks = None;

    let mut before_insert_hook: Option<Span> = None;
    let mut before_update_hook: Option<Span> = None;
    let mut before_delete_hook: Option<Span> = None;
    let mut before_soft_delete_hook: Option<Span> = None;
    let mut after_insert_hook: Option<Span> = None;
    let mut after_update_hook: Option<Span> = None;
    let mut after_delete_hook: Option<Span> = None;
    let mut after_soft_delete_hook: Option<Span> = None;

    let mut methods = None;
    let mut update_method = None;
    let mut delete_method = None;
    let mut soft_delete_method: Option<SoftDeleteMethodArgument> = None;

    parser(|meta| {
        match_meta!(match meta {
            singleton => {
                check_duplicate(&is_singleton, &meta)?;
                is_singleton = Some(());

                // `#[dsl(singleton)]` carries no list, `#[dsl(singleton(with_default))]` does.
                if meta.input.peek(syn::token::Paren) {
                    meta.parse_nested_meta(|meta| {
                        match_meta!(match meta {
                            with_default => {
                                check_duplicate(&singleton_with_default, &meta)?;
                                singleton_with_default = Some(meta.path.span());
                            }
                        });
                        Ok(())
                    })?;
                }
            }
            plural_name => {
                check_duplicate(&name_plural, &meta)?;
                let value = meta.value()?;
                name_plural = Some(value.parse()?);
            }
            unique_index => unique_indices.push(try_parse_unique_index(meta)?),
            hook => {
                check_duplicate(&hooks, &meta)?;
                hooks = Some(());

                meta.parse_nested_meta(|meta| {
                    match_meta!(match meta {
                        before => {
                            check_duplicate(&before_hooks, &meta)?;
                            before_hooks = Some(());

                            meta.parse_nested_meta(|meta| {
                                match_meta!(match meta {
                                    insert => {
                                        check_duplicate(&before_insert_hook, &meta)?;
                                        before_insert_hook = Some(meta.path.span());
                                    }
                                    update => {
                                        check_duplicate(&before_update_hook, &meta)?;
                                        before_update_hook = Some(meta.path.span());
                                    }
                                    delete => {
                                        check_duplicate(&before_delete_hook, &meta)?;
                                        before_delete_hook = Some(meta.path.span());
                                    }
                                    soft_delete => {
                                        check_duplicate(&before_soft_delete_hook, &meta)?;
                                        before_soft_delete_hook = Some(meta.path.span());
                                    }
                                });
                                Ok(())
                            })?;
                        }
                        after => {
                            check_duplicate(&after_hooks, &meta)?;
                            after_hooks = Some(());

                            meta.parse_nested_meta(|meta| {
                                match_meta!(match meta {
                                    insert => {
                                        check_duplicate(&after_insert_hook, &meta)?;
                                        after_insert_hook = Some(meta.path.span());
                                    }
                                    update => {
                                        check_duplicate(&after_update_hook, &meta)?;
                                        after_update_hook = Some(meta.path.span());
                                    }
                                    delete => {
                                        check_duplicate(&after_delete_hook, &meta)?;
                                        after_delete_hook = Some(meta.path.span());
                                    }
                                    soft_delete => {
                                        check_duplicate(&after_soft_delete_hook, &meta)?;
                                        after_soft_delete_hook = Some(meta.path.span());
                                    }
                                });
                                Ok(())
                            })?;
                        }
                    });
                    Ok(())
                })?;
            }
            method => {
                check_duplicate(&methods, &meta)?;
                methods = Some(());

                meta.parse_nested_meta(|meta| {
                    match_meta!(match meta {
                        update => {
                            check_duplicate(&update_method, &meta)?;
                            update_method = Some(meta.value()?.parse::<syn::LitBool>()?.value);
                        }
                        delete => {
                            check_duplicate(&delete_method, &meta)?;
                            delete_method = Some(meta.value()?.parse::<syn::LitBool>()?.value);
                        }
                        soft_delete => {
                            check_duplicate(&soft_delete_method, &meta)?;
                            let path = meta.path.clone();
                            let value = meta.value()?.parse::<syn::LitBool>()?;
                            soft_delete_method = Some(SoftDeleteMethodArgument { path, value });
                        }
                    });
                    Ok(())
                })?;
            }
        });
        Ok(())
    })
    .parse2(args.clone())?;

    if let Some(soft_delete_method) = &soft_delete_method
        && delete_method.is_none()
    {
        return Err(error::soft_delete_method_without_delete_method(
            &soft_delete_method.path,
        ));
    }

    if !update_method.unwrap_or(true) {
        if let Some(span) = before_update_hook {
            return Err(error::before_update_hook_without_update_method(span));
        }
        if let Some(span) = after_update_hook {
            return Err(error::after_update_hook_without_update_method(span));
        }
    }

    if !delete_method.unwrap_or(true) {
        if let Some(span) = before_delete_hook {
            return Err(error::before_delete_hook_without_delete_method(span));
        }

        if let Some(span) = after_delete_hook {
            return Err(error::after_delete_hook_without_delete_method(span));
        }
    }

    if !soft_delete_method
        .as_ref()
        .is_some_and(SoftDeleteMethodArgument::is_enabled)
    {
        if let Some(span) = before_soft_delete_hook {
            return Err(error::before_soft_delete_hook_on_table_not_soft_deletable(
                span,
            ));
        }

        if let Some(span) = after_soft_delete_hook {
            return Err(error::after_soft_delete_hook_on_table_not_soft_deletable(
                span,
            ));
        }
    }

    let is_singleton = is_singleton.is_some();

    if let Some(span) = singleton_with_default
        && update_method == Some(false)
    {
        return Err(error::update_method_disabled_on_singleton_with_default(
            span,
        ));
    }

    if is_singleton && let Some(name_plural) = &name_plural {
        return Err(error::plural_name_on_singleton(name_plural));
    }

    // For singletons, plural_name will be set later from the table accessor.
    // Use a placeholder for now.
    let parsed_plural_name = if is_singleton {
        syn::Ident::new("__singleton_placeholder", proc_macro2::Span::call_site())
    } else {
        name_plural.ok_or_else(|| error::missing_plural_name(args))?
    };

    // `singleton` itself is the attribute symbol in this scope, so the parsed kind needs a
    // name of its own.
    let singleton_kind = is_singleton.then_some(match singleton_with_default {
        None => SingletonKind::WithoutDefault,
        Some(_) => SingletonKind::WithDefault,
    });

    Ok(DSLData {
        singleton: singleton_kind,
        plural_name: parsed_plural_name,
        unique_indices,
        before_insert_hook: before_insert_hook.is_some(),
        before_update_hook: before_update_hook.is_some(),
        before_delete_hook: before_delete_hook.is_some(),
        before_soft_delete_hook: before_soft_delete_hook.is_some(),
        after_insert_hook: after_insert_hook.is_some(),
        after_update_hook: after_update_hook.is_some(),
        after_delete_hook: after_delete_hook.is_some(),
        after_soft_delete_hook: after_soft_delete_hook.is_some(),
        update_method,
        delete_method,
        soft_delete_method,
    })
}

struct DSLData {
    singleton: Option<SingletonKind>,
    plural_name: Ident,
    unique_indices: Vec<Ident>,
    before_insert_hook: bool,
    before_update_hook: bool,
    before_delete_hook: bool,
    before_soft_delete_hook: bool,
    after_insert_hook: bool,
    after_update_hook: bool,
    after_delete_hook: bool,
    after_soft_delete_hook: bool,
    update_method: Option<bool>,
    delete_method: Option<bool>,
    soft_delete_method: Option<SoftDeleteMethodArgument>,
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

    let name = name.ok_or_else(|| error::missing_unique_index_name(&meta))?;

    Ok(name)
}
