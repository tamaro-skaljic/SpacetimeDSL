//! Snapshots of the exact tokens the generator emits for every table shape the
//! codebase supports, so any change to the generated output shows up as a reviewable
//! diff instead of a silent behavioral change.
//!
//! Each fixture in `tests/fixtures` isolates one feature and names the branch it covers.
//! Its snapshots live in `tests/snapshots/<fixture>/<StructName>`: `table.snap` holds
//! everything the macro emits that is not a DSL method, plus a manifest of the generated
//! method names, and one `<method_name>.snap` holds each DSL method.
//!
//! Run `cargo insta review` to inspect and accept changed snapshots.

use std::{collections::BTreeSet, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rust_format::{Formatter, PrettyPlease};
use syn::{Attribute, Item, ItemStruct};

use crate::{ExpandedDSLAttribute, expand_dsl_attribute_parts, output::GeneratedOutput};

#[test]
fn plain_table() {
    snapshot_fixture("plain_table");
}

/// Expands every struct of the fixture the way the attribute macro would
/// and snapshots the generated output of each.
fn snapshot_fixture(fixture_name: &str) {
    let fixture = read_fixture(fixture_name);
    let fixture: syn::File = syn::parse_str(&fixture)
        .unwrap_or_else(|error| panic!("`{fixture_name}.rs` should be parsable Rust: {error}"));

    let mut snapshotted_struct_names = vec![];

    for item in fixture.items {
        let Item::Struct(mut item_struct) = item else {
            continue;
        };

        let Some(dsl_attribute_args) = take_first_dsl_attribute_args(&mut item_struct) else {
            continue;
        };

        snapshotted_struct_names.push(item_struct.ident.to_string());

        snapshot_struct(
            fixture_name,
            &item_struct.ident.to_string(),
            dsl_attribute_args,
            quote!(#item_struct),
        );
    }

    assert!(
        !snapshotted_struct_names.is_empty(),
        "`{fixture_name}.rs` should contain at least one struct with a `#[dsl]` attribute"
    );
}

fn snapshot_struct(
    fixture_name: &str,
    struct_name: &str,
    dsl_attribute_args: TokenStream,
    item: TokenStream,
) {
    let ExpandedDSLAttribute {
        generated_output, ..
    } = expand_dsl_attribute_parts(dsl_attribute_args, item).unwrap_or_else(|error| {
        panic!("`{fixture_name}.rs` / `{struct_name}` should expand: {error}")
    });

    insta::with_settings!({
        snapshot_path => format!("../tests/snapshots/{fixture_name}/{struct_name}"),
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!("table", table_snapshot(&generated_output, struct_name));

        for dsl_method in &generated_output.dsl_methods {
            insta::assert_snapshot!(
                dsl_method.method_name.to_string(),
                format_tokens(&dsl_method.tokens)
            );
        }
    });
}

/// Everything the macro emits that is not a DSL method, followed by a manifest of the
/// generated method names. The manifest makes an added or removed method fail this
/// snapshot instead of only orphaning a file.
fn table_snapshot(generated_output: &GeneratedOutput, struct_name: &str) -> String {
    let method_names: BTreeSet<String> = generated_output
        .dsl_methods
        .iter()
        .map(|dsl_method| dsl_method.method_name.to_string())
        .collect();

    assert_eq!(
        method_names.len(),
        generated_output.dsl_methods.len(),
        "`{struct_name}` should generate each DSL method name only once, otherwise its methods cannot be snapshotted separately"
    );

    let items_outside_dsl_methods = format_tokens(&generated_output.items_outside_dsl_methods);
    let manifest = method_names
        .iter()
        .map(|method_name| format!("// {method_name}\n"))
        .collect::<String>();

    format!("{items_outside_dsl_methods}\n// Generated DSL methods:\n{manifest}")
}

/// The args of the first `#[dsl]` attribute, removed from the item - which is exactly
/// what the item looks like when the attribute macro receives it, because an attribute
/// macro strips only itself.
fn take_first_dsl_attribute_args(item_struct: &mut ItemStruct) -> Option<TokenStream> {
    let position = item_struct.attrs.iter().position(is_dsl_attribute)?;

    let dsl_attribute = item_struct.attrs.remove(position);

    Some(
        dsl_attribute
            .meta
            .require_list()
            .expect("a `#[dsl]` attribute which is a list was just found")
            .tokens
            .clone(),
    )
}

fn is_dsl_attribute(attribute: &Attribute) -> bool {
    let Ok(list) = attribute.meta.require_list() else {
        return false;
    };

    let path = list.path.to_token_stream().to_string();

    path == "dsl" || path == "spacetimedsl :: dsl"
}

fn read_fixture(fixture_name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{fixture_name}.rs"));

    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("`{}` should be readable: {error}", path.display()))
}

fn format_tokens(tokens: &TokenStream) -> String {
    PrettyPlease::default()
        .format_tokens(tokens.clone())
        .unwrap_or_else(|error| {
            panic!("the generated tokens should be formattable Rust: {error}\n\n{tokens}")
        })
}
