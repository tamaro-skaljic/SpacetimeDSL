//! How `#[spacetimedsl::dsl]` analyses a struct and generates its DSL methods.
//!
//! The module is private, so its items are `pub` without leaving the crate. A method of an
//! `api` type is the exception: it is callable wherever its type is, whichever module holds
//! the `impl` block, so a method only this crate calls stays `pub(crate)`.

use {
    crate::{
        api::dsl::{
            hook::{HookKind, Operation, Timing},
            table::{SingletonKind, SpacetimeDSLTableKind},
        },
        internal::dsl::{
            after, before, delete, hook, insert, method, plural_name, singleton,
            soft_delete::SoftDeleteMethodArgument, unique_index, update, with_default,
        },
    },
    proc_macro2::Span,
    spacetime_bindings_macro_input::{match_meta, sym, table::TableArgs, util::check_duplicate},
    std::collections::{BTreeMap, BTreeSet},
    syn::{
        Ident,
        meta::{ParseNestedMeta, parser},
        parse::Parser,
        spanned::Spanned,
    },
};

pub mod attribute;

mod integration;

mod table;

mod column;

mod rust;

mod db;

pub mod dsl;

mod error;

mod spacetimedb;

pub fn try_parse(
    args: proc_macro2::TokenStream,
    input: &syn::DeriveInput,
) -> syn::Result<crate::api::Table> {
    let dsl_data = try_parse_dsl(&args)?;

    let (table_args, column_args) = integration::select_table_attribute(
        input,
        dsl_data.table_selector.as_ref(),
        dsl_data.kind.singleton().is_some(),
    )?;

    if let SpacetimeDSLTableKind::Singleton(_) = dsl_data.kind {
        reject_unique_index_on_singleton(&dsl_data.unique_indices, &table_args)?;
    }

    reject_unknown_or_repeated_unique_indices(&dsl_data.unique_indices, &table_args)?;

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

/// `unique_index(name = …)` makes an index declared in `#[table]` unique, so it has to
/// name one of those, and each only once.
fn reject_unknown_or_repeated_unique_indices(
    unique_indices: &[Ident],
    table_args: &TableArgs,
) -> syn::Result<()> {
    let declared_indices: Vec<&Ident> = table_args
        .indices
        .iter()
        .map(|index| &index.accessor)
        .collect();

    for (position, unique_index_name) in unique_indices.iter().enumerate() {
        if !declared_indices.contains(&unique_index_name) {
            return Err(error::unknown_unique_index(
                unique_index_name,
                &declared_indices,
            ));
        }

        if unique_indices[..position].contains(unique_index_name) {
            return Err(error::repeated_unique_index(unique_index_name));
        }
    }

    Ok(())
}

fn try_parse_dsl(args: &proc_macro2::TokenStream) -> syn::Result<DSLData> {
    validate(parse_dsl_arguments(args)?, args)
}

/// Every `#[dsl(...)]` argument as written, with the spans the diagnostics point at.
struct ParsedDSLArguments {
    is_singleton: bool,
    singleton_with_default: Option<Span>,
    plural_name: Option<Ident>,
    table_selector: Option<Ident>,
    unique_indices: Vec<Ident>,
    declared_hook_spans: BTreeMap<HookKind, Span>,
    update_method: Option<bool>,
    delete_method: Option<bool>,
    soft_delete_method: Option<SoftDeleteMethodArgument>,
}

/// Parses every `#[dsl(...)]` argument. It rejects only what cannot be parsed: an unknown
/// or repeated argument, or a value of the wrong kind. How the arguments combine is left
/// to [`validate`].
fn parse_dsl_arguments(args: &proc_macro2::TokenStream) -> syn::Result<ParsedDSLArguments> {
    let mut name_plural: Option<Ident> = None;
    let mut table_selector: Option<Ident> = None;
    let mut is_singleton: Option<()> = None;
    let mut singleton_with_default: Option<Span> = None;

    let mut unique_indices = vec![];

    let mut hooks = None;
    let mut before_hooks = None;
    let mut after_hooks = None;
    let mut declared_hook_spans = BTreeMap::new();

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
            dsl::table => {
                check_duplicate(&table_selector, &meta)?;
                table_selector = Some(meta.value()?.parse()?);
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
                            parse_hooks_of_timing(meta, Timing::Before, &mut declared_hook_spans)?;
                        }
                        after => {
                            check_duplicate(&after_hooks, &meta)?;
                            after_hooks = Some(());
                            parse_hooks_of_timing(meta, Timing::After, &mut declared_hook_spans)?;
                        }
                    });
                    Ok(())
                })?;
            }
            method => {
                check_duplicate(&methods, &meta)?;
                methods = Some(());

                use dsl::soft_delete;

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

    Ok(ParsedDSLArguments {
        is_singleton: is_singleton.is_some(),
        singleton_with_default,
        plural_name: name_plural,
        table_selector,
        unique_indices,
        declared_hook_spans,
        update_method,
        delete_method,
        soft_delete_method,
    })
}

/// Parses the operations of `before(...)` or `after(...)` into `declared_hook_spans`.
fn parse_hooks_of_timing(
    meta: ParseNestedMeta<'_>,
    timing: Timing,
    declared_hook_spans: &mut BTreeMap<HookKind, Span>,
) -> syn::Result<()> {
    meta.parse_nested_meta(|meta| {
        use dsl::soft_delete;
        let operation = match_meta!(match meta {
            insert => Operation::Insert,
            update => Operation::Update,
            delete => Operation::Delete,
            soft_delete => Operation::SoftDelete,
        });
        let kind = HookKind { timing, operation };

        check_duplicate(&declared_hook_spans.get(&kind), &meta)?;
        declared_hook_spans.insert(kind, meta.path.span());

        Ok(())
    })
}

/// Checks how the parsed arguments combine. The checks run in a fixed order, which decides
/// the diagnostic an input breaking several rules gets.
fn validate(parsed: ParsedDSLArguments, args: &proc_macro2::TokenStream) -> syn::Result<DSLData> {
    let ParsedDSLArguments {
        is_singleton,
        singleton_with_default,
        plural_name: name_plural,
        table_selector,
        unique_indices,
        declared_hook_spans,
        update_method,
        delete_method,
        soft_delete_method,
    } = parsed;

    let declared_hook_span = |kind| declared_hook_spans.get(&kind).copied();

    if let Some(soft_delete_method) = &soft_delete_method
        && delete_method.is_none()
    {
        return Err(error::soft_delete_method_without_delete_method(
            &soft_delete_method.path,
        ));
    }

    if !update_method.unwrap_or(true) {
        if let Some(span) = declared_hook_span(HookKind::BEFORE_UPDATE) {
            return Err(error::before_update_hook_without_update_method(span));
        }
        if let Some(span) = declared_hook_span(HookKind::AFTER_UPDATE) {
            return Err(error::after_update_hook_without_update_method(span));
        }
    }

    if !delete_method.unwrap_or(true) {
        if let Some(span) = declared_hook_span(HookKind::BEFORE_DELETE) {
            return Err(error::before_delete_hook_without_delete_method(span));
        }

        if let Some(span) = declared_hook_span(HookKind::AFTER_DELETE) {
            return Err(error::after_delete_hook_without_delete_method(span));
        }
    }

    if !soft_delete_method
        .as_ref()
        .is_some_and(SoftDeleteMethodArgument::is_enabled)
    {
        if let Some(span) = declared_hook_span(HookKind::BEFORE_SOFT_DELETE) {
            return Err(error::before_soft_delete_hook_on_table_not_soft_deletable(
                span,
            ));
        }

        if let Some(span) = declared_hook_span(HookKind::AFTER_SOFT_DELETE) {
            return Err(error::after_soft_delete_hook_on_table_not_soft_deletable(
                span,
            ));
        }
    }

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

    let kind = if is_singleton {
        SpacetimeDSLTableKind::Singleton(match singleton_with_default {
            None => SingletonKind::WithoutDefault,
            Some(_) => SingletonKind::WithDefault,
        })
    } else {
        SpacetimeDSLTableKind::Normal {
            plural_name: name_plural.ok_or_else(|| error::missing_plural_name(args))?,
        }
    };

    Ok(DSLData {
        kind,
        table_selector,
        unique_indices,
        declared_hooks: declared_hook_spans.into_keys().collect(),
        update_method,
        delete_method,
        soft_delete_method,
    })
}

pub struct DSLData {
    kind: SpacetimeDSLTableKind,
    /// `table = <accessor>`: the `#[table]` attribute this `#[dsl]` belongs to.
    table_selector: Option<Ident>,
    unique_indices: Vec<Ident>,
    declared_hooks: BTreeSet<HookKind>,
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
