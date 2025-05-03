mod integration;

mod table;

mod column;

mod dsl;

mod utils;

pub(crate) fn try_parse(item: &syn::DeriveInput) -> syn::Result<crate::api::Table> {
    let (table_args, column_args) = integration::spacetime_bindings_macro_input(item)?;
    table::try_parse(item, &table_args, &column_args)
}
