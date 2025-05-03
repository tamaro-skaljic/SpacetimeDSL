use quote::quote;
use spacetimedsl_derive_input::api as input;
mod output;

/// Add `#[spacetimedsl::table]` to your structs with `#[spacetimedb::table]`
/// to interact in a more ergonomic way than SpacetimeDB allows you by default.
#[proc_macro_attribute]
pub fn table(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // put this on the struct so we don't get unknown attribute errors
    let derive_table_helper: syn::Attribute = derive_table_helper_attr();

    ok_or_compile_error(|| {
        // Parse the input tokens into syntax trees
        let args: syn::Attribute = syn::parse2(args)?;
        let mut item: syn::DeriveInput = syn::parse2(item)?;

        // Add `derive(SpacetimeDSL)` only if it's not already in the attributes of the item.
        // If multiple `#[spacetimedsl::table]` attributes are applied to the same `struct` item,
        // this will ensure that we don't emit multiple conflicting implementations.
        if !item.attrs.contains(&derive_table_helper) {
            item.attrs.push(derive_table_helper);
        }

        let input = input::api::parse(args, item)?;

        // Build the output, possibly using quasi-quotation
        let output = output::output(input)?;

        Ok(quote! {
            #item
            #output
        })
    })
}

fn derive_table_helper_attr() -> syn::Attribute {
    let source = quote!(#[derive(spacetimedsl::SpacetimeDSL)]);

    syn::parse::Parser::parse2(syn::Attribute::parse_outer, source)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

/// Provides helper attributes for `#[spacetimedsl::table]` because proc_macro_attribute's currently don't support them.
/// TODO: Remove if https://github.com/rust-lang/rust/issues/65823 is implemented.
#[proc_macro_derive(SpacetimeDSL, attributes(wrap))]
pub fn table_helper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
