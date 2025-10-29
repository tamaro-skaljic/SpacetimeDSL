use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::api::dsl::{
    hook::{SpacetimeDSLMethodHook, SpacetimeDSLMethodHooks},
    method::{SpacetimeDSLArg, SpacetimeDSLArgType},
};

pub(crate) fn build(
    singular_table_name: &syn::Ident,
    before_insert: bool,
    before_update: bool,
    before_delete: bool,
    after_insert: bool,
    after_update: bool,
    after_delete: bool,
) -> SpacetimeDSLMethodHooks {
    let before_insert = build_any(
        before_insert,
        Timing::Before,
        singular_table_name,
        Operation::Insert,
    );
    let before_update = build_any(
        before_update,
        Timing::Before,
        singular_table_name,
        Operation::Update,
    );
    let before_delete = build_any(
        before_delete,
        Timing::Before,
        singular_table_name,
        Operation::Delete,
    );
    let after_insert = build_any(
        after_insert,
        Timing::After,
        singular_table_name,
        Operation::Insert,
    );
    let after_update = build_any(
        after_update,
        Timing::After,
        singular_table_name,
        Operation::Update,
    );
    let after_delete = build_any(
        after_delete,
        Timing::After,
        singular_table_name,
        Operation::Delete,
    );

    SpacetimeDSLMethodHooks {
        before_insert,
        before_delete,
        before_update,
        after_insert,
        after_update,
        after_delete,
    }
}

fn build_any(
    should_exist: bool,
    timing: Timing,
    singular_table_name: &syn::Ident,
    operation: Operation,
) -> Option<SpacetimeDSLMethodHook> {
    if !should_exist {
        return None;
    }
    
    let singular_table_name_pascal_case =
        RenameRule::PascalCase.apply_to_field(singular_table_name.to_string());

    Some(SpacetimeDSLMethodHook {
        trait_name: get_trait_name(&timing, &singular_table_name_pascal_case, &operation),
        function_name: get_function_name(&timing, singular_table_name, &operation),
        function_args: get_function_args(&timing, singular_table_name, &singular_table_name_pascal_case, &operation),
        return_type: quote! {
            Result<(), spacetimedsl::SpacetimeDSLError>
        },
    })
}

enum Timing {
    Before,
    After,
}

enum Operation {
    Insert,
    Update,
    Delete,
}

fn get_trait_name(
    timing: &Timing,
    singular_table_name_pascal_case: &str,
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
    };

    format_ident!("{}{}{}Hook", timing, singular_table_name_pascal_case, operation,)
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
    };

    format_ident!("{}_{}_{}", timing, singular_table_name, operation)
}

fn get_function_args(
    timing: &Timing,
    singular_table_name: &syn::Ident,
    singular_table_name_pascal_case: &str,
    operation: &Operation,
) -> Vec<SpacetimeDSLArg> {
    let table_type = format_ident!("{singular_table_name_pascal_case}");

    match (timing, operation) {
        (Timing::Before, Operation::Insert) => {
            // FIXME: Single Source of Truth Violation for arg type name
            let arg_type = format_ident!("Create{singular_table_name_pascal_case}");

            vec![
                build_dsl_function_arg(),
                build_function_arg(
                    format_ident!("create_{singular_table_name}_request"),
                    quote! { &#arg_type },
                ),
            ]
        }
        (Timing::After, Operation::Insert) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("new_{singular_table_name}"),
                quote! { &#table_type },
            ),
        ],
        (_, Operation::Update) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("old_{singular_table_name}"),
                quote! { &#table_type },
            ),
            build_function_arg(
                format_ident!("new_{singular_table_name}"),
                quote! { &#table_type },
            ),
        ],
        (_, Operation::Delete) => vec![
            build_dsl_function_arg(),
            build_function_arg(
                format_ident!("old_{singular_table_name}"),
                quote! { &#table_type },
            ),
        ],
    }
}

fn build_function_arg(name: syn::Ident, ty: TokenStream) -> SpacetimeDSLArg {
    SpacetimeDSLArg {
        is_mut: false,
        is_option: false,
        arg_name: name,
        arg_type: SpacetimeDSLArgType::Normal(ty),
    }
}

fn build_dsl_function_arg() -> SpacetimeDSLArg {
    SpacetimeDSLArg {
        is_mut: false,
        is_option: false,
        arg_name: format_ident!("dsl"),
        arg_type: SpacetimeDSLArgType::Normal(quote! {
            &spacetimedsl::DSL
        }),
    }
}
