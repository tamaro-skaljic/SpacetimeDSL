//! The methods a table adds to the wrapper types of its foreign key columns.
//!
//! Each one looks rows up through the DSL method of the column's index, so the lookup, its
//! name and its return type stay defined once, in `get.rs`.

use super::context::MethodGenerationContext;
use crate::api::{
    Column,
    dsl::{
        column::SpacetimeDSLColumnMethods,
        wrapper::{WrapperMethod, WrapperType},
    },
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// One method per column of `columns_with_foreign_key`, which all reference
/// `referenced_table_name`.
pub(in crate::internal) fn for_wrapper_methods(
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &[&Column],
    context: &MethodGenerationContext,
) -> Vec<WrapperMethod> {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    let takes_name_of_dsl_method =
        referenced_table_name == singular_table_name || columns_with_foreign_key.len() > 1;

    columns_with_foreign_key
        .iter()
        .filter_map(|column| {
            let column_methods = column.spacetimedsl_methods.as_ref()?;

            let wrapper_type = column.spacetimedsl_column.wrapper_type.as_ref().expect(
                "`internal/dsl/column.rs` rejects a `#[foreign_key]` column without `#[use_wrapper]`",
            );

            let (dsl_method, short_method_name, rows_found, return_type, collect_rows) =
                match column_methods {
                SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => (
                    &methods.get_one_option,
                    format_ident!("get_{singular_table_name}"),
                    format!("the `{struct_name}`"),
                    methods.get_one_option.return_type.clone(),
                    TokenStream::default(),
                ),
                SpacetimeDSLColumnMethods::ForIndex(methods) => (
                    &methods.get_many,
                    format_ident!("get_{plural_table_name}"),
                    format!("all `{struct_name}` rows"),
                    quote! { Vec<#struct_name> },
                    quote! { .collect() },
                ),
            };

            let method_name = match takes_name_of_dsl_method {
                true => dsl_method.method_name.clone(),
                false => short_method_name,
            };

            let dsl_method_name = &dsl_method.method_name;
            let column_name = &column.rust_field.name;
            let wrapper_struct_name = wrapper_type.struct_name();
            let wrapper_variable_name =
                RenameRule::SnakeCase.apply_to_variant(wrapper_struct_name.to_string());

            Some(WrapperMethod {
                wrapper_type: WrapperType::map(wrapper_type),
                doc_comment: format!(
                    "Get {rows_found} whose `{column_name}` column references this `{wrapper_struct_name}`.\n\nUse it like `{wrapper_variable_name}.{method_name}(&dsl)`."
                ),
                method_name,
                return_type,
                method_impl: quote! {
                    dsl.into().#dsl_method_name(self)#collect_rows
                },
            })
        })
        .collect()
}
