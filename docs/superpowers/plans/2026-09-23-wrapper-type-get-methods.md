# Wrapper Type Get Methods Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every `#[foreign_key]` column with a single-column index adds a lookup method to its `#[use_wrapper]` type, so `entity_id.get_position(&dsl)` returns the rows that reference that wrapper value.

**Architecture:** `derive-input` builds one `WrapperMethod` per foreign key column. It builds them from the column's existing `get_<table>_by_<index>` or `get_<tables>_by_<index>` DSL method, so the names and return types come from one place. `derive` renders each `WrapperMethod` as an `impl <WrapperType> { … }` block, and the characterization tests snapshot it into `wrapper_methods.snap`. The `spacetimedsl!` macro gets two `From` impls, so `&DSL` and `&ReadOnlyDSL` both convert into the `ReadOnlyDSL` the wrapper methods take.

**Tech Stack:** Rust edition 2024 (toolchain pinned in `rust-toolchain.toml`), `syn`/`quote`/`proc-macro2`, `ident_case`, `insta` snapshots, SpacetimeDB 2.7.0 (`examples/test` runtime test).

**Spec:** GitHub issue [#64](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/64), with the decisions below. They were agreed with the maintainer on 2026-09-23 and they override the issue text where the two differ.

## Decisions Agreed With the Maintainer

1. **Many-row return shape.** First try the issue's signature, `-> impl Iterator<Item = Row>` with the body `dsl.into().get_<tables>_by_<index>(self)`. Edition 2024 makes the DSL method's `impl Iterator` capture the `'a` of `&'a self` ([edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html)). So the iterator is expected to borrow the temporary `ReadOnlyDSL`, and compilation is expected to fail with E0515 "cannot return value referencing temporary value". If it fails, return `Vec<Row>` and use the body `dsl.into().get_<tables>_by_<index>(self).collect()`. Task 1 Step 9 is where this is decided.
2. **Context bound.** Use `T: 'a + crate::spacetimedsl::ReadContext` and the parameter `dsl: impl Into<crate::spacetimedsl::ReadOnlyDSL<'a, T>>`. The `spacetimedsl!` macro gets `From<&'a DSL<'a, T>>` and `From<&'a ReadOnlyDSL<'a, T>>` for `ReadOnlyDSL<'a, T>`. Reducers pass `&dsl`, and views pass `&read_only_dsl`.
3. **Self-referencing foreign key.** When the referenced table is the table itself, always use the long name.
4. **Naming.** Group the foreign key columns of one table by `foreign_key(table = …)`.
   - If exactly one column references another table, use the short name: `get_<singular>` for a unique index (`#[primary_key]` or `#[unique]`) and `get_<plural>` for a non-unique one.
   - Otherwise (two or more columns, or a self-reference), use the long name, which is the exact DSL method name, for example `get_shipments_by_origin_warehouse_id`.
5. **Singletons.** A singleton's foreign key columns have no index, so they get no method.
6. **Doc comment.** Write one sentence, then the usage line. For example: ``Get the `Position` whose `entity_id` column references this `EntityId`.`` / ``Use it like `entity_id.get_position(&dsl)`.``
7. **Data model.** Add a new API struct `WrapperMethod` and a new field `SpacetimeDSLTableMethods::wrapper_methods`. The generator goes in its own module, and the renderer goes in its own module.
8. **Snapshots.** Snapshot all wrapper methods of a struct into one `wrapper_methods.snap`, and only when the struct has wrapper methods.
9. **New fixture.** `derive/tests/fixtures/wrapper_methods.rs` covers the self-reference, a foreign key on the primary key, a qualified wrapper path and a struct with two `#[dsl]` passes.
10. **Runtime proof.** Add calls and assertions to the `tester` reducer and to the `my_view` view in `examples/test`.
11. **Docs.** Add a new subsection to `docs/DOCUMENTATION.md`, update the two existing foreign key examples, and add one `README.md` bullet.
12. **Commits.** Make one commit per task, in the repository style (`Add …`). End each commit message with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
13. **Argument.** Pass `self` to the DSL method, as the issue shows. This works with every wrapper SpacetimeDSL generates, through the generated `From<&W> for W`.

## Global Constraints

- Toolchain: the exact version in `rust-toolchain.toml`. Edition `2024`. Do not add dependencies.
- Every `crate::spacetimedsl::…` path in generated code comes from `derive-input/src/api/runtime.rs`. Do not spell a runtime path in a `quote!` anywhere else.
- `AGENTS.md`: no abbreviations in identifiers. Comments state only "what" or "why" when the code does not show it.
- Generate nothing for multi-column indices, whether they are unique or not.
- Generated signature: `pub fn <name><'a, T: 'a + crate::spacetimedsl::ReadContext>(&self, dsl: impl Into<crate::spacetimedsl::ReadOnlyDSL<'a, T>>) -> <return type>`.
- Snapshots: accept them only after you read every new or changed `.snap` file. Nothing outside the files listed in each task may change.
- On Windows, use `.\x.ps1`. On Linux and macOS, use `./x.sh`. Both are generated by `gen-x.sh`, so do not edit them.

## Review Focus

1. **The many-row method in a real edition-2024 crate.** Snapshots do not compile the code, so the E0515 risk shows only in `examples/test`. Covered by Task 1 Step 9 (`cargo check -p spacetimedsl_test`) and by the calls in Task 2.
2. **A view that holds only a `ReadOnlyDSL`.** It must be able to call `wrapper.get_…(&dsl)`. Covered by the `my_view` calls in Task 2.
3. **A qualified wrapper path from another module** (`#[use_wrapper(crate::entity::EntityId)]`). It must produce `impl crate::entity::EntityId { … }`, and that must compile. Covered by the `AccountNote` snapshot (Task 1) and by `examples/test` `Position`/`UniquePosition` (Task 1 Step 9 and Task 2).
4. **Two tables that each have one foreign key to the same wrapper.** Both must get short names that do not collide (`get_position` and `get_unique_position` on `EntityId`). Covered by the Task 2 assertions.
5. **One struct with two `#[dsl]` passes and a foreign key.** Each pass must add the method for its own table. Covered by the `Membership` fixture (`pass_1` and `pass_2` snapshots) in Task 1.

---

### Task 1: Generate the wrapper methods and snapshot them

**Files:**

- Create: `derive/tests/fixtures/wrapper_methods.rs`
- Create: `derive-input/src/internal/dsl/method/wrapper_method.rs`
- Create: `derive/src/output/wrapper_method.rs`
- Modify: `derive-input/src/api/dsl/wrapper.rs` (new struct `WrapperMethod`)
- Modify: `derive-input/src/api/dsl/table.rs` (field `SpacetimeDSLTableMethods::wrapper_methods`)
- Modify: `derive-input/src/internal/dsl/wrapper.rs` (helper `WrapperType::struct_name`)
- Modify: `derive-input/src/internal/dsl/method.rs` (call the generator in `SpacetimeDSLTableMethods::generate`, lines 222-311)
- Modify: `derive-input/src/api/runtime.rs` (new `read_only_dsl_type_with_lifetime`)
- Modify: `derive/src/output.rs` (field `GeneratedOutput::wrapper_methods`, render call)
- Modify: `derive/src/characterization_tests.rs` (new test, new snapshot kind, module doc)
- Test: new `wrapper_methods.snap` files under `derive/tests/snapshots/**`

**Interfaces:**

- Consumes: `SpacetimeDSLColumnMethods::{ForUniqueIndex, ForIndex}` (`derive-input/src/api/dsl/column.rs`). `SpacetimeDSLMethod { method_name, return_type, .. }` (`derive-input/src/api/dsl/method.rs`). `MethodGenerationContext { struct_name, singular_table_name, plural_table_name, .. }` (`derive-input/src/internal/dsl/method/context.rs`). `WrapperType::map(&WrapperType) -> syn::Type` (`derive-input/src/internal/dsl/wrapper.rs:207`). `runtime::read_context()`.
- Produces:
  - `pub struct WrapperMethod { pub wrapper_type: syn::Type, pub doc_comment: String, pub method_name: syn::Ident, pub return_type: TokenStream, pub method_impl: TokenStream }` in `spacetimedsl_derive_input::api::dsl::wrapper`.
  - `SpacetimeDSLTableMethods::wrapper_methods: Vec<WrapperMethod>`.
  - `runtime::read_only_dsl_type_with_lifetime(lifetime: &impl ToTokens) -> TokenStream`.
  - `GeneratedOutput::wrapper_methods: TokenStream`.

- [ ] **Step 1: Write the new fixture**

Create `derive/tests/fixtures/wrapper_methods.rs`:

```rust
//! Covers the lookup methods a `#[foreign_key]` column adds to its wrapper type, in the
//! shapes the other fixtures with foreign keys do not reach:
//!
//! - `Category` references its own table, so its method takes the long name although only
//!   one column references the table: `category_id.get_categories()` would read like a
//!   lookup of the category itself.
//! - `Profile` carries its foreign key on the primary key, a one-to-one relationship.
//! - `AccountNote` names its wrapper by a qualified path, so the `impl` block is emitted on
//!   the path while the doc comment names the last segment.
//! - `Membership` is expanded once per `#[dsl]` attribute, and each pass adds the method for
//!   its own table.
//!
//! The fixture is only expanded, never compiled, so `crate::accounts` does not have to exist.

#[spacetimedsl::dsl(plural_name = accounts, method(update = true))]
#[spacetimedb::table(accessor = account, public)]
pub struct Account {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(AccountId)]
    #[referenced_by(path = self, table = profile)]
    #[referenced_by(path = self, table = account_note)]
    #[referenced_by(path = self, table = active_membership)]
    #[referenced_by(path = self, table = expired_membership)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = categories, method(update = true))]
#[spacetimedb::table(accessor = category, public)]
pub struct Category {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = category)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CategoryId)]
    #[foreign_key(path = self, table = category, column = id, on_delete = Error)]
    pub parent_category_id: u64,
}

#[spacetimedsl::dsl(plural_name = profiles, method(update = true))]
#[spacetimedb::table(accessor = profile, public)]
pub struct Profile {
    #[primary_key]
    #[use_wrapper(AccountId)]
    #[foreign_key(path = self, table = account, column = id, on_delete = Error)]
    account_id: u64,

    pub display_name: String,
}

#[spacetimedsl::dsl(plural_name = account_notes, method(update = true))]
#[spacetimedb::table(accessor = account_note, public)]
pub struct AccountNote {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(crate::accounts::AccountId)]
    #[foreign_key(path = crate::accounts, table = account, column = id, on_delete = Error)]
    pub account_id: u64,
}

#[spacetimedsl::dsl(plural_name = active_memberships, method(update = true))]
#[spacetimedb::table(accessor = active_membership, public)]
#[spacetimedsl::dsl(plural_name = expired_memberships, method(update = true))]
#[spacetimedb::table(accessor = expired_membership, public)]
pub struct Membership {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[use_wrapper(AccountId)]
    #[foreign_key(path = self, table = account, column = id, on_delete = Error)]
    pub account_id: u64,
}
```

If a struct in this fixture does not expand (the test panics with "should expand in pass"), stop and report the error. Do not remove the struct.

- [ ] **Step 2: Register the fixture and the new snapshot kind in the test harness**

In `derive/src/characterization_tests.rs`, add the test after `delete_hooks_with_foreign_key_on_unique_index`:

```rust
#[test]
fn wrapper_methods() {
    snapshot_fixture("wrapper_methods");
}
```

Add this constant after `INTERNAL_METHODS_SNAPSHOT_NAME`:

```rust
/// The name of the snapshot that holds every wrapper method of a struct at once.
///
/// A wrapper method which takes the long name has the same name as the DSL method it calls,
/// so a file per wrapper method would overwrite the snapshot of that DSL method.
const WRAPPER_METHODS_SNAPSHOT_NAME: &str = "wrapper_methods";
```

In `snapshot_fixture`, inside the `insta::with_settings!` block, after the `internal_methods` assertion:

```rust
            if !generated_output.wrapper_methods.is_empty() {
                insta::assert_snapshot!(
                    WRAPPER_METHODS_SNAPSHOT_NAME,
                    format_tokens(&generated_output.wrapper_methods)
                );
            }
```

In the module doc comment at the top, replace the sentence that begins "Its snapshots live in" (lines 6-11) with:

```rust
//! Its snapshots live in `tests/snapshots/<fixture>/<StructName>`: `table.snap` holds
//! everything the macro emits that is not a DSL method or a wrapper method, plus a manifest
//! of the generated method names, one `<method_name>.snap` holds each public DSL method,
//! `internal_methods.snap` holds all internal DSL methods together, and
//! `wrapper_methods.snap` holds all methods the struct adds to the wrapper types of its
//! foreign key columns. A struct carrying more than one `#[dsl]` attribute is expanded
//! once per attribute and snapshotted into a `pass_<n>` directory per expansion.
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p spacetimedsl_derive`
Expected: compile error E0609 `no field`wrapper_methods` on type `GeneratedOutput``.

- [ ] **Step 4: Add the API struct and the table field**

In `derive-input/src/api/dsl/wrapper.rs`, change the `syn` import to `use syn::{Ident, Path, Type};` and append:

```rust
/// A method on the wrapper type of a foreign key column which looks up the rows that
/// reference one value of that wrapper, like `entity_id.get_position(&dsl)`.
///
/// `method_impl` reads the DSL from the argument `dsl`, which the code that renders this
/// method declares.
#[derive(Clone)]
pub struct WrapperMethod {
    pub wrapper_type: Type,
    pub doc_comment: String,
    pub method_name: Ident,
    pub return_type: TokenStream,
    pub method_impl: TokenStream,
}
```

In `derive-input/src/api/dsl/table.rs`, import `crate::api::dsl::wrapper::WrapperMethod` (extend the existing `use crate::api::dsl::{…}` list with `wrapper::WrapperMethod`). Then add the last field of `SpacetimeDSLTableMethods`:

```rust
    /// Methods this table adds to the wrapper types of its foreign key columns.
    pub wrapper_methods: Vec<WrapperMethod>,
```

- [ ] **Step 5: Add `WrapperType::struct_name`**

In `derive-input/src/internal/dsl/wrapper.rs`, inside the second `impl WrapperType` block (after `struct_name_or_path_tokens`):

```rust
    /// The wrapper's own name without its module path, as doc comments name it.
    pub(in crate::internal) fn struct_name(&self) -> Ident {
        match self {
            WrapperType::Created(created_wrapper) => created_wrapper.wrapper_struct_name.clone(),
            WrapperType::Used(used_wrapper) => used_wrapper
                .wrapper_struct_name_or_path
                .segments
                .last()
                .expect("A parsed path always has a last segment")
                .ident
                .clone(),
        }
    }
```

- [ ] **Step 6: Write the generator**

Create `derive-input/src/internal/dsl/method/wrapper_method.rs`:

```rust
//! The methods a table adds to the wrapper types of its foreign key columns.
//!
//! Each one looks rows up through the DSL method of the column's index, so the lookup, its
//! name and its return type stay defined once, in `get.rs`.

use super::context::MethodGenerationContext;
use crate::api::{
    Column,
    dsl::{
        column::SpacetimeDSLColumnMethods,
        wrapper::{WrapperMethod, WrapperType},
    },
};
use ident_case::RenameRule;
use quote::{format_ident, quote};
use syn::Ident;

/// One method per column of `columns_with_foreign_key`, which all reference
/// `referenced_table_name`.
///
/// The short name `get_<table>` or `get_<tables>` is only clear when one column references
/// another table. Two such columns would add the same name twice to the same wrapper type,
/// and a column which references its own table would read like a lookup of the wrapper's
/// own row. Both take the name of the DSL method instead, which names the index.
pub(in crate::internal) fn for_wrapper_methods(
    referenced_table_name: &Ident,
    columns_with_foreign_key: &[&Column],
    context: &MethodGenerationContext,
) -> Vec<WrapperMethod> {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    let takes_name_of_dsl_method =
        referenced_table_name == singular_table_name || columns_with_foreign_key.len() > 1;

    columns_with_foreign_key
        .iter()
        .filter_map(|column| {
            // A singleton's foreign key columns carry no index, so there is no DSL method
            // to look their rows up through.
            let column_methods = column.spacetimedsl_methods.as_ref()?;

            let wrapper_type = column.spacetimedsl_column.wrapper_type.as_ref().expect(
                "`internal/dsl/column.rs` rejects a `#[foreign_key]` column without `#[use_wrapper]`",
            );

            let (dsl_method, short_method_name, rows_found) = match column_methods {
                SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => (
                    &methods.get_one_option,
                    format_ident!("get_{singular_table_name}"),
                    format!("the `{struct_name}`"),
                ),
                SpacetimeDSLColumnMethods::ForIndex(methods) => (
                    &methods.get_many,
                    format_ident!("get_{plural_table_name}"),
                    format!("all `{struct_name}` rows"),
                ),
            };

            let method_name = match takes_name_of_dsl_method {
                true => dsl_method.method_name.clone(),
                false => short_method_name,
            };

            let dsl_method_name = &dsl_method.method_name;
            let column_name = &column.rust_field.name;
            let wrapper_struct_name = wrapper_type.struct_name();
            let wrapper_variable_name =
                RenameRule::SnakeCase.apply_to_variant(wrapper_struct_name.to_string());

            Some(WrapperMethod {
                wrapper_type: WrapperType::map(wrapper_type),
                doc_comment: format!(
                    "Get {rows_found} whose `{column_name}` column references this `{wrapper_struct_name}`.\n\nUse it like `{wrapper_variable_name}.{method_name}(&dsl)`."
                ),
                method_name,
                return_type: dsl_method.return_type.clone(),
                method_impl: quote! {
                    dsl.into().#dsl_method_name(self)
                },
            })
        })
        .collect()
}
```

This is the first-attempt form (decision 1): it keeps the DSL method's return type for both index kinds. Step 9 may change the `ForIndex` arm.

- [ ] **Step 7: Call the generator from `SpacetimeDSLTableMethods::generate`**

In `derive-input/src/internal/dsl/method.rs`:

- Add `mod wrapper_method;` to the module list, in alphabetical order after `mod upsert;`.
- Add `use wrapper_method::for_wrapper_methods;` after `use upsert::for_singleton_upsert;`.
- Add the struct import: extend `dsl::{ … }` so that it also imports `wrapper::WrapperMethod`. Only do this if the compiler needs it. The `vec![]` type is inferred, so the import is probably unnecessary.
- After `let mut on_delete_strategies_of_this_table = vec![];` add `let mut wrapper_methods = vec![];`.
- At the end of the body of `for (referenced_table_name, columns_with_foreign_key) in columns_with_foreign_keys_by_table { … }`, after the `on_delete_strategies_of_this_table.push(…)`, add:

```rust
                wrapper_methods.extend(for_wrapper_methods(
                    referenced_table_name,
                    &columns_with_foreign_key,
                    context,
                ));
```

- Add `wrapper_methods,` as the last field in the `SpacetimeDSLTableMethods { … }` literal.

- [ ] **Step 8: Render the methods**

In `derive-input/src/api/runtime.rs`, after `read_only_dsl_type`:

```rust
/// `ReadOnlyDSL<#lifetime, T>`, the DSL a wrapper method reads through, for a lifetime the
/// method declares itself.
pub fn read_only_dsl_type_with_lifetime(lifetime: &impl ToTokens) -> TokenStream {
    quote! {
        crate::spacetimedsl::ReadOnlyDSL<#lifetime, T>
    }
}
```

Create `derive/src/output/wrapper_method.rs`:

```rust
use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::{dsl::wrapper::WrapperMethod, runtime};

/// One `impl` block per method, because two tables can add methods to the same wrapper type
/// without knowing each other.
pub fn build(wrapper_method: &WrapperMethod) -> TokenStream {
    let WrapperMethod {
        wrapper_type,
        doc_comment,
        method_name,
        return_type,
        method_impl,
    } = wrapper_method;

    let read_context = runtime::read_context();
    let read_only_dsl_type = runtime::read_only_dsl_type_with_lifetime(&quote! { 'a });

    quote! {
        impl #wrapper_type {
            #[doc = #doc_comment]
            pub fn #method_name<'a, T: 'a + #read_context>(
                &self,
                dsl: impl Into<#read_only_dsl_type>,
            ) -> #return_type {
                #method_impl
            }
        }
    }
}
```

In `derive/src/output.rs`:

- Add `mod wrapper_method;` after `mod hook;`.
- Change the doc comment of `items_outside_dsl_methods` to: `/// Compile-error checks, wrapper types, the accessor`impl`, the create-argument struct and the hook traits - everything the macro emits that is neither a DSL method nor a wrapper method.`
- Add the last field of `GeneratedOutput`:

```rust
    /// The `impl` blocks which add lookup methods to the wrapper types of foreign key columns.
    pub wrapper_methods: TokenStream,
```

- In `into_token_stream`, add `let wrapper_methods = self.wrapper_methods;` and emit `#wrapper_methods` after `#(#dsl_methods)*`.
- In `build`, before `Ok(GeneratedOutput {`:

```rust
    let wrapper_methods = input
        .spacetimedsl_methods
        .wrapper_methods
        .iter()
        .map(wrapper_method::build)
        .collect();
```

  Then add `wrapper_methods,` to the `GeneratedOutput { … }` literal.

- [ ] **Step 9: Compile the generated code in a real edition-2024 crate and apply decision 1**

Run: `cargo check -p spacetimedsl_test`

`examples/test` has `EntityRelationship`, which has two non-unique foreign keys to `entity`. So it generates `EntityId::get_entity_relationships_by_parent_entity_id` in the first-attempt `impl Iterator` form.

- **If the check passes:** keep `impl Iterator`. Write the result, with the rustc version, in the commit message body.
- **If it fails with E0515 (expected)**, the error points at `dsl.into().get_entity_relationships_by_…(self)`. In `wrapper_method.rs`, replace everything from `let (dsl_method, short_method_name, rows_found) = match column_methods {` down to and including the `Some(WrapperMethod { … })` expression with the code below. The unique arm keeps its return type and body. Only the many-row arm collects:

```rust
            let (dsl_method, short_method_name, rows_found, return_type, collect_rows) =
                match column_methods {
                    SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => (
                        &methods.get_one_option,
                        format_ident!("get_{singular_table_name}"),
                        format!("the `{struct_name}`"),
                        methods.get_one_option.return_type.clone(),
                        TokenStream::default(),
                    ),
                    // The DSL method's iterator borrows the `ReadOnlyDSL` this method creates,
                    // which does not outlive the call (edition 2024 lifetime capture), so the
                    // rows are collected here.
                    SpacetimeDSLColumnMethods::ForIndex(methods) => (
                        &methods.get_many,
                        format_ident!("get_{plural_table_name}"),
                        format!("all `{struct_name}` rows"),
                        quote! { Vec<#struct_name> },
                        quote! { .collect() },
                    ),
                };

            let method_name = match takes_name_of_dsl_method {
                true => dsl_method.method_name.clone(),
                false => short_method_name,
            };

            let dsl_method_name = &dsl_method.method_name;
            let column_name = &column.rust_field.name;
            let wrapper_struct_name = wrapper_type.struct_name();
            let wrapper_variable_name =
                RenameRule::SnakeCase.apply_to_variant(wrapper_struct_name.to_string());

            Some(WrapperMethod {
                wrapper_type: WrapperType::map(wrapper_type),
                doc_comment: format!(
                    "Get {rows_found} whose `{column_name}` column references this `{wrapper_struct_name}`.\n\nUse it like `{wrapper_variable_name}.{method_name}(&dsl)`."
                ),
                method_name,
                return_type,
                method_impl: quote! {
                    dsl.into().#dsl_method_name(self)#collect_rows
                },
            })
```

  Add `use proc_macro2::TokenStream;` to the imports of `wrapper_method.rs`. Then run `cargo check -p spacetimedsl_test` again. Expected: success.

Any other error: stop and report it with the shortest decisive line.

- [ ] **Step 10: Generate the snapshots and read every one of them**

Run (PowerShell): `$env:INSTA_UPDATE='always'; cargo test -p spacetimedsl_derive; Remove-Item Env:INSTA_UPDATE`
Then run: `git status --porcelain derive/tests/snapshots`

Expected: only new `wrapper_methods.snap` files. No existing `.snap` changes. There is a new file in each directory below:

- `delete_hooks_with_foreign_key_on_unique_index/ChildMarker`: `impl ParentRecordId { pub fn get_child_marker … -> Result<ChildMarker, crate::spacetimedsl::error::SpacetimeDSLError> { dsl.into().get_child_marker_by_parent_id(self) } }`
- `foreign_key_and_referenced_by/Shipment`: two `impl WarehouseId` blocks, `get_shipments_by_origin_warehouse_id` and then `get_shipments_by_destination_warehouse_id`
- `foreign_key_and_referenced_by/Inspection`: `impl WarehouseId { pub fn get_inspections … }`
- `on_delete_error/Book`, `on_delete_delete/Book`, `on_delete_set_zero/Book`, `on_delete_ignore/Book`: `impl AuthorId { pub fn get_books … }`
- `wrapper_methods/Category`: `impl CategoryId { pub fn get_categories_by_parent_category_id … }`
- `wrapper_methods/Profile`: `impl AccountId { pub fn get_profile … { dsl.into().get_profile_by_account_id(self) } }`
- `wrapper_methods/AccountNote`: `impl crate::accounts::AccountId { pub fn get_account_notes … }`, with a doc comment that says ``this `AccountId` `` and ``account_id.get_account_notes(&dsl)``
- `wrapper_methods/Membership/pass_1`: `impl AccountId { pub fn get_active_membership … { dsl.into().get_active_membership_by_account_id(self) } }`
- `wrapper_methods/Membership/pass_2`: the same with `expired_membership`

The other `wrapper_methods/*` snapshots (`table.snap`, `<method>.snap`, `internal_methods.snap`) are new too, because the fixture is new. Other fixtures that contain a `#[foreign_key]` column may also get a new `wrapper_methods.snap`. Check each against decisions 3-5. `singleton_with_foreign_key/*` must NOT get one.

Read each new `wrapper_methods.snap` and check:

- The signature matches the Global Constraints.
- The return type matches Step 9.
- The doc comment matches decision 6.

- [ ] **Step 11: Run the full unit suite, the formatter and clippy**

Run: `.\x.ps1 unit-test`
Expected: both `cargo test -p spacetimedsl_derive` and `cargo test -p spacetimedsl-compile-tests` pass, with no pending `.snap.new` files. The trybuild `.stderr` files must not change. If one changes, stop and report it.

Run: `cargo fmt --all` and then `cargo clippy --workspace --all-targets --all-features`
Expected: no new warnings in `derive`, `derive-input` or `examples/test`.

- [ ] **Step 12: Commit**

```bash
git add derive-input/src derive/src derive/tests/fixtures/wrapper_methods.rs derive/tests/snapshots
git commit -m "Add get methods to wrapper types of foreign key columns

<one line: which return type Step 9 kept and why>

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Let `&DSL` and `&ReadOnlyDSL` convert into `ReadOnlyDSL`, and prove the methods at runtime

**Files:**

- Modify: `src/lib.rs` (inside `macro_rules! spacetimedsl`, `pub mod spacetimedsl`, after the `impl<T: ::spacetimedsl::ReadContext> ReadOnlyDSL<'_, T>` block, lines 147-157)
- Modify: `examples/test/src/lib.rs` (`test::my_view` at line 1402, `test::tester` at line 1435)

**Interfaces:**

- Consumes: from Task 1, the generated methods `EntityId::get_entity_relationships_by_parent_entity_id`, `EntityId::get_entity_relationships_by_child_entity_id`, `EntityId::get_position` and `EntityId::get_unique_position`. Each takes `dsl: impl Into<ReadOnlyDSL<'a, T>>`. The many-row ones return `impl Iterator<Item = EntityRelationship>` or `Vec<EntityRelationship>`, from Task 1 Step 9.
- Produces: `impl<'a, T: WriteContext + ReadContext> From<&'a DSL<'a, T>> for ReadOnlyDSL<'a, T>` and `impl<'a, T: ReadContext> From<&'a ReadOnlyDSL<'a, T>> for ReadOnlyDSL<'a, T>`.

- [ ] **Step 1: Write the failing usage**

In `examples/test/src/lib.rs`, in `my_view`, before `dsl.get_entity_by_obj_id(EntityId::new(0)).ok()`:

```rust
        let _ = EntityId::new(0).get_position(&dsl);
        let _ = EntityId::new(0)
            .get_entity_relationships_by_parent_entity_id(&dsl)
            .into_iter()
            .count();
```

In `tester`, directly after the block that returns `Err("Count of entity relationships should be 3!"…)` (around line 1537-1539). At that point `player` is the parent of `player2` and `player3`, and `player2` is the parent of `player3`:

```rust
        if player
            .get_obj_id()
            .get_entity_relationships_by_parent_entity_id(&dsl)
            .into_iter()
            .count()
            .ne(&2)
        {
            return Err(
                "EntityId::get_entity_relationships_by_parent_entity_id should find the 2 relationships whose parent is the player!"
                    .to_string(),
            );
        }

        if player3
            .get_obj_id()
            .get_entity_relationships_by_child_entity_id(&dsl)
            .into_iter()
            .count()
            .ne(&2)
        {
            return Err(
                "EntityId::get_entity_relationships_by_child_entity_id should find the 2 relationships whose child is player3!"
                    .to_string(),
            );
        }
```

Directly after the `match` that binds `mut player_position` from `dsl.create_position(CreatePosition { entity_id: player.get_obj_id(), … })` (around line 1823), and before any later update of that row:

```rust
        if player
            .get_obj_id()
            .get_position(&dsl)?
            .get_id()
            .ne(&player_position.get_id())
        {
            return Err(
                "EntityId::get_position should find the Position of the player!".to_string(),
            );
        }
```

Directly after the `match` that binds `mut unique_player_position` from `dsl.create_unique_position(CreateUniquePosition { entity_id: player.get_obj_id(), … })` (around line 1924):

```rust
        if player
            .get_obj_id()
            .get_unique_position(&dsl)?
            .get_id()
            .ne(&unique_player_position.get_id())
        {
            return Err(
                "EntityId::get_unique_position should find the UniquePosition of the player!"
                    .to_string(),
            );
        }
```

If a getter name differs from the one used above (`get_id`, `get_obj_id`), use the getter the struct actually generates. Its name is `get_<column>`.

- [ ] **Step 2: Run the check to verify it fails**

Run: `cargo check -p spacetimedsl_test`
Expected: error E0277. `the trait bound`ReadOnlyDSL<'_, _>: From<&DSL<'_, ReducerContext>>`is not satisfied` (in `tester`), and the same for `&ReadOnlyDSL<'_, ViewContext>` (in `my_view`).

- [ ] **Step 3: Add the conversions**

In `src/lib.rs`, inside `pub mod spacetimedsl { … }`, after the `impl<T: ::spacetimedsl::ReadContext> ReadOnlyDSL<'_, T> { … }` block:

```rust
            // Wrapper methods such as `entity_id.get_position(&dsl)` take
            // `impl Into<ReadOnlyDSL>`, so reducers and views can pass the DSL they hold.
            impl<'a, T: ::spacetimedsl::WriteContext + ::spacetimedsl::ReadContext>
                From<&'a DSL<'a, T>> for ReadOnlyDSL<'a, T>
            {
                fn from(dsl: &'a DSL<'a, T>) -> Self {
                    read_only_dsl(dsl.ctx)
                }
            }

            impl<'a, T: ::spacetimedsl::ReadContext> From<&'a ReadOnlyDSL<'a, T>>
                for ReadOnlyDSL<'a, T>
            {
                fn from(dsl: &'a ReadOnlyDSL<'a, T>) -> Self {
                    ReadOnlyDSL {
                        ctx: dsl.ctx,
                        db: dsl.db,
                    }
                }
            }
```

(`dsl.ctx` is the `&'a T` field. The `ctx()` getter would tie the result to the borrow of `dsl` instead.)

- [ ] **Step 4: Run the check to verify it passes**

Run: `cargo check -p spacetimedsl_test`
Expected: success, with no new warnings.

- [ ] **Step 5: Run the runtime test against a local SpacetimeDB**

Prerequisite: a running local SpacetimeDB (`spacetime start` in a second terminal).
Run: `.\x.ps1 test`
Expected: `spacetime call … tester` succeeds. The logs contain none of the four new error messages, and `blackholio` publishes too. Blackholio's `EntityId` and `PlayerId` also get wrapper methods now, so the publish proves that they compile. If `spacetime` is not available, say so in the report. Do not mark this step done.

- [ ] **Step 6: Run the unit suite again**

Run: `.\x.ps1 unit-test`
Expected: pass, with no snapshot changes. `src/lib.rs` is not part of the snapshots.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs examples/test/src/lib.rs
git commit -m "Let DSL and ReadOnlyDSL references convert into ReadOnlyDSL

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Document the wrapper methods

**Files:**

- Modify: `docs/DOCUMENTATION.md` (section "Foreign Keys & Referential Integrity", lines 1222-1395)
- Modify: `README.md` (list "Data Integrity by Construction", lines 212-217)

**Interfaces:**

- Consumes: the final names and return types from Tasks 1 and 2.
- Produces: user documentation only.

- [ ] **Step 1: Add the subsection**

In `docs/DOCUMENTATION.md`, insert a new subsection before `### Critical Rule` (line 1380). Replace `<ManyReturnType>` with `impl Iterator<Item = Position>` or `Vec<Position>`, whichever Task 1 Step 9 kept, and remove the note about `Vec` if the iterator was kept:

````markdown
### Look Up Referencing Rows From a Wrapper

Every `#[foreign_key]` column with a single-column index adds a method to its `#[use_wrapper]` type. The method looks up the rows which reference one value of that wrapper. You do not have to add anything.

```rust
// Position has `#[unique] #[use_wrapper(EntityId)] #[foreign_key(… table = entity …)] entity_id`:
let position: Position = entity.get_id().get_position(&dsl)?;

// Circle has `#[index(btree)] #[use_wrapper(PlayerId)] #[foreign_key(… table = player …)] player_id`:
let circles: <ManyReturnType> = player.get_id().get_circles(&dsl);
```

- A unique column (`#[primary_key]` or `#[unique]`) adds `get_<table>`. It returns `Result<Row, SpacetimeDSLError>`, like `get_<table>_by_<column>`.
- A non-unique column (`#[index]`) adds `get_<tables>`. It returns `<ManyReturnType>`.
- Two or more columns of one table reference the same table, or a column references its own table: each method takes the full name of the DSL method it calls, for example `get_entity_relationships_by_parent_entity_id`.
- Multi-column indices and the foreign key columns of singleton tables add no method.
- Pass `&dsl` from a reducer or `&read_only_dsl` from a view.
````

If Task 1 kept `Vec`, add this line to the list: "- A non-unique column returns a `Vec` rather than the DSL method's iterator, because the method creates the `ReadOnlyDSL` which that iterator would borrow."

- [ ] **Step 2: Name the generated methods in the two existing examples**

In `### Self-Referencing Tables`, after the code block (line 1341), add:

```markdown
`parent_entity_id` references its own table, so `EntityId` gets `get_entities_by_parent_entity_id(&dsl)` - the children of an entity - rather than `get_entities(&dsl)`.
```

In `### Multiple Foreign Keys to Same Table`, after the code block (line 1378), add:

```markdown
Both columns reference `entity`, so `EntityId` gets `get_entity_relationships_by_parent_entity_id(&dsl)` and `get_entity_relationships_by_child_entity_id(&dsl)`.
```

- [ ] **Step 3: Add the README bullet**

In `README.md`, after the bullet that begins "🏷️ Wrapper types" (line 214):

```markdown
- 🧭 Foreign-key columns add lookups to their wrapper types, like `entity_id.get_position(&dsl)`.
```

- [ ] **Step 4: Check the Markdown**

Run: `git diff --stat` and read the rendered diff. Expected: only `docs/DOCUMENTATION.md` and `README.md` change. Every method name in the new text exists in a `wrapper_methods.snap` from Task 1, or in `examples/test`.

- [ ] **Step 5: Commit**

```bash
git add docs/DOCUMENTATION.md README.md
git commit -m "Document get methods on wrapper types

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Out of Scope

- Optional (`Option<…>`) foreign key columns. `OnDeleteStrategy::SetNone` does not exist yet (issue #32), so no current table has this shape.
- Methods for multi-column indices, and for singleton foreign key columns.
- Lazy many-row iteration through a redesigned DSL parameter (options 3 and 4 of the planning discussion). Track it as a follow-up issue if the `Vec` fallback is taken.

## How to Test (after all tasks)

- `.\x.ps1 unit-test`: snapshots and trybuild diagnostics pass.
- `spacetime start`, then `.\x.ps1 test`: the `tester` reducer passes with the new wrapper-method assertions, and `blackholio` publishes.
- Smoke test: in any reducer of `examples/test`, write `player.get_obj_id().get_position(&dsl)?` and hover the method in the IDE. The doc comment reads "Get the `Position` whose `entity_id` column references this `EntityId`."
