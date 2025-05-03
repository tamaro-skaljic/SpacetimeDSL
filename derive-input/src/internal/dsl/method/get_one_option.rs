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
        get_column_value, get_column_value_from_wrapper, get_method_arg_column_type,
        get_method_arg_into_wrapper_type, get_method_arg_into_wrapper_type_option,
        get_return_table_type_option,
    },
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, quote};

pub(in crate::internal) fn for_single_column_index(
    rust_struct: &RustStruct,
    spacetimedb_table: &SpacetimeDBTable,
    rust_field: &RustField,
    spacetimedsl_column: &SpacetimeDSLColumn,
) -> SpacetimeDSLMethod {
    let struct_name = &rust_struct.name;
    let table_name = &spacetimedb_table.singular_name;
    let column_name = &rust_field.name;

    let doc_comment = format!(
        "Get an Option<{}> row inside the {} table filtered by {}.",
        struct_name, table_name, column_name,
    )
    .into();

    let trait_name = format!(
        "Get{}RowOptionBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(column_name)
    )
    .into();

    let method_name = format!(
        "get_{}_by_{}",
        &spacetimedb_table.singular_name, column_name
    )
    .into();

    let mut method_arg = quote! { #column_name: };

    let return_type = get_return_table_type_option(&rust_struct.name);

    let column_value;

    let mut into_option = TokenStream::default();

    match &spacetimedsl_column.wrapper_type {
        Some(wrapper_type) => {
            let wrapper_type_name_or_path = match wrapper_type {
                WrapperType::Wrap(wrap) => &wrap.wrapper_struct_name,
                WrapperType::Wrapped(wrapped) => &wrapped.wrapper_struct_name_or_path,
            };

            if spacetimedsl_column.is_option {
                method_arg.append_all(get_method_arg_into_wrapper_type_option(
                    wrapper_type_name_or_path,
                ));

                into_option = wrapper_type_into_option(column_name, wrapper_type_name_or_path);
                column_value = get_column_value(column_name);
            } else {
                method_arg.append_all(get_method_arg_into_wrapper_type(wrapper_type_name_or_path));

                column_value = get_column_value_from_wrapper(column_name);
            }
        }
        None => {
            method_arg.append_all(get_method_arg_column_type(&rust_field.type_name_or_path));

            column_value = get_column_value(column_name);
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
            .find(#column_value);
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
    let table_name = &spacetimedb_table.singular_name;
    let index_name = &multi_column_index.name;

    let doc_comment = format!(
        "Get an Option<{}> row inside the {} table filtered by {}.",
        struct_name, table_name, index_name,
    )
    .into();

    let trait_name = format!(
        "Get{}RowOptionBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(index_name)
    )
    .into();

    let method_name = format!("get_{}_by_{}", &spacetimedsl_table.plural_name, index_name).into();

    let mut method_args = vec![];

    let return_type = get_return_table_type_option(&rust_struct.name);

    let mut column_values = vec![];

    let mut into_options = vec![];

    for column in columns {
        let column_name = &column.rust_field.name;
        let column_type = &column.rust_field.type_name_or_path;

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

                    let mut column_value = format!("{column_name}: ");
                    column_value.push_str(&get_column_value(column_name));
                    column_values.push(column_value.into());
                } else {
                    let mut method_arg = quote! {
                        #column_name:
                    };
                    method_arg
                        .append_all(get_method_arg_into_wrapper_type(wrapper_type_name_or_path));
                    method_args.push(method_arg);

                    let mut column_value = format!("{column_name}: ");
                    column_value.push_str(&get_column_value_from_wrapper(column_name));
                    column_values.push(column_value.into());
                }
            }
            None => {
                let mut method_arg = quote! {
                    #column_name:
                };
                method_arg.append_all(get_method_arg_column_type(column_type));
                method_args.push(method_arg);

                column_values.push(get_column_value(column_name));
            }
        };
    }

    let mut panic_msg = format!(
        "There must be only one {struct_name} row inside the {table_name} table when filtering on the unique multi-column index {index_name} with value "
    );
    panic_msg.push_str("{:?}. Found: {:#?}");

    let method_args = method_args.iter().map(|ts| ts.to_string().into()).collect();
    let method_impl = quote! {
        use itertools::Itertools;

        #(#into_options)*

        match ctx.db.#table_name().#index_name().filter((#(#column_values),*)).exactly_one() {
            Ok(#table_name) => Some(#table_name),
            Err(e) => {
                panic!(
                    #panic_msg,
                    (#(#column_values),*),
                    e.collect()
                );
            }
        }
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
