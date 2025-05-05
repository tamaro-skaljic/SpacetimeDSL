use super::get_unique_multi_column_index_checks;
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
use quote::{format_ident, quote};
use syn::{Type, parse_str};

// TODO: Create a UniqueConstraintViolation error if a unique multi-column index is violated. (return a Result)
pub(in crate::internal) fn build(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    columns: &Vec<Column>,
) -> SpacetimeDSLMethod {
    let struct_name = format_ident!("{}", *rust_struct.name);
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);

    let doc_comment = format!("Create a {} row.", struct_name).into();

    let trait_name = format!("Create{}Row", struct_name,).into();

    let method_name = format!("create_{}", table_name,).into();

    let mut method_args = vec![];

    let try_insert_error_generic_type = format_ident!("{table_name}__TableHandle");
    let return_type = quote! {
        Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>>
    }
    .to_string()
    .into();

    let mut into_options = vec![];
    let mut constructor_args = vec![];

    for column in columns {
        let column_name = format_ident!("{}", *column.rust_field.name);
        let column_type: Type =
            parse_str(&column.rust_field.type_name_or_path).expect("create.build");

        if column.spacetimedb_column.is_auto_inc
            || column.rust_field.name.eq(&"created_at".to_string().into())
            || column.rust_field.name.eq(&"modified_at".to_string().into())
        {
            if column.spacetimedb_column.is_auto_inc {
                constructor_args.push(quote! {
                    #column_name: #column_type::default()
                });
            } else if column.rust_field.name.eq(&"created_at".to_string().into()) {
                constructor_args.push(quote! {
                    created_at: self.ctx().timestamp
                });
            } else if column.rust_field.name.eq(&"modified_at".to_string().into()) {
                constructor_args.push(quote! {
                    modified_at: self.ctx().timestamp
                });
            }
            continue;
        }

        match &column.spacetimedsl_column.wrapper_type {
            Some(wrapper_type) => match wrapper_type {
                WrapperType::Wrap(wrapper_type) => {
                    if column.rust_field.type_name_or_path.eq(&"String".into()) {
                        method_args.push(quote! {
                            #column_name: &str
                        });

                        constructor_args.push(quote! {
                            #column_name: #column_name.to_string()
                        });
                    } else {
                        let wrapped_type_name_or_path =
                            &WrapperType::map_to_wrapped_type(wrapper_type);
                        let ma = get_method_arg_column_type(wrapped_type_name_or_path);
                        method_args.push(quote! {
                            #column_name: #ma
                        });

                        constructor_args.push(get_column_value(&column_name));
                    }
                }
                WrapperType::Wrapped(_) => {
                    let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                    if column.spacetimedsl_column.is_option {
                        let ma = get_method_arg_into_wrapper_type_option(wrapper_type_name_or_path);
                        method_args.push(quote! {
                            #column_name: #ma
                        });

                        into_options.push(wrapper_type_into_option(
                            &column_name,
                            wrapper_type_name_or_path,
                        ));

                        constructor_args.push(get_column_value(&column_name));
                    } else {
                        let ma = get_method_arg_into_wrapper_type(wrapper_type_name_or_path);
                        method_args.push(quote! {
                            #column_name: #ma
                        });

                        let column_value = &get_column_value_from_wrapper(&column_name);
                        constructor_args.push(quote! {
                            #column_name: #column_value
                        });
                    }
                }
            },
            None => {
                if column.rust_field.type_name_or_path.eq(&"String".into()) {
                    method_args.push(quote! {
                        #column_name: &str
                    });
                    constructor_args.push(quote! {
                        #column_name: #column_name.to_string()
                    });
                } else {
                    let ma = get_method_arg_column_type(&column_type);
                    method_args.push(quote! {
                        #column_name: #ma
                    });
                    constructor_args.push(get_column_value(&column_name));
                }
            }
        };
    }

    let multi_column_index_checks =
        get_unique_multi_column_index_checks(rust_struct, spacetimedb_table);

    let multi_column_index_checks: Vec<TokenStream> = multi_column_index_checks
        .into_iter()
        .map(|mcic| mcic.check)
        .collect();

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();

    let use_itertools = if multi_column_index_checks.len() > 0 {
        quote! {
            use spacetimedsl::itertools::Itertools;
        }
    } else {
        TokenStream::default()
    };

    let method_impl = quote! {
        #use_itertools

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
