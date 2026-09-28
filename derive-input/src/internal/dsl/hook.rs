use std::collections::BTreeSet;

use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::api::{
    dsl::{
        hook::{HookKind, Operation, SpacetimeDSLMethodHook, SpacetimeDSLMethodHooks, Timing},
        method::{SpacetimeDSLArg, SpacetimeDSLArgType},
        table::SingletonKind,
    },
    runtime,
};

/// What the `before_insert` hook of a table receives.
///
/// Every table hands it the `Create<Table>` request, except a
/// `SingletonKind::WithDefault` table: that one has no create method and therefore no
/// `Create<Table>` struct, so its row reaches the hook whole, the way `upsert_<table>`
/// holds it.
#[derive(Clone, Copy, PartialEq)]
enum InsertedValue {
    CreateRequest,
    WholeRow,
}

impl InsertedValue {
    fn of(singleton: Option<SingletonKind>) -> InsertedValue {
        match singleton {
            Some(SingletonKind::WithDefault) => InsertedValue::WholeRow,
            _ => InsertedValue::CreateRequest,
        }
    }
}

pub fn build(
    singular_table_name: &syn::Ident,
    singleton: Option<SingletonKind>,
    declared: &BTreeSet<HookKind>,
) -> SpacetimeDSLMethodHooks {
    let inserted_value = InsertedValue::of(singleton);

    SpacetimeDSLMethodHooks {
        declared: declared
            .iter()
            .map(|&kind| (kind, build_one(kind, singular_table_name, inserted_value)))
            .collect(),
    }
}

fn build_one(
    HookKind { timing, operation }: HookKind,
    singular_table_name: &syn::Ident,
    inserted_value: InsertedValue,
) -> SpacetimeDSLMethodHook {
    let singular_table_name_pascal_case = format_ident!(
        "{}",
        RenameRule::PascalCase.apply_to_field(singular_table_name.to_string())
    );

    SpacetimeDSLMethodHook {
        trait_name: get_trait_name(&timing, &singular_table_name_pascal_case, &operation),
        function_name: get_function_name(&timing, singular_table_name, &operation),
        function_args: get_function_args(
            &timing,
            singular_table_name,
            &singular_table_name_pascal_case,
            &operation,
            inserted_value,
        ),
        return_type: get_return_type(
            &timing,
            &operation,
            &singular_table_name_pascal_case,
            inserted_value,
        ),
    }
}

fn get_trait_name(
    timing: &Timing,
    singular_table_name_pascal_case: &syn::Ident,
    operation: &Operation,
) -> syn::Ident {
    let timing = match timing {
        Timing::Before => "Before",
        Timing::After => "After",
    };

    let operation = match operation {
        Operation::Insert => "Insert",
        Operation::Update => "Update",
        Operation::Delete => "Delete",
        Operation::SoftDelete => "SoftDelete",
    };

    format_ident!(
        "{}{}{}Hook",
        timing,
        singular_table_name_pascal_case,
        operation,
    )
}

fn get_function_name(
    timing: &Timing,
    singular_table_name: &syn::Ident,
    operation: &Operation,
) -> syn::Ident {
    let timing = match timing {
        Timing::Before => "before",
        Timing::After => "after",
    };

    let operation = match operation {
        Operation::Insert => "insert",
        Operation::Update => "update",
        Operation::Delete => "delete",
        Operation::SoftDelete => "soft_delete",
    };

    format_ident!("{}_{}_{}", timing, singular_table_name, operation)
}

fn get_function_args(
    timing: &Timing,
    singular_table_name: &syn::Ident,
    singular_table_name_pascal_case: &syn::Ident,
    operation: &Operation,
    inserted_value: InsertedValue,
) -> Vec<SpacetimeDSLArg> {
    match (timing, operation) {
        (Timing::Before, Operation::Insert) => match inserted_value {
            InsertedValue::CreateRequest => {
                // FIXME: Single Source of Truth Violation for arg type name
                let arg_type = format_ident!("Create{singular_table_name_pascal_case}");

                vec![
                    build_dsl_function_arg(),
                    build_function_arg(
                        format_ident!("create_{singular_table_name}_request"),
                        quote! { #arg_type },
                    ),
                ]
            }
            InsertedValue::WholeRow => vec![
                build_dsl_function_arg(),
                build_function_arg(
                    format_ident!("new_{singular_table_name}"),
                    quote! { #singular_table_name_pascal_case },
                ),
            ],
        },
        (Timing::After, Operation::Insert) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("new_{singular_table_name}"),
                quote! { &#singular_table_name_pascal_case },
            ),
        ],
        // A soft deletion writes the row rather than removing it, so both its hooks take
        // the shape of the update hooks.
        (Timing::Before, Operation::Update | Operation::SoftDelete) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("old_{singular_table_name}"),
                quote! { &#singular_table_name_pascal_case },
            ),
            build_function_arg(
                format_ident!("new_{singular_table_name}"),
                quote! { #singular_table_name_pascal_case },
            ),
        ],
        (Timing::After, Operation::Update | Operation::SoftDelete) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("old_{singular_table_name}"),
                quote! { &#singular_table_name_pascal_case },
            ),
            build_function_arg(
                format_ident!("new_{singular_table_name}"),
                quote! { &#singular_table_name_pascal_case },
            ),
        ],
        (_, Operation::Delete) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("old_{singular_table_name}"),
                quote! { &#singular_table_name_pascal_case },
            ),
        ],
    }
}

fn get_return_type(
    timing: &Timing,
    operation: &Operation,
    singular_table_name_pascal_case: &syn::Ident,
    inserted_value: InsertedValue,
) -> TokenStream {
    let error_type = runtime::spacetimedsl_error_type();

    match (timing, operation) {
        (Timing::Before, Operation::Insert) => {
            let returned_type = match inserted_value {
                InsertedValue::CreateRequest => {
                    format_ident!("Create{singular_table_name_pascal_case}")
                }
                InsertedValue::WholeRow => singular_table_name_pascal_case.clone(),
            };
            quote! {
                Result<#returned_type, #error_type>
            }
        }
        (Timing::Before, Operation::Update | Operation::SoftDelete) => quote! {
            Result<#singular_table_name_pascal_case, #error_type>
        },
        _ => quote! {
            Result<(), #error_type>
        },
    }
}

fn build_function_arg(name: syn::Ident, ty: TokenStream) -> SpacetimeDSLArg {
    SpacetimeDSLArg {
        is_option: false,
        arg_name: name,
        arg_type: SpacetimeDSLArgType::Normal(ty),
    }
}

fn build_dsl_function_arg() -> SpacetimeDSLArg {
    SpacetimeDSLArg {
        is_option: false,
        arg_name: format_ident!("dsl"),
        arg_type: SpacetimeDSLArgType::Normal(runtime::dsl_reference_type()),
    }
}
