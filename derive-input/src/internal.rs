mod integration;

mod table;

mod column;

mod dsl;

mod utils;

pub(crate) fn try_parse(input: &syn::DeriveInput) -> syn::Result<crate::api::Table> {
    let (table_args, column_args) = integration::spacetime_bindings_macro_input(input)?;
    table::try_parse(input, &table_args, &column_args)
}
