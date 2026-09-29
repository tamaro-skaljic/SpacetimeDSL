//! The shape of the "which row" part every generated error carries, such as
//! `{ id : 7 }` or `{ database_id : 1, name : main }`.
//!
//! Each function returns the finished `format!(…)` call, so no caller builds a format
//! string at generation time and escapes it for the `format!` that runs later.

use {
    crate::internal::dsl::singleton,
    itertools::Itertools,
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::Ident,
};

/// `format!("{{ a : {}, b : {} }}", a_value, b_value)`: one `column : value` pair per column,
/// in the order given.
///
/// The columns and the values are zipped by position, so a caller that builds its values
/// from the same column list cannot get the two out of step.
pub fn column_names_and_row_values(
    column_names: &[Ident],
    row_values: &[impl ToTokens],
) -> TokenStream {
    let pairs = column_names
        .iter()
        .map(|column_name| format!("{column_name} : {{}}"))
        .join(", ");
    let format_string = format!("{{{{ {pairs} }}}}");

    quote! { format!(#format_string, #(#row_values),*) }
}

/// `format!("{{ column : {} }}", value)` for one column.
pub fn single_column_and_value(column_name: &Ident, value: &impl ToTokens) -> TokenStream {
    column_names_and_row_values(std::slice::from_ref(column_name), &[value])
}

/// `"{ id : 0 }"`: a singleton's injected primary key and its only value. Both are known at
/// generation time, so the text is a plain literal.
pub fn singleton_primary_key() -> TokenStream {
    let text = format!(
        "{{ {} : {} }}",
        singleton::PRIMARY_KEY_NAME,
        singleton::rendered_primary_key_value()
    );

    quote! { #text }
}
