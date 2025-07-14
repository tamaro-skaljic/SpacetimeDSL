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

        // Check if this is the last DSL attribute to avoid generating conflicting wrapper types
        let is_last_dsl_attribute = is_last_dsl_attribute(&args, &derive_input)?;

        let input = Table::try_parse(args, &derive_input)?;

        // Build the output, possibly using quasi-quotation
        let output = output::output(&input, is_last_dsl_attribute)?;

        // TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/38
        Ok(proc_macro2::TokenStream::from_iter([
            quote!(#derive_input),
            output,
        ]))
    })
}

fn derive_table_helper_attr() -> syn::Attribute {
    let source = quote!(#[derive(Clone, Debug, PartialEq, spacetimedsl::SpacetimeDSL)]); // TODO: Add PartialOrd if ScheduledAt has implemented it

    syn::parse::Parser::parse2(syn::Attribute::parse_outer, source)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

/// Provides helper attributes for `#[dsl]` because proc_macro_attribute's currently don't support them.
// TODO: Remove if https://github.com/rust-lang/rust/issues/65823 is implemented.
#[proc_macro_derive(
    SpacetimeDSL,
    attributes(create_wrapper, use_wrapper, foreign_key, referenced_by)
)]
pub fn table_helper(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::default()
}

/// Check if this DSL attribute is the last one on the struct to avoid generating conflicting wrapper types
fn is_last_dsl_attribute(
    current_args: &proc_macro2::TokenStream,
    derive_input: &syn::DeriveInput,
) -> syn::Result<bool> {
    let mut dsl_attributes = Vec::new();
    
    // Find all dsl attributes on the struct
    for attr in &derive_input.attrs {
        if let syn::Meta::List(meta_list) = &attr.meta {
            if meta_list.path.segments.len() == 2 
                && meta_list.path.segments[0].ident == "spacetimedsl"
                && meta_list.path.segments[1].ident == "dsl" {
                dsl_attributes.push(&meta_list.tokens);
            }
        }
    }
    
    // If there's only one DSL attribute, it's the last one
    if dsl_attributes.len() <= 1 {
        return Ok(true);
    }
    
    // Find which DSL attribute this invocation corresponds to by comparing arguments
    let current_args_str = current_args.to_string();
    let mut found_index = None;
    
    for (index, attr_tokens) in dsl_attributes.iter().enumerate() {
        if attr_tokens.to_string() == current_args_str {
            found_index = Some(index);
            break;
        }
    }
    
    // If this is the last DSL attribute (highest index), generate wrapper types
    match found_index {
        Some(index) => Ok(index == dsl_attributes.len() - 1),
        None => Ok(true), // Fallback: if we can't match, generate wrapper types to be safe
    }
}

fn ok_or_compile_error<Res: Into<proc_macro::TokenStream>>(
    f: impl FnOnce() -> syn::Result<Res>,
) -> proc_macro::TokenStream {
    match f() {
        Ok(ok) => ok.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
