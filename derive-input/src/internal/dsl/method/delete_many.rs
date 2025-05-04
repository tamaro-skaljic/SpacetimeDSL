use crate::api::db::IndexType;
use crate::internal::utils::wrapper_type_into_option;
use crate::{
    api::{
        Column,
        db::{Index, SpacetimeDBTable},
        dsl::{
            column::SpacetimeDSLColumn, method::SpacetimeDSLMethod, table::SpacetimeDSLTable,
            wrapper::WrapperType,
        },
        rust::{RustField, RustStruct},
    },
    internal::dsl::quote::{
        get_column_value, get_column_value_from_wrapper, get_method_arg_column_type_reference,
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
    spacetimedsl_table: &SpacetimeDSLTable,
    rust_field: &RustField,
    spacetimedsl_column: &SpacetimeDSLColumn,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let column_name = format_ident!("{}", *rust_field.name);

    let doc_comment = format!(
        "Delete all {} rows inside the {} table filtered by the single-column index {}.",
        struct_name, table_name, column_name,
    )
    .into();

    let trait_name = format!(
        "Delete{}RowsBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(column_name.to_string())
    )
    .into();

    let method_name = format!(
        "delete_{}_by_{}",
        &spacetimedsl_table.plural_name, column_name
    )
    .into();

    let method_arg;

    let return_type = "u64".into();

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
                .expect("delete_many.for_single_column_index");
            let ma = get_method_arg_column_type_reference(&column_type);
            method_arg = quote! { #column_name: #ma };

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
    spacetimedsl_table: &SpacetimeDSLTable,
    columns: &[Column],
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = format_ident!("{}", *spacetimedb_table.singular_name);
    let index_name = &multi_column_index.name;
    let index_columns = match &multi_column_index.index_type {
        IndexType::BTreeMultiColumn { columns } => columns,
        i => {
            panic!(
                "There shouldn't be an index with another type when delete_many.for_multi_column_index is running. Found: {:#?}",
                i
            )
        }
    };

    let doc_comment = format!(
        "Delete all {} rows inside the {} table filtered by the multi-column index {}.",
        struct_name, table_name, index_name,
    )
    .into();

    let trait_name = format!(
        "Delete{}RowsBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(index_name)
    )
    .into();

    let method_name = format!(
        "delete_{}_by_{}",
        &spacetimedsl_table.plural_name, index_name
    )
    .into();

    let mut method_args = vec![];

    let return_type = "u64".into();

    let mut column_values = vec![];

    let mut into_options = vec![];

    for column in columns {
        let column_name = format_ident!("{}", *column.rust_field.name);
        let column_type: Type = parse_str(&column.rust_field.type_name_or_path)
            .expect("delete_many.for_multi_column_index");

        if !index_columns.contains(&column_name.to_string().into()) {
            continue;
        }
        let column_name = format_ident!("{}", *column.rust_field.name);
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
                let ma = get_method_arg_column_type_reference(&column_type);
                method_args.push(quote! {
                    #column_name: #ma
                });

                column_values.push(get_column_value(&column_name));
            }
        };
    }

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();
    let index_name = format_ident!("{index_name}");
    let method_impl = quote! {
        #(#into_options)*
        return self
            .ctx()
            .db()
            .#table_name()
            .#index_name()
            .delete((#(#column_values),*));
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
