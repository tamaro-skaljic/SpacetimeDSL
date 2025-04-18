use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod input;
mod output;

// TODO: Multi Column Indices
// TODO: Scheduled Args

// TODO: Use a attribute macro "table" instead of the plural_table_name helper attribute
// TODO: possibly add the derive #[derive(SpacetimeDSL)] through the attribute macro instead of needing the user to automatically annotate it.
// TODO: Remove visibility modifiers through the attribute macro from all struct members AFTER the derive macro has created getters and setters.

// TODO: If https://github.com/rust-lang/rust/issues/105077 is released, generate setters based on that instead of having to generate setters for struct members which are pub / pub(super) / pub(crate) (not for private ones)

// TODO: Use the lib_derive library only as compile-time dependency, use the lib library as runtime dependency

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
    let output = output::output(input.unwrap());

    // Hand the output tokens back to the compiler
    TokenStream::from(output)
}
