use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use spacetimedsl_derive_output as input;
mod output;

/// Add `#[derive(SpacetimeDSL)]` to your structs with `#[spacetimedb::table]`
/// to interact in a more ergonomic way than SpacetimeDB allows you by default.
#[proc_macro_derive(SpacetimeDSL, attributes(plural_table_name, wrap))]
pub fn dsl(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = input::parse(parse_macro_input!(input as DeriveInput));

    // No-op if it's not annotated with `#[spacetimedb::table]`
    if input.is_none() {
        return TokenStream::default();
    }

    // Build the output, possibly using quasi-quotation
    let output = output::output(input.expect("Expected input, found none."));

    // Hand the output tokens back to the compiler
    TokenStream::from(output)
}
