//! Snapshots of the exact tokens the generator emits for every table shape the
//! codebase supports, so any change to the generated output shows up as a reviewable
//! diff instead of a silent behavioral change.
//!
//! Each fixture in `tests/fixtures` isolates one feature and names the branch it covers.
//! Its snapshots live in `tests/snapshots/<fixture>/<StructName>`: `table.snap` holds
//! everything the macro emits that is not a DSL method, plus a manifest of the generated
//! method names, one `<method_name>.snap` holds each public DSL method,
//! `internal_methods.snap` holds all internal DSL methods together, and
//! `wrapper_methods.snap` holds all methods the struct adds to the wrapper types of its
//! foreign key columns. A struct carrying more than one `#[dsl]` attribute is expanded
//! once per attribute and snapshotted into a `pass_<n>` directory per expansion.
//!
//! Run `cargo insta review` to inspect and accept changed snapshots.

use {
    crate::{ExpandedDSLAttribute, expand_dsl_attribute_parts, output::GeneratedOutput},
    proc_macro2::TokenStream,
    quote::ToTokens,
    rust_format::{Formatter, PrettyPlease},
    spacetimedsl_derive_input::api::attribute::{FIELD_ATTRIBUTE_NAMES, is_dsl_attribute},
    std::{
        collections::BTreeSet,
        fs,
        path::{Path, PathBuf},
    },
    syn::{Attribute, DeriveInput, Expr, ExprLit, Item, Lit, Stmt},
};

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
fn wrapper_created_uuid() {
    snapshot_fixture("wrapper_created_uuid");
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
fn singleton_with_uuid_foreign_key() {
    snapshot_fixture("singleton_with_uuid_foreign_key");
}

#[test]
fn table_named_singleton() {
    snapshot_fixture("table_named_singleton");
}

#[test]
fn foreign_key_and_referenced_by() {
    snapshot_fixture("foreign_key_and_referenced_by");
}

#[test]
fn foreign_keys_with_equivalent_spellings() {
    snapshot_fixture("foreign_keys_with_equivalent_spellings");
}

#[test]
fn foreign_key_with_table_level_index() {
    snapshot_fixture("foreign_key_with_table_level_index");
}

#[test]
fn foreign_key_to_table_without_delete_methods() {
    snapshot_fixture("foreign_key_to_table_without_delete_methods");
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
fn on_delete_set_zero_uuid() {
    snapshot_fixture("on_delete_set_zero_uuid");
}

#[test]
fn on_delete_set_zero_with_update_hooks_and_set_on_update() {
    snapshot_fixture("on_delete_set_zero_with_update_hooks_and_set_on_update");
}

#[test]
fn on_delete_ignore() {
    snapshot_fixture("on_delete_ignore");
}

#[test]
fn insert_update_and_delete_hooks() {
    snapshot_fixture("insert_update_and_delete_hooks");
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
fn update_hook_with_set_on_update() {
    snapshot_fixture("update_hook_with_set_on_update");
}

#[test]
fn uuid_auto_gen() {
    snapshot_fixture("uuid_auto_gen");
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
fn wrapper_methods() {
    snapshot_fixture("wrapper_methods");
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

#[test]
fn on_soft_delete_cascade() {
    snapshot_fixture("on_soft_delete_cascade");
}

#[test]
fn on_soft_delete_cascade_with_soft_delete_hooks() {
    snapshot_fixture("on_soft_delete_cascade_with_soft_delete_hooks");
}

#[test]
fn self_referencing_cascade() {
    snapshot_fixture("self_referencing_cascade");
}

#[test]
fn restricted_accessor_visibility() {
    snapshot_fixture("restricted_accessor_visibility");
}

#[test]
fn absolute_attribute_paths() {
    snapshot_fixture("absolute_attribute_paths");
}

#[test]
fn every_field_attribute() {
    snapshot_fixture("every_field_attribute");
}

/// `proc_macro_derive(attributes(...))` needs literal identifiers, so the helper
/// attributes of the `SpacetimeDSL` derive cannot be generated from
/// `FIELD_ATTRIBUTE_NAMES`. A name missing from the helper list surfaces only as an
/// unknown-attribute error in user code, which no snapshot notices, so the two lists are
/// compared here.
#[test]
fn helper_attributes_match_field_attributes() {
    let lib_rs: syn::File =
        syn::parse_str(include_str!("lib.rs")).expect("`lib.rs` should be parsable Rust");

    let proc_macro_derive = lib_rs
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(function) => Some(&function.attrs),
            _ => None,
        })
        .flatten()
        .find(|attribute| attribute.path().is_ident("proc_macro_derive"))
        .expect("`lib.rs` should declare the `SpacetimeDSL` derive");

    let mut helper_attributes = vec![];

    proc_macro_derive
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("attributes") {
                meta.parse_nested_meta(|meta| {
                    helper_attributes.push(
                        meta.path
                            .get_ident()
                            .expect("a helper attribute is a plain identifier")
                            .to_string(),
                    );
                    Ok(())
                })?;
            }
            Ok(())
        })
        .expect("the `proc_macro_derive` arguments should be parsable");

    let mut field_attributes: Vec<String> = FIELD_ATTRIBUTE_NAMES
        .iter()
        .map(ToString::to_string)
        .collect();

    helper_attributes.sort();
    field_attributes.sort();

    assert_eq!(
        helper_attributes, field_attributes,
        "the helper attributes of `#[proc_macro_derive(SpacetimeDSL, attributes(...))]` should be exactly `FIELD_ATTRIBUTE_NAMES`"
    );
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

/// A fixture is only snapshotted through a `#[test]` above that calls `snapshot_fixture`
/// with its name. A fixture without such a test would never run, and the snapshots of a
/// deleted fixture would stay behind, both without anything reporting it.
#[test]
fn every_fixture_is_registered_under_its_own_name() {
    let registrations = fixture_registrations();
    let registered_fixture_names: BTreeSet<&str> = registrations
        .iter()
        .map(|registration| registration.fixture_name.as_str())
        .collect();
    let fixture_names = entry_names_in("tests/fixtures", |path| {
        path.extension().is_some_and(|extension| extension == "rs")
    });
    let snapshot_directory_names = entry_names_in("tests/snapshots", Path::is_dir);

    let unregistered_fixtures: Vec<&String> = fixture_names
        .iter()
        .filter(|fixture_name| !registered_fixture_names.contains(fixture_name.as_str()))
        .collect();
    assert!(
        unregistered_fixtures.is_empty(),
        "every fixture should have a `#[test]` calling `snapshot_fixture` with its name, but these have none: {unregistered_fixtures:?}"
    );

    let snapshot_directories_without_fixture: Vec<&String> = snapshot_directory_names
        .difference(&fixture_names)
        .collect();
    assert!(
        snapshot_directories_without_fixture.is_empty(),
        "every snapshot directory should belong to a fixture of the same name, but these have none: {snapshot_directories_without_fixture:?}"
    );

    let misnamed_registrations: Vec<&FixtureRegistration> = registrations
        .iter()
        .filter(|registration| registration.function_name != registration.fixture_name)
        .collect();
    assert!(
        misnamed_registrations.is_empty(),
        "every `#[test]` calling `snapshot_fixture` should be named after its fixture, but these are not: {misnamed_registrations:?}"
    );

    let registrations_without_fixture: Vec<&str> = registered_fixture_names
        .into_iter()
        .filter(|fixture_name| !fixture_names.contains(*fixture_name))
        .collect();
    assert!(
        registrations_without_fixture.is_empty(),
        "every `snapshot_fixture` call should name an existing fixture, but these do not: {registrations_without_fixture:?}"
    );
}

/// A `#[test]` of this file whose body is a single `snapshot_fixture("<fixture_name>")` call.
#[derive(Debug)]
struct FixtureRegistration {
    function_name: String,
    fixture_name: String,
}

fn fixture_registrations() -> Vec<FixtureRegistration> {
    let this_file = syn::parse_file(include_str!("characterization_tests.rs"))
        .expect("this file should be parsable Rust");

    this_file
        .items
        .iter()
        .filter_map(|item| {
            let Item::Fn(function) = item else {
                return None;
            };
            if !function
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("test"))
            {
                return None;
            }
            let [Stmt::Expr(Expr::Call(call), _)] = function.block.stmts.as_slice() else {
                return None;
            };
            let Expr::Path(callee) = call.func.as_ref() else {
                return None;
            };
            if !callee.path.is_ident("snapshot_fixture") {
                return None;
            }
            let arguments: Vec<&Expr> = call.args.iter().collect();
            let [
                Expr::Lit(ExprLit {
                    lit: Lit::Str(fixture_name),
                    ..
                }),
            ] = arguments.as_slice()
            else {
                return None;
            };

            Some(FixtureRegistration {
                function_name: function.sig.ident.to_string(),
                fixture_name: fixture_name.value(),
            })
        })
        .collect()
}

/// The names, without extension, of the entries of `directory` (relative to this crate)
/// which `keep` accepts.
fn entry_names_in(directory: &str, keep: impl Fn(&Path) -> bool) -> BTreeSet<String> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(directory);

    fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("`{}` should be readable: {error}", directory.display()))
        .map(|entry| {
            entry
                .expect("an entry of a readable directory should be readable")
                .path()
        })
        .filter(|path| keep(path))
        .map(|path| {
            path.file_stem()
                .expect("an entry of a directory should have a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
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

            if !generated_output.wrapper_methods.is_empty() {
                insta::assert_snapshot!(
                    WRAPPER_METHODS_SNAPSHOT_NAME,
                    format_tokens(&generated_output.wrapper_methods)
                );
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

/// The name of the snapshot that holds every wrapper method of a struct at once.
const WRAPPER_METHODS_SNAPSHOT_NAME: &str = "wrapper_methods";

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
