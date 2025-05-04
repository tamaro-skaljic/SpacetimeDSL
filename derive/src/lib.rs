use proc_macro::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::Table;
mod output;

/// Add `#[dsl]` to your structs with `#[table]`
/// to interact in a more ergonomic way than SpacetimeDB allows you by default.
#[proc_macro_attribute]
pub fn dsl(args: TokenStream, item: TokenStream) -> TokenStream {
    // put this on the struct so we don't get unknown attribute errors
    let derive_table_helper = derive_table_helper_attr();

    ok_or_compile_error(|| {
        // Parse the input tokens into a syntax tree
        let args = proc_macro2::TokenStream::from(args);
        let mut derive_input: syn::DeriveInput = syn::parse(item)?;

        // Add `derive(SpacetimeDSL)` only if it's not already in the attributes of the item.
        // If multiple `#[dsl]` attributes are applied to the same `struct` item,
        // this will ensure that we don't emit multiple conflicting implementations.
        if !derive_input.attrs.contains(&derive_table_helper) {
            derive_input.attrs.push(derive_table_helper);
        }

        let input = Table::try_parse(args, &derive_input)?;

        // Build the output, possibly using quasi-quotation
        let output = output::output(&input)?;

        Ok(proc_macro2::TokenStream::from_iter([
            quote!(#derive_input),
            output,
        ]))
    })
}

fn derive_table_helper_attr() -> syn::Attribute {
    let source = quote!(#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedsl::SpacetimeDSL)]);

    syn::parse::Parser::parse2(syn::Attribute::parse_outer, source)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

/// Provides helper attributes for `#[dsl]` because proc_macro_attribute's currently don't support them.
// TODO: Remove if https://github.com/rust-lang/rust/issues/65823 is implemented.
#[proc_macro_derive(SpacetimeDSL, attributes(wrap, wrapped, foreign_key))]
pub fn table_helper(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::default()
}

fn ok_or_compile_error<Res: Into<proc_macro::TokenStream>>(
    f: impl FnOnce() -> syn::Result<Res>,
) -> proc_macro::TokenStream {
    match f() {
        Ok(ok) => ok.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
