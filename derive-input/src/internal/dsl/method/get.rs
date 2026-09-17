use super::reference_integrity::Action;
use super::{
    context::MethodGenerationContext,
    index::{IndexColumnArguments, IndexShape, index_accessor, index_column_arguments},
    reference_integrity::get_unique_multi_column_index_check,
};
use crate::{
    api::{dsl::method::SpacetimeDSLMethod, runtime},
    internal::dsl::one_or_multiple::OneOrMultiple,
};
use quote::{format_ident, quote};

/// `get_all_<tables>`: iterate every row of the table.
pub(in crate::internal) fn for_get_all(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    SpacetimeDSLMethod {
        doc_comment: format!("Get all rows inside the `{singular_table_name}` table."),
        method_name: format_ident!("get_all_{}", plural_table_name),
        method_args: vec![],
        return_type: quote! {
            impl Iterator<Item = #struct_name>
        },
        method_impl: quote! {
            self
                .db()
                .#singular_table_name()
                .iter()
        },
        read_context_compatible: false,
    }
}

/// `count_of_all_<tables>`: how many rows the table holds.
pub(in crate::internal) fn for_get_count(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    SpacetimeDSLMethod {
        doc_comment: format!("Count all rows inside the `{singular_table_name}` table."),
        method_name: format_ident!("count_of_all_{}", plural_table_name),
        method_args: vec![],
        return_type: quote! {
            u64
        },
        method_impl: quote! {
            self
                .db()
                .#singular_table_name()
                .count()
        },
        read_context_compatible: true,
    }
}

/// `get_<tables>_by_<index>`: iterate the rows an index matches.
pub(in crate::internal) fn for_get_many(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::Multiple, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    // A multi-column index is filtered by the tuple of its columns.
    let method_impl = match shape.is_multi_column {
        true => quote! {
            #(#wrapper_option_mappers)*

            #index_accessor
                .filter((#(#row_value_getters),*))
        },
        false => quote! {
            #(#wrapper_option_mappers)*

            #index_accessor
                .filter(#(#row_value_getters),*)
        },
    };

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Get a `{struct_name}` iterator that contains all rows in the `{singular_table_name}` table {described_as}."
        ),
        method_name: format_ident!("get_{plural_table_name}_by_{index_name}"),
        method_args,
        return_type: quote! {
            impl Iterator<Item = #struct_name>
        },
        method_impl,
        read_context_compatible: true,
    }
}

/// `get_<table>_by_<index>`: look one row up by a unique index.
pub(in crate::internal) fn for_get_one(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        field_name_for_found_value,
        ..
    } = context;

    let index_name = &shape.index_name;
    let described_as = &shape.described_as;
    let column_names_and_row_values = &shape.column_names_and_row_values;
    let unique_multi_column_index_hint = shape.unique_multi_column_hint;

    let IndexColumnArguments {
        method_args,
        row_value_getters,
        wrapper_option_mappers,
    } = index_column_arguments(shape, &OneOrMultiple::One, context);

    let index_accessor = index_accessor(singular_table_name, index_name);

    let method_impl = match shape.is_multi_column {
        true => {
            // FIXME: Row Value Getters of Wrapper Types shouldn't be `id.clone().into().value()`, they should be `let id = id.into();` at the method beginning and then `id.value()` anywhere else
            let multi_column_index_check = get_unique_multi_column_index_check(
                &Action::Get,
                singular_table_name,
                index_name,
                column_names_and_row_values,
                &row_value_getters,
            );

            let not_found_error = runtime::not_found_error(
                singular_table_name_as_string,
                &quote! {
                    format!(#column_names_and_row_values, #(#row_value_getters),*)
                },
            );

            let itertools_import = runtime::itertools_import();

            quote! {
                #(#wrapper_option_mappers)*

                #itertools_import

                let mut #field_name_for_found_value: Option<#struct_name> = None;

                #multi_column_index_check

                match #field_name_for_found_value {
                    Some(#singular_table_name) => Ok(#singular_table_name),
                    None => {
                        return Err(#not_found_error);
                    }
                }
            }
        }
        false => {
            let not_found_error = runtime::not_found_error(
                singular_table_name_as_string,
                &quote! {
                    format!(#column_names_and_row_values, #(#row_value_getters),*)
                },
            );

            quote! {
                #(#wrapper_option_mappers)*

                match #index_accessor.find(#(#row_value_getters),*) {
                    Some(#singular_table_name) => Ok(#singular_table_name),
                    None => return Err(#not_found_error)
                }
            }
        }
    };

    SpacetimeDSLMethod {
        doc_comment: format!(
            "{unique_multi_column_index_hint}\n\nTry to get a `{struct_name}` from the `{singular_table_name}` table {described_as}."
        ),
        method_name: format_ident!("get_{singular_table_name}_by_{index_name}"),
        method_args,
        return_type: runtime::error_result_type(struct_name),
        method_impl,
        read_context_compatible: true,
    }
}
