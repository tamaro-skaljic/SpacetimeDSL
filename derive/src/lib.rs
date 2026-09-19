use ident_case::RenameRule;
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use spacetimedsl_derive_input::api::{Table, runtime};

#[cfg(test)]
mod characterization_tests;
mod output;

/// Add `#[dsl]` to your structs with `#[table]`
/// to interact in a more ergonomic way than SpacetimeDB allows you by default.
#[proc_macro_attribute]
pub fn dsl(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = proc_macro2::TokenStream::from(args);
    let item = proc_macro2::TokenStream::from(item);

    ok_or_compile_error(|| expand_dsl_attribute(args, item))
}

/// The whole `#[dsl]` expansion on [`proc_macro2`] types, so it can be called
/// outside of a procedural macro invocation - by the characterization tests, for example.
fn expand_dsl_attribute(
    args: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let ExpandedDSLAttribute {
        derive_input,
        generated_output,
    } = expand_dsl_attribute_parts(args, item)?;

    Ok(proc_macro2::TokenStream::from_iter([
        quote!(#derive_input),
        generated_output.into_token_stream(),
    ]))
}

/// The expansion before its halves are concatenated, so the characterization tests
/// can snapshot each generated DSL method on its own without parsing the [`Table`] twice.
struct ExpandedDSLAttribute {
    /// The item the macro echoes back, with the `derive(SpacetimeDSL)` helper attached.
    derive_input: syn::DeriveInput,
    generated_output: output::GeneratedOutput,
}

fn expand_dsl_attribute_parts(
    args: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> syn::Result<ExpandedDSLAttribute> {
    // put this on the struct so we don't get unknown attribute errors
    let derive_table_helper = derive_table_helper_attr();

    // Parse the input tokens into a syntax tree
    let mut derive_input: syn::DeriveInput = syn::parse2(item)?;

    // Check if this is a singleton table by scanning args for the `singleton` keyword
    let is_singleton = args.clone().into_iter().any(|token| {
        if let proc_macro2::TokenTree::Ident(ident) = token {
            ident == "singleton"
        } else {
            false
        }
    });

    // For singletons, inject `#[primary_key] id: u8` into the struct
    if is_singleton {
        inject_singleton_primary_key(&mut derive_input)?;
    }

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
    let generated_output = output::build(&input, first_dsl_attribute)?;

    // Check if this is the last #[dsl] attribute by counting remaining ones
    let _is_last_dsl_attribute = is_last_dsl_attribute(&derive_input);

    // If this is the last #[dsl] attribute, make all struct fields private
    // We do this AFTER parsing and generating methods so the setter logic works correctly
    // TODO: Temporarily disabled to allow public primary key columns
    // if is_last_dsl_attribute {
    //     make_struct_fields_private(&mut derive_input);
    // }

    Ok(ExpandedDSLAttribute {
        derive_input,
        generated_output,
    })
}

fn derive_table_helper_attr() -> syn::Attribute {
    let source = quote!(#[derive(Clone, Debug, PartialEq, ::spacetimedsl::SpacetimeDSL)]); // TODO: Add PartialOrd if ScheduledAt has implemented it

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
    attributes(
        create_wrapper,
        use_wrapper,
        foreign_key,
        referenced_by,
        created_at,
        updated_at
    )
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

/// For singleton tables, inject `#[primary_key] id: u8` as the first field.
/// Errors if the user already has a field named `id`.
///
/// The name, the type and the value this field is filled with are spelled out again in
/// `derive-input`'s `internal::dsl::singleton`, which is what generates the DSL methods
/// that read it. The two crates do not share the definition, so a change here needs the
/// same change there.
fn inject_singleton_primary_key(derive_input: &mut syn::DeriveInput) -> syn::Result<()> {
    if let syn::Data::Struct(data_struct) = &mut derive_input.data
        && let syn::Fields::Named(fields) = &mut data_struct.fields
    {
        // Check if user manually defined an `id` field
        for field in fields.named.iter() {
            if let Some(ident) = &field.ident
                && ident == "id"
            {
                return Err(syn::Error::new_spanned(
                    field,
                    "Singleton tables automatically add `#[primary_key] id: u8`. Do not define an `id` field manually!",
                ));
            }
        }

        // Create the field: `#[primary_key] id: u8`
        let pk_field = syn::Field {
            attrs: vec![syn::parse_quote!(#[primary_key])],
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(syn::Ident::new("id", proc_macro2::Span::call_site())),
            colon_token: Some(syn::token::Colon::default()),
            ty: syn::parse_quote!(u8),
        };

        // Insert as the first field
        fields.named.insert(0, pk_field);
    } else {
        return Err(syn::Error::new_spanned(
            &derive_input.ident,
            "Singleton tables must be structs with named fields!",
        ));
    }

    Ok(())
}

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

        let write_context = runtime::write_context();
        let dsl_method_hooks_type = runtime::dsl_method_hooks_type();

        Ok(quote! {
            impl<T: #write_context> #trait_name<T> for #dsl_method_hooks_type {
                #function_input
            }
        })
    })
}

//endregion Hooks
