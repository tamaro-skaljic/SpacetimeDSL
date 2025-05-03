use proc_macro2::TokenStream;
use quote::quote;

//region Method Arguments

pub(in crate::internal::dsl) fn get_method_arg_vec(method_arg: TokenStream) -> TokenStream {
    quote! {
        Vec<#method_arg>
    }
}

pub(in crate::internal::dsl) fn get_method_arg_column_type(column_type: &Box<str>) -> TokenStream {
    quote! {
        #column_type
    }
}

pub(in crate::internal::dsl) fn get_method_arg_column_type_reference(
    column_type: &Box<str>,
) -> TokenStream {
    quote! {
        &'a #column_type
    }
}

pub(in crate::internal::dsl) fn get_method_arg_into_wrapper_type(
    wrapper_type_name_or_path: &Box<str>,
) -> TokenStream {
    quote! {
        impl Into<#wrapper_type_name_or_path>
    }
}

pub(in crate::internal::dsl) fn get_method_arg_into_wrapper_type_option(
    wrapper_type_name_or_path: &Box<str>,
) -> TokenStream {
    quote! {
        impl Into<Option<#wrapper_type_name_or_path>>
    }
}

//endregion Method Arguments

//region Return Types

pub(in crate::internal::dsl) fn get_return_table_type_option(table_type: &Box<str>) -> Box<str> {
    quote! {
        Option<#table_type>
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_table_type_iterator(table_type: &Box<str>) -> Box<str> {
    quote! {
        impl Iterator<Item = #table_type>
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_table_type_option_iterator(
    table_type: &Box<str>,
) -> Box<str> {
    quote! {
        impl Iterator<Item = Option<#table_type>>
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_column_type(column_type: &Box<str>) -> Box<str> {
    quote! {
        #column_type
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_column_type_reference(
    column_type: &Box<str>,
) -> Box<str> {
    quote! {
        &#column_type
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_wrapper_type(
    wrapper_type_name_or_path: &Box<str>,
) -> Box<str> {
    quote! {
        #wrapper_type_name_or_path
    }
    .to_string()
    .into()
}

pub(in crate::internal::dsl) fn get_return_wrapper_type_option(
    wrapper_type_name_or_path: &Box<str>,
) -> Box<str> {
    quote! {
        Option<#wrapper_type_name_or_path>
    }
    .to_string()
    .into()
}

//endregion Return Types

//region Column Values

pub fn get_column_value(column_name: &Box<str>) -> Box<str> {
    quote! {
        #column_name
    }
    .to_string()
    .into()
}

pub fn get_column_value_from_wrapper(column_name: &Box<str>) -> Box<str> {
    quote! {
        #column_name.into().value()
    }
    .to_string()
    .into()
}

//endregion Column Values
