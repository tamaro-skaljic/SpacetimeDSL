use crate::internal::dsl::{
    after, before, delete, hook, insert, method, plural_name, singleton, unique_index, update,
};
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
    // Parse DSL attribute arguments
    let mut dsl_data = try_parse_dsl(&args)?;

    // Pass plural_name to integration for intelligent table selection
    let (table_args, column_args) = integration::spacetime_bindings_macro_input(
        input,
        &dsl_data.plural_name,
        dsl_data.is_singleton,
    )?;

    // For singletons, set plural_name to the singular name from the table accessor
    // (it's only used for get_all/count_of_all which won't be generated)
    if dsl_data.is_singleton {
        dsl_data.plural_name = crate::internal::table::rm_rsharp(table_args.accessor.clone());
    }

    // Pass the parsed plural_name to avoid re-parsing
    table::try_parse(input, dsl_data, &table_args, &column_args)
}

// Parse plural_name from DSL arguments
fn try_parse_dsl(args: &proc_macro2::TokenStream) -> syn::Result<DSLData> {
    let mut name_plural: Option<Ident> = None;
    let mut is_singleton: Option<()> = None;

    let mut unique_indices = vec![];

    let mut hooks = None;
    let mut before_hooks = None;
    let mut after_hooks = None;

    let mut before_insert_hook = None;
    let mut before_update_hook = None;
    let mut before_delete_hook = None;
    let mut after_insert_hook = None;
    let mut after_update_hook = None;
    let mut after_delete_hook = None;

    let mut methods = None;
    let mut update_method = None;
    let mut delete_method = None;

    parser(|meta| {
        match_meta!(match meta {
            singleton => {
                check_duplicate(&is_singleton, &meta)?;
                is_singleton = Some(());
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
                                        before_insert_hook = Some(());
                                    }
                                    update => {
                                        check_duplicate(&before_update_hook, &meta)?;
                                        before_update_hook = Some(());
                                    }
                                    delete => {
                                        check_duplicate(&before_delete_hook, &meta)?;
                                        before_delete_hook = Some(());
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
                                        after_insert_hook = Some(());
                                    }
                                    update => {
                                        check_duplicate(&after_update_hook, &meta)?;
                                        after_update_hook = Some(());
                                    }
                                    delete => {
                                        check_duplicate(&after_delete_hook, &meta)?;
                                        after_delete_hook = Some(());
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
                    });
                    Ok(())
                })?;
            }
        });
        Ok(())
    })
    .parse2(args.clone())?;

    if !update_method.unwrap_or(true) {
        if before_update_hook.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Cannot have a `before_update` hook when the `update` method is disabled in `#[dsl(method(update = false))]`",
            ));
        }
        if after_update_hook.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Cannot have an `after_update` hook when the `update` method is disabled in `#[dsl(method(update = false))]`",
            ));
        }
    }

    if !delete_method.unwrap_or(true) {
        if before_delete_hook.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Cannot have a `before_delete` hook when the `delete` method is disabled in `#[dsl(method(delete = false))]`",
            ));
        }

        if after_delete_hook.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Cannot have an `after_delete` hook when the `delete` method is disabled in `#[dsl(method(delete = false))]`",
            ));
        }
    }

    let is_singleton = is_singleton.is_some();

    if is_singleton {
        if name_plural.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`plural_name` is not allowed on singleton tables! Use `#[dsl(singleton)]` without `plural_name`.",
            ));
        }

        if !unique_indices.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
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
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "PluralName must be set in `#[dsl(plural_name = PluralName)]`",
            )
        })?
    };

    Ok(DSLData {
        is_singleton,
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
    })
}

struct DSLData {
    is_singleton: bool,
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
