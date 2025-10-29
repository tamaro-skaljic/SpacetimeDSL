use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use spacetimedsl_derive_input::api::{
    Table,
    dsl::{column::SpacetimeDSLColumnMethods, method::SpacetimeDSLArg, wrapper::WrapperType},
};

mod accessor;
mod create_method_arg;
mod function;
mod hook;

pub(crate) fn output(input: &Table, first_dsl_attribute: bool) -> syn::Result<TokenStream> {
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

    dsl_methods.push(function::associated::without_lifetime::build(
        &input.spacetimedsl_methods.create,
    )?);
    dsl_methods.push(function::associated::with_lifetime::build(
        &input.spacetimedsl_methods.get_all,
    )?);
    dsl_methods.push(function::associated::with_lifetime::build(
        &input.spacetimedsl_methods.get_count,
    )?);

    if let Some(method) = &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted
    {
        dsl_methods.push(function::build(method)?);
    }

    if let Some(method) = &input
        .spacetimedsl_methods
        .execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted {
        dsl_methods.push(function::build(method)?);
    }

    for execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted {
        dsl_methods.push(function::build(execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted)?);
    }

    for execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted in &input.spacetimedsl_methods.execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted {
        dsl_methods.push(function::build(execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted)?);
    }

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        dsl_methods.push(get_column_dsl_methods(multi_column_index)?);
    }

    for column in &input.columns {
        if first_dsl_attribute {
            table_methods.push(accessor::getter(&column.spacetimedsl_column.getter)?);

            if let Some(data) = &column.spacetimedsl_column.setter {
                table_methods.push(accessor::setter(data)?)
            }
        }

        if let Some(methods) = &column.spacetimedsl_methods {
            dsl_methods.push(get_column_dsl_methods(methods)?)
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

    Ok(quote! {
        #(#compile_error_checks)*

        #(#wrapper_types)*

        impl #struct_name {
            #(#table_methods)*
        }

        #create_dsl_method_arg

        #(#hooks)*

        #(#dsl_methods)*
    })
}

fn get_column_dsl_methods(methods: &SpacetimeDSLColumnMethods) -> syn::Result<TokenStream> {
    let mut token_streams = vec![];

    match methods {
        SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => {
            token_streams.push(function::associated::without_lifetime::build(
                &methods.get_one_option,
            )?);

            if let Some(method) = &methods.update {
                token_streams.push(function::associated::without_lifetime::build(method)?)
            };

            token_streams.push(function::associated::without_lifetime::build(
                &methods.delete_one,
            )?);
        }
        SpacetimeDSLColumnMethods::ForIndex(methods) => {
            token_streams.push(function::associated::with_lifetime::build(
                &methods.get_many,
            )?);

            token_streams.push(function::associated::with_lifetime::build(
                &methods.delete_many,
            )?);
        }
    };

    Ok(quote! {
        #(#token_streams)*
    })
}

pub fn malformed_code_generation_result(result: String) -> String {
    let mut result = result.replace("\n", "");

    for _ in 0..20 {
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

        if arg.is_mut {
            function_args.push(quote! {
                mut #arg_name: #arg_type
            });
        } else {
            function_args.push(quote! {
                #arg_name: #arg_type
            });
        }
    }

    function_args
}
