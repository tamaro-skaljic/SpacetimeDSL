# Soft Deletion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A table opting into `#[dsl(method(soft_delete = true))]` earns `soft_delete_*` methods that retire rows by writing a marker column, with hooks and foreign key cascades of their own.

**Architecture:** `derive-input` parses the `#[dsl]` attribute into the `api` types and generates every DSL method body as a `TokenStream`; `derive` emits what `derive-input` produced. Soft deletion is added as a marker column detected at parse time, a pair of method generators sharing one skeleton with the existing delete generators, and a second set of cascade entry points beside the existing ones. Two tables in a foreign key relationship verify each other through identifiers both sides build from the same two table names; that mechanism gains a soft-deletion half.

**Tech Stack:** Rust 2024, `syn` 2, `quote`, `proc-macro2`, `strum` for enum iteration, `insta` for snapshot tests, `trybuild` for diagnostics tests, `itertools`.

**Spec:** [`docs/superpowers/specs/2026-09-19-soft-deletion-design.md`](../specs/2026-09-19-soft-deletion-design.md)

## Global Constraints

- Rust edition 2024, `rust-version = 1.93.0`, toolchain pinned in `rust-toolchain.toml`. The `.stderr` files are only reproducible against that exact compiler.
- Every workspace dependency is pinned with `=`. Do not add a dependency; everything needed is already in `[workspace.dependencies]`.
- No abbreviations in identifiers. `InputOutput`, not `Io`. `singular_table_name`, not `name`.
- No comment may restate what the code says. Document only what is not obvious and why.
- Write the test before the production code. Snapshot tests are written by adding a fixture and a test function; diagnostics tests by adding a `tests/ui/*.rs` file.
- `./x unit-test` must be green at the end of every task. It runs `cargo test -p spacetimedsl_derive` and `cargo test -p spacetimedsl-compile-tests`.
- `.\x.ps1 test` must be clean at the end of every task. The snapshot tests compare token streams and never compile them, so they cannot catch a generated body that does not build, nor one that builds and then misbehaves. This step does both: it publishes `examples/test` and `examples/blackholio` to the local SpacetimeDB server and runs the `tester` reducer. The server is already running.
  - **Read its output; do not trust its exit code.** The script runs each `spacetime` command without checking the result and always exits 0. A task is clean only when both modules publish without an error, `spacetime call ... tester` reports no error, and the logs hold no panic and no assertion failure.
  - When a publish fails, `cargo check -p spacetimedsl_test` points at the offending generated code far faster than the publish output does. It is a debugging aid, not a substitute for this step.
- `method(delete)` keeps its default of `true`. It becomes mandatory only when `method(soft_delete)` is present.
- Accepted strategies: `on_delete` takes `Error`, `Delete`, `SoftDelete`, `SetZero`, `Ignore`; `on_soft_delete` takes `Error`, `SoftDelete`, `Ignore` only.
- `OnDeleteStrategy` variant order is `Error, Delete, SoftDelete, SetZero, Ignore`, in both copies of the enum.

## How to run the two test harnesses

**Snapshot tests** (`derive/tests/fixtures/*.rs` → `derive/tests/snapshots/<fixture>/<Struct>/*.snap`):

```bash
cargo test -p spacetimedsl_derive
```

A new or changed snapshot makes the test fail and writes `*.snap.new` beside the old file. Read every `.snap.new`, then accept them:

```bash
cargo insta accept            # if cargo-insta is installed
# otherwise, per file:
mv path/to/name.snap.new path/to/name.snap
```

**Diagnostics tests** (`compile-tests/tests/ui/*.rs` + `*.stderr`):

```bash
cargo test -p spacetimedsl-compile-tests
TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests   # regenerate .stderr
```

A `tests/ui/*.rs` file that compiles makes the test fail with `expected test case to fail to compile, but it succeeded`. That is the red state for every diagnostics task here.

## File Structure

**Created**

| File                                                         | Responsibility                                                                                                     |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `derive-input/src/api/dsl/soft_delete.rs`                    | `SoftDeleteMarker` and `SoftDeleteMarkerKind`: which column carries the marker and which of the two shapes it has. |
| `derive-input/src/internal/dsl/soft_delete.rs`               | Marker detection from the struct's fields, and the marker rejections.                                              |
| `derive-input/src/internal/dsl/method/removal.rs`            | `Removal`, and the body skeleton `delete.rs` and `soft_delete.rs` both produce.                                    |
| `derive-input/src/internal/dsl/method/soft_delete.rs`        | `for_soft_delete_one`, `for_soft_delete_many`, and the two marker code fragments.                                  |
| `derive/tests/fixtures/soft_delete_flag.rs`                  | Snapshot fixture: `bool` marker.                                                                                   |
| `derive/tests/fixtures/soft_delete_timestamp.rs`             | Snapshot fixture: `Option<Timestamp>` marker claimed by the attribute.                                             |
| `derive/tests/fixtures/soft_delete_without_delete_method.rs` | Snapshot fixture: `delete = false, soft_delete = true`.                                                            |
| `derive/tests/fixtures/soft_delete_hooks.rs`                 | Snapshot fixture: both soft-delete hooks.                                                                          |
| `derive/tests/fixtures/on_soft_delete_cascade.rs`            | Snapshot fixture: `on_soft_delete` cascade, one and many, recursing one level.                                     |
| `compile-tests/tests/ui/*.rs` + `*.stderr`                   | One pair per rejection; named in the task that adds it.                                                            |

**Modified**

| File                                                             | Change                                                                     |
| ---------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `src/delete.rs`                                                  | `OnDeleteStrategy::SoftDelete`.                                            |
| `src/error.rs`                                                   | `Display` arm for it, `Action::SoftDelete`.                                |
| `derive-input/src/api/dsl/foreign_key.rs`                        | `SoftDelete` variant, both strategy fields optional.                       |
| `derive-input/src/api/dsl/table.rs`                              | `soft_delete_marker`, `CascadeEntryPoints`, the two strategy-pair structs. |
| `derive-input/src/api/dsl/column.rs`                             | `soft_delete_one`, `soft_delete_many`.                                     |
| `derive-input/src/api/dsl/hook.rs`                               | `before_soft_delete`, `after_soft_delete`.                                 |
| `derive-input/src/internal.rs`                                   | `method(soft_delete)` and hook parsing, rejections 1 and 8.                |
| `derive-input/src/internal/dsl.rs`                               | `symbol!(soft_delete)`, `symbol!(on_soft_delete)`, the two new modules.    |
| `derive-input/src/internal/dsl/table.rs`                         | Call marker detection, carry the marker.                                   |
| `derive-input/src/internal/dsl/hook.rs`                          | `Operation::SoftDelete`.                                                   |
| `derive-input/src/internal/dsl/column.rs`                        | Pass the marker down to the foreign key parser.                            |
| `derive-input/src/internal/dsl/foreign_key.rs`                   | `on_soft_delete`, optional `on_delete`, per-field strategy sets.           |
| `derive-input/src/internal/dsl/reference.rs`                     | `#[referenced_by]` gate widened.                                           |
| `derive-input/src/internal/dsl/method/naming.rs`                 | Four compile-error checks, dispatcher names parameterized by `Removal`.    |
| `derive-input/src/internal/dsl/method/delete.rs`                 | Reduced to two calls into `removal.rs`.                                    |
| `derive-input/src/internal/dsl/method/update.rs`                 | Hook before stamp.                                                         |
| `derive-input/src/internal/dsl/method/upsert.rs`                 | Hook before stamp on both paths, module doc rewritten.                     |
| `derive-input/src/internal/dsl/method/create.rs`                 | Marker excluded from `Create<Table>`.                                      |
| `derive-input/src/internal/dsl/method/on_delete_strategy.rs`     | `SoftDelete` arm.                                                          |
| `derive-input/src/internal/dsl/method/foreign_key.rs`            | Per-`Removal` generation, new checks.                                      |
| `derive-input/src/internal/dsl/method/referenced_by.rs`          | Per-`Removal` entry points, new checks.                                    |
| `derive-input/src/internal/dsl/method.rs`                        | Wire the soft methods and the soft strategy pairs.                         |
| `derive/src/lib.rs`                                              | Register `set_on_soft_delete`.                                             |
| `derive/src/output.rs`                                           | Emit the soft methods, the soft entry points, the two hooks.               |
| `derive/src/characterization_tests.rs`                           | One test function per new fixture.                                         |
| `docs/DOCUMENTATION.md`, `README.md`, `examples/test/src/lib.rs` | Documentation and the runtime example.                                     |

---

### Task 1: Hooks run before the framework writes its own columns

The spec reverses a decision documented in `upsert.rs`. Today `update_<table>_by_<primary_key>` and both paths of `upsert_<table>` stamp `created_at` and `updated_at` and then call the hook, so a hook can overrule a timestamp. After this task all write paths call the hook first, and the framework writes the columns it owns afterwards. `create_<table>` already does this, so nothing in `create.rs` changes.

This task changes generated output only. No new test is written; the existing snapshots are the test, and they must change in exactly the expected way.

**Files:**

- Modify: `derive-input/src/internal/dsl/method/update.rs:118-135`
- Modify: `derive-input/src/internal/dsl/method/upsert.rs:1-12` (module documentation), `:430-450` (update path), insert path in the same `method_impl`
- Test: `derive/tests/snapshots/timestamps/**`, `derive/tests/snapshots/hooks_all_six/**`, `derive/tests/snapshots/singleton_with_default/**`

**Interfaces:**

- Consumes: nothing from earlier tasks.
- Produces: no new names. Later tasks rely on the rule that a hook runs before framework-owned columns are written.

- [ ] **Step 1: Record the current snapshots as the baseline**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -5
git status --short        # must be clean
```

Expected: PASS, working tree clean. This is the "before" state the diff in step 5 is read against.

- [ ] **Step 2: Move the stamp below the hook in `update.rs`**

In `for_update`, the `method_impl` currently reads:

```rust
            #on_update_set_current_timestamp

            #before_update_hook
```

Swap the two interpolations so it reads:

```rust
            #before_update_hook

            #on_update_set_current_timestamp
```

Nothing else in the function changes. `before_update_hook` already contains the `if #field_name_for_found_value.is_none() { ... }` prelude that loads the old row, and that prelude does not depend on the timestamp.

- [ ] **Step 3: Move the stamp below the hook on both paths of `upsert.rs`**

On the update path of `for_singleton_upsert`'s `method_impl`, change:

```rust
                #keep_created_at
                #set_updated_at_on_update

                #use_before_update_hook_trait
                #before_update_hook_call
```

to:

```rust
                #use_before_update_hook_trait
                #before_update_hook_call

                #keep_created_at
                #set_updated_at_on_update
```

On the insert path of the same `method_impl`, move the `created_at` and `updated_at` assignments below the `before_insert` hook call in the same way: the hook interpolation comes first, the two assignments follow it.

- [ ] **Step 4: Rewrite the module documentation of `upsert.rs`**

Replace the paragraph at `upsert.rs:10-12`:

```rust
//! Both paths set the columns the framework owns before they call their hook, so a hook can
//! overrule a timestamp. `create_<table>` orders the two the other way round; inside one
//! method the two paths agreeing with each other matters more.
```

with:

```rust
//! Both paths call their hook before they set the columns the framework owns, so the
//! framework has the last word on a timestamp. `create_<table>`, `update_<table>_by_<key>`
//! and `soft_delete_<table>_by_<index>` order the two the same way.
```

- [ ] **Step 5: Run the snapshots and read every diff**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -20
```

Expected: FAIL, with `.snap.new` files under `derive/tests/snapshots/timestamps`, `hooks_all_six` and `singleton_with_default`.

Read each one. Every diff must be a pure reordering: the hook call moves above the assignment to the timestamp column, and nothing else moves. A diff that changes a method body in any other way means a wrong interpolation was moved — fix it before accepting.

- [ ] **Step 6: Accept the snapshots and verify green**

```bash
cargo insta accept
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
```

Expected: both PASS.

- [ ] **Step 7: Commit**

```bash
git add derive-input/src/internal/dsl/method/update.rs \
        derive-input/src/internal/dsl/method/upsert.rs \
        derive/tests/snapshots
git commit -m "refactor: call hooks before writing framework-owned columns

The framework now has the last word on created_at and updated_at in
update_<table>_by_<key> and in both paths of upsert_<table>, which is the
order create_<table> already used.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 2: `SoftDelete` in the runtime crate

The `OnDeleteStrategy` enum exists twice: once in the runtime crate, which generated code names at run time, and once in `derive-input`, which generates those names. Both copies change together; the comment above each says so.

**Files:**

- Modify: `src/delete.rs:8-48`
- Modify: `src/error.rs:36-53` (`Action`), `:66-75` (`Display for OnDeleteStrategy`)
- Modify: `derive-input/src/api/dsl/foreign_key.rs:10-73`

**Interfaces:**

- Consumes: nothing.
- Produces: `spacetimedsl::delete::OnDeleteStrategy::SoftDelete`; `spacetimedsl::error::Action::SoftDelete`; `derive_input::api::dsl::foreign_key::OnDeleteStrategy::SoftDelete`, which `quote::ToTokens` renders as the runtime path.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `src/delete.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::OnDeleteStrategy;

    #[test]
    fn soft_delete_renders_its_own_name() {
        assert_eq!(OnDeleteStrategy::SoftDelete.to_string(), "SoftDelete");
    }
}
```

- [ ] **Step 2: Run it to make sure it fails**

```bash
cargo test -p spacetimedsl soft_delete_renders_its_own_name
```

Expected: FAIL, `no variant named SoftDelete found for enum OnDeleteStrategy`.

- [ ] **Step 3: Add the variant to the runtime enum**

In `src/delete.rs`, insert between `Delete` and the commented-out `SetNone`:

```rust
    /**
     * Available only for tables with `#[dsl(method(soft_delete = true))]`.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the referencing rows are soft-deleted, which marks them through their marker column instead of removing them.
     * Like `Delete`, this cascades further into the tables which reference the soft-deleted rows.
     */
    SoftDelete,
```

Delete the `// TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/59 SoftDelete Feature` line at `src/delete.rs:5`.

In `src/error.rs`, add the `Display` arm between `Delete` and `SetZero`:

```rust
            OnDeleteStrategy::SoftDelete => write!(f, "SoftDelete"),
```

Add the `Action` variant after `Delete`:

```rust
    SoftDelete,
```

and its `Display` arm after the `Action::Delete` arm:

```rust
            Action::SoftDelete => write!(f, "soft delete"),
```

`ReferenceIntegrityViolationError::OnCreateOrUpdate`'s `Display` panics for `Action::Get | Action::Delete`; extend that pattern to `Action::Get | Action::Delete | Action::SoftDelete`, because a soft deletion is no more a create or an update than a deletion is.

- [ ] **Step 4: Run it to make sure it passes**

```bash
cargo test -p spacetimedsl soft_delete_renders_its_own_name
```

Expected: PASS.

- [ ] **Step 5: Mirror the variant into the macro copy**

In `derive-input/src/api/dsl/foreign_key.rs`, add to the enum between `Delete` and the commented-out `SetNone`:

```rust
    /**
     * Available only for tables with `#[dsl(method(soft_delete = true))]`.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the referencing rows are soft-deleted, which marks them through their marker column instead of removing them.
     * Like `Delete`, this cascades further into the tables which reference the soft-deleted rows.
     */
    SoftDelete,
```

and to the `ToTokens` implementation, between the `Delete` and `SetZero` arms:

```rust
            OnDeleteStrategy::SoftDelete => {
                crate::api::runtime::on_delete_strategy(&quote::quote! { SoftDelete })
            }
```

- [ ] **Step 6: Verify the whole workspace still builds and both harnesses are green**

```bash
cargo build --workspace 2>&1 | tail -5
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
```

Expected: all PASS. The new variant has no `#[foreign_key]` spelling yet, so no generated output changes and no snapshot moves. `OnDeleteStrategy::iter()` in `method/foreign_key.rs` now yields a fifth strategy with no columns behind it, which produces an empty match arm — that is an added arm in the snapshots. If snapshots move, read each diff, confirm it adds only an empty `SoftDelete => {}` arm, and accept.

- [ ] **Step 7: Commit**

```bash
git add src/delete.rs src/error.rs derive-input/src/api/dsl/foreign_key.rs derive/tests/snapshots
git commit -m "feat: add the SoftDelete on-delete strategy to the runtime crate

Both copies of OnDeleteStrategy gain the variant, and Action gains a
SoftDelete variant so error prose can name the operation.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 3: `method(soft_delete)` requires an explicit `method(delete)`

The first of the eight rejections. It lives in `try_parse_dsl`, because that is the only place where both flags are still `Option<bool>` and "absent" can be told apart from "false".

**Files:**

- Modify: `derive-input/src/internal.rs:70-72` (the `let mut` block), `:155-170` (the `method` arm), `:186-205` (the checks), `:245-275` (`DSLData`)
- Modify: `derive-input/src/internal/dsl.rs` (symbol)
- Create: `compile-tests/tests/ui/soft_delete_method_without_delete_method.rs` + `.stderr`

**Interfaces:**

- Consumes: nothing.
- Produces: `DSLData.soft_delete_method: Option<bool>`, read by Task 4 and Task 7.

- [ ] **Step 1: Write the failing test**

Create `compile-tests/tests/ui/soft_delete_method_without_delete_method.rs`:

```rust
//! `method(soft_delete)` decides whether a table retires rows instead of removing them,
//! which only means something next to a statement about whether it removes them at all.
//! Leaving `method(delete)` to its default would hide that decision, so mentioning
//! `soft_delete` makes `delete` mandatory.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, soft_delete = true),
    )]
    #[spacetimedb::table(
        accessor = ticket,
        public,
    )]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,

        deleted: bool,
    }
}

fn main() {}
```

- [ ] **Step 2: Run it to make sure it fails**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -20
```

Expected: FAIL. Either `expected test case to fail to compile, but it succeeded`, or a compile error about an unknown `soft_delete` meta. Both are the red state; the parser does not know the keyword yet.

- [ ] **Step 3: Parse the flag and reject the missing `delete`**

In `derive-input/src/internal/dsl.rs`, add beside the other symbols:

```rust
symbol!(soft_delete);
```

In `derive-input/src/internal.rs`, add the local beside `delete_method`:

```rust
    let mut soft_delete_method = None;
```

Add an arm to the nested `method` matcher, after the `delete` arm:

```rust
                        soft_delete => {
                            check_duplicate(&soft_delete_method, &meta)?;
                            soft_delete_method =
                                Some(meta.value()?.parse::<syn::LitBool>()?.value);
                        }
```

The arm must be able to report the span of the `soft_delete` keyword later, so capture it too. Replace the local and the arm with:

```rust
    let mut soft_delete_method: Option<bool> = None;
    let mut soft_delete_method_span: Option<Span> = None;
```

```rust
                        soft_delete => {
                            check_duplicate(&soft_delete_method, &meta)?;
                            soft_delete_method_span = Some(meta.path.span());
                            soft_delete_method =
                                Some(meta.value()?.parse::<syn::LitBool>()?.value);
                        }
```

After the `parse2` call and beside the other checks, add:

```rust
    if let Some(span) = soft_delete_method_span
        && delete_method.is_none()
    {
        return Err(syn::Error::new(
            span,
            "`#[dsl(method(soft_delete = ...))]` requires `#[dsl(method(delete = ...))]` to be set as well, e.g. `method(delete = true, soft_delete = true)`.\nSoft deletion retires a row instead of removing it, which only says something next to a decision about whether the table removes rows at all.",
        ));
    }
```

Add the field to `DSLData` and to the `Ok(DSLData { .. })` that builds it:

```rust
    soft_delete_method: Option<bool>,
```

```rust
        soft_delete_method,
```

- [ ] **Step 4: Regenerate the diagnostic and read it**

```bash
TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests 2>&1 | tail -10
cat compile-tests/tests/ui/soft_delete_method_without_delete_method.stderr
```

Expected: the `.stderr` holds the message above, underlining `soft_delete` inside the `method(...)` list. If it underlines the whole struct instead, `meta.path.span()` was not captured — fix and regenerate.

- [ ] **Step 5: Run both harnesses**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
cargo test -p spacetimedsl_derive 2>&1 | tail -5
```

Expected: both PASS. No table in any fixture mentions `soft_delete`, so no snapshot moves.

- [ ] **Step 6: Commit**

```bash
git add derive-input/src/internal.rs derive-input/src/internal/dsl.rs compile-tests/tests/ui
git commit -m "feat: parse method(soft_delete) and require an explicit method(delete)

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 4: The marker column and its rejections

Detection of the column a soft deletion writes, and rejections 2 through 7. This is the task that makes a table soft-deletable as far as the `api` types are concerned; nothing generates a method yet.

**Files:**

- Create: `derive-input/src/api/dsl/soft_delete.rs`
- Create: `derive-input/src/internal/dsl/soft_delete.rs`
- Modify: `derive-input/src/api/dsl.rs`, `derive-input/src/api/dsl/table.rs:26-50`
- Modify: `derive-input/src/internal/dsl.rs`, `derive-input/src/internal/dsl/table.rs:17-50` and `:205-235`
- Modify: `derive/src/lib.rs:112-123`
- Create: six `compile-tests/tests/ui/*.rs` + `.stderr` pairs, named in step 1

**Interfaces:**

- Consumes: `DSLData.soft_delete_method` (Task 3).
- Produces:
  - `api::dsl::soft_delete::SoftDeleteMarkerKind { Flag, Timestamp }`
  - `api::dsl::soft_delete::SoftDeleteMarker { column_name: Ident, kind: SoftDeleteMarkerKind }`
  - `SpacetimeDSLTable.soft_delete_marker: Option<SoftDeleteMarker>` and `SpacetimeDSLTable::is_soft_deletable(&self) -> bool`
  - `internal::dsl::soft_delete::try_parse(has_soft_delete_method: Option<bool>, singleton: Option<SingletonKind>, column_args: &ColumnArgs<'_>, original_struct_name: &Ident) -> syn::Result<Option<SoftDeleteMarker>>`

- [ ] **Step 1: Write the six failing tests**

Create these files under `compile-tests/tests/ui/`. Each ends in `fn main() {}` and starts with `::spacetimedsl::spacetimedsl!();`, like every other case there.

`marker_column_without_soft_delete_method.rs`:

```rust
//! A column named `deleted` claims the soft-delete marker role. On a table which never
//! soft-deletes, nothing would ever write it, so the name would promise something the
//! generated code does not do.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, delete = true),
    )]
    #[spacetimedb::table(
        accessor = ticket,
        public,
    )]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,

        deleted: bool,
    }
}

fn main() {}
```

`soft_delete_method_without_marker_column.rs`: same table, `method(update = true, delete = true, soft_delete = true)`, and no `deleted` column.

`marker_column_with_wrong_type.rs`: `soft_delete = true` and `deleted: u8`.

`two_marker_columns.rs`: `soft_delete = true`, both `deleted: bool` and `deleted_at: Option<spacetimedb::Timestamp>`.

`public_marker_column.rs`: `soft_delete = true` and `pub deleted: bool`.

`soft_delete_method_on_singleton.rs`: `#[dsl(singleton, method(update = true, delete = true, soft_delete = true))]` with a `deleted: bool` column.

- [ ] **Step 2: Run them to make sure they fail**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -30
```

Expected: FAIL, `expected test case to fail to compile, but it succeeded`, for all six.

- [ ] **Step 3: Add the api type**

Create `derive-input/src/api/dsl/soft_delete.rs`:

```rust
use syn::Ident;

/// Which of the two shapes a soft-delete marker column has.
///
/// The shape decides what a soft deletion writes into the column and how the generated
/// code asks whether a row is already retired.
#[derive(Clone, Copy, PartialEq)]
pub enum SoftDeleteMarkerKind {
    /// A `bool`, set to `true`.
    Flag,
    /// An `Option<Timestamp>`, set to the current timestamp.
    Timestamp,
}

/// The one column a soft deletion writes.
///
/// A table has at most one. Its presence is what makes the table soft-deletable: the
/// `method(soft_delete)` flag and this column are rejected unless they agree.
#[derive(Clone)]
pub struct SoftDeleteMarker {
    pub column_name: Ident,
    pub kind: SoftDeleteMarkerKind,
}
```

Add `pub mod soft_delete;` to `derive-input/src/api/dsl.rs`.

In `derive-input/src/api/dsl/table.rs`, add the field to `SpacetimeDSLTable` after `has_delete_method`:

```rust
    /// The column a soft deletion writes, if the table is soft-deletable.
    ///
    /// `Some` and `#[dsl(method(soft_delete = true))]` imply each other: the parser rejects
    /// either one without the other, so this is the single question every generator asks.
    pub soft_delete_marker: Option<SoftDeleteMarker>,
```

and the query beside `is_singleton`:

```rust
    pub fn is_soft_deletable(&self) -> bool {
        self.soft_delete_marker.is_some()
    }
```

- [ ] **Step 4: Add the detection module**

Create `derive-input/src/internal/dsl/soft_delete.rs`:

```rust
//! Which column a soft deletion writes, and the rules that column obeys.
//!
//! A column claims the marker role by its name or by carrying `#[set_on_soft_delete]`, and
//! the claim fixes its type. The role and the `method(soft_delete)` flag imply each other,
//! so this module rejects either one without the other and every generator afterwards
//! reads one `Option`.

use crate::api::dsl::{
    soft_delete::{SoftDeleteMarker, SoftDeleteMarkerKind},
    table::SingletonKind,
};
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::{sats::SatsField, table::ColumnArgs};
use syn::Ident;

const FLAG_COLUMN_NAMES: [&str; 2] = ["deleted", "removed"];
const TIMESTAMP_COLUMN_NAMES: [&str; 2] = ["deleted_at", "removed_at"];

pub(in crate::internal) fn try_parse(
    has_soft_delete_method: Option<bool>,
    singleton: Option<SingletonKind>,
    column_args: &ColumnArgs<'_>,
    original_struct_name: &Ident,
) -> syn::Result<Option<SoftDeleteMarker>> {
    let is_soft_deletable = has_soft_delete_method == Some(true);

    if is_soft_deletable && singleton.is_some() {
        return Err(syn::Error::new_spanned(
            original_struct_name,
            "`#[dsl(method(soft_delete = true))]` is not allowed on a singleton table!\nA singleton holds one row which the DSL looks up by its injected primary key, so retiring that row would leave the table with a row no method can reach.",
        ));
    }

    let mut marker: Option<SoftDeleteMarker> = None;

    for field in &column_args.fields {
        let kind = match claimed_kind(field)? {
            None => continue,
            Some(kind) => kind,
        };

        let column_name = field.name.as_ref().expect("should have a name");
        let field_type = field.ty.to_token_stream().to_string();

        if !type_fits(kind, &field_type) {
            return Err(syn::Error::new_spanned(
                field.ty,
                format!(
                    "A column with the soft-delete marker role should have the type `{}`! Found: {field_type}",
                    match kind {
                        SoftDeleteMarkerKind::Flag => "bool",
                        SoftDeleteMarkerKind::Timestamp => "Option<spacetimedb::Timestamp>",
                    }
                ),
            ));
        }

        if marker.is_some() {
            return Err(syn::Error::new_spanned(
                field.ident.expect("a named field has an identifier"),
                "Multiple columns claim the soft-delete marker role! Only one column is allowed.",
            ));
        }

        if !matches!(field.vis, syn::Visibility::Inherited) {
            return Err(syn::Error::new_spanned(
                field.vis,
                "A column with the soft-delete marker role should have `Visibility::Inherited`!\n`soft_delete_<table>_by_<index>` is its only writer, so it has a getter but no setter.",
            ));
        }

        marker = Some(SoftDeleteMarker {
            column_name: format_ident!("{column_name}"),
            kind,
        });
    }

    match (is_soft_deletable, &marker) {
        (true, None) => Err(syn::Error::new_spanned(
            original_struct_name,
            "`#[dsl(method(soft_delete = true))]` requires a column which the soft deletion writes!\nName a column `deleted` or `removed` and give it the type `bool`, name a column `deleted_at` or `removed_at` and give it the type `Option<spacetimedb::Timestamp>`, or put `#[set_on_soft_delete]` on a column of either type.",
        )),
        (false, Some(marker)) => Err(syn::Error::new_spanned(
            &marker.column_name,
            "This column claims the soft-delete marker role, but the table is not soft-deletable!\nAdd `#[dsl(method(soft_delete = true))]` to the table, or rename the column and remove `#[set_on_soft_delete]` from it.",
        )),
        _ => Ok(marker),
    }
}

/// Which role a column's name or attribute claims, if any.
fn claimed_kind(field: &SatsField<'_>) -> syn::Result<Option<SoftDeleteMarkerKind>> {
    let column_name = field.name.as_ref().expect("should have a name");

    let has_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("set_on_soft_delete"));

    let claims_flag = FLAG_COLUMN_NAMES.contains(&column_name.as_ref());
    let claims_timestamp = TIMESTAMP_COLUMN_NAMES.contains(&column_name.as_ref());

    if claims_flag {
        return Ok(Some(SoftDeleteMarkerKind::Flag));
    }

    if claims_timestamp {
        return Ok(Some(SoftDeleteMarkerKind::Timestamp));
    }

    if !has_attribute {
        return Ok(None);
    }

    // The attribute names no shape, so the column's type picks one.
    let field_type = field.ty.to_token_stream().to_string();

    if type_fits(SoftDeleteMarkerKind::Flag, &field_type) {
        return Ok(Some(SoftDeleteMarkerKind::Flag));
    }

    if type_fits(SoftDeleteMarkerKind::Timestamp, &field_type) {
        return Ok(Some(SoftDeleteMarkerKind::Timestamp));
    }

    Err(syn::Error::new_spanned(
        field.ty,
        format!(
            "A column with `#[set_on_soft_delete]` should have the type `bool` or `Option<spacetimedb::Timestamp>`! Found: {field_type}"
        ),
    ))
}

fn type_fits(kind: SoftDeleteMarkerKind, field_type: &str) -> bool {
    match kind {
        SoftDeleteMarkerKind::Flag => field_type.eq("bool"),
        SoftDeleteMarkerKind::Timestamp => {
            field_type.eq("Option < Timestamp >")
|   | field_type.eq("Option < spacetimedb :: Timestamp >") |
        }
    }
}
```

The types this relies on, confirmed against the dependency: `ColumnArgs.original_struct_name` is an owned `Ident`, so `&column_args.original_struct_name` is the `&Ident` the signature takes. `SatsField.name` is an `Option<String>`, so `field.name.as_ref().expect(..)` is a `&String` and `&column_name.as_ref()` is the `&&str` that `[&str; 2]::contains` wants.

Add `pub mod soft_delete;` to `derive-input/src/internal/dsl.rs`.

- [ ] **Step 5: Call detection and carry the marker**

In `derive-input/src/internal/dsl/table.rs`, inside `SpacetimeDSLTable::try_parse`, before the field loop:

```rust
        let soft_delete_marker = super::soft_delete::try_parse(
            dsl_data.soft_delete_method,
            dsl_data.singleton,
            column_args,
            &column_args.original_struct_name,
        )?;
```

and add to the `SpacetimeDSLTable { .. }` that is returned, after `has_delete_method`:

```rust
                soft_delete_marker,
```

- [ ] **Step 6: Register the helper attribute**

In `derive/src/lib.rs`, add `set_on_soft_delete` to the `attributes(...)` list of the `SpacetimeDSL` derive, after `updated_at`.

- [ ] **Step 7: Regenerate the six diagnostics and read them**

```bash
TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests 2>&1 | tail -10
for f in compile-tests/tests/ui/marker_column_without_soft_delete_method \
         compile-tests/tests/ui/soft_delete_method_without_marker_column \
         compile-tests/tests/ui/marker_column_with_wrong_type \
         compile-tests/tests/ui/two_marker_columns \
         compile-tests/tests/ui/public_marker_column \
         compile-tests/tests/ui/soft_delete_method_on_singleton; do
    echo "== $f"; cat "$f.stderr"; done
```

Expected: six distinct messages, each underlining the column, the type, the visibility or the struct name as the code above chose. A message pointing at the wrong span is a bug in the `new_spanned` argument, not in the test.

- [ ] **Step 8: Run both harnesses**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
cargo test -p spacetimedsl_derive 2>&1 | tail -5
```

Expected: both PASS. No fixture is soft-deletable, so no snapshot moves.

- [ ] **Step 9: Commit**

```bash
git add derive-input/src/api/dsl.rs derive-input/src/api/dsl/soft_delete.rs \
        derive-input/src/api/dsl/table.rs derive-input/src/internal/dsl.rs \
        derive-input/src/internal/dsl/soft_delete.rs \
        derive-input/src/internal/dsl/table.rs derive/src/lib.rs \
        compile-tests/tests/ui
git commit -m "feat: detect the soft-delete marker column

A column claims the marker role by its name or by #[set_on_soft_delete], the
claim fixes its type, and the role and method(soft_delete) imply each other.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 5: The marker is not part of `Create<Table>`

A new row is never born retired, so the create generator fills the marker in rather than asking the caller for it. `create_method_column_parts` already does this for `created_at` and `updated_at`; the marker is a third such column.

**Files:**

- Modify: `derive-input/src/internal/dsl/method/create.rs:71-95`
- Create: `derive/tests/fixtures/soft_delete_flag.rs`
- Modify: `derive/src/characterization_tests.rs`

**Interfaces:**

- Consumes: `SpacetimeDSLTable.soft_delete_marker` (Task 4).
- Produces: the first soft-deletable fixture, reused as a reading aid by later tasks.

- [ ] **Step 1: Write the failing test**

Create `derive/tests/fixtures/soft_delete_flag.rs`:

```rust
//! Covers a soft-deletable table whose marker is a `bool` claimed by its name.
//!
//! The marker is private, so it earns a getter and no setter, and the create method fills
//! it in rather than asking for it.

#[spacetimedsl::dsl(plural_name = tickets, method(update = true, delete = true, soft_delete = true))]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub title: String,

    #[index(btree)]
    pub priority: u8,

    deleted: bool,
}
```

Add to `derive/src/characterization_tests.rs`, beside the other fixture tests:

```rust
#[test]
fn soft_delete_flag() {
    snapshot_fixture("soft_delete_flag");
}
```

- [ ] **Step 2: Run it to see the marker in `CreateTicket`**

```bash
cargo test -p spacetimedsl_derive soft_delete_flag 2>&1 | tail -20
```

Expected: FAIL, with new `.snap.new` files. Read `derive/tests/snapshots/soft_delete_flag/Ticket/table.snap.new`: `pub struct CreateTicket` currently holds a `deleted` member. That is the defect this task removes. Do not accept these snapshots yet.

- [ ] **Step 3: Fill the marker in instead of asking for it**

In `create_method_column_parts`, add a branch to the `else if` chain that already handles the two timestamp roles, after the `on_update_set_current_timestamp_column_name` branch:

```rust
    } else if let Some(marker) = &spacetimedsl_table.soft_delete_marker
        && { internal_column.rust_field_name.eq(&marker.column_name) }
    {
        let initial_value = match marker.kind {
            SoftDeleteMarkerKind::Flag => quote! { false },
            SoftDeleteMarkerKind::Timestamp => quote! { None },
        };
        constructor_arg = Some(quote! {
            let #column_name = #initial_value;
        });
    }
```

The chain binds `column_name` from the outer scope in the timestamp branches; keep the same spelling here so the generated `let` names the column.

Add the import:

```rust
use crate::api::dsl::soft_delete::SoftDeleteMarkerKind;
```

- [ ] **Step 4: Run it again and read the snapshots**

```bash
cargo test -p spacetimedsl_derive soft_delete_flag 2>&1 | tail -20
```

Expected: FAIL again, because the snapshots are still new. Read them. `CreateTicket` must now hold `id` is absent (auto inc), `title` and `priority`, and no `deleted`. `create_ticket` must contain `let deleted = false;`. The table must expose `get_deleted` and no `set_deleted`.

There is no `soft_delete_ticket_by_id` yet; that arrives in Task 8.

- [ ] **Step 5: Accept and verify**

```bash
cargo insta accept
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
```

Expected: both PASS.

- [ ] **Step 6: Commit**

```bash
git add derive-input/src/internal/dsl/method/create.rs \
        derive/tests/fixtures/soft_delete_flag.rs \
        derive/src/characterization_tests.rs derive/tests/snapshots
git commit -m "feat: keep the soft-delete marker out of Create<Table>

The create method initializes it, the way it already initializes created_at,
so a row is never born retired.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 6: The `before_soft_delete` and `after_soft_delete` hooks

The hooks take the shape of the update hooks, because a soft deletion writes a row. Nothing calls them yet; Task 8 does.

**Files:**

- Modify: `derive-input/src/api/dsl/hook.rs:7-14`
- Modify: `derive-input/src/internal/dsl/hook.rs:37-47` (`DeclaredHooks`), `:49-105` (`build`), `:140-146` (`Operation`), `:150-200` (name builders), `:202-260` (args), `:262-290` (return type)
- Modify: `derive-input/src/internal.rs` (parsing, rejection 8), `derive-input/src/internal/dsl/table.rs:34-45`
- Modify: `derive/src/output.rs:158-166`
- Create: `derive/tests/fixtures/soft_delete_hooks.rs`, `compile-tests/tests/ui/soft_delete_hook_without_soft_delete_method.rs` + `.stderr`
- Modify: `derive/src/characterization_tests.rs`

**Interfaces:**

- Consumes: `DSLData.soft_delete_method` (Task 3).
- Produces: `SpacetimeDSLMethodHooks.before_soft_delete` and `.after_soft_delete`, both `Option<SpacetimeDSLMethodHook>`, read by Task 8 and Task 11.

- [ ] **Step 1: Write the two failing tests**

Create `compile-tests/tests/ui/soft_delete_hook_without_soft_delete_method.rs`:

```rust
//! A `before_soft_delete` hook runs when a row is retired. On a table which never retires
//! a row, the trait would be emitted and never used, and the developer would wait for a
//! call that cannot come.

::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = true, delete = true),
        hook(before(soft_delete)),
    )]
    #[spacetimedb::table(
        accessor = ticket,
        public,
    )]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,
    }
}

fn main() {}
```

Create `derive/tests/fixtures/soft_delete_hooks.rs`:

```rust
//! Covers both soft-delete hooks. They take the shape of the update hooks, because a soft
//! deletion writes the row rather than removing it, and they run before the marker is
//! written.

#[spacetimedsl::dsl(
    plural_name = tickets,
    method(update = true, delete = true, soft_delete = true),
    hook(before(soft_delete), after(soft_delete)),
)]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub title: String,

    deleted_at: Option<spacetimedb::Timestamp>,
}
```

Add to `derive/src/characterization_tests.rs`:

```rust
#[test]
fn soft_delete_hooks() {
    snapshot_fixture("soft_delete_hooks");
}
```

- [ ] **Step 2: Run them to make sure they fail**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -10
cargo test -p spacetimedsl_derive soft_delete_hooks 2>&1 | tail -10
```

Expected: the diagnostics case fails because the table compiles or because `soft_delete` is an unknown meta inside `before(...)`; the snapshot test fails because the fixture cannot be expanded. Both are red.

- [ ] **Step 3: Parse the two hooks and reject them without the method**

In `derive-input/src/internal.rs`, add the two locals beside the others:

```rust
    let mut before_soft_delete_hook: Option<Span> = None;
    let mut after_soft_delete_hook: Option<Span> = None;
```

Add an arm to the `before` matcher, after the `delete` arm:

```rust
                                    soft_delete => {
                                        check_duplicate(&before_soft_delete_hook, &meta)?;
                                        before_soft_delete_hook = Some(meta.path.span());
                                    }
```

and the matching arm to the `after` matcher:

```rust
                                    soft_delete => {
                                        check_duplicate(&after_soft_delete_hook, &meta)?;
                                        after_soft_delete_hook = Some(meta.path.span());
                                    }
```

After the `parse2` call, beside the other hook checks:

```rust
    if soft_delete_method != Some(true) {
        if let Some(span) = before_soft_delete_hook {
            return Err(syn::Error::new(
                span,
                "Cannot have a `before_soft_delete` hook when the table is not soft-deletable. Enable it with `#[dsl(method(soft_delete = true))]`",
            ));
        }

        if let Some(span) = after_soft_delete_hook {
            return Err(syn::Error::new(
                span,
                "Cannot have an `after_soft_delete` hook when the table is not soft-deletable. Enable it with `#[dsl(method(soft_delete = true))]`",
            ));
        }
    }
```

Add the two `bool` fields to `DSLData` and fill them with `before_soft_delete_hook.is_some()` and `after_soft_delete_hook.is_some()`, matching how the six existing hook flags are carried.

- [ ] **Step 4: Build the two hooks**

In `derive-input/src/api/dsl/hook.rs`, add to `SpacetimeDSLMethodHooks`:

```rust
    pub before_soft_delete: Option<SpacetimeDSLMethodHook>,
    pub after_soft_delete: Option<SpacetimeDSLMethodHook>,
```

In `derive-input/src/internal/dsl/hook.rs`:

- add `pub before_soft_delete: bool,` and `pub after_soft_delete: bool,` to `DeclaredHooks`;
- add `SoftDelete` to `Operation`;
- in `build`, add two `build_any` calls mirroring the update ones, with `Operation::SoftDelete`, and put both into the returned `SpacetimeDSLMethodHooks`;
- in `get_trait_name`, map `Operation::SoftDelete => "SoftDelete"`;
- in `get_function_name`, map `Operation::SoftDelete => "soft_delete"`;
- in `get_function_args`, give `(Timing::Before, Operation::SoftDelete)` the same three arguments as `(Timing::Before, Operation::Update)` and `(Timing::After, Operation::SoftDelete)` the same three as `(Timing::After, Operation::Update)`. The existing `(_, Operation::Delete)` catch-all arm matches any timing, so the two soft-delete arms must be written **above** it or the compiler will route them into the delete shape. Write them directly after the update arms;
- in `get_return_type`, add `(Timing::Before, Operation::SoftDelete)` to the arm that returns `Result<#singular_table_name_pascal_case, #error_type>`, beside `(Timing::Before, Operation::Update)`.

In `derive-input/src/internal/dsl/table.rs`, pass the two new flags into the `DeclaredHooks { .. }` literal.

In `derive/src/output.rs`, add to the `hooks` vector:

```rust
        hook::build(&input.spacetimedsl_table.hooks.before_soft_delete)?,
        hook::build(&input.spacetimedsl_table.hooks.after_soft_delete)?,
```

- [ ] **Step 5: Regenerate the diagnostic and read the snapshots**

```bash
TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests 2>&1 | tail -10
cat compile-tests/tests/ui/soft_delete_hook_without_soft_delete_method.stderr
cargo test -p spacetimedsl_derive soft_delete_hooks 2>&1 | tail -10
cat derive/tests/snapshots/soft_delete_hooks/Ticket/table.snap.new
```

Expected: the diagnostic underlines `soft_delete` inside `before(...)`. The snapshot holds:

```rust
pub trait BeforeTicketSoftDeleteHook<T: crate::spacetimedsl::WriteContext> {
    fn before_ticket_soft_delete(
        dsl: &crate::spacetimedsl::DSL<'_, T>,
        old_ticket: &Ticket,
        new_ticket: Ticket,
    ) -> Result<Ticket, crate::spacetimedsl::error::SpacetimeDSLError>;
}
pub trait AfterTicketSoftDeleteHook<T: crate::spacetimedsl::WriteContext> {
    fn after_ticket_soft_delete(
        dsl: &crate::spacetimedsl::DSL<'_, T>,
        old_ticket: &Ticket,
        new_ticket: &Ticket,
    ) -> Result<(), crate::spacetimedsl::error::SpacetimeDSLError>;
}
```

A `before_ticket_soft_delete` taking only `old_ticket` means the arm was written below the `(_, Operation::Delete)` catch-all.

- [ ] **Step 6: Accept and verify**

```bash
cargo insta accept
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
```

Expected: both PASS.

- [ ] **Step 7: Commit**

```bash
git add derive-input/src/api/dsl/hook.rs derive-input/src/internal/dsl/hook.rs \
        derive-input/src/internal.rs derive-input/src/internal/dsl/table.rs \
        derive/src/output.rs derive/src/characterization_tests.rs \
        derive/tests/fixtures/soft_delete_hooks.rs derive/tests/snapshots \
        compile-tests/tests/ui
git commit -m "feat: add the before and after soft_delete hooks

They take the shape of the update hooks, because a soft deletion writes the
row rather than removing it.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 7: Extract the removal skeleton from `delete.rs`

A pure refactor. `for_delete_one` and `for_delete_many` keep producing byte-identical output; the body they produce moves into a skeleton the soft-delete generators will call with the other variant in Task 8. The snapshots are the test: if any of them moves, the extraction changed behavior and is wrong.

Do this before writing the soft-delete generators, so the skeleton is shaped by working code rather than by guesswork.

**Files:**

- Create: `derive-input/src/internal/dsl/method/removal.rs`
- Modify: `derive-input/src/internal/dsl/method/delete.rs` (whole file)
- Modify: `derive-input/src/internal/dsl/method.rs:22-50` (module list and `use`s)

**Interfaces:**

- Consumes: `IndexShape`, `MethodGenerationContext`, the helpers `delete.rs` already imports.
- Produces:

```rust
pub(in crate::internal) enum Removal { Hard, Soft }

pub(in crate::internal) fn for_removal_one(
    removal: Removal,
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod;

pub(in crate::internal) fn for_removal_many(
    removal: Removal,
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod;
```

- [ ] **Step 1: Record the baseline**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -5
git status --short
```

Expected: PASS, clean tree.

- [ ] **Step 2: Create the module with the variant and the two entry points**

Create `derive-input/src/internal/dsl/method/removal.rs` beginning with:

```rust
//! What retiring a row and removing it have in common.
//!
//! `delete_<tables>_by_<index>` and `soft_delete_<tables>_by_<index>` differ in four
//! places: the statement that writes, the hooks they call, the dispatcher they cascade
//! through and the strategy they report. Everything around those four — finding the rows,
//! building the entries, refusing before writing, running the remaining strategy passes —
//! is stated once here, because two copies of that order would drift.

/// Whether a generated method removes the rows it matched or retires them.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::internal) enum Removal {
    Hard,
    Soft,
}
```

Move the whole current body of `for_delete_many` into `for_removal_many` and the whole body of `for_delete_one` into `for_removal_one`, taking `removal: Removal` as the first parameter. At this point every use of `removal` is `Removal::Hard`; leave the bodies otherwise untouched.

Add `mod removal;` to the module list in `derive-input/src/internal/dsl/method.rs`.

- [ ] **Step 3: Reduce `delete.rs` to two calls**

Replace the whole of `derive-input/src/internal/dsl/method/delete.rs` with:

```rust
//! `delete_<table>_by_<index>` and `delete_<tables>_by_<index>`: remove the rows an index
//! matches.
//!
//! The body they produce is [`super::removal`]'s, with `Removal::Hard`. Retiring rows
//! instead of removing them is [`super::soft_delete`].

use super::{
    context::MethodGenerationContext,
    index::IndexShape,
    removal::{Removal, for_removal_many, for_removal_one},
};
use crate::api::dsl::method::SpacetimeDSLMethod;

/// `delete_<tables>_by_<index>`: delete every row an index matches.
pub(in crate::internal) fn for_delete_many(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_many(Removal::Hard, shape, context)
}

/// `delete_<table>_by_<index>`: delete the one row a unique index finds.
pub(in crate::internal) fn for_delete_one(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_one(Removal::Hard, shape, context)
}
```

`super::soft_delete` does not exist yet, so write that doc line only after Task 8, or write it now and add the module in Task 8 — the doc comment is prose and does not have to resolve.

- [ ] **Step 4: Mark the four variation points**

Inside `removal.rs`, find the four places that will differ and give each one a `match removal` with both arms, where the `Removal::Soft` arm is for now a copy of the `Removal::Hard` arm. Task 8 fills them in. The four are:

1. the method name and doc comment (`format_ident!("delete_{plural_table_name}_by_{index_name}")` and its `format!`);
2. the strategy the `DeletionResultEntry` reports (`runtime::on_delete_strategy(&quote! { Delete })`);
3. the hooks (`spacetimedsl_table.hooks.before_delete` / `.after_delete`);
4. the statement that writes (`#index_accessor.delete(#index_name)` and the one-row `.delete(&row_to_delete.#primary_key_column_name)`), together with the count check around it.

Writing them as `match removal` now keeps Task 8 from having to re-find them, and each arm is still `Removal::Hard`'s code, so output does not change.

- [ ] **Step 5: Verify nothing moved**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
git status --short
```

Expected: both PASS and **no `.snap.new` file anywhere**. A moved snapshot means the extraction changed a body. Find the difference and remove it; do not accept the snapshot.

- [ ] **Step 6: Commit**

```bash
git add derive-input/src/internal/dsl/method/removal.rs \
        derive-input/src/internal/dsl/method/delete.rs \
        derive-input/src/internal/dsl/method.rs
git commit -m "refactor: extract the removal skeleton out of delete.rs

The body of the delete methods moves behind a Removal variant, so the soft
delete methods can produce the same shape without a second copy of the
cascade order. Generated output is unchanged.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 8: The `soft_delete_*` methods

**Files:**

- Create: `derive-input/src/internal/dsl/method/soft_delete.rs`
- Modify: `derive-input/src/internal/dsl/method/removal.rs` (fill the four `Removal::Soft` arms)
- Modify: `derive-input/src/api/dsl/column.rs:28-45`
- Modify: `derive-input/src/internal/dsl/method.rs:64-100` (`column_methods_for`)
- Modify: `derive/src/output.rs:196-225` (`get_column_dsl_methods`)
- Create: `derive/tests/fixtures/soft_delete_timestamp.rs`, `derive/tests/fixtures/soft_delete_without_delete_method.rs`
- Modify: `derive/src/characterization_tests.rs`

**Interfaces:**

- Consumes: `Removal`, `for_removal_one`, `for_removal_many` (Task 7); `SoftDeleteMarker` (Task 4); the two hooks (Task 6).
- Produces:
  - `for_soft_delete_one(shape, context) -> SpacetimeDSLMethod`, `for_soft_delete_many(shape, context) -> SpacetimeDSLMethod`
  - `set_marker(marker: &SoftDeleteMarker, receiver: &TokenStream, row: &Ident) -> TokenStream`
  - `is_marked(marker: &SoftDeleteMarker, row: &TokenStream) -> TokenStream`
  - `SpacetimeDSLColumnMethodsForUniqueIndex.soft_delete_one: Option<SpacetimeDSLMethod>`, `SpacetimeDSLColumnMethodsForIndex.soft_delete_many: Option<SpacetimeDSLMethod>`

- [ ] **Step 1: Write the failing tests**

Create `derive/tests/fixtures/soft_delete_timestamp.rs`:

```rust
//! Covers a marker claimed by `#[set_on_soft_delete]` rather than by its name, with the
//! `Option<Timestamp>` shape, on a table with a unique and a non-unique index.

#[spacetimedsl::dsl(plural_name = tickets, method(update = true, delete = true, soft_delete = true))]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[create_wrapper]
    pub reference: String,

    #[index(btree)]
    pub priority: u8,

    #[set_on_soft_delete]
    retired_at: Option<spacetimedb::Timestamp>,
}
```

Create `derive/tests/fixtures/soft_delete_without_delete_method.rs`:

```rust
//! Covers a table whose rows can only be retired, never removed: `method(delete = false)`
//! beside `method(soft_delete = true)`. It earns `soft_delete_*` and no `delete_*`.

#[spacetimedsl::dsl(plural_name = tickets, method(update = true, delete = false, soft_delete = true))]
#[spacetimedb::table(accessor = ticket, public)]
pub struct Ticket {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub title: String,

    deleted: bool,
}
```

Add both to `derive/src/characterization_tests.rs`:

```rust
#[test]
fn soft_delete_timestamp() {
    snapshot_fixture("soft_delete_timestamp");
}

#[test]
fn soft_delete_without_delete_method() {
    snapshot_fixture("soft_delete_without_delete_method");
}
```

- [ ] **Step 2: Run them to make sure they fail**

```bash
cargo test -p spacetimedsl_derive soft_delete 2>&1 | tail -20
```

Expected: FAIL. The `.snap.new` for every soft-deletable fixture holds no `soft_delete_*` method. That absence is what this task fixes.

- [ ] **Step 3: Write the marker fragments and the two generators**

Create `derive-input/src/internal/dsl/method/soft_delete.rs`:

```rust
//! `soft_delete_<table>_by_<index>` and `soft_delete_<tables>_by_<index>`: retire the rows
//! an index matches by writing the table's marker column.
//!
//! The body they produce is [`super::removal`]'s, with `Removal::Soft`. Removing rows
//! instead of retiring them is [`super::delete`].

use super::{
    context::MethodGenerationContext,
    index::IndexShape,
    removal::{Removal, for_removal_many, for_removal_one},
};
use crate::api::dsl::{
    method::SpacetimeDSLMethod,
    soft_delete::{SoftDeleteMarker, SoftDeleteMarkerKind},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// `soft_delete_<tables>_by_<index>`: retire every row an index matches.
pub(in crate::internal) fn for_soft_delete_many(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_many(Removal::Soft, shape, context)
}

/// `soft_delete_<table>_by_<index>`: retire the one row a unique index finds.
pub(in crate::internal) fn for_soft_delete_one(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> SpacetimeDSLMethod {
    for_removal_one(Removal::Soft, shape, context)
}

/// The statement that retires one row.
///
/// `receiver` is what the surrounding body calls `ctx()` on: `self` inside a DSL method,
/// `dsl` inside a cascade function, which is a free function taking the DSL as an argument.
pub(in crate::internal) fn set_marker(
    marker: &SoftDeleteMarker,
    receiver: &TokenStream,
    row: &Ident,
) -> TokenStream {
    let column_name = &marker.column_name;

    match marker.kind {
        SoftDeleteMarkerKind::Flag => quote! {
            #row.#column_name = true;
        },
        SoftDeleteMarkerKind::Timestamp => quote! {
            #row.#column_name = Some(#receiver.ctx().timestamp()?);
        },
    }
}

/// The expression that asks whether a row is already retired.
pub(in crate::internal) fn is_marked(marker: &SoftDeleteMarker, row: &TokenStream) -> TokenStream {
    let column_name = &marker.column_name;

    match marker.kind {
        SoftDeleteMarkerKind::Flag => quote! { #row.#column_name },
        SoftDeleteMarkerKind::Timestamp => quote! { #row.#column_name.is_some() },
    }
}
```

Add `mod soft_delete;` to `derive-input/src/internal/dsl/method.rs` and `use soft_delete::{for_soft_delete_many, for_soft_delete_one};`.

- [ ] **Step 4: Fill the four `Removal::Soft` arms in `removal.rs`**

The marker is `context.spacetimedsl_table.soft_delete_marker.as_ref().expect("a soft removal is only generated for a soft-deletable table")`.

1. **Name and doc comment.**

```rust
    let method_name = match removal {
        Removal::Hard => format_ident!("delete_{plural_table_name}_by_{index_name}"),
        Removal::Soft => format_ident!("soft_delete_{plural_table_name}_by_{index_name}"),
    };
```

and, in `for_removal_one`, the same with `singular_table_name`. The doc comment reads `"Try to delete all ..."` for `Hard` and `"Try to soft-delete all ..."` for `Soft`; keep the rest of the sentence identical so the two read as a pair.

2. **Reported strategy.**

```rust
    let reported_strategy = match removal {
        Removal::Hard => runtime::on_delete_strategy(&quote! { Delete }),
        Removal::Soft => runtime::on_delete_strategy(&quote! { SoftDelete }),
    };
```

Pass `&reported_strategy` where the current code passes `&runtime::on_delete_strategy(&quote! { Delete })`.

3. **Hooks.**

```rust
    let (before_hook, after_hook) = match removal {
        Removal::Hard => (
            &spacetimedsl_table.hooks.before_delete,
            &spacetimedsl_table.hooks.after_delete,
        ),
        Removal::Soft => (
            &spacetimedsl_table.hooks.before_soft_delete,
            &spacetimedsl_table.hooks.after_soft_delete,
        ),
    };
```

The soft hooks take three arguments and the before hook returns the row, so their call is built like `upsert.rs` builds `before_update_hook_call`, not like `delete.rs` builds `before_delete_hook`. For `Removal::Soft` the before-hook call is:

```rust
    let hook_call = runtime::dsl_method_hooks_call(
        hook_function_name,
        &quote! { self, &old_row, new_row },
    );

    quote! {
        let new_row = #hook_call?;
    }
```

and the after-hook call is:

```rust
    let hook_call = runtime::dsl_method_hooks_call(
        hook_function_name,
        &quote! { self, &old_row, &new_row },
    );

    quote! {
        #hook_call?;
    }
```

4. **The write.** For `Removal::Soft`, in place of the delete statement and its count check:

```rust
    let mut new_row = old_row.clone();

    // hook call goes here (see 3)

    #set_marker_statement

    let new_row = self
        .db()
        .#singular_table_name()
        .#primary_key_column_name()
        .update(new_row);
```

where `#set_marker_statement` is `soft_delete::set_marker(marker, &quote! { self }, &format_ident!("new_row"))`.

The many-row form loops over the rows it collected, because SpacetimeDB has no bulk update, and the already-retired filter runs before the loop:

```rust
    let rows_to_soft_delete: Vec<#struct_name> = #index_accessor
        .filter(#index_name)
        .filter(|row| !(#is_marked_expression))
        .collect();
```

where `#is_marked_expression` is `soft_delete::is_marked(marker, &quote! { row })`.

The one-row form keeps the not-found error and adds the early return:

```rust
    if #is_marked_expression {
        return Ok(#empty_deletion_result);
    }
```

placed after the not-found check and before the entry is built, with `#is_marked_expression` built over `quote! { row_to_delete }`.

`#empty_deletion_result` is `runtime::deletion_result` with `&quote! { vec![] }` and `&quote! { None }`, which the many-row generator already builds for the no-match case.

The soft body must **not** contain `set_updated_at_on_update`. A soft deletion leaves `updated_at` alone: `deleted_at` records the retirement, `updated_at` keeps meaning the last ordinary edit. The skeleton came from `delete.rs`, which never stamped, so this holds as long as nothing is pulled in from `upsert.rs` to add it.

- [ ] **Step 5: Carry the methods through the api types and the output**

In `derive-input/src/api/dsl/column.rs`:

```rust
pub struct SpacetimeDSLColumnMethodsForUniqueIndex {
    pub get_one_option: SpacetimeDSLMethod,
    // Only `Some(T)` if the table has an update method and this index is the primary key.
    pub update: Option<SpacetimeDSLMethod>,
    // Only `Some(T)` if the table has a delete method.
    pub delete_one: Option<SpacetimeDSLMethod>,
    // Only `Some(T)` if the table is soft-deletable.
    pub soft_delete_one: Option<SpacetimeDSLMethod>,
}

pub struct SpacetimeDSLColumnMethodsForIndex {
    pub get_many: SpacetimeDSLMethod,
    // Only `Some(T)` if the table has a delete method.
    pub delete_many: Option<SpacetimeDSLMethod>,
    // Only `Some(T)` if the table is soft-deletable.
    pub soft_delete_many: Option<SpacetimeDSLMethod>,
}
```

In `column_methods_for` in `derive-input/src/internal/dsl/method.rs`, add to each branch:

```rust
            soft_delete_many: match spacetimedsl_table.is_soft_deletable() {
                false => None,
                true => Some(for_soft_delete_many(&shape, context)),
            },
```

```rust
                soft_delete_one: match spacetimedsl_table.is_soft_deletable() {
                    false => None,
                    true => Some(for_soft_delete_one(&shape, context)),
                },
```

`SpacetimeDSLColumnMethods::map` also builds the two singleton branches; a singleton is never soft-deletable, so give both `soft_delete_one: None`.

In `get_column_dsl_methods` in `derive/src/output.rs`, emit them after the delete methods:

```rust
            if let Some(method) = &methods.soft_delete_one {
                dsl_methods.push(build_public_dsl_method(method)?)
            };
```

```rust
            if let Some(method) = &methods.soft_delete_many {
                dsl_methods.push(build_public_dsl_method(method)?)
            };
```

- [ ] **Step 6: Read every snapshot**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -20
cat derive/tests/snapshots/soft_delete_flag/Ticket/soft_delete_ticket_by_id.snap.new
cat derive/tests/snapshots/soft_delete_flag/Ticket/soft_delete_tickets_by_priority.snap.new
cat derive/tests/snapshots/soft_delete_without_delete_method/Ticket/table.snap.new
```

Check each of these:

- `soft_delete_ticket_by_id` returns `Result<DeletionResult, SpacetimeDSLError>`, errors when the row is missing, returns early when `row_to_delete.deleted` is already true, sets `deleted = true`, and calls `.update(...)` rather than `.delete(...)`.
- `soft_delete_tickets_by_priority` filters `!row.deleted` before collecting.
- `soft_delete_timestamp`'s methods write `Some(self.ctx().timestamp()?)` into `retired_at` and test `retired_at.is_some()`.
- `soft_delete_without_delete_method`'s `table.snap` manifest lists `soft_delete_ticket_by_id` and lists **no** `delete_ticket_by_id`.
- `soft_delete_hooks`' method calls `before_ticket_soft_delete` before the assignment to `deleted_at` and `after_ticket_soft_delete` after the `.update(...)`.
- No non-soft fixture moved. If one did, a `Removal::Hard` arm was changed; revert that.

- [ ] **Step 7: Accept and verify**

```bash
cargo insta accept
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
```

Expected: both PASS.

- [ ] **Step 8: Commit**

```bash
git add derive-input/src/internal/dsl/method/soft_delete.rs \
        derive-input/src/internal/dsl/method/removal.rs \
        derive-input/src/internal/dsl/method/delete.rs \
        derive-input/src/internal/dsl/method.rs \
        derive-input/src/api/dsl/column.rs derive/src/output.rs \
        derive/src/characterization_tests.rs derive/tests/fixtures derive/tests/snapshots
git commit -m "feat: generate the soft_delete_* DSL methods

One per index, retiring the rows it matches by writing the marker column and
updating the row. Already retired rows are skipped, so the methods are
idempotent.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 9: `on_soft_delete` on a foreign key

Parsing only. The strategies are not generated yet, and no cascade calls them; Task 11 does that. Splitting it this way keeps the rejections reviewable apart from the code generation.

**Files:**

- Modify: `derive-input/src/api/dsl/foreign_key.rs:3-9` (`ForeignKey`)
- Modify: `derive-input/src/internal/dsl/foreign_key.rs` (whole parser), `derive-input/src/internal/dsl/column.rs:13-35`, `derive-input/src/internal/column.rs:60-67`
- Modify: `derive-input/src/internal/dsl.rs` (symbol)
- Modify: `derive-input/src/internal/dsl/reference.rs:8-42`
- Modify: `derive-input/src/internal/dsl/method/foreign_key.rs:92-115` (grouping by strategy)
- Create: six `compile-tests/tests/ui/*.rs` + `.stderr` pairs, named in step 1

**Interfaces:**

- Consumes: `OnDeleteStrategy::SoftDelete` (Task 2), `SpacetimeDSLTable.soft_delete_marker` (Task 4).
- Produces:
  - `ForeignKey.on_delete_strategy: Option<OnDeleteStrategy>`
  - `ForeignKey.on_soft_delete_strategy: Option<OnDeleteStrategy>`
  - `ForeignKey::try_parse(has_delete_method: &bool, is_soft_deletable: bool, is_singleton: bool, field: &SatsField<'_>) -> syn::Result<Option<ForeignKey>>`

- [ ] **Step 1: Write the six failing tests**

Under `compile-tests/tests/ui/`, each a two-table module pair like `referenced_by_without_delete_method.rs`:

- `foreign_key_without_any_on_delete_field.rs` — `#[foreign_key(path = .., table = .., column = id)]` with neither strategy field.
- `on_soft_delete_with_delete_strategy.rs` — `on_soft_delete = Delete`.
- `on_soft_delete_with_set_zero_strategy.rs` — `on_soft_delete = SetZero`.
- `on_delete_soft_delete_on_table_that_is_not_soft_deletable.rs` — `on_delete = SoftDelete` on a table without `method(soft_delete = true)`.
- `on_soft_delete_soft_delete_on_table_that_is_not_soft_deletable.rs` — `on_soft_delete = SoftDelete` on the same kind of table.
- `referenced_by_without_delete_or_soft_delete_method.rs` — `#[referenced_by]` on a table with `method(delete = false)` and no `soft_delete`. This replaces the existing `referenced_by_without_delete_method.rs`; delete that file and its `.stderr` in step 4, because the message it pins is being rewritten.

- [ ] **Step 2: Run them to make sure they fail**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -30
```

Expected: FAIL for all six.

- [ ] **Step 3: Parse both fields and enforce the per-field strategy sets**

Add `symbol!(on_soft_delete);` to `derive-input/src/internal/dsl.rs`.

In `derive-input/src/api/dsl/foreign_key.rs`:

```rust
pub struct ForeignKey {
    pub path: Path,
    pub table_name: Ident,
    pub primary_key_column_name: Ident,
    /// What happens to the rows of this table when a row of the referenced table is
    /// deleted. `None` while the referenced table is not deletable.
    pub on_delete_strategy: Option<OnDeleteStrategy>,
    /// What happens to the rows of this table when a row of the referenced table is
    /// soft-deleted. `None` while the referenced table is not soft-deletable.
    pub on_soft_delete_strategy: Option<OnDeleteStrategy>,
}
```

In `derive-input/src/internal/dsl/foreign_key.rs`:

- add `on_soft_delete` to the `attr.parse_nested_meta` matcher, parsed the same way as `on_delete` but through a parser that accepts only the three legal strategies;
- replace the `ok_or_else` that made `on_delete` mandatory with:

```rust
            if on_delete_strategy.is_none() && on_soft_delete_strategy.is_none() {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "A `#[foreign_key]` must set `on_delete`, `on_soft_delete`, or both, e.g. `on_delete = Delete`.\nSet `on_delete` when the referenced table has a delete method, set `on_soft_delete` when it is soft-deletable, and set both when it is both. The referenced table's own `#[referenced_by]` decides which of them is required; leaving out a required one is an unresolved import naming the field to add.",
                ));
            }
```

- give `OnDeleteStrategy` two parsers rather than one, so the accepted set is stated where it applies:

```rust
impl OnDeleteStrategy {
    fn try_parse_for_on_delete(
        meta: &ParseNestedMeta<'_>,
        tokens: &Meta,
    ) -> syn::Result<OnDeleteStrategy> {
        let action_variant: Ident = meta.value()?.parse()?;

        match action_variant.to_string().as_str() {
            "Error" => Ok(OnDeleteStrategy::Error),
            "Delete" => Ok(OnDeleteStrategy::Delete),
            "SoftDelete" => Ok(OnDeleteStrategy::SoftDelete),
            "SetNone" => Err(syn::Error::new_spanned(
                tokens,
                "Because Option is currently not allowed on primary_key and unique/btree indices, `OnDeleteStrategy::SetNone` isn't implemented yet. `OnDeleteStrategy` must be one of `Error`, `Delete`, `SoftDelete`, `SetZero` or `Ignore` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.".to_string(),
            )),
            "SetZero" => Ok(OnDeleteStrategy::SetZero),
            "Ignore" => Ok(OnDeleteStrategy::Ignore),
            _ => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy` must be one of `Error`, `Delete`, `SoftDelete`, `SetNone`, `SetZero` or `Ignore` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.".to_string(),
            )),
        }
    }

    fn try_parse_for_on_soft_delete(
        meta: &ParseNestedMeta<'_>,
        tokens: &Meta,
    ) -> syn::Result<OnDeleteStrategy> {
        let action_variant: Ident = meta.value()?.parse()?;

        match action_variant.to_string().as_str() {
            "Error" => Ok(OnDeleteStrategy::Error),
            "SoftDelete" => Ok(OnDeleteStrategy::SoftDelete),
            "Ignore" => Ok(OnDeleteStrategy::Ignore),
            "Delete" => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy::Delete` is not allowed in `on_soft_delete`! Soft-deleting a row must not physically remove the rows which reference it. `on_soft_delete` must be one of `Error`, `SoftDelete` or `Ignore`.".to_string(),
            )),
            "SetZero" => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy::SetZero` is not allowed in `on_soft_delete`! Soft deletion preserves the row, so clearing the foreign key column would destroy what it preserved. `on_soft_delete` must be one of `Error`, `SoftDelete` or `Ignore`.".to_string(),
            )),
            _ => Err(syn::Error::new_spanned(
                tokens,
                "`OnDeleteStrategy` must be one of `Error`, `SoftDelete` or `Ignore` in `#[foreign_key(on_soft_delete = OnDeleteStrategy)]`, e.g. `on_soft_delete = SoftDelete`.".to_string(),
            )),
        }
    }
}
```

- keep the two existing checks and extend them to both fields. `SetZero` on a private column stays rejected. `Delete` without a delete method stays rejected. Add:

```rust
            let uses_soft_delete = on_delete_strategy == Some(OnDeleteStrategy::SoftDelete)
|   | on_soft_delete_strategy == Some(OnDeleteStrategy::SoftDelete); |

            if uses_soft_delete && !is_soft_deletable {
                return Err(syn::Error::new_spanned(
                    &attr.meta,
                    "`OnDeleteStrategy::SoftDelete` is only allowed when this table is soft-deletable (`#[dsl(method(soft_delete = true))]`)!\nThe strategy retires the rows of this table, which needs a marker column for the DSL to write.",
                ));
            }
```

Thread `is_soft_deletable` through: `ForeignKey::try_parse` gains the parameter, `SpacetimeDSLColumn::try_parse` passes it, and `internal/column.rs` supplies `spacetimedsl_table.is_soft_deletable()` beside the `&spacetimedsl_table.has_delete_method` it already passes.

- [ ] **Step 4: Widen the `#[referenced_by]` gate**

In `derive-input/src/internal/dsl/reference.rs`, `ReferencingTable::try_parse` takes `is_soft_deletable: bool` beside `has_delete_method`, and the check becomes:

```rust
            if !has_delete_method && !is_soft_deletable {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`#[referenced_by]` is only allowed when the table has a delete method (`#[dsl(method(delete = true))]`) or is soft-deletable (`#[dsl(method(soft_delete = true))]`)!\nThe on-delete strategies it declares run when a row of this table is deleted or soft-deleted, neither of which the DSL can do while both are disabled.",
                ));
            }
```

The call site in `internal/dsl/table.rs` currently passes `&has_delete_method.unwrap_or(true)`. It must now also pass whether the table is soft-deletable, which the marker detection already produced earlier in the same function.

Delete `compile-tests/tests/ui/referenced_by_without_delete_method.rs` and its `.stderr`; the new case replaces it.

- [ ] **Step 5: Keep the strategy grouping compiling**

`method/foreign_key.rs` groups the foreign key columns by `on_delete_strategy`, which is now an `Option`. For this task, group only the columns whose `on_delete_strategy` is `Some`, so generated output for existing tables does not change:

```rust
        let on_delete_strategy = match &column_with_foreign_key
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .expect("every column of a foreign key group carries a foreign key")
            .on_delete_strategy
        {
            None => continue,
            Some(on_delete_strategy) => on_delete_strategy,
        };
```

Task 11 gives the soft strategies their own grouping.

- [ ] **Step 6: Regenerate the diagnostics and read them**

```bash
TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests 2>&1 | tail -10
git diff --stat compile-tests/tests/ui
```

Read every `.stderr` that changed or appeared. Six new messages, one deleted pair, and no unrelated `.stderr` may move. If one does, a message was edited that this task should not touch.

- [ ] **Step 7: Run both harnesses**

```bash
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
cargo test -p spacetimedsl_derive 2>&1 | tail -5
```

Expected: both PASS, and no snapshot moves. Every existing fixture sets `on_delete`, so the grouping still sees the same columns.

- [ ] **Step 8: Commit**

```bash
git add derive-input/src/api/dsl/foreign_key.rs \
        derive-input/src/internal/dsl/foreign_key.rs \
        derive-input/src/internal/dsl/column.rs derive-input/src/internal/column.rs \
        derive-input/src/internal/dsl/reference.rs derive-input/src/internal/dsl.rs \
        derive-input/src/internal/dsl/table.rs \
        derive-input/src/internal/dsl/method/foreign_key.rs \
        compile-tests/tests/ui
git commit -m "feat: accept on_soft_delete on a foreign key

on_delete becomes optional and a foreign key must set at least one of the two.
on_soft_delete takes Error, SoftDelete or Ignore, and #[referenced_by] is now
allowed on a table that is only soft-deletable.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 10: The four compile-error-check identifiers

The rename and the split, with no new cascade code behind them yet. After this task a hard-deletion pairing still verifies exactly as before, under a longer name.

**Files:**

- Modify: `derive-input/src/internal/dsl/method/naming.rs` (whole file)
- Modify: `derive-input/src/internal/dsl/method/foreign_key.rs:220-240`, `derive-input/src/internal/dsl/method/referenced_by.rs:190-210`

**Interfaces:**

- Consumes: nothing new.
- Produces:

```rust
pub(in crate::internal) fn referencing_table_compile_error_check_for_deletions(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident;

pub(in crate::internal) fn referencing_table_compile_error_check_for_soft_deletions(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident;

pub(in crate::internal) fn referenced_table_compile_error_check_for_deletions(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident;

pub(in crate::internal) fn referenced_table_compile_error_check_for_soft_deletions(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident;
```

- [ ] **Step 1: Record the baseline**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -5
git status --short
```

Expected: PASS, clean.

- [ ] **Step 2: Rewrite `naming.rs`'s four check builders**

Replace `referencing_table_compile_error_check` and `referenced_table_compile_error_check` with:

```rust
pub(in crate::internal) fn referencing_table_compile_error_check_for_deletions(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_with_on_delete_defined_referencing_the_{referenced_table_name}_table"
    )
}

pub(in crate::internal) fn referencing_table_compile_error_check_for_soft_deletions(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_with_on_soft_delete_defined_referencing_the_{referenced_table_name}_table"
    )
}

pub(in crate::internal) fn referenced_table_compile_error_check_for_deletions(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

pub(in crate::internal) fn referenced_table_compile_error_check_for_soft_deletions(
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_soft_deletable_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}
```

Extend the module documentation to say that each direction is split by capability, and that a table emits the half it can perform and imports the half it needs from the other side.

- [ ] **Step 3: Point the two call sites at the deletion halves**

In `method/foreign_key.rs`, `referencing_table_compile_error_check(...)` becomes `referencing_table_compile_error_check_for_deletions(...)` and `referenced_table_compile_error_check(...)` becomes `referenced_table_compile_error_check_for_deletions(...)`.

In `method/referenced_by.rs`, make the same two substitutions.

Nothing else changes yet: both sides still emit and import exactly one identifier per pair, and it is the deletion one.

- [ ] **Step 4: Read the snapshots**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -20
```

Expected: FAIL, with `.snap.new` for every fixture that has a foreign key: `foreign_key_and_referenced_by`, the four `on_delete_*`, `delete_hooks_with_foreign_key_on_unique_index`, `singleton_with_foreign_key`.

Every diff must be a pure rename of the two identifiers. A diff that adds or removes a `use` or a `trait` means a call site was missed.

- [ ] **Step 5: Accept and verify**

```bash
cargo insta accept
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
```

Expected: both PASS.

- [ ] **Step 6: Commit**

```bash
git add derive-input/src/internal/dsl/method/naming.rs \
        derive-input/src/internal/dsl/method/foreign_key.rs \
        derive-input/src/internal/dsl/method/referenced_by.rs \
        derive/tests/snapshots
git commit -m "refactor: split the compile error checks by removal kind

Each direction of the paired check now names the inner field it is about, so
a table that is deletable, soft-deletable or both pairs with its referencing
tables one capability at a time.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 11: The soft cascade

The last generation task: a second pair of cascade entry points on a soft-deletable referenced table, a second set of strategy functions on the referencing side, the `SoftDelete` strategy arm, and the soft halves of both compile-error checks wired to the flags that decide them.

**Files:**

- Modify: `derive-input/src/api/dsl/table.rs:60-95`
- Modify: `derive-input/src/internal/dsl/method/naming.rs` (dispatcher names)
- Modify: `derive-input/src/internal/dsl/method/referenced_by.rs`, `method/foreign_key.rs`, `method/on_delete_strategy.rs`, `method.rs`, `method/removal.rs`
- Modify: `derive/src/output.rs:84-115`
- Create: `derive/tests/fixtures/on_soft_delete_cascade.rs`
- Modify: `derive/src/characterization_tests.rs`

**Interfaces:**

- Consumes: everything from Tasks 2, 4, 6, 8, 9, 10.
- Produces:

```rust
// api/dsl/table.rs
pub struct CascadeEntryPoints {
    pub after_one_row: SpacetimeDSLMethod,
    pub after_multiple_rows: SpacetimeDSLMethod,
}

pub struct OnDeleteStrategiesOfReferencingTables {
    pub on_deletion: Option<CascadeEntryPoints>,
    pub on_soft_deletion: Option<CascadeEntryPoints>,
}

pub struct OnDeleteStrategiesOfTheReferencedTable {
    pub on_deletion: Option<CascadeEntryPoints>,
    pub on_soft_deletion: Option<CascadeEntryPoints>,
}
```

```rust
// method/naming.rs
pub(in crate::internal) fn referenced_table_function_name(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    referenced_table_name: &Ident,
) -> Ident;

pub(in crate::internal) fn referencing_table_function_name(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident;
```

- [ ] **Step 1: Write the failing test**

Create `derive/tests/fixtures/on_soft_delete_cascade.rs`:

```rust
//! Covers `on_soft_delete`: retiring a referenced row retires the rows which reference it,
//! and those rows' own referencing table is consulted in turn.
//!
//! `Author` is soft-deletable and deletable, so its referencing table sets both fields.
//! `Book` is soft-deletable and is itself referenced by `Review`, which is what makes the
//! cascade recurse.

#[spacetimedsl::dsl(
    plural_name = authors,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(accessor = author, public)]
pub struct Author {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(AuthorId)]
    #[referenced_by(path = self, table = book)]
    id: u64,

    pub name: String,

    deleted: bool,
}

#[spacetimedsl::dsl(
    plural_name = books,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(accessor = book, public)]
pub struct Book {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(BookId)]
    #[referenced_by(path = self, table = review)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(AuthorId)]
    #[foreign_key(path = self, table = author, column = id, on_delete = Delete, on_soft_delete = SoftDelete)]
    pub author_id: u64,

    deleted: bool,
}

#[spacetimedsl::dsl(plural_name = reviews, method(update = true, delete = true))]
#[spacetimedb::table(accessor = review, public)]
pub struct Review {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(BookId)]
    #[foreign_key(path = self, table = book, column = id, on_delete = Delete, on_soft_delete = Ignore)]
    pub book_id: u64,
}
```

Add to `derive/src/characterization_tests.rs`:

```rust
#[test]
fn on_soft_delete_cascade() {
    snapshot_fixture("on_soft_delete_cascade");
}
```

- [ ] **Step 2: Run it to make sure it fails**

```bash
cargo test -p spacetimedsl_derive on_soft_delete_cascade 2>&1 | tail -20
```

Expected: FAIL. The `.snap.new` files hold the deletion cascade only, and the `on_soft_delete` fields are parsed and then ignored.

- [ ] **Step 3: Parameterize the dispatcher names**

In `naming.rs`, give both function-name builders a `removal: Removal` first parameter, and build the suffix from it:

```rust
fn removal_suffix(removal: Removal, one_or_multiple: &OneOrMultiple) -> &'static str {
    match (removal, one_or_multiple) {
        (Removal::Hard, OneOrMultiple::One) => "was_deleted",
        (Removal::Hard, OneOrMultiple::Multiple) => "were_deleted",
        (Removal::Soft, OneOrMultiple::One) => "was_soft_deleted",
        (Removal::Soft, OneOrMultiple::Multiple) => "were_soft_deleted",
    }
}
```

The two builders keep their current prefixes and end in that suffix, so `Removal::Hard` reproduces today's names exactly.

- [ ] **Step 4: Give both strategy-pair structs two optional pairs**

Rewrite the two structs in `api/dsl/table.rs` as in the Interfaces block above, with documentation saying that a pair is present when the table can perform that kind of removal, and that both members of a pair exist or neither does.

In `method.rs`:

- build the referenced-table pair for `Removal::Hard` when `referencing_tables` is non-empty **and** `has_delete_method`, and for `Removal::Soft` when it is non-empty **and** `is_soft_deletable()`;
- group the foreign key columns twice, once by `on_delete_strategy` and once by `on_soft_delete_strategy`, and build the referencing-table pair for a `Removal` only when that grouping is non-empty.

In `derive/src/output.rs`, emit each present pair, keeping the existing rule that every one-row method is emitted before any many-row method.

- [ ] **Step 5: Wire the soft halves of both checks**

In `method/referenced_by.rs`, for each referencing table:

- emit `referenced_table_compile_error_check_for_deletions` when the referenced table has a delete method, and `..._for_soft_deletions` when it is soft-deletable;
- import `referencing_table_compile_error_check_for_deletions` in the `Removal::Hard` entry points and `..._for_soft_deletions` in the `Removal::Soft` ones.

In `method/foreign_key.rs`, for each referenced table:

- emit `referencing_table_compile_error_check_for_deletions` when this table's foreign keys to it set `on_delete`, and `..._for_soft_deletions` when they set `on_soft_delete`;
- import `referenced_table_compile_error_check_for_deletions` from the referenced table's path in the `Removal::Hard` functions and `..._for_soft_deletions` in the `Removal::Soft` ones.

- [ ] **Step 6: Add the `SoftDelete` strategy arm**

In `method/on_delete_strategy.rs`, add a `OnDeleteStrategy::SoftDelete` arm built from the `OnDeleteStrategy::Delete` arm, with four substitutions:

1. the hooks are `spacetimedsl_table.hooks.before_soft_delete` / `.after_soft_delete`, called with `&dsl, &old_row, new_row` and `&dsl, &old_row, &new_row` — the before hook returns the row;
2. the write is `set_marker` followed by `#spacetimedb_call_prefix.#primary_key_column_name().update(row)` rather than `.delete(...)`, with `soft_delete::set_marker(marker, &quote! { dsl }, ...)` because a cascade function receives the DSL as `dsl`, not `self`;
3. rows already retired are skipped, using `soft_delete::is_marked`;
4. the recursion calls `referenced_table_function_name(Removal::Soft, ..)`, and the strategies it fans out to are the three `on_soft_delete` accepts: `Error`, `SoftDelete`, `Ignore`.

Give `on_delete_strategy_implementation` a `removal: Removal` parameter so the `Error` and `Ignore` arms, which are identical for both kinds, are not duplicated: they only report a different strategy value and recurse through a different dispatcher.

- [ ] **Step 7: Call the soft dispatchers from the soft methods**

In `removal.rs`, the `Removal::Soft` body runs the same four strategy passes the `Removal::Hard` body runs — `Error` before writing, then the rest — but through `referenced_table_function_call_for_dsl_method` with `Removal::Soft`, and over the three strategies `on_soft_delete` accepts. Give that function in `referenced_by.rs` a `removal: Removal` parameter and pass it through to `referenced_table_function_name`.

- [ ] **Step 8: Read the snapshots**

```bash
cargo test -p spacetimedsl_derive 2>&1 | tail -20
ls derive/tests/snapshots/on_soft_delete_cascade/*/
```

Check:

- `Author` holds four cascade entry points: `..._after_one_row_of_the_author_table_was_deleted`, `..._were_deleted`, `..._was_soft_deleted`, `..._were_soft_deleted`.
- `Book` holds four strategy functions towards `author`, two per kind, and four cascade entry points of its own towards `review`.
- `Review` holds two strategy functions towards `book` for deletions and two for soft deletions, the latter with only `Error`, `SoftDelete` and `Ignore` arms.
- `Author`'s `table.snap` emits both `..._is_not_deletable_or_...` and `..._is_not_soft_deletable_or_...` traits for `book`, and `Book` emits both `..._with_on_delete_defined_...` and `..._with_on_soft_delete_defined_...` traits for `author`.
- `soft_delete_author_by_id` calls the soft dispatcher, not the hard one.
- No fixture without `on_soft_delete` moved, beyond what Task 10 already renamed.

- [ ] **Step 9: Accept and verify**

```bash
cargo insta accept
cargo test -p spacetimedsl_derive 2>&1 | tail -5
cargo test -p spacetimedsl-compile-tests 2>&1 | tail -5
cargo build --workspace 2>&1 | tail -5
```

Expected: all PASS.

- [ ] **Step 10: Commit**

```bash
git add derive-input/src derive/src derive/tests
git commit -m "feat: cascade on_soft_delete into the referencing tables

A soft-deletable referenced table earns a second pair of cascade entry points,
a referencing table earns a second set of strategy functions, and the
SoftDelete strategy retires the rows it reaches and recurses.

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 12: Documentation and the runtime example

The last task. It proves the feature against a real SpacetimeDB instance and writes it down for the people who will use it.

**Files:**

- Modify: `docs/DOCUMENTATION.md`, `README.md`, `examples/test/src/lib.rs`

**Interfaces:**

- Consumes: the whole feature.
- Produces: nothing other tasks depend on.

- [ ] **Step 1: Add a soft-deletable pair of tables to the example module**

Add a module to `examples/test/src/lib.rs`, beside the existing ones:

```rust
pub mod soft_deletion {
    use spacetimedb::Timestamp;

    /// A retired `Archive` keeps its row, so a reducer can still read what was retired.
    #[spacetimedsl::dsl(
        plural_name = archives,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(
        accessor = archive,
        public,
    )]
    pub struct Archive {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(ArchiveId)]
        #[referenced_by(path = crate::soft_deletion, table = archive_entry)]
        id: u64,

        pub label: String,

        #[updated_at]
        modified_at: Option<Timestamp>,

        deleted: bool,
    }

    /// Retiring an `Archive` retires its entries; deleting one removes them.
    #[spacetimedsl::dsl(
        plural_name = archive_entries,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(
        accessor = archive_entry,
        public,
    )]
    pub struct ArchiveEntry {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(ArchiveId)]
        #[foreign_key(
            path = crate::soft_deletion,
            table = archive,
            column = id,
            on_delete = Delete,
            on_soft_delete = SoftDelete,
        )]
        pub archive_id: u64,

        #[set_on_soft_delete]
        retired_at: Option<Timestamp>,
    }
}
```

Import what the reducer needs at the top of the module the `tester` reducer lives in, matching how that module already imports the other tables' types and `Create...` structs.

- [ ] **Step 2: Assert the behavior in the `tester` reducer**

Append to the body of `tester`, before its final `Ok(())`:

```rust
        let archive = dsl.create_archive(CreateArchive {
            label: "first".to_string(),
        })?;
        let archive_id = archive.get_id();

        let entry = dsl.create_archive_entry(CreateArchiveEntry {
            archive_id: archive_id.clone(),
        })?;
        let entry_id = entry.get_id();

        let modified_at_before_soft_delete = *archive.get_modified_at();

        dsl.soft_delete_archive_by_id(&archive_id)?;

        let retired_archive = match dsl.get_archive_by_id(&archive_id)? {
            None => {
                return Err("A soft-deleted Archive row should still be readable!".to_string());
            }
            Some(retired_archive) => retired_archive,
        };

        if !retired_archive.get_deleted() {
            return Err("soft_delete_archive_by_id should have set the marker!".to_string());
        }

        if retired_archive
            .get_modified_at()
            .ne(&modified_at_before_soft_delete)
        {
            return Err("A soft deletion should leave modified_at alone!".to_string());
        }

        let retired_entry = match dsl.get_archive_entry_by_id(&entry_id)? {
            None => {
                return Err(
                    "An ArchiveEntry of a soft-deleted Archive should still exist!".to_string(),
                );
            }
            Some(retired_entry) => retired_entry,
        };

        if retired_entry.get_retired_at().is_none() {
            return Err(
                "on_soft_delete = SoftDelete should have retired the ArchiveEntry!".to_string(),
            );
        }

        let repeated = dsl.soft_delete_archive_by_id(&archive_id)?;

        if !repeated.entries.is_empty() {
            return Err(
                "Soft-deleting an already retired row should report no entries!".to_string(),
            );
        }
```

The accessor names follow the generated ones: `get_<column>` for every column, including the private marker. If a getter's return type makes a comparison awkward — `get_deleted` returns `&bool`, `get_retired_at` returns `&Option<Timestamp>` — dereference or borrow at the call rather than changing the generator.

- [ ] **Step 3: Run the module against a local SpacetimeDB**

```bash
./x test
```

Expected: the module publishes, the reducer runs and the logs show no assertion failure. The local `spacetime` server is already running.

- [ ] **Step 4: Write the reference documentation**

In `docs/DOCUMENTATION.md`, add a soft deletion section covering: `method(soft_delete)` and its requirement on `method(delete)`; the marker column's two shapes, the three ways to claim the role and the privacy rule; the generated method names, their return type and their idempotency; the two hooks and their signatures; `on_soft_delete` and which strategies it takes; the `SoftDelete` strategy and what it requires; and the fact that `get_*` still returns retired rows.

Match the surrounding sections' heading depth and tone.

- [ ] **Step 5: Mention the feature in the README**

Add soft deletion to the feature list in `README.md`, in one line, in the style of the lines around it.

- [ ] **Step 6: Format the markdown tables**

```bash
./format-tables.sh docs/DOCUMENTATION.md
./format-tables.sh README.md
```

- [ ] **Step 7: Verify everything**

```bash
./x unit-test
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features 2>&1 | tail -20
```

Expected: tests pass, formatting clean, no new clippy warning.

- [ ] **Step 8: Commit**

```bash
git add docs/DOCUMENTATION.md README.md examples/test/src/lib.rs
git commit -m "docs: document soft deletion and exercise it in examples/test

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

## Deferred to a follow-up story

Recorded here so they are not rediscovered as gaps:

- Read filtering: `get_*`, `get_all_*` and `count_of_all_*` return retired rows.
- No restore or undelete method.
- `docs/entity_dsl_methods.png` is not regenerated.
