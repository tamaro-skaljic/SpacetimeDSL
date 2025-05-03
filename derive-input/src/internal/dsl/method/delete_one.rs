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
        "Delete a {} row inside the {} table filtered by the unique single-column index {}.",
        struct_name, table_name, column_name,
    )
    .into();

    let trait_name = format!(
        "Delete{}RowBy{}",
        struct_name,
        RenameRule::PascalCase.apply_to_field(column_name)
    )
    .into();

    let method_name = format!(
        "delete_{}_by_{}",
        &spacetimedb_table.singular_name, column_name
    )
    .into();

    let mut method_arg = quote! { #column_name: };

    let return_type = "bool".into();

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
    todo!()
}
