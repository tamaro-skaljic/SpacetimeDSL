use crate::api::dsl::table::SingletonKind;
use crate::internal::dsl::{
    after, before, delete, hook, insert, method, plural_name, singleton, soft_delete, unique_index,
    update, with_default,
};
use proc_macro2::Span;
use spacetime_bindings_macro_input::{match_meta, sym, util::check_duplicate};
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

    // For singletons, set plural_name to the singular name from the table accessor
    // (it's only used for get_all/count_of_all which won't be generated)
    if dsl_data.singleton.is_some() {
        dsl_data.plural_name = crate::internal::table::rm_rsharp(table_args.accessor.clone());
    }

    // Pass the parsed plural_name to avoid re-parsing
    table::try_parse(input, dsl_data, &table_args, &column_args)
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
    let mut after_insert_hook: Option<Span> = None;
    let mut after_update_hook: Option<Span> = None;
    let mut after_delete_hook: Option<Span> = None;

    let mut methods = None;
    let mut update_method = None;
    let mut delete_method = None;
    let mut soft_delete_method: Option<bool> = None;
    let mut soft_delete_method_span: Option<Span> = None;

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
                            soft_delete_method_span = Some(meta.path.span());
                            soft_delete_method = Some(meta.value()?.parse::<syn::LitBool>()?.value);
                        }
                    });
                    Ok(())
                })?;
            }
        });
        Ok(())
    })
    .parse2(args.clone())?;

    if let Some(span) = soft_delete_method_span
        && delete_method.is_none()
    {
        return Err(syn::Error::new(
            span,
            "`#[dsl(method(soft_delete = ...))]` requires `#[dsl(method(delete = ...))]` to be set as well, e.g. `method(delete = true, soft_delete = true)`.\nSoft deletion retires a row instead of removing it, which only says something next to a decision about whether the table removes rows at all.",
        ));
    }

    if !update_method.unwrap_or(true) {
        if let Some(span) = before_update_hook {
            return Err(syn::Error::new(
                span,
                "Cannot have a `before_update` hook when the `update` method is disabled with `#[dsl(method(update = false))]`",
            ));
        }
        if let Some(span) = after_update_hook {
            return Err(syn::Error::new(
                span,
                "Cannot have an `after_update` hook when the `update` method is disabled with `#[dsl(method(update = false))]`",
            ));
        }
    }

    if !delete_method.unwrap_or(true) {
        if let Some(span) = before_delete_hook {
            return Err(syn::Error::new(
                span,
                "Cannot have a `before_delete` hook when the `delete` method is disabled with `#[dsl(method(delete = false))]`",
            ));
        }

        if let Some(span) = after_delete_hook {
            return Err(syn::Error::new(
                span,
                "Cannot have an `after_delete` hook when the `delete` method is disabled with `#[dsl(method(delete = false))]`",
            ));
        }
    }

    let is_singleton = is_singleton.is_some();

    if let Some(span) = singleton_with_default
        && update_method == Some(false)
    {
        return Err(syn::Error::new(
            span,
            "Cannot disable the `update` method with `#[dsl(method(update = false))]` on a table with `#[dsl(singleton(with_default))]`!\n`upsert_<table>` is the only method which writes the row of such a table, so the table could never hold one.",
        ));
    }

    if is_singleton {
        if let Some(name_plural) = &name_plural {
            return Err(syn::Error::new_spanned(
                name_plural,
                "`plural_name` is not allowed on singleton tables! Use `#[dsl(singleton)]` without `plural_name`.",
            ));
        }

        if let Some(first_unique_index_name) = unique_indices.first() {
            return Err(syn::Error::new_spanned(
                first_unique_index_name,
                "`unique_index` is not allowed on singleton tables!",
            ));
        }
    }

    // For singletons, plural_name will be set later from the table accessor.
    // Use a placeholder for now.
    let parsed_plural_name = if is_singleton {
        syn::Ident::new("__singleton_placeholder", proc_macro2::Span::call_site())
    } else {
        name_plural.ok_or_else(|| {
            syn::Error::new_spanned(
                args,
                "PluralName must be set in `#[dsl(plural_name = PluralName)]`",
            )
        })?
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
        after_insert_hook: after_insert_hook.is_some(),
        after_update_hook: after_update_hook.is_some(),
        after_delete_hook: after_delete_hook.is_some(),
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
    after_insert_hook: bool,
    after_update_hook: bool,
    after_delete_hook: bool,
    update_method: Option<bool>,
    delete_method: Option<bool>,
    soft_delete_method: Option<bool>,
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
