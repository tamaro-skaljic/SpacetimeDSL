use ident_case::RenameRule;
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
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
        let first_dsl_attribute = if !derive_input.attrs.contains(&derive_table_helper) {
            derive_input.attrs.push(derive_table_helper);
            true
        } else {
            false
        };

        let input = Table::try_parse(args, &derive_input)?;

        // Build the output, possibly using quasi-quotation
        let output = output::output(&input, first_dsl_attribute)?;

        // Check if this is the last #[dsl] attribute by counting remaining ones
        let _is_last_dsl_attribute = is_last_dsl_attribute(&derive_input);

        // If this is the last #[dsl] attribute, make all struct fields private
        // We do this AFTER parsing and generating methods so the setter logic works correctly
        // TODO: Temporarily disabled to allow public primary key columns
        // if is_last_dsl_attribute {
        //     make_struct_fields_private(&mut derive_input);
        // }

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

fn ok_or_compile_error<Res: Into<proc_macro::TokenStream>>(
    f: impl FnOnce() -> syn::Result<Res>,
) -> proc_macro::TokenStream {
    match f() {
        Ok(ok) => ok.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Check if this is the last #[dsl] attribute on the struct.
/// Each attribute removes itself before the macro function runs, so the last one
/// will see 0 remaining DSL attributes in the attributes list.
fn is_last_dsl_attribute(derive_input: &syn::DeriveInput) -> bool {
    // Find all remaining dsl attributes similar to how integration.rs finds table attributes
    let mut dsl_attr_count = 0;

    for attr in &derive_input.attrs {
        // Check for #[dsl(...)] attributes with require_list()
        if let Ok(list) = attr.meta.require_list() {
            let path_string = list.path.to_token_stream().to_string();
            if path_string == "dsl" || path_string == "spacetimedsl :: dsl" {
                dsl_attr_count += 1;
            }
        }
    }

    // If there are 0 dsl attributes left, this is the last one being processed
    dsl_attr_count == 0
}

// TODO: Temporarily disabled to allow public primary key columns
// /// Make all struct fields private by setting their visibility to Inherited,
// /// except for fields with #[primary_key] which preserve their original visibility
// fn make_struct_fields_private(derive_input: &mut syn::DeriveInput) {
//     if let syn::Data::Struct(data_struct) = &mut derive_input.data
//         && let syn::Fields::Named(fields) = &mut data_struct.fields
//     {
//         for field in &mut fields.named {
//             // Check if this field has the #[primary_key] attribute
//             let is_primary_key = field.attrs.iter().any(|attr| {
//                 attr.path().is_ident("primary_key")
//             });
//
//             // Only make non-primary-key fields private
//             if !is_primary_key {
//                 field.vis = syn::Visibility::Inherited;
//             }
//         }
//     }
// }

//region Hooks

/// Add `#[hook]` to your functions to add the trait implementation line required for SpacetimeDSL hooks to work.
#[proc_macro_attribute]
pub fn hook(_args: TokenStream, item: TokenStream) -> TokenStream {
    ok_or_compile_error(|| {
        let function_input: syn::ItemFn = syn::parse(item)?;

        let trait_name = format_ident!(
            "{}Hook",
            RenameRule::PascalCase.apply_to_field(function_input.sig.ident.to_string())
        );

        Ok(quote! {
            impl<T: spacetimedb::DbContext<DbView = spacetimedb::Local> + spacetimedsl::Context> #trait_name<T> for spacetimedsl::DSLMethodHooks {
                #function_input
            }
        })
    })
}

//endregion Hooks
