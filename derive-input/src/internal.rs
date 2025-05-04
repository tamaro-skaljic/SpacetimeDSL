mod integration;

mod table;

mod column;

mod dsl;

mod utils;

pub(crate) fn try_parse(args: proc_macro2::TokenStream, input: &syn::DeriveInput) -> syn::Result<crate::api::Table> {
    let (table_args, column_args) = integration::spacetime_bindings_macro_input(input)?;
    table::try_parse(args, input, &table_args, &column_args)
}
