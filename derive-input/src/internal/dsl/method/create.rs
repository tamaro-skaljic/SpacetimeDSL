use crate::internal::utils::wrapper_type_into_option;
use crate::{
    api::{
        Column,
        db::SpacetimeDBTable,
        dsl::{method::SpacetimeDSLMethod, wrapper::WrapperType},
        rust::RustStruct,
    },
    internal::dsl::quote::{
        get_column_value, get_column_value_from_wrapper, get_method_arg_column_type,
        get_method_arg_into_wrapper_type, get_method_arg_into_wrapper_type_option,
    },
};
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, quote};

use super::get_unique_multi_column_index_checks;

// TODO: Use try_update instead of update and create a UniqueConstraintViolation error if a unique multi-column index is violated. (return a Result)
pub(in crate::internal) fn build(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    columns: &Vec<Column>,
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = &spacetimedb_table.singular_name;

    let doc_comment = format!("Create a {} row.", struct_name).into();

    let trait_name = format!("Create{}Row", struct_name,).into();

    let method_name = format!("create_{}", table_name,).into();

    let mut method_args = vec![];

    let try_insert_error_generic_type = format!("{table_name}__TableHandle");
    let return_type = quote! {
        Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
    }
    .to_string()
    .into();

    let mut into_options = vec![];
    let mut constructor_args = vec![];

    for column in columns {
        let column_name = &column.rust_field.name;
        let column_type = &column.rust_field.type_name_or_path;

        if column.spacetimedb_column.is_auto_inc
            || column.rust_field.name.eq(&"created_at".to_string().into())
            || column.rust_field.name.eq(&"modified_at".to_string().into())
        {
            method_args.push(TokenStream::default());

            if column.spacetimedb_column.is_auto_inc {
                constructor_args.push(
                    quote! {
                        #column_name: #column_type::default()
                    }
                    .to_string()
                    .into(),
                );
            } else if column.rust_field.name.eq(&"created_at".to_string().into()) {
                constructor_args.push(
                    quote! {
                        created_at: self.ctx().timestamp
                    }
                    .to_string()
                    .into(),
                );
            } else if column.rust_field.name.eq(&"modified_at".to_string().into()) {
                constructor_args.push(
                    quote! {
                        modified_at: self.ctx().timestamp
                    }
                    .to_string()
                    .into(),
                );
            }
            continue;
        }

        match &column.spacetimedsl_column.wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_name_or_path = match wrapper_type {
                    WrapperType::Wrap(wrap) => &wrap.wrapper_struct_name,
                    WrapperType::Wrapped(wrapped) => &wrapped.wrapper_struct_name_or_path,
                };

                if column.spacetimedsl_column.is_option {
                    let mut method_arg = quote! {
                        #column_name:
                    };
                    method_arg.append_all(get_method_arg_into_wrapper_type_option(
                        wrapper_type_name_or_path,
                    ));
                    method_args.push(method_arg);

                    into_options.push(wrapper_type_into_option(
                        column_name,
                        wrapper_type_name_or_path,
                    ));

                    let mut constructor_arg = format!("{column_name}: ");
                    constructor_arg.push_str(&get_column_value(column_name));
                    constructor_args.push(constructor_arg.into());
                } else {
                    let mut method_arg = quote! {
                        #column_name:
                    };
                    method_arg
                        .append_all(get_method_arg_into_wrapper_type(wrapper_type_name_or_path));
                    method_args.push(method_arg);

                    let mut constructor_arg = format!("{column_name}: ");
                    constructor_arg.push_str(&get_column_value_from_wrapper(column_name));
                    constructor_args.push(constructor_arg.into());
                }
            }
            None => {
                let mut method_arg = quote! {
                    #column_name:
                };
                method_arg.append_all(get_method_arg_column_type(column_type));
                method_args.push(method_arg);

                constructor_args.push(get_column_value(column_name));
            }
        };
    }

    let multi_column_index_checks = get_unique_multi_column_index_checks(
        rust_struct,
        spacetimedb_table,
        primary_key_column_name,
    );

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();
    let method_impl = quote! {
        use itertools::Itertools;

        #(#into_options)*
        let #table_name = #struct_name {
            #(#constructor_args),*
        };

        #(#multi_column_index_checks)*

        return self
                .ctx()
                .db()
                .#table_name()
                .try_insert(#table_name);
    }
    .to_string()
    .into();

    SpacetimeDSLMethod {
        doc_comment,
        trait_name,
        method_name,
        method_args,
        return_type,
        method_impl,
    }
}
