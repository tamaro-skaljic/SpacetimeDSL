use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

//region Method Arguments

pub(in crate::internal::dsl) fn get_method_arg_column_type(column_type: &Type) -> TokenStream {
    quote! {
        #column_type
    }
}

pub(in crate::internal::dsl) fn get_method_arg_column_type_reference(
    column_type: &Type,
) -> TokenStream {
    quote! {
        &'a #column_type
    }
}

pub(in crate::internal::dsl) fn get_method_arg_into_wrapper_type(
    wrapper_type_name_or_path: &Type,
) -> TokenStream {
    quote! {
        impl Into<#wrapper_type_name_or_path>
    }
}

pub(in crate::internal::dsl) fn get_method_arg_into_wrapper_type_option(
    wrapper_type_name_or_path: &Type,
) -> TokenStream {
    quote! {
        impl Into<Option<#wrapper_type_name_or_path>>
    }
}

//endregion Method Arguments

//region Return Types

pub(in crate::internal::dsl) fn get_return_table_type_option(table_type: &Type) -> Box<str> {
    quote! {
        Option<#table_type>
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_table_type_iterator(table_type: &Type) -> Box<str> {
    quote! {
        impl Iterator<Item = #table_type>
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_column_type(column_type: &Type) -> Box<str> {
    quote! {
        #column_type
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_column_type_reference(column_type: &Type) -> Box<str> {
    quote! {
        &#column_type
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_wrapper_type(
    wrapper_type_name_or_path: &Type,
) -> Box<str> {
    quote! {
        #wrapper_type_name_or_path
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_wrapper_type_option(
    wrapper_type_name_or_path: &Type,
) -> Box<str> {
    quote! {
        Option<#wrapper_type_name_or_path>
    }
    .to_string()
    .into()
}

//endregion Return Types

//region Column Values

pub(in crate::internal::dsl) fn get_column_value(column_name: &Ident) -> TokenStream {
    quote! {
        #column_name
    }
}

pub(in crate::internal::dsl) fn get_column_value_from_wrapper(column_name: &Ident) -> TokenStream {
    quote! {
        #column_name.into().value()
    }
}

//endregion Column Values
