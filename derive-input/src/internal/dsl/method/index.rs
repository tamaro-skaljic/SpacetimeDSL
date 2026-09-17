//! What an index tells the generators that look rows up through it.
//!
//! [`IndexShape`] is the per-index analysis: its name, its columns, whether it is the
//! primary key, and the prose fragments every generated doc comment ends with.
//! [`index_column_arguments`] is the per-column analysis: the method argument, the row-value
//! getter and the wrapper unwrapping for each column, built from one walk so the n-th of
//! each belongs to the same column.

use super::context::MethodGenerationContext;
use crate::{
    api::{
        db::index::{Index, IndexType},
        dsl::{
            method::{SpacetimeDSLArg, SpacetimeDSLArgType},
            wrapper::WrapperType,
        },
    },
    internal::{column::ColumnTypeKind, dsl::one_or_multiple::OneOrMultiple},
};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::VecDeque;
use syn::{Ident, parse_str};

/// `self.db().<table>().<index>()`, where every index-based body starts.
pub(in crate::internal) fn index_accessor(
    singular_table_name: &Ident,
    index_name: &Ident,
) -> TokenStream {
    quote! {
        self
            .db()
            .#singular_table_name()
            .#index_name()
    }
}

/// Everything the five index-based generators derive from the index they are given.
///
/// This used to sit inside `for_method`, which is why no generator could be lifted out of
/// it. The four prose fragments the doc comments were built from are assembled here into
/// the one phrase all five of them built identically.
pub(in crate::internal) struct IndexShape {
    pub index_name: Ident,
    pub index_columns: Vec<Ident>,
    pub is_multi_column: bool,
    /// Whether this is the table's primary key index.
    ///
    /// SpacetimeDB defines `update` on the primary key index alone, so no other index can
    /// carry an update method.
    pub is_primary_key: bool,
    /// Whether this is the injected primary key of a singleton table.
    ///
    /// Only `for_update` reads it. Getting and deleting a singleton's row are their own
    /// generators, because those bodies have nothing in common with the index-based ones;
    /// updating it is the ordinary update with one statement added, so splitting it would
    /// copy the whole body to change three lines.
    pub is_singleton_primary_key: bool,
    /// `{{ a : {}, b : {} }}`, with one placeholder per index column.
    pub column_names_and_row_values: String,
    /// "whose value matches the value from the unique single-column btree index on the
    /// `x` column", the tail every generated doc comment ends with.
    pub described_as: String,
    /// The experimental-feature warning a unique multi-column index carries, or empty.
    pub unique_multi_column_hint: &'static str,
}

impl IndexShape {
    pub(in crate::internal) fn of(index: &Index, context: &MethodGenerationContext) -> IndexShape {
        let index_name = index.name.clone();

        let (index_columns, is_multi_column, value_matches, single_or_multi, on_the_columns) =
            match &index.index_type {
                IndexType::BTreeSingleColumn { column }
                | IndexType::HashSingleColumn { column } => (
                    vec![column.clone()],
                    false,
                    "value matches the value from",
                    "single",
                    format!("`{column}` column"),
                ),
                IndexType::BTreeMultiColumn { columns }
                | IndexType::HashMultiColumn { columns } => (
                    columns.clone(),
                    true,
                    "values match the values from",
                    "multi",
                    documentation_on_columns(columns),
                ),
                IndexType::Direct { column } => (
                    vec![column.clone()],
                    false,
                    "value matches",
                    "single",
                    format!("`{column}` column"),
                ),
            };

        let index_documentation = match is_multi_column {
            false => format!("{} index", index_kind(&index.index_type)),
            true => format!("{} index `{index_name}`", index_kind(&index.index_type)),
        };

        // Only a unique index reaches the one-row generators, and only they say "unique".
        let unique = match index.is_unique {
            false => "",
            true => "unique ",
        };

        let is_primary_key = !is_multi_column
            && index_columns
                .first()
                .is_some_and(|c| context.primary_key_column.rust_field_name == *c);

        IndexShape {
            is_primary_key,
            is_singleton_primary_key: context.spacetimedsl_table.is_singleton && is_primary_key,
            column_names_and_row_values: column_names_and_row_values(&index_columns),
            described_as: format!(
                "whose {value_matches} the {unique}{single_or_multi}-column {index_documentation} on the {on_the_columns}"
            ),
            unique_multi_column_hint: match index.is_unique && is_multi_column {
                false => "",
                true => {
                    "Warning: The unique multi-column index feature of SpacetimeDSL is experimental.\n- It will be removed if unique multi-column indices are implemented in SpacetimeDB.\n- SpacetimeDSL is only able to enforce referential integrity if you never use the (mutating) `insert`, `update` and `delete` methods of `spacetimedb::ReducerContext` yourself."
                }
            },
            index_name,
            index_columns,
            is_multi_column,
        }
    }
}

/// What an index's columns contribute to the method that looks rows up by them.
///
/// The three lists are built from one walk over the index's columns, in index order, so
/// the n-th argument, the n-th row value and the n-th mapper all belong to the same
/// column.
pub(in crate::internal) struct IndexColumnArguments {
    pub method_args: Vec<SpacetimeDSLArg>,
    /// How the body reads each argument back out when it renders a not-found message.
    pub row_value_getters: Vec<TokenStream>,
    /// The `let` that unwraps an optional wrapper argument, or empty for a column that
    /// needs no unwrapping.
    pub wrapper_option_mappers: Vec<TokenStream>,
}

/// The arguments the four index-based lookup generators take.
///
/// `for_get_many`, `for_delete_many`, `for_get_one` and `for_delete_one` differ here in
/// one thing: whether the method returns many rows or one. A many-row method borrows its
/// arguments for the iterator that outlives the call, while a one-row method also renders
/// its arguments into a not-found message, so a wrapped argument has to be cloned before
/// it is consumed and a single-column string has to be owned.
pub(in crate::internal) fn index_column_arguments(
    shape: &IndexShape,
    one_or_multiple: &OneOrMultiple,
    context: &MethodGenerationContext,
) -> IndexColumnArguments {
    let mut arguments = IndexColumnArguments {
        method_args: vec![],
        row_value_getters: vec![],
        wrapper_option_mappers: vec![],
    };

    for column_name in &shape.index_columns {
        let column = context
            .internal_columns
            .iter()
            .find(|c| c.rust_field_name == *column_name)
            .expect("An index column is always one of the table's columns");

        let column_is_string = column.rust_field_type_kind == ColumnTypeKind::String;

        let wrapper_option_mapper;
        let method_arg;
        let row_value_getter;

        match &column.spacetimedsl_column_wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_ty = &WrapperType::map(wrapper_type);

                if column_is_string {
                    wrapper_option_mapper = TokenStream::default();

                    method_arg = SpacetimeDSLArg {
                        is_option: false,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Normal(quote! { &str }),
                    };

                    row_value_getter = match one_or_multiple {
                        OneOrMultiple::Multiple => quote! { #column_name },
                        // A multi-column message renders the whole tuple with `{:?}`,
                        // which a `&str` already satisfies.
                        OneOrMultiple::One => match shape.is_multi_column {
                            true => quote! { #column_name },
                            false => quote! { #column_name.to_string() },
                        },
                    };
                } else if column.spacetimedsl_column_is_option {
                    wrapper_option_mapper = match wrapper_type {
                        // A created wrapper wraps the whole Option, so
                        // value() already yields it.
                        WrapperType::Created(_) => quote! {
                            let #column_name = match #column_name.into() {
                                None => None,
                                Some(#column_name) => Into::<#wrapper_type_ty>::into(#column_name).value(),
                            };
                        },
                        // A used wrapper wraps the inner type, so the
                        // Option has to be rebuilt around value().
                        WrapperType::Used(_) => quote! {
                            let #column_name = match #column_name.into() {
                                None => None,
                                Some(#column_name) => Some(Into::<#wrapper_type_ty>::into(#column_name).value()),
                            };
                        },
                    };

                    method_arg = SpacetimeDSLArg {
                        is_option: true,
                        arg_name: column_name.clone(),
                        arg_type: SpacetimeDSLArgType::Wrapped {
                            wrapped_type: WrapperType::map_to_wrapped_type(wrapper_type)
                                .to_token_stream(),
                            actual_type: quote! { &impl Into<Option<#wrapper_type_ty>> },
                        },
                    };

                    row_value_getter = quote! { #column_name };
                } else {
                    wrapper_option_mapper = TokenStream::default();

                    let wrapped_type =
                        WrapperType::map_to_wrapped_type(wrapper_type).to_token_stream();

                    match one_or_multiple {
                        OneOrMultiple::Multiple => {
                            method_arg = SpacetimeDSLArg {
                                is_option: false,
                                arg_name: column_name.clone(),
                                arg_type: SpacetimeDSLArgType::Wrapped {
                                    wrapped_type,
                                    actual_type: quote! { impl Into<#wrapper_type_ty> },
                                },
                            };

                            row_value_getter = quote! { #column_name.into().value() };
                        }
                        // `into()` consumes the argument, and the not-found message needs
                        // it afterwards.
                        OneOrMultiple::One => {
                            method_arg = SpacetimeDSLArg {
                                is_option: false,
                                arg_name: column_name.clone(),
                                arg_type: SpacetimeDSLArgType::Wrapped {
                                    wrapped_type,
                                    actual_type: quote! { impl Into<#wrapper_type_ty> + Clone },
                                },
                            };

                            row_value_getter = quote! { #column_name.clone().into().value() };
                        }
                    }
                }
            }
            None => {
                wrapper_option_mapper = TokenStream::default();

                // TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
                let column_type = if column_is_string {
                    parse_str("str").expect("`str` is a valid type path")
                } else {
                    column.rust_field_type_name_or_path.clone()
                };

                match one_or_multiple {
                    OneOrMultiple::Multiple => {
                        method_arg = SpacetimeDSLArg {
                            is_option: column.spacetimedsl_column_is_option,
                            arg_name: column_name.clone(),
                            arg_type: SpacetimeDSLArgType::Normal(quote! { &'a #column_type }),
                        };

                        row_value_getter = quote! { #column_name };
                    }
                    OneOrMultiple::One => {
                        method_arg = SpacetimeDSLArg {
                            is_option: column.spacetimedsl_column_is_option,
                            arg_name: column_name.clone(),
                            arg_type: SpacetimeDSLArgType::Normal(quote! { &#column_type }),
                        };

                        row_value_getter = match column_is_string && !shape.is_multi_column {
                            true => quote! { #column_name.to_string() },
                            false => quote! { #column_name },
                        };
                    }
                }
            }
        }

        arguments.wrapper_option_mappers.push(wrapper_option_mapper);
        arguments.method_args.push(method_arg);
        arguments.row_value_getters.push(row_value_getter);
    }

    arguments
}

/// "`a`, `b` and `c`", as the doc comment of a multi-column index names its columns.
fn documentation_on_columns(columns: &[Ident]) -> String {
    let mut columns: VecDeque<&Ident> = columns.iter().collect();

    let first_column = columns.pop_front().expect(
        "A multi-column index is only built from two or more columns, so it has a first one",
    );
    let last_column = columns.pop_back().expect(
        "A multi-column index is only built from two or more columns, so it has a last one",
    );

    let mut documentation = format!("columns `{first_column}`");

    for any_other_column in columns {
        documentation.push_str(&format!(", `{any_other_column}`"));
    }

    documentation.push_str(&format!(" and `{last_column}`"));

    documentation
}

/// The format string behind every "these columns had these values" message:
/// `{{ id : {} }}` for one column, `{{ a : {}, b : {} }}` for several.
///
/// One placeholder per column, in the order given, so a caller that builds its row-value
/// getters from the same list cannot get the two out of step.
pub(in crate::internal) fn column_names_and_row_values(column_names: &[Ident]) -> String {
    let placeholders = column_names
        .iter()
        .map(|column_name| format!("{column_name} : {{}}"))
        .collect_vec()
        .join(", ");

    format!("{{{{ {placeholders} }}}}")
}

/// The kind of index, as the doc comments of the generated methods name it.
fn index_kind(index_type: &IndexType) -> &'static str {
    match index_type {
        IndexType::BTreeSingleColumn { .. } | IndexType::BTreeMultiColumn { .. } => "btree",
        IndexType::HashSingleColumn { .. } | IndexType::HashMultiColumn { .. } => "hash",
        IndexType::Direct { .. } => "direct",
    }
}
