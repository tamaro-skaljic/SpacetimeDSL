//! Snapshots of the exact tokens the generator emits for every table shape the
//! codebase supports, so any change to the generated output shows up as a reviewable
//! diff instead of a silent behavioral change.
//!
//! Each fixture in `tests/fixtures` isolates one feature and names the branch it covers.
//! Its snapshots live in `tests/snapshots/<fixture>/<StructName>`: `table.snap` holds
//! everything the macro emits that is not a DSL method, plus a manifest of the generated
//! method names, one `<method_name>.snap` holds each public DSL method, and
//! `internal_methods.snap` holds all internal DSL methods together. A struct carrying
//! more than one `#[dsl]` attribute is expanded once per attribute and snapshotted into
//! a `pass_<n>` directory per expansion.
//!
//! Run `cargo insta review` to inspect and accept changed snapshots.

use std::{collections::BTreeSet, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::ToTokens;
use rust_format::{Formatter, PrettyPlease};
use syn::{Attribute, DeriveInput, Item};

use crate::{ExpandedDSLAttribute, expand_dsl_attribute_parts, output::GeneratedOutput};

#[test]
fn plain_table() {
    snapshot_fixture("plain_table");
}

#[test]
fn unique_single_column_index() {
    snapshot_fixture("unique_single_column_index");
}

#[test]
fn non_unique_single_column_index() {
    snapshot_fixture("non_unique_single_column_index");
}

#[test]
fn unique_multi_column_index() {
    snapshot_fixture("unique_multi_column_index");
}

#[test]
fn non_unique_multi_column_index() {
    snapshot_fixture("non_unique_multi_column_index");
}

#[test]
fn direct_index() {
    snapshot_fixture("direct_index");
}

#[test]
fn hash_index() {
    snapshot_fixture("hash_index");
}

#[test]
fn string_index_column() {
    snapshot_fixture("string_index_column");
}

#[test]
fn qualified_type_spellings() {
    snapshot_fixture("qualified_type_spellings");
}

#[test]
fn wrapper_created_unnamed() {
    snapshot_fixture("wrapper_created_unnamed");
}

#[test]
fn wrapper_created_named() {
    snapshot_fixture("wrapper_created_named");
}

#[test]
fn wrapper_used() {
    snapshot_fixture("wrapper_used");
}

#[test]
fn wrapper_optional_index() {
    snapshot_fixture("wrapper_optional_index");
}

#[test]
fn singleton() {
    snapshot_fixture("singleton");
}

#[test]
fn singleton_with_default() {
    snapshot_fixture("singleton_with_default");
}

#[test]
fn singleton_with_foreign_key() {
    snapshot_fixture("singleton_with_foreign_key");
}

#[test]
fn foreign_key_and_referenced_by() {
    snapshot_fixture("foreign_key_and_referenced_by");
}

#[test]
fn on_delete_error() {
    snapshot_fixture("on_delete_error");
}

#[test]
fn on_delete_delete() {
    snapshot_fixture("on_delete_delete");
}

#[test]
fn on_delete_set_zero() {
    snapshot_fixture("on_delete_set_zero");
}

#[test]
fn on_delete_ignore() {
    snapshot_fixture("on_delete_ignore");
}

#[test]
fn hooks_all_six() {
    snapshot_fixture("hooks_all_six");
}

#[test]
fn methods_disabled() {
    snapshot_fixture("methods_disabled");
}

#[test]
fn timestamps() {
    snapshot_fixture("timestamps");
}

#[test]
fn update_hook_with_updated_at() {
    snapshot_fixture("update_hook_with_updated_at");
}

#[test]
fn scheduled_table() {
    snapshot_fixture("scheduled_table");
}

#[test]
fn multiple_dsl_attributes() {
    snapshot_fixture("multiple_dsl_attributes");
}

#[test]
fn multiple_table_attributes() {
    snapshot_fixture("multiple_table_attributes");
}

#[test]
fn delete_hooks_with_foreign_key_on_unique_index() {
    snapshot_fixture("delete_hooks_with_foreign_key_on_unique_index");
}

#[test]
fn soft_delete_flag() {
    snapshot_fixture("soft_delete_flag");
}

#[test]
fn soft_delete_hooks() {
    snapshot_fixture("soft_delete_hooks");
}

#[test]
fn soft_delete_timestamp() {
    snapshot_fixture("soft_delete_timestamp");
}

#[test]
fn soft_delete_without_delete_method() {
    snapshot_fixture("soft_delete_without_delete_method");
}

/// Expanding the same fixture twice must produce byte-identical output, otherwise the
/// snapshots above would fail at random and a regenerated module would differ from the
/// previous one for no reason.
///
/// Repeating the expansion inside a single process is enough to catch that: every
/// `HashMap` and `HashSet` draws its own seed from a per-thread counter, so a collection
/// whose iteration order reaches the output reorders it between expansions of the same
/// run - not only between runs.
#[test]
fn expansion_is_deterministic() {
    /// The fixture with the most order-sensitive input: several foreign keys,
    /// several referencing tables and more than one on-delete strategy.
    const FIXTURE_NAME: &str = "foreign_key_and_referenced_by";

    const EXPANSION_COUNT: usize = 50;

    let first_expansion = expand_fixture_to_string(FIXTURE_NAME);

    for expansion_number in 2..=EXPANSION_COUNT {
        assert_eq!(
            expand_fixture_to_string(FIXTURE_NAME),
            first_expansion,
            "expansion {expansion_number} of `{FIXTURE_NAME}.rs` should be identical to the first one"
        );
    }
}

/// One `#[dsl]` expansion of one struct of a fixture.
struct FixtureExpansion {
    struct_name: String,
    /// Which `#[dsl]` attribute of the struct produced this expansion, counted from the
    /// outermost one - which is the one the compiler expands first.
    pass_number: usize,
    /// How many `#[dsl]` attributes the struct carries in total.
    pass_count: usize,
    generated_output: GeneratedOutput,
}

fn snapshot_fixture(fixture_name: &str) {
    for expansion in expand_fixture(fixture_name) {
        let FixtureExpansion {
            struct_name,
            pass_number,
            pass_count,
            generated_output,
        } = expansion;

        // Structs with a single `#[dsl]` attribute - almost all of them - would otherwise
        // get a `pass_1` directory which never has a sibling.
        let snapshot_directory = match pass_count {
            1 => format!("../tests/snapshots/{fixture_name}/{struct_name}"),
            _ => format!("../tests/snapshots/{fixture_name}/{struct_name}/pass_{pass_number}"),
        };

        insta::with_settings!({
            snapshot_path => snapshot_directory,
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            insta::assert_snapshot!("table", table_snapshot(&generated_output, &struct_name));

            for dsl_method in &generated_output.dsl_methods {
                if dsl_method.is_internal {
                    continue;
                }

                insta::assert_snapshot!(
                    dsl_method.method_name.to_string(),
                    format_tokens(&dsl_method.tokens)
                );
            }

            if let Some(internal_methods) = internal_methods_snapshot(&generated_output) {
                insta::assert_snapshot!(INTERNAL_METHODS_SNAPSHOT_NAME, internal_methods);
            }
        });
    }
}

/// Expands every struct of the fixture the way the attribute macro would, once per
/// `#[dsl]` attribute the struct carries.
///
/// Each pass receives the item the previous pass echoed back, exactly as the compiler
/// feeds one attribute macro's output into the next one. The second and later passes
/// therefore see the `derive(SpacetimeDSL)` helper already present, which is what makes
/// `first_dsl_attribute` false and suppresses the wrapper types and the accessors.
fn expand_fixture(fixture_name: &str) -> Vec<FixtureExpansion> {
    let fixture = read_fixture(fixture_name);
    let fixture: syn::File = syn::parse_str(&fixture)
        .unwrap_or_else(|error| panic!("`{fixture_name}.rs` should be parsable Rust: {error}"));

    let mut expansions = vec![];

    for item in fixture.items {
        let Item::Struct(item_struct) = item else {
            continue;
        };

        let mut derive_input: DeriveInput = syn::parse2(item_struct.to_token_stream())
            .unwrap_or_else(|error| panic!("a `struct` item should be a `DeriveInput`: {error}"));

        let struct_name = derive_input.ident.to_string();
        let pass_count = derive_input
            .attrs
            .iter()
            .filter(|attribute| is_dsl_attribute(attribute))
            .count();

        for pass_number in 1..=pass_count {
            let dsl_attribute_args = take_first_dsl_attribute_args(&mut derive_input.attrs)
                .expect("the `#[dsl]` attributes of the struct were just counted");

            let ExpandedDSLAttribute {
                derive_input: echoed_item,
                generated_output,
            } = expand_dsl_attribute_parts(dsl_attribute_args, derive_input.to_token_stream())
                .unwrap_or_else(|error| {
                    panic!(
                        "`{fixture_name}.rs` / `{struct_name}` should expand in pass {pass_number}: {error}"
                    )
                });

            expansions.push(FixtureExpansion {
                struct_name: struct_name.clone(),
                pass_number,
                pass_count,
                generated_output,
            });

            derive_input = echoed_item;
        }
    }

    assert!(
        !expansions.is_empty(),
        "`{fixture_name}.rs` should contain at least one struct with a `#[dsl]` attribute"
    );

    expansions
}

/// Everything the fixture generates, concatenated - the whole output, not only the parts
/// the snapshots split out.
fn expand_fixture_to_string(fixture_name: &str) -> String {
    expand_fixture(fixture_name)
        .into_iter()
        .map(|expansion| expansion.generated_output.into_token_stream().to_string())
        .collect()
}

/// The name of the snapshot that holds every internal DSL method of a struct at once.
///
/// An internal method carries the name of both the referencing and the referenced table on
/// top of a long fixed phrase, so a file per internal method produced paths that no longer
/// fit into the Windows path limit once the repository was checked out into a worktree.
const INTERNAL_METHODS_SNAPSHOT_NAME: &str = "internal_methods";

/// Every internal DSL method of the struct, each one preceded by its name, or `None` when
/// the struct generates no internal method at all.
///
/// The names stay visible in the snapshot itself, so a renamed method still shows up as a
/// diff even though the file name no longer carries it.
fn internal_methods_snapshot(generated_output: &GeneratedOutput) -> Option<String> {
    let internal_methods: String = generated_output
        .dsl_methods
        .iter()
        .filter(|dsl_method| dsl_method.is_internal)
        .map(|dsl_method| {
            let method_name = &dsl_method.method_name;
            let method = format_tokens(&dsl_method.tokens);

            format!("// {method_name}\n{method}\n")
        })
        .collect();

    (!internal_methods.is_empty()).then_some(internal_methods)
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

/// The args of the first `#[dsl]` attribute, removed from the attributes - which is
/// exactly what the item looks like when the attribute macro receives it, because an
/// attribute macro strips only itself.
fn take_first_dsl_attribute_args(attributes: &mut Vec<Attribute>) -> Option<TokenStream> {
    let position = attributes.iter().position(is_dsl_attribute)?;

    let dsl_attribute = attributes.remove(position);

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
