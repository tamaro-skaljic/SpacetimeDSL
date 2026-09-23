# `#[auto_gen(v4|v7)]` UUID Columns Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a `spacetimedb::Uuid` column carry `#[auto_gen(v4)]` or `#[auto_gen(v7)]` so `create_<table>` fills it with a fresh UUID, and make `#[create_wrapper]` work on `Uuid` columns at all.

**Architecture:** The runtime crate gets a `NewUUID` context trait (`new_uuid_v4`/`new_uuid_v7`) next to `GetTimestamp`. The `Wrapper` trait loses its `Default` bounds; a created wrapper over a `Uuid` column gets no `Default` impl and gains `v4(dsl)`/`v7(dsl)` constructors instead. `#[auto_gen]` is parsed per column into `Option<UUIDVersion>`, validated, and read by the create generator, which emits `let id = Wrapper::v7(self)?.value();`.

**Tech Stack:** Rust 2024, proc-macro (`syn`, `quote`), SpacetimeDB 2.7.0, `insta` snapshots, `trybuild` UI tests, `spacetime` CLI for the end-to-end module test.

**Spec:** GitHub issue #1 "UUID" plus the decisions made in the planning conversation (listed under Global Constraints).

## Global Constraints

- Attribute syntax: exactly `#[auto_gen(v4)]` or `#[auto_gen(v7)]`; bare `#[auto_gen]`, unknown versions and a repeated `#[auto_gen]` are compile errors.
- Allowed column type: `Uuid` or `spacetimedb::Uuid` only (no `Option`).
- An `#[auto_gen]` column must have `#[create_wrapper]` or `#[create_wrapper(Name)]`; missing wrapper and `#[use_wrapper]` are compile errors.
- An `#[auto_gen]` column must be private (inherited visibility).
- `#[auto_gen]` is rejected on `#[dsl(singleton(with_default))]` (it has no create method); allowed on `#[dsl(singleton)]` and ordinary tables.
- Several `#[auto_gen]` columns per table are allowed.
- Context trait name: `NewUUID` (`src/new_uuid.rs`); version enum name: `UUIDVersion { V4, V7 }`.
- Every `#[create_wrapper]` over a `Uuid` column (with or without `#[auto_gen]`) gets `v4`/`v7` and no `Default`; it keeps `Wrapper::new` for wrapping stored values (getters, setters, delete messages, foreign keys).
- `v4`/`v7` signature: `pub fn v4(dsl: &crate::spacetimedsl::DSL<'_, impl crate::spacetimedsl::WriteContext>) -> Result<Self, SpacetimeDSLError>`.
- UUID generation errors map to `SpacetimeDSLError::Error(format!("Failed to generate a UUID v7: {error}"))` (and `v4` alike).
- Docs: `docs/DOCUMENTATION.md` and `README.md` only.
- A shape that SpacetimeDSL cannot handle is fixed in this branch; a shape SpacetimeDB rejects stays commented out with a `FIXME` and an upstream link, and is noted in the docs.
- One commit per task on branch `issue-1-uuid`.

## Review Focus

1. A getter on a UUID wrapper column (`row.get_id()`) must return the stored UUID, never a freshly generated one - pinned by the `tester` reducer comparing `get_*_by_*` results with `create_*` results (Task 4).
2. Two rows created in the same reducer must get different UUIDs, and v7 UUIDs must sort in creation order (v7 shares one counter per reducer call) - pinned in Task 4.
3. A foreign key (`#[use_wrapper]`) pointing at a UUID primary key in another module must still compile, because it uses `Wrapper::new` on the stored value - pinned by the snapshot fixture (Task 3) and the example module (Task 4).
4. A unique multi-column index containing an auto-generated UUID must still run the DSL uniqueness check on the generated value, not on a default - pinned in Task 4.
5. `#[dsl(singleton)]` with a UUID column must generate the UUID on its only create call - pinned in Task 4.

---

### Task 1: Runtime - `NewUUID` trait and `Default`-free `Wrapper`

**Files:**
- Create: `src/new_uuid.rs`
- Modify: `src/lib.rs` (module list, `Context` supertraits, `Wrapper` bounds, prelude)

**Interfaces:**
- Produces: `spacetimedsl::new_uuid::NewUUID` with `fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError>` and `fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError>`; implemented for `ReducerContext`, `TxContext` (Ok) and `ViewContext`, `AnonymousViewContext` (Err). `Context: ... + NewUUID`. `Wrapper<WrappedType: Clone, WrapperType>: Clone + PartialEq + PartialOrd + SpacetimeType + Display`.

- [ ] **Step 1:** Create `src/new_uuid.rs`, same macro layout as `src/get_timestamp.rs`:

```rust
use crate::{ContextType, SpacetimeDSLError};

pub trait NewUUID {
    fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError>;
    fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError>;
}

macro_rules! impl_new_uuid_err {
    ($context:ident, $variant:ident) => {
        impl NewUUID for spacetimedb::$context {
            fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                Err(crate::get_err("UUIDs are only accessible from Reducer Contexts", ContextType::$variant))
            }
            fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                Err(crate::get_err("UUIDs are only accessible from Reducer Contexts", ContextType::$variant))
            }
        }
    };
}

macro_rules! impl_new_uuid_ok {
    ($context:ident) => {
        impl NewUUID for spacetimedb::$context {
            fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                spacetimedb::ReducerContext::new_uuid_v4(self).map_err(|error| {
                    SpacetimeDSLError::Error(format!("Failed to generate a UUID v4: {error}"))
                })
            }
            fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                spacetimedb::ReducerContext::new_uuid_v7(self).map_err(|error| {
                    SpacetimeDSLError::Error(format!("Failed to generate a UUID v7: {error}"))
                })
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_new_uuid_err!(AnonymousViewContext, AnonymousView);
impl_new_uuid_ok!(ReducerContext);
impl_new_uuid_ok!(TxContext);
// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_new_uuid_err!(ViewContext, View);
```

`spacetimedb::ReducerContext::new_uuid_v4(self)` is called as a path so the inherent method is used (and `&TxContext` deref-coerces to `&ReducerContext`).

- [ ] **Step 2:** In `src/lib.rs`: add `pub mod new_uuid;`, `use new_uuid::NewUUID;`, `+ NewUUID` in `Context`, `pub use ::spacetimedsl::new_uuid::NewUUID;` in the prelude, and change `Wrapper` to:

```rust
pub trait Wrapper<WrappedType: Clone, WrapperType>:
    Clone + PartialEq + PartialOrd + spacetimedb::SpacetimeType + Display
```

- [ ] **Step 3:** Run `cargo build -p spacetimedsl` and `cargo test -p spacetimedsl_derive`. Expected: both pass, no snapshot changes.
- [ ] **Step 4:** Commit `Add NewUUID context trait and drop Default from Wrapper bounds`.

### Task 2: `#[create_wrapper]` over `Uuid` columns

**Files:**
- Modify: `derive-input/src/internal/column.rs` (`ColumnTypeKind::UUID`)
- Modify: `derive-input/src/internal/dsl/method/reference_integrity.rs` (exhaustive match)
- Modify: `derive-input/src/internal/dsl/wrapper.rs` (`get_wrapper_impl`)
- Create: `derive/tests/fixtures/wrapper_created_uuid.rs`, test fn in `derive/src/characterization_tests.rs`, snapshots under `derive/tests/snapshots/wrapper_created_uuid/`

**Interfaces:**
- Produces: `ColumnTypeKind::UUID` - `Uuid` bare or `spacetimedb::Uuid`. Generated UUID wrapper: no `impl Default`; `impl Wrapper { pub fn v4(dsl) -> Result<Self, _>; pub fn v7(dsl) -> Result<Self, _> }`.

- [ ] **Step 1:** Write fixture `wrapper_created_uuid.rs` (UUID PK with `#[create_wrapper]`, no `#[auto_gen]`; one `spacetimedb::Uuid` spelled column with `#[unique] #[create_wrapper(ExternalReference)]`) and the `#[test] fn wrapper_created_uuid()`.
- [ ] **Step 2:** Run `cargo test -p spacetimedsl_derive wrapper_created_uuid`; review `table.snap` - it must contain `impl Default` today (the bug).
- [ ] **Step 3:** Extend `ColumnTypeKind::of`: accept `spacetimedb` as root for `Uuid` (`"Uuid" if is_bare || is_rooted_in_spacetimedb => ColumnTypeKind::UUID`). Add `ColumnTypeKind::UUID` to the `String | Other` arm in `reference_integrity.rs`.
- [ ] **Step 4:** In `get_wrapper_impl` take `is_uuid: bool` (from `ColumnTypeKind::of(&rust_field.type_name_or_path) == ColumnTypeKind::UUID`); emit `impl Default` only when `!is_uuid`; when `is_uuid` emit:

```rust
impl #wrapper_struct_name {
    pub fn v4(dsl: &#dsl_type_with_impl_write_context) -> #result_self {
        Ok(Self { value: #new_uuid_trait::new_uuid_v4(dsl.ctx())? })
    }
    pub fn v7(dsl: &#dsl_type_with_impl_write_context) -> #result_self {
        Ok(Self { value: #new_uuid_trait::new_uuid_v7(dsl.ctx())? })
    }
}
```

with the paths from new `runtime.rs` constructors `dsl_type_for_any_write_context()` (`crate::spacetimedsl::DSL<'_, impl crate::spacetimedsl::WriteContext>`) and `new_uuid_trait()` (`crate::spacetimedsl::NewUUID`, re-exported in `spacetimedsl!()`), and `error_result_type(&quote!(Self))`.
- [ ] **Step 5:** `cargo insta test -p spacetimedsl_derive --accept`; read the new snapshots; confirm no other snapshot changed.
- [ ] **Step 6:** Commit `Support #[create_wrapper] on Uuid columns`.

### Task 3: `#[auto_gen(v4|v7)]` parse, validation and create method

**Files:**
- Create: `derive-input/src/api/dsl/auto_gen.rs` (`pub enum UUIDVersion { V4, V7 }`)
- Create: `derive-input/src/internal/dsl/auto_gen.rs` (`UUIDVersion::try_parse`, `wrapper_constructor_name`)
- Modify: `derive-input/src/api/dsl.rs`, `derive-input/src/internal/dsl.rs` (modules + `symbol!(auto_gen)`)
- Modify: `derive-input/src/api/dsl/column.rs` (`auto_generated_uuid_version`), `derive-input/src/internal/dsl/column.rs`, `derive-input/src/internal/column.rs` (`InternalColumn`)
- Modify: `derive-input/src/internal/dsl/method/create.rs`
- Modify: `derive/src/lib.rs` (register helper attribute `auto_gen`)
- Create: `derive/tests/fixtures/uuid_auto_gen.rs` + test fn + snapshots
- Create: `compile-tests/tests/ui/auto_gen_*.rs` / `.stderr`

**Interfaces:**
- Consumes: Task 2 `v4`/`v7` wrapper constructors, `ColumnTypeKind::UUID`.
- Produces: `SpacetimeDSLColumn.auto_generated_uuid_version: Option<UUIDVersion>`; `InternalColumn.spacetimedsl_column_auto_generated_uuid_version: Option<UUIDVersion>`.

- [ ] **Step 1:** Write fixture `uuid_auto_gen.rs`: v7 PK table; table with `#[unique]` v4 column and a module-external `#[use_wrapper]` foreign key to the v7 PK; `#[dsl(singleton)]` table with a v4 column. Add test fn. Write UI tests (one file each): `auto_gen_without_wrapper`, `auto_gen_with_use_wrapper`, `auto_gen_wrong_type`, `auto_gen_public`, `auto_gen_on_singleton_with_default`, `auto_gen_without_version`, `auto_gen_unknown_version`, `auto_gen_repeated`.
- [ ] **Step 2:** Run `cargo test -p spacetimedsl_derive uuid_auto_gen` and `cargo test -p spacetimedsl-compile-tests`; expect failures (attribute ignored / unknown attribute).
- [ ] **Step 3:** Implement `UUIDVersion::try_parse(field, rust_field, wrapper_type, singleton_has_default) -> syn::Result<Option<UUIDVersion>>` with the checks, in this order: repeated attribute; argument is exactly the ident `v4` or `v7`; not `singleton(with_default)`; `ColumnTypeKind::UUID`; private; `WrapperType::Created`. Call it from `SpacetimeDSLColumn::try_parse` (pass `singleton_has_default`), store the result, copy into `InternalColumn`.
- [ ] **Step 4:** In `create_method_column_parts`, before the `auto_inc` branch:

```rust
if let Some(uuid_version) = &internal_column.spacetimedsl_column_auto_generated_uuid_version {
    let wrapper_type = WrapperType::map(internal_column.spacetimedsl_column_wrapper_type.as_ref()
        .expect("an #[auto_gen] column has a #[create_wrapper]"));
    let constructor_name = uuid_version.wrapper_constructor_name();
    constructor_arg = Some(quote! { let #column_name = #wrapper_type::#constructor_name(self)?.value(); });
}
```

- [ ] **Step 5:** Register `auto_gen` in `#[proc_macro_derive(SpacetimeDSL, attributes(...))]`.
- [ ] **Step 6:** `cargo insta test -p spacetimedsl_derive --accept`, `TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests`; read every new snapshot and `.stderr`; rerun both without overwrite - PASS.
- [ ] **Step 7:** Commit `Add #[auto_gen(v4|v7)] for Uuid columns`.

### Task 4: End-to-end shapes in `examples/test`

**Files:**
- Modify: `examples/test/src/lib.rs` (replace `uuid_test` FIXME module, extend `tester`)

- [ ] **Step 1:** In `pub mod uuid_test`, add tables: (a) `UUIDPrimaryKeyRecord` - `#[primary_key] #[create_wrapper] #[auto_gen(v7)] id: Uuid`; (b) `UUIDUniqueRecord` - `auto_inc` PK + `#[unique] #[create_wrapper] #[auto_gen(v4)] token: Uuid`; (c) `UUIDIndexRecord` - `auto_inc` PK + `#[index(btree)] #[create_wrapper] #[auto_gen(v4)] token: Uuid`; (d) `UUIDMultiColumnIndexRecord` - `auto_inc` PK + `token: Uuid (v4)` + `pub group: u32`, `index(accessor = token_and_group, btree(columns = [token, group]))`; (e) same as (d) with `#[dsl(unique_index(name = token_and_group))]`; (f) `#[dsl(singleton)]` `UUIDSingletonRecord` with `#[create_wrapper] #[auto_gen(v4)] token: Uuid`.
- [ ] **Step 2:** In `tester`, for (a)-(e): create two rows, return `Err` if UUIDs are equal, if `get_version()` is not the configured version, or if the `get_*_by_*` lookup does not return the created row; for (a) also require the second v7 > first. For (f): create, then `get_*` and compare token + version.
- [ ] **Step 3:** Run `spacetime start` (if not running) and `./x.sh test`. Expected: publish succeeds, `tester` returns Ok. A shape that fails because of SpacetimeDSL is fixed; a shape rejected by SpacetimeDB is commented out with a `FIXME` + upstream link.
- [ ] **Step 4:** Commit `Test #[auto_gen] UUID columns end to end`.

### Task 5: Documentation

**Files:**
- Modify: `docs/DOCUMENTATION.md` (column attribute list, wrapper section + `Wrapper` trait signature, excluded create fields table, `NewUUID` in prelude/context API)
- Modify: `README.md` (feature bullet)

- [ ] **Step 1:** Update docs, then `cargo fmt --all -- --check` and `./x.sh unit-test`.
- [ ] **Step 2:** Commit `Document #[auto_gen] UUID columns`.
