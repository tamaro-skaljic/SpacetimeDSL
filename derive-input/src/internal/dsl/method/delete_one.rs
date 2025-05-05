use super::get_unique_multi_column_index_check;
use crate::api::db::IndexType;
use crate::internal::utils::wrapper_type_into_option;
use crate::{
    api::{
        Column,
        db::{Index, SpacetimeDBTable},
        dsl::{column::SpacetimeDSLColumn, method::SpacetimeDSLMethod, wrapper::WrapperType},
        rust::{RustField, RustStruct},
    },
    internal::dsl::quote::{
        get_column_value, get_column_value_from_wrapper, get_method_arg_column_type,
        get_method_arg_into_wrapper_type, get_method_arg_into_wrapper_type_option,
    },
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Type, parse_str};

pub(in crate::internal) fn for_single_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    rust_field: &RustField,
    spacetimedsl_column: &SpacetimeDSLColumn,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let column_name = format_ident!("{}", *rust_field.name);

    let doc_comment = format!(
        "Delete a {} row inside the {} table filtered by the unique single-column index {}.",
        struct_name, table_name, column_name,
    )
    .into();

    let trait_name = format!(
        "Delete{}RowBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(column_name.to_string())
    )
    .into();

    let method_name = format!(
        "delete_{}_by_{}",
        &spacetimedb_table.singular_name, column_name
    )
    .into();

    let method_arg;

    let return_type = "bool".into();

    let column_value;

    let mut into_option = TokenStream::default();

    match &spacetimedsl_column.wrapper_type {
        Some(wrapper_type) => {
            let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

            if spacetimedsl_column.is_option {
                let ma = get_method_arg_into_wrapper_type_option(wrapper_type_name_or_path);
                method_arg = quote! { #column_name: &#ma };

                into_option = wrapper_type_into_option(&column_name, wrapper_type_name_or_path);
                column_value = get_column_value(&column_name);
            } else {
                let ma = get_method_arg_into_wrapper_type(wrapper_type_name_or_path);
                method_arg = quote! { #column_name: #ma };

                column_value = get_column_value_from_wrapper(&column_name);
            }
        }
        None => {
            let column_type: Type = parse_str(&rust_field.type_name_or_path)
                .expect("delete_one.for_single_column_index");
            let ma = get_method_arg_column_type(&column_type);
            method_arg = quote! { #column_name: &#ma };

            column_value = get_column_value(&column_name);
        }
    };

    let method_args = vec![method_arg.to_string().into()];
    let method_impl = quote! {
        #into_option
        return self
            .ctx()
            .db()
            .#table_name()
            .#column_name()
            .delete(#column_value);
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

pub(in crate::internal) fn for_multi_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    multi_column_index: &Index,
    columns: &[Column],
    primary_key_column_name: &Box<str>,
) -> SpacetimeDSLMethod {
    let struct_name = format_ident!("{}", *rust_struct.name);
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let index_name = &multi_column_index.name;
    let index_columns = match &multi_column_index.index_type {
        IndexType::BTreeMultiColumn { columns } => columns,
        i => {
            panic!(
                "There shouldn't be an index with another type when delete_one.for_multi_column_index is running. Found: {:#?}",
                i
            )
        }
    };

    let doc_comment = format!(
        "Delete a {} row inside the {} table by the unique multi-column index {}.\n\nPanics if it finds more than one, because then the unique constraint is violated somewhere.",
        struct_name, table_name, index_name,
    )
    .into();

    let trait_name = format!(
        "Delete{}RowBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(index_name)
    )
    .into();

    let method_name = format!(
        "delete_{}_by_{}",
        &spacetimedb_table.singular_name, index_name
    )
    .into();

    let mut method_args = vec![];

    let return_type = "bool".into();

    let mut column_values = vec![];

    let mut into_options = vec![];

    for column in columns {
        let column_name = format_ident!("{}", *column.rust_field.name);
        let column_type: Type = parse_str(&column.rust_field.type_name_or_path)
            .expect("delete_one.for_multi_column_index");

        if !index_columns.contains(&column_name.to_string().into()) {
            continue;
        }

        match &column.spacetimedsl_column.wrapper_type {
            Some(wrapper_type) => {
                let wrapper_type_name_or_path = &WrapperType::map(wrapper_type);

                if column.spacetimedsl_column.is_option {
                    let ma = get_method_arg_into_wrapper_type_option(wrapper_type_name_or_path);
                    method_args.push(quote! {
                        #column_name: &#ma
                    });

                    into_options.push(wrapper_type_into_option(
                        &column_name,
                        wrapper_type_name_or_path,
                    ));

                    let column_value = &get_column_value(&column_name);
                    column_values.push(quote! {
                        #column_name: #column_value
                    });
                } else {
                    let ma = get_method_arg_into_wrapper_type(wrapper_type_name_or_path);
                    method_args.push(quote! {
                        #column_name: #ma
                    });

                    let column_value = &get_column_value_from_wrapper(&column_name);
                    column_values.push(quote! {
                        #column_name: #column_value
                    });
                }
            }
            None => {
                let ma = get_method_arg_column_type(&column_type);
                method_args.push(quote! {
                    #column_name: &#ma
                });

                column_values.push(get_column_value(&column_name));
            }
        };
    }

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();

    let multi_column_index_check = get_unique_multi_column_index_check(
        &struct_name.to_string().into(),
        &table_name,
        multi_column_index,
        column_values,
    )
    .check;

    let primary_key_column_name = format_ident!("{primary_key_column_name}");
    let field_name_for_found_value = format_ident!("the_same_or_another_{table_name}");

    let method_impl = quote! {
        use spacetimedsl::itertools::Itertools;

        #(#into_options)*

        #multi_column_index_check

        return self
            .ctx()
            .db()
            .#table_name()
            .#primary_key_column_name()
            .delete(#field_name_for_found_value.unwrap().#primary_key_column_name);
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
