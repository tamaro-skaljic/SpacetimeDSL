use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use spacetimedsl_derive_input::api::{
    Table,
    dsl::{
        column::SpacetimeDSLColumnMethods,
        method::{SpacetimeDSLArg, SpacetimeDSLMethod},
        wrapper::WrapperType,
    },
};
use syn::Ident;

mod accessor;
mod create_method_arg;
mod doc_comment;
mod function;
mod hook;

/// The generated output, split so callers can inspect every DSL method on its own
/// instead of walking the [`Table`] a second time themselves.
pub(crate) struct GeneratedOutput {
    /// Compile-error checks, wrapper types, the accessor `impl`, the create-argument
    /// struct and the hook traits - everything the macro emits that is not a DSL method.
    pub items_outside_dsl_methods: TokenStream,
    pub dsl_methods: Vec<GeneratedDSLMethod>,
}

impl GeneratedOutput {
    /// Concatenates the halves in the order the macro emits them.
    pub(crate) fn into_token_stream(self) -> TokenStream {
        let items_outside_dsl_methods = self.items_outside_dsl_methods;
        let dsl_methods = self
            .dsl_methods
            .into_iter()
            .map(|dsl_method| dsl_method.tokens);

        quote! {
            #items_outside_dsl_methods

            #(#dsl_methods)*
        }
    }
}

pub(crate) struct GeneratedDSLMethod {
    /// Only read by the characterization tests, which snapshot each method under its own name.
    #[cfg_attr(not(test), allow(dead_code))]
    pub method_name: Ident,
    pub tokens: TokenStream,
}

pub(crate) fn build(input: &Table, first_dsl_attribute: bool) -> syn::Result<GeneratedOutput> {
    let struct_name = format_ident!("{}", &input.rust_struct.name.to_string());
    let mut wrapper_types = vec![];

    // Only generate wrapper types if this is the last DSL attribute to avoid conflicts
    if first_dsl_attribute {
        for column in &input.columns {
            if let Some(WrapperType::Created(wrapper_type)) =
                &column.spacetimedsl_column.wrapper_type
            {
                wrapper_types.push(&wrapper_type.wrapper_impl);
            }
        }
    }

    let mut table_methods = vec![];
    let mut dsl_methods = vec![];

    dsl_methods.push(build_public_dsl_method(&input.spacetimedsl_methods.create)?);

    if let Some(method) = &input.spacetimedsl_methods.get_all {
        dsl_methods.push(build_public_dsl_method(method)?);
    }
    if let Some(method) = &input.spacetimedsl_methods.get_count {
        dsl_methods.push(build_public_dsl_method(method)?);
    }

    if let Some(method) = &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted
    {
        dsl_methods.push(build_internal_dsl_method(method)?);
    }

    if let Some(method) = &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted {
        dsl_methods.push(build_internal_dsl_method(method)?);
    }

    for execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted {
        dsl_methods.push(build_internal_dsl_method(
            execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted
        )?);
    }

    for execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted {
        dsl_methods.push(build_internal_dsl_method(
            execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted
        )?);
    }

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        dsl_methods.extend(get_column_dsl_methods(multi_column_index)?);
    }

    for column in &input.columns {
        if first_dsl_attribute {
            if let Some(getter) = &column.spacetimedsl_column.getter {
                table_methods.push(accessor::build(accessor::Accessor::Getter(getter))?);
            }

            if let Some(mut_getter) = &column.spacetimedsl_column.mut_getter {
                table_methods.push(accessor::build(accessor::Accessor::MutGetter(mut_getter))?)
            }

            if let Some(setter) = &column.spacetimedsl_column.setter {
                table_methods.push(accessor::build(accessor::Accessor::Setter(setter))?)
            }
        }

        if let Some(methods) = &column.spacetimedsl_methods {
            dsl_methods.extend(get_column_dsl_methods(methods)?)
        }
    }

    let mut compile_error_checks = vec![];

    input
        .spacetimedsl_table
        .compile_error_checks
        .iter()
        .for_each(|compile_error_check| {
            compile_error_checks.push(quote! {
                pub trait #compile_error_check {}
            });
        });

    let create_dsl_method_arg = match &input.spacetimedsl_table.create_dsl_method_arg {
        Some(arg) => create_method_arg::build(&arg.struct_impl)?,
        None => TokenStream::default(),
    };

    let hooks = vec![
        hook::build(&input.spacetimedsl_table.hooks.before_insert)?,
        hook::build(&input.spacetimedsl_table.hooks.before_update)?,
        hook::build(&input.spacetimedsl_table.hooks.before_delete)?,
        hook::build(&input.spacetimedsl_table.hooks.after_insert)?,
        hook::build(&input.spacetimedsl_table.hooks.after_update)?,
        hook::build(&input.spacetimedsl_table.hooks.after_delete)?,
    ];

    Ok(GeneratedOutput {
        items_outside_dsl_methods: quote! {
            #(#compile_error_checks)*

            #(#wrapper_types)*

            impl #struct_name {
                #(#table_methods)*
            }

            #create_dsl_method_arg

            #(#hooks)*
        },
        dsl_methods,
    })
}

fn build_public_dsl_method(method: &SpacetimeDSLMethod) -> syn::Result<GeneratedDSLMethod> {
    Ok(GeneratedDSLMethod {
        method_name: method.method_name.clone(),
        tokens: function::build_public(method)?,
    })
}

fn build_internal_dsl_method(method: &SpacetimeDSLMethod) -> syn::Result<GeneratedDSLMethod> {
    Ok(GeneratedDSLMethod {
        method_name: method.method_name.clone(),
        tokens: function::build_internal(method)?,
    })
}

fn get_column_dsl_methods(
    methods: &SpacetimeDSLColumnMethods,
) -> syn::Result<Vec<GeneratedDSLMethod>> {
    let mut dsl_methods = vec![];

    match methods {
        SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => {
            dsl_methods.push(build_public_dsl_method(&methods.get_one_option)?);

            if let Some(method) = &methods.update {
                dsl_methods.push(build_public_dsl_method(method)?)
            };

            dsl_methods.push(build_public_dsl_method(&methods.delete_one)?);
        }
        SpacetimeDSLColumnMethods::ForIndex(methods) => {
            dsl_methods.push(build_public_dsl_method(&methods.get_many)?);

            dsl_methods.push(build_public_dsl_method(&methods.delete_many)?);
        }
    };

    Ok(dsl_methods)
}

pub fn malformed_code_generation_result(result: String) -> String {
    let mut result = result.replace("\n", " ");

    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    format!("

Congratulations, you have found a bug in SpacetimeDSL!

We would be very pleased if you can create an issue in our GitHub repository: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/new

Please include your table definition as well as the following, malformed, code generation result - thank you very much!

{result}

")
}

fn map_args(args: &Vec<SpacetimeDSLArg>) -> Vec<TokenStream> {
    let mut function_args = vec![];

    for arg in args {
        let arg_name = &arg.arg_name;
        let arg_type = match &arg.arg_type {
            spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLArgType::Normal(
                actual_type,
            ) => actual_type,
            spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLArgType::Wrapped {
                wrapped_type: _,
                actual_type,
            } => actual_type,
        };

        function_args.push(quote! {
            #arg_name: #arg_type
        });
    }

    function_args
}
