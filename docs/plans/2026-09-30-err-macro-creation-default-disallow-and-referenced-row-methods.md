# `err!`, `#[creation_default]`, `#[disallow]` and Referenced-Row Methods — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve issues #187, #188, #186 and #190 in one pull request: an `err!` macro for users, a `#[creation_default(...)]` helper attribute which fills a column on create, a `#[disallow(...)]` helper attribute which refuses forbidden values on every row write, and wrapper-type methods which look up the row a foreign key references.

**Architecture:** `err!` is a `macro_rules!` of the runtime crate (`src/lib.rs`). The two helper attributes are parsed in the new modules `derive-input/src/internal/dsl/creation_default.rs` and `derive-input/src/internal/dsl/disallow.rs`, carried as fields of the public `SpacetimeDSLColumn`, and turned into generated code by `derive-input/src/internal/dsl/method/create.rs` (defaults) and the new `derive-input/src/internal/dsl/method/disallow.rs` (checks), which every generator that writes a row calls. The referenced-row methods are generated in `derive-input/src/internal/dsl/method/wrapper_method.rs` next to the existing wrapper methods, into the new list `SpacetimeDSLTableMethods::referenced_row_methods`, which `spacetimedsl_derive` emits only for a struct with a single `#[dsl]` attribute.

**Tech Stack:** Rust 2024 (toolchain pinned in `rust-toolchain.toml`), `syn` 3 / `quote` / `proc-macro2`, SpacetimeDB `=2.10.1`, `insta` snapshots, `trybuild` compile tests, the `examples/test` SpacetimeDB module as the runtime gate, PowerShell `x.ps1` as the only build entry point.

**Spec:** The four GitHub issues, read together with *Decisions taken while planning* below, which fill the gaps the issues left open:

- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/187 — an `err!` macro, a shortcut for `Err(SpacetimeDSLError::Error(format!(…)))`
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/188 — `#[creation_default(...)]` leaves a column out of `Create<Table>` and fills it with the given value; not allowed on `singleton(with_default)` tables
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/186 — `#[disallow(zero)]`, `#[disallow(decreasing)]` and `#[disallow(increasing)]`, checked after any before hook
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/190 — wrapper-type methods in the other direction: `alliance.get_server_id().get_season(dsl)?` looks up the season the server references

## Global Constraints

- Build and test only through `x.ps1`. Never run `cargo` directly: a workspace member built on its own fails to link against SpacetimeDB.
- The crates stay at version `0.24.0`, which is not released yet. `docs/MIGRATION.md` gets entries only in its existing section `## 0.23 → 0.24`, and only for what affects existing code and for every change of a public `derive-input` type; the new features themselves are documented in `docs/DOCUMENTATION.md`.
- One branch, one pull request, one commit per task, in the order of this plan.
- Every commit message ends with the line `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Names follow AGENTS.md *Self-Documenting Code*: no abbreviations, no comment that repeats the code, no comment about the past or about an issue or task.
- Every user-facing `syn::Error` is written in `derive-input/src/internal/error.rs`.
- `derive-input/src/api/` gains only public fields and types another crate reads. Helper functions go into `derive-input/src/internal/` as plain `pub`; a method `internal` adds to an `api` type stays `pub(crate)`.
- A change of a public `derive-input` type updates `derive/src/data_transfer_contract_tests.rs` and adds an entry under *For crates building on `spacetimedsl_derive-input`* in `docs/MIGRATION.md`, in the same commit.
- A new field attribute joins `api::attribute::FIELD_ATTRIBUTE_NAMES`, the `attributes(...)` list of the `SpacetimeDSL` derive in `derive/src/lib.rs` and the fixture `every_field_attribute`, in the same commit.
- A runtime test group is one file in `examples/test/src` with a `pub(crate) fn run_tests`, registered in `TEST_GROUPS` in `examples/test/src/lib.rs`. Its table accessors are unique across the whole module, and every count it asserts is relative.
- A compile-test fixture is valid except for the one rejection it pins, carries the companions the fix relies on, and names the follow-on errors of a rejected `#[dsl]` in its `//!` comment (AGENTS.md *Diagnostics*).
- Knowledge in `AGENTS.md`, `README.md`, `docs/DOCUMENTATION.md`, `docs/MIGRATION.md` and `CODE_QUALITY_REPORT.md` changes in the same commit as the code it describes.

## Decisions taken while planning

| Topic | Decision |
| --- | --- |
| Delivery | One pull request, one commit per task, in the order #187, #188, #186, #190: `#[creation_default]` comes before `#[disallow]`, because `#[disallow(zero)]` rejects `#[creation_default(0)]`. |
| Version | No bump: `0.24.0` is unreleased. See *Global Constraints* for what `docs/MIGRATION.md` lists. |
| Plan location | `docs/plans/`, as the previous plans. |
| #187 value | `err!(…)` evaluates to `Err(SpacetimeDSLError::Error(…))`. A caller `return`s it or ends a function with it. |
| #187 arguments | A literal first argument is the `format!` string; it and every token after it are handed to `format!` unchanged, so inline and named arguments work. Any other single expression is the whole message, turned into a `String` with `ToString`. |
| #187 export | `#[macro_export]` at the root of the `spacetimedsl` crate, re-exported by `spacetimedsl::prelude`, and through it by the prelude `spacetimedsl!()` generates. |
| #187 adoption | `docs/DOCUMENTATION.md` (reducer example, prelude list, new *Refusing With `err!`*), `README.md`, `examples/blackholio` (two sites) and `examples/test` (two sites) switch to `err!`. |
| #188 syntax | `#[creation_default(<expression>)]`, one per column. The expression is parsed as `syn::Expr`, so `derive-input`'s `syn` dependency gains the `full` feature. |
| #188 type | The expression has the field's own type, also on a column with a wrapper type: `create_<table>` emits `let <column>: <field type> = <expression>;`. |
| #188 allowed | Private and public columns, wrapped columns, foreign key columns (`0` and `Uuid::NIL` reference no row, so create skips their reference check), tables with `#[dsl(singleton)]`. |
| #188 rejected | `singleton(with_default)` tables (the issue); `#[auto_inc]`, `#[auto_gen]`, the `set_on_create` and `set_on_update` columns and the soft-delete marker, which create fills in already; `#[primary_key]` and `#[unique]` columns, where every created row would repeat the value; a missing expression; a second attribute. |
| #188 docs | `create_<table>` gets a `# Defaults` section which lists each defaulted column with its expression, written by a small token printer as `String::new()` rather than `proc_macro2`'s `String :: new ()`. |
| API model | `SpacetimeDSLColumn::creation_default: Option<syn::Expr>` and `SpacetimeDSLColumn::disallowed: BTreeSet<Disallowed>`, next to `auto_generated_uuid_version`. |
| #186 syntax | One `#[disallow(...)]` per column, with a list of rules: `zero`, `decreasing`, `increasing`. A second attribute, a rule named twice and an empty list are rejected. |
| #186 types | `zero`: `u8`–`u128` and `Uuid`. `decreasing` and `increasing`: `u8`–`u128`, `i8`–`i128`, `f32` and `f64`, in every spelling; `ColumnTypeKind` gains `SignedInteger` and `Float`. |
| #186 rejected combinations | `decreasing` together with `increasing` (fix: remove both and make the column private); `decreasing` or `increasing` on the primary key, which an update never changes, or on a private column, which has no setter; `zero` or `decreasing` on a column with `on_delete = SetZero`; `zero` with a literal `#[creation_default(0)]` or `#[creation_default(…Uuid::NIL)]` (one compile test per shape). |
| #186 allowed | `zero` on `#[auto_inc]` and `#[auto_gen]` columns: a row can reach `0` through raw SpacetimeDB access in the table's own module, for example a system user, and the rule then keeps the DSL from writing it. `create_<table>` skips an `#[auto_inc]` column, whose placeholder `0` SpacetimeDB replaces. |
| #186 where | Every row write: `create_<table>`, `update_<table>_by_<key>`, both paths of `upsert_<table>`, `soft_delete_*`, and the writes of `SetZero` and `SoftDelete` cascades. `zero` is checked in all of them, `decreasing` and `increasing` wherever a stored row exists. Each check runs after the write's before hook and before the framework writes the columns it owns. |
| #186 error | `SpacetimeDSLError::Error(String)`: *Disallowed Value Error while trying to create a row in the `player` table because `level` is `0`, which `#[disallow(zero)]` forbids!*, *…while trying to update the row `{ id : 7 }` in the `player` table because `score` would decrease from `10` to `5`, which `#[disallow(decreasing)]` forbids!* and *…while trying to soft delete the row `{ id : 7 }` in…*. A `Uuid` column says `Uuid::NIL`; a singleton's row is `{ id : 0 }`. |
| #186 floats | An unchanged value — the same bits, NaN included — never breaks a rule. Otherwise `decreasing` requires `new >= old` and `increasing` requires `new <= old`, both through `partial_cmp`, so a change to or from NaN breaks both. |
| #186 cascades | A broken rule inside a cascade stops it like a hook error and is carried in `error_from_hook`. The field keeps its name; its documentation and the `Display` prefix of `DeletionResult` become *Error which stopped the cascade:*. |
| #186 docs | The setter of a column with rules says which values a write refuses. The create, update, upsert and soft-delete methods get a `# Disallowed values` section. |
| #190 receivers | The wrapper type of the primary key of a non-singleton table, created or used: one method per foreign key column besides the primary key. |
| #190 names | `get_<referenced table>`. Where a table references the same table through several columns or references itself: `get_<stem>`, the column name without `_<referenced primary key>`, else without `_id`, else whole. Two equal names in one table are rejected with a diagnostic. |
| #190 return type | `Result<<path::<table>__TableHandle as ::spacetimedb::Table>::Row, SpacetimeDSLError>`: SpacetimeDB 2.10.1 generates `<accessor>__TableHandle` next to every table, whose `Table::Row` is the struct the foreign key does not name. A missing row, and a column holding `0` or `Uuid::NIL`, give a `NotFoundError`. |
| #190 several `#[dsl]` | The tables of such a struct share its wrapper types, so the lookup would be ambiguous: `spacetimedsl_derive` emits `referenced_row_methods` only for a struct with a single `#[dsl]` attribute. |
| #190 collisions | Two tables referencing each other through unique foreign keys, and two tables sharing a primary key wrapper which reference the same table, add a method of the same name to one wrapper type (rustc E0592). One UNSUPPORTED COMBINATION compile test per shape pins it, as the issue decided; the fix is the new `#[foreign_key(..., referenced_row_method = false)]`. |
| #190 opt-out | `referenced_row_method = false` switches off the method of one foreign key column. The argument is rejected where it has no effect — on a foreign key on the primary key and on a singleton table — and accepted without effect on a struct with several `#[dsl]` attributes, which `derive-input` cannot see. |

## Found while planning and left out of scope

Worth an issue each; none is changed by this plan:

1. The before-update prelude of `update_<table>_by_<key>` looks up the stored row with `.expect("Row should exist for update")`, so updating a row that no longer exists panics instead of returning a `NotFoundError` when the table has an update hook.
2. `update_<table>_by_<key>` runs its reference-integrity checks before the `before_update` hook, so a hook which changes a foreign key column writes an unchecked reference.
3. `CODE_QUALITY_REPORT.md`'s `InternalColumn` entry grows by two copied fields (`creation_default`, `disallowed`); the refactoring it recommends stays open.

## Review Focus

These inputs are the most likely to bite a user, and no test pins them today. Each line names the test its owning task adds.

1. **`#[disallow(zero)]` on an `#[auto_inc]` primary key.** `create_<table>` writes the placeholder `0` there, so a naive check would refuse every create. Pinned by `create_skips_the_auto_inc_primary_key` in Task 5's runtime group.
2. **A struct with several `#[dsl]` attributes and a foreign key.** Every pass would add the same referenced-row method to the shared wrapper type, so the struct would stop compiling. Pinned by `LookupMembership` in Task 8's runtime group, and by the unchanged `Membership` snapshots of the fixture `wrapper_methods`.
3. **`#[creation_default(0)]` on a foreign key column.** `0` references no row, so the reference check of `create_<table>` has to skip it rather than fail. Pinned by `team_id` of `CreationDefaultPlayer` in Task 2's runtime group.
4. **A NaN stored in a float column with `#[disallow(decreasing)]`.** NaN compares unordered with itself, so a naive check would refuse every later update of that row. Pinned by `an_unchanged_nan_is_no_change` in Task 6's runtime group.
5. **A unique foreign key to the own table, a linked list.** Its referencing-rows method is `get_stage_by_next_stage_id`; the referenced-row method must not take the same name. Pinned by `LookupStage` in Task 8's runtime group.

---

## Conventions for every task

**The local server** has to run for the runtime gate: start `spacetime start` in a terminal of its own, and check it with `spacetime server ping local`.

**The unit gate** (snapshots and compile tests):

```powershell
.\x.ps1 unit-test 2>&1 | Select-String -Pattern "test result:|FAILED|^error|^warning: " | Select-Object -First 20
```

**The runtime gate:**

```powershell
$output = .\x.ps1 test 2>&1 | Out-String
if ($LASTEXITCODE -eq 0) {
    "PASSED"
} else {
    "FAILED - relevant output:"
    $output -split "`n" | Select-String -Pattern "^error|-->|panic|should|failed" | Select-Object -First 30
}
```

**Regenerating snapshots**, then reading every hunk of the diff:

```powershell
$env:INSTA_FORCE_UPDATE = "1"
.\x.ps1 unit-test
$env:INSTA_FORCE_UPDATE = $null
git diff --stat -- derive/tests/snapshots
git diff -- derive/tests/snapshots
git status --short -- derive/tests/snapshots | Select-String "^\?\?"
```

The last command lists the snapshot files a new fixture created; read each of them whole, because `git diff` does not show untracked files.

**Regenerating compile-test output**, then reading every hunk of the diff and every new `.stderr` file:

```powershell
$env:TRYBUILD = "overwrite"
.\x.ps1 unit-test
$env:TRYBUILD = $null
git diff -- compile-tests/tests/ui
git status --short -- compile-tests/tests/ui | Select-String "^\?\?"
```

The regeneration rewrites the line endings of every file it touches, so `git status` lists far more files than the diff shows. `git diff` normalizes line endings, so trust it. Stage with `git add .`; git handles the line endings.

**Green includes the formatter:** run `.\x.ps1 format` until a second run changes nothing, then `.\x.ps1 lint`, which must exit 0.

**Red is an observation:** read the failure text of a new test before changing production code, and check that it fails for the reason under test. A new compile test for a rejection that does not exist yet fails with trybuild's *Expected test case to fail to compile, but it succeeded*; that is red for the right reason.

---

## Task 0: Record the baseline and commit this plan

**Files:**
- Create: `docs/plans/2026-09-30-err-macro-creation-default-disallow-and-referenced-row-methods.md` (this file)

- [ ] **Step 1: Run the unit gate**

Expected: every `test result:` line says `ok`, no `FAILED`.

- [ ] **Step 2: Run the runtime gate**

Expected: `PASSED`.

- [ ] **Step 3: Run the lint gate**

Run: `.\x.ps1 lint; $LASTEXITCODE`
Expected: `0`.

A red baseline is fixed or reported before Task 1 starts; it is not carried into the plan.

- [ ] **Step 4: Commit the plan**

```powershell
git add docs/plans/2026-09-30-err-macro-creation-default-disallow-and-referenced-row-methods.md
git commit -m @'
Add the plan for err!, #[creation_default], #[disallow] and referenced-row methods

Plans issues #187, #188, #186 and #190.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
'@
```

---

## Task 1: `err!`, the shortcut for a custom error (#187)

**Files:**
- Modify: `src/lib.rs` (the macro, the prelude)
- Create: `examples/test/src/err_macro_test.rs`; register it in `examples/test/src/lib.rs`
- Modify: `examples/test/src/cascade_hook_error_test.rs`, `examples/test/src/update_and_soft_delete_hook_test.rs`, `examples/blackholio/src/lib.rs`
- Modify: `docs/DOCUMENTATION.md` (*Prelude Exports*, *Reducers*, *Error Handling*), `README.md`, `docs/MIGRATION.md`

**Interfaces:**
- Produces: `spacetimedsl::err!`, reachable as `crate::spacetimedsl::prelude::err` in a user's crate. Later tasks do not call it; generated code keeps building its errors through `derive-input`'s `runtime::generic_error`.

`err!` changes what the runtime crate offers, not what the generator emits, so the runtime gate is the only test kind that can fail for it: no snapshot and no diagnostic changes.

- [ ] **Step 1: Write the failing runtime group**

Create `examples/test/src/err_macro_test.rs`:

```rust
//! `err!`, the shortcut for `Err(SpacetimeDSLError::Error(…))`, reached through the prelude
//! like every other runtime item.

use crate::spacetimedsl::prelude::*;

const MAXIMUM_LEVEL: u8 = 10;

pub(crate) fn run_tests<T: WriteContext>(_dsl: &DSL<'_, T>) -> Result<(), String> {
    let level = 12;

    expect_message(
        err!("Level {level} is above {}", MAXIMUM_LEVEL),
        "Level 12 is above 10",
    )?;

    expect_message(
        err!("Level {} is above {maximum}", level, maximum = MAXIMUM_LEVEL),
        "Level 12 is above 10",
    )?;

    expect_message(err!("No level given"), "No level given")?;

    let message = format!("Level {level} is too high");
    expect_message(err!(message), "Level 12 is too high")?;

    expect_message(err!(MAXIMUM_LEVEL), "10")?;

    Ok(())
}

/// Checks that `result` is the error `err!` builds, with `expected_message` as its message.
fn expect_message(result: Result<(), SpacetimeDSLError>, expected_message: &str) -> Result<(), String> {
    match result {
        Err(SpacetimeDSLError::Error(message)) if message == expected_message => Ok(()),
        other => Err(format!(
            "err! should give Err(SpacetimeDSLError::Error({expected_message:?}))! Got: {other:?}"
        )),
    }
}
```

In `examples/test/src/lib.rs`, add `pub mod err_macro_test;` after `pub mod entity;`, and append to `TEST_GROUPS`, after the `referenced_table_without_delete_methods_test` entry:

```rust
    ("err_macro_test", err_macro_test::run_tests),
```

- [ ] **Step 2: Observe the red**

Run the runtime gate.
Expected: `FAILED`, with `error: cannot find macro `err` in this scope` pointing into `err_macro_test.rs`.

- [ ] **Step 3: Define the macro and export it**

In `src/lib.rs`, add after `pub mod error;`:

```rust
/// `Err(SpacetimeDSLError::Error(message))`, the error a reducer, a hook or a helper refuses
/// with, its message built like `format!` builds a `String`.
///
/// ```rust,ignore
/// return err!("Level {level} is above the maximum of {}", MAXIMUM_LEVEL);
/// ```
///
/// A first argument which is not a literal is the whole message, turned into a `String`
/// with `ToString`, so a message kept in a variable or a constant needs no `"{}"`:
///
/// ```rust,ignore
/// return err!(LOCKED_MESSAGE);
/// ```
#[macro_export]
macro_rules! err {
    ($format_string:literal $($format_arguments:tt)*) => {
        ::core::result::Result::Err($crate::error::SpacetimeDSLError::Error(::std::format!(
            $format_string $($format_arguments)*
        )))
    };
    ($message:expr $(,)?) => {
        ::core::result::Result::Err($crate::error::SpacetimeDSLError::Error(
            ::std::string::ToString::to_string(&$message),
        ))
    };
}
```

In `pub mod prelude`, add `err,` to the `pub use crate::{…}` list, between `delete::{…},` and `error::{…},`.

- [ ] **Step 4: Observe the green**

Run the runtime gate.
Expected: `PASSED`. The group reaches `err!` through `use crate::spacetimedsl::prelude::*;` alone, which proves both re-exports.

- [ ] **Step 5: Use `err!` in the examples**

In `examples/test/src/cascade_hook_error_test.rs`, replace

```rust
        true => Err(SpacetimeDSLError::Error(LOCKED_MESSAGE.to_string())),
```

by

```rust
        true => err!(LOCKED_MESSAGE),
```

In `examples/test/src/update_and_soft_delete_hook_test.rs`, replace

```rust
        return Err(SpacetimeDSLError::Error(
            LOCKED_GUILD_MEMBER_MESSAGE.to_string(),
        ));
```

by

```rust
        return err!(LOCKED_GUILD_MEMBER_MESSAGE);
```

In `examples/blackholio/src/lib.rs`, replace

```rust
                    return Err(SpacetimeDSLError::Error(format!(
                        "Player (ID: {}, Identity: {}, Name: {}) is already logged in!",
                        player.get_id(),
                        player.get_identity(),
                        player.get_name()
                    )));
```

by

```rust
                    return err!(
                        "Player (ID: {}, Identity: {}, Name: {}) is already logged in!",
                        player.get_id(),
                        player.get_identity(),
                        player.get_name()
                    );
```

and the same shape with *is not logged in!* further down.

Run the runtime gate. Expected: `PASSED`; it publishes `blackholio` as well, so that module compiles with the macro too.

- [ ] **Step 6: Document the macro**

In `docs/DOCUMENTATION.md`:

Under *Prelude Exports*, in the list of `spacetimedsl::prelude` items, add after the line about `SpacetimeDSLError`:

```markdown
  - `err!` — `Err(SpacetimeDSLError::Error(…))` with a message built like `format!`
```

Under *Reducers*, replace `return Err(SpacetimeDSLError::Error("Message cannot be empty".to_string()));` by `return err!("Message cannot be empty");`.

Under *Error Handling*, add after the section *Recommended Pattern: `?` Propagation*:

````markdown
### Refusing With `err!`

`err!` is the shortcut for `Err(SpacetimeDSLError::Error(…))`. It builds the message like `format!`, inline arguments included, and a single argument which is not a literal becomes the whole message through `ToString`:

```rust
if level > MAXIMUM_LEVEL {
    return err!("Level {level} is above the maximum of {MAXIMUM_LEVEL}");
}

return err!(LOCKED_MESSAGE);
```

It evaluates to the `Err`, so return it or let it end a function.
````

In `README.md`, replace in `before_position_hook_helper`

```rust
        return Err(spacetimedsl::SpacetimeDSLError::Error(
            "Position out of bounds".to_string(),
        ));
```

by

```rust
        return spacetimedsl::err!("Position out of bounds");
```

In `docs/MIGRATION.md`, add after the entry *The crate root re-exports `spacetimedsl::prelude`*:

```markdown
#### The preludes export `err!`

`spacetimedsl::prelude`, and with it the prelude `spacetimedsl!()` generates, exports the new macro `err!`. A macro of your own called `err` which another glob import brings into the same scope becomes ambiguous where it is called; import that one by name, which takes precedence over a glob.
```

- [ ] **Step 7: Format, lint, commit**

Commit message:

```text
Add err!, the shortcut for Err(SpacetimeDSLError::Error(..))

err!("…", args) builds the message like format!, inline and named arguments included;
err!(message) turns any other single expression into the message with ToString. The macro
is exported at the crate root and re-exported by spacetimedsl::prelude, so the prelude
spacetimedsl!() generates reaches it too. The documentation, the README and the examples
use it.

Tests: the runtime group err_macro_test.

Closes #187

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 2: `#[creation_default(...)]` fills a column on create (#188, part 1)

**Files:**
- Create: `derive-input/src/internal/dsl/method/doc.rs` (the rustdoc helpers, moved out of `relationship_doc.rs`, and the token printer)
- Modify: `derive-input/src/internal/dsl/method/relationship_doc.rs`, `method.rs`, `context.rs`, `create.rs`, `update.rs`, `upsert.rs`, `removal.rs` (the moved helpers)
- Modify: `derive-input/Cargo.toml` (`syn` gains `full`)
- Modify: `derive-input/src/internal/dsl.rs` (module and symbol), `derive-input/src/api/attribute.rs`, `derive/src/lib.rs` (helper attribute)
- Create: `derive-input/src/internal/dsl/creation_default.rs`
- Modify: `derive-input/src/api/dsl/column.rs`, `derive-input/src/internal/dsl/column.rs`, `derive-input/src/internal/column.rs`
- Modify: `derive-input/src/internal/error.rs` (two diagnostics)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Create: `compile-tests/tests/ui/creation_default_without_expression.rs`, `compile-tests/tests/ui/creation_default_repeated.rs` (+ `.stderr`)
- Create: `derive/tests/fixtures/creation_default.rs`; register it in `derive/src/characterization_tests.rs`
- Modify: `derive/tests/fixtures/every_field_attribute.rs`
- Create: `examples/test/src/creation_default_test.rs`; register it in `examples/test/src/lib.rs`
- Modify: `docs/DOCUMENTATION.md` (*Column Attributes*, *Create Structs & Create Methods*), `docs/MIGRATION.md`, `README.md`
- Recorded output: the new `derive/tests/snapshots/creation_default/**`; `every_field_attribute/Gadget/table.snap` and `create_gadget.snap`

**Interfaces:**
- Produces: `SpacetimeDSLColumn::creation_default: Option<syn::Expr>`; `InternalColumn::spacetimedsl_column_creation_default: Option<syn::Expr>`; `internal::dsl::creation_default::try_parse(field: &SatsField<'_>) -> syn::Result<Option<Expr>>` (Task 3 widens it); `CreateColumnRole::CreationDefault`; `method::doc::{with_section, paragraphs, section, written_tokens}`, where `section(heading: &str, lead: Option<&str>, bullets: &[String]) -> String` and `written_tokens(tokens: TokenStream) -> String`; `error::multiple_creation_default_attributes`, `error::creation_default_without_expression`. The symbol `internal::dsl::creation_default`.

- [ ] **Step 1: Move the rustdoc helpers into their own module**

The `# Defaults` section of this task and the `# Disallowed values` section of Task 5 are not about relationships, so the helpers every section is built with leave `relationship_doc.rs`. This step changes no generated token.

Create `derive-input/src/internal/dsl/method/doc.rs` holding `with_section`, `paragraphs` and `section`, moved unchanged from `relationship_doc.rs`, with `section` now `pub`:

```rust
//! The pieces every generated rustdoc text is assembled from: paragraphs, sections, and code
//! the way a person writes it.

use std::borrow::Borrow;

/// `doc`, followed by `section` as a paragraph of its own, or `doc` alone when `section` is
/// empty.
pub fn with_section(doc: String, section: String) -> String {
    paragraphs([doc, section])
}

/// The non-empty `parts` as paragraphs, each separated from the next by a blank line.
pub fn paragraphs<Part: Borrow<str>>(parts: impl IntoIterator<Item = Part>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.borrow().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A doc comment section: its heading, a lead sentence when there is one, and its bullets.
/// Nothing when there are no bullets.
pub fn section(heading: &str, lead: Option<&str>, bullets: &[String]) -> String {
    if bullets.is_empty() {
        return String::new();
    }

    let lead = lead.map(|lead| format!("{lead}\n\n")).unwrap_or_default();

    format!("# {heading}\n\n{lead}{}", bullets.join("\n"))
}
```

In `relationship_doc.rs`, delete `with_section`, `paragraphs` and `section`, drop `std::borrow::Borrow` from the imports and add `doc::{paragraphs, section}` to `super::{…}`.

In `method.rs`, declare `pub mod doc;` after `mod delete;`.

In `context.rs`, import `super::doc` instead of `super::relationship_doc`, and call `doc::paragraphs` in both places that called `relationship_doc::paragraphs`.

In `create.rs`, `update.rs`, `upsert.rs` and `removal.rs`, add `doc` to the `super::{…}` import and replace every `relationship_doc::with_section(` by `doc::with_section(`.

Run the unit gate, then `git diff --stat -- derive/tests/snapshots compile-tests/tests/ui`.
Expected: all `ok`, and no file listed.

- [ ] **Step 2: Declare the attribute**

In `derive-input/Cargo.toml`, give `syn` the `full` feature, because a default can be any expression:

```toml
syn = { workspace = true, default-features = true, features = ["full"] }
```

In `derive-input/src/internal/dsl.rs`, add `pub mod creation_default;` after `pub mod auto_gen;`, and `symbol!(creation_default);` after `symbol!(auto_gen);`.

In `derive-input/src/api/attribute.rs`, make the list nine long:

```rust
pub const FIELD_ATTRIBUTE_NAMES: [&str; 9] = [
    dsl::create_wrapper.0,
    dsl::use_wrapper.0,
    dsl::foreign_key.0,
    dsl::referenced_by.0,
    dsl::set_on_create.0,
    dsl::set_on_update.0,
    dsl::set_on_soft_delete.0,
    dsl::auto_gen.0,
    dsl::creation_default.0,
];
```

In `derive/src/lib.rs`, add `creation_default` after `auto_gen` in `#[proc_macro_derive(SpacetimeDSL, attributes(…))]`.

Create `derive-input/src/internal/dsl/creation_default.rs` with only its module comment for now, so the declaration compiles:

```rust
//! `#[creation_default(<expression>)]`: the value `create_<table>` fills a column with,
//! instead of asking the caller for it in `Create<Table>`.
```

Run the unit gate. Expected: all `ok`; `helper_attributes_match_field_attributes` agrees with the two lists.

- [ ] **Step 3: Write the failing runtime group and compile tests**

Create `examples/test/src/creation_default_test.rs`:

```rust
//! `#[creation_default(...)]`: the value `create_<table>` fills a column with, so
//! `Create<Table>` does not ask the caller for it.
//!
//! `create_creation_default_player` is called with `name` alone, which compiles only while
//! every defaulted column is left out of `CreateCreationDefaultPlayer`. `team_id` defaults to
//! `0`, which references no row, so the reference check of the create method skips it.

use crate::spacetimedsl::prelude::*;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum CreationDefaultMembership {
    Trial,
    Paid,
}

#[spacetimedsl::dsl(
    plural_name = creation_default_teams,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = creation_default_team)]
pub struct CreationDefaultTeam {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::creation_default_test, table = creation_default_player)]
    id: u64,
}

#[spacetimedsl::dsl(
    plural_name = creation_default_players,
    method(update = true, delete = true)
)]
#[spacetimedb::table(accessor = creation_default_player)]
pub struct CreationDefaultPlayer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub name: String,

    #[creation_default(100)]
    pub coins: u32,

    #[creation_default(-1)]
    rank: i32,

    #[creation_default(CreationDefaultMembership::Trial)]
    membership: CreationDefaultMembership,

    #[creation_default(String::from("rookie"))]
    pub title: String,

    #[index(btree)]
    #[use_wrapper(CreationDefaultTeamId)]
    #[foreign_key(
        path = crate::creation_default_test,
        table = creation_default_team,
        column = id
    )]
    #[creation_default(0)]
    pub team_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let player = dsl.create_creation_default_player(CreateCreationDefaultPlayer {
        name: "Ada".to_string(),
    })?;

    if *player.get_coins() != 100 {
        return Err(format!(
            "`coins` should hold its creation default 100! Got: {}",
            player.get_coins()
        ));
    }

    if *player.get_rank() != -1 {
        return Err(format!(
            "`rank` should hold its creation default -1! Got: {}",
            player.get_rank()
        ));
    }

    if *player.get_membership() != CreationDefaultMembership::Trial {
        return Err(format!(
            "`membership` should hold its creation default Trial! Got: {:?}",
            player.get_membership()
        ));
    }

    if player.get_title() != "rookie" {
        return Err(format!(
            "`title` should hold its creation default \"rookie\"! Got: {}",
            player.get_title()
        ));
    }

    if player.get_team_id().value() != 0 {
        return Err(format!(
            "`team_id` should hold its creation default 0, which references no row! Got: {}",
            player.get_team_id()
        ));
    }

    Ok(())
}
```

In `examples/test/src/lib.rs`, add `pub mod creation_default_test;` after `pub mod component;`, and append to `TEST_GROUPS`:

```rust
    ("creation_default_test", creation_default_test::run_tests),
```

Create `compile-tests/tests/ui/creation_default_without_expression.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = true))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[creation_default]
        pub coins: u32,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_repeated.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = true))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[creation_default(100)]
        #[creation_default(200)]
        pub coins: u32,
    }
}

fn main() {}
```

- [ ] **Step 4: Observe the red**

Run the unit gate.
Expected: `FAILED` in `invalid_table_definitions_are_rejected`; trybuild reports both new files with *Expected test case to fail to compile, but it succeeded*: the attribute is known, and still ignored.

Run the runtime gate.
Expected: `FAILED` with `error[E0063]: missing fields `coins`, `membership`, `rank` and 2 other fields in initializer of `CreateCreationDefaultPlayer``: `create_*` still asks for the defaulted columns.

- [ ] **Step 5: Pin the model in the contract test (the red)**

In `derive/src/data_transfer_contract_tests.rs`, in the `Gadget` fixture of `the_model_holds_what_a_table_declares`, add after `pub name: String,`:

```rust
                #[creation_default(1)]
                pub revision: u32,
```

Change `assert_eq!(gadget.columns.len(), 6);` to `assert_eq!(gadget.columns.len(), 7);`. Add after the three assertions on `name`:

```rust
    let revision = column(&gadget, "revision");
    assert_eq!(
        revision
            .spacetimedsl_column
            .creation_default
            .as_ref()
            .expect("`revision` has `#[creation_default(1)]`")
            .to_token_stream()
            .to_string(),
        "1"
    );
    assert!(
        create_dsl_method_arg
            .struct_members
            .iter()
            .all(|member| member.arg_name != "revision"),
        "a column with `#[creation_default]` is not asked of the caller"
    );
```

In `visit_spacetimedsl_column`, add `creation_default: _,` after `auto_generated_uuid_version,` in the destructuring of `SpacetimeDSLColumn`.

Run the unit gate.
Expected: `FAILED` with `error[E0026]: struct `SpacetimeDSLColumn` does not have a field named `creation_default`` (and `E0609` for the field access).

- [ ] **Step 6: Parse the attribute and fill the column**

In `derive-input/src/api/dsl/column.rs`, add to `SpacetimeDSLColumn` after `auto_generated_uuid_version`:

```rust
    /// `Some` when the field has `#[creation_default(...)]`: the expression `create_<table>`
    /// fills the column with instead of asking the caller for it.
    pub creation_default: Option<syn::Expr>,
```

Fill `derive-input/src/internal/dsl/creation_default.rs`:

```rust
//! `#[creation_default(<expression>)]`: the value `create_<table>` fills a column with,
//! instead of asking the caller for it in `Create<Table>`.

use {
    super::creation_default,
    crate::internal::error,
    spacetime_bindings_macro_input::sats::SatsField,
    syn::{Attribute, Expr},
};

/// Reads `#[creation_default(<expression>)]` from a column. A column may have one.
pub fn try_parse(field: &SatsField<'_>) -> syn::Result<Option<Expr>> {
    let mut creation_default_attributes = field
        .original_attrs
        .iter()
        .filter(|attribute| attribute.path() == creation_default);

    let Some(creation_default_attribute) = creation_default_attributes.next() else {
        return Ok(None);
    };

    if let Some(repeated_attribute) = creation_default_attributes.next() {
        return Err(error::multiple_creation_default_attributes(
            repeated_attribute,
        ));
    }

    parse_expression(creation_default_attribute).map(Some)
}

fn parse_expression(creation_default_attribute: &Attribute) -> syn::Result<Expr> {
    creation_default_attribute
        .meta
        .require_list()
        .and_then(|list| list.parse_args())
        .map_err(|_| error::creation_default_without_expression(creation_default_attribute))
}
```

In `derive-input/src/internal/error.rs`, add after the `#[auto_gen]` section:

```rust
// `#[creation_default]`

pub fn multiple_creation_default_attributes(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "Only one `#[creation_default]` is allowed per column!",
    )
}

pub fn creation_default_without_expression(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "Expected an expression in `#[creation_default(...)]`, e.g. `#[creation_default(0)]` or `#[creation_default(String::new())]`: the value `create_<table>` fills this column with!",
    )
}
```

In `derive-input/src/internal/dsl/column.rs`, add `dsl::creation_default` to the `internal::{…}` import, call the parser after the block that checks the wrapper of a foreign key column:

```rust
        let creation_default = creation_default::try_parse(field)?;
```

and add `creation_default,` to the returned `SpacetimeDSLColumn` after `auto_generated_uuid_version,`.

In `derive-input/src/internal/column.rs`, add to `InternalColumn` after `spacetimedsl_column_auto_generated_uuid_version`:

```rust
    pub spacetimedsl_column_creation_default: Option<Expr>,
```

(import `syn::Expr`), and fill it in `try_parse` after `spacetimedsl_column_auto_generated_uuid_version`:

```rust
            spacetimedsl_column_creation_default: spacetimedsl_column.creation_default.clone(),
```

In `derive-input/src/internal/dsl/method/create.rs`, add the role after `SoftDeleteMarker(SoftDeleteMarkerKind),`:

```rust
    /// A `#[creation_default(...)]` column, filled with its expression.
    CreationDefault,
```

In `CreateColumnRole::of`, add after the block that returns `CreateColumnRole::SoftDeleteMarker`:

```rust
        if internal_column.spacetimedsl_column_creation_default.is_some() {
            return CreateColumnRole::CreationDefault;
        }
```

In `create_method_column_parts`, add the arm after the two `SoftDeleteMarker` arms:

```rust
        CreateColumnRole::CreationDefault => {
            let creation_default = internal_column
                .spacetimedsl_column_creation_default
                .as_ref()
                .expect("the creation default role is only given to a column with a creation default");

            CreateMethodColumnParts {
                arg: None,
                wrapper_option_mapper: None,
                constructor_arg: Some(quote! {
                    let #column_name: #column_type = #creation_default;
                }),
                constructor_arg_name: quote! { #column_name },
            }
        }
```

The type annotation makes rustc report a default of the wrong type at the expression.

In `for_create`, replace the `doc_comment` by:

```rust
        doc_comment: doc::paragraphs([
            format!("Create a row in the `{singular_table_name}` table."),
            defaults_section(internal_columns),
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each column references a row:",
                &internal_columns.iter().collect_vec(),
            ),
        ]),
```

and add after `for_create`:

```rust
/// The `# Defaults` section of `create_<table>`: the columns it fills with their
/// `#[creation_default]` instead of asking the caller for them. Empty when there is none.
fn defaults_section(internal_columns: &[InternalColumn]) -> String {
    let bullets: Vec<String> = internal_columns
        .iter()
        .filter_map(|internal_column| {
            let creation_default = internal_column
                .spacetimedsl_column_creation_default
                .as_ref()?;

            Some(format!(
                "- `{}`: `{}`",
                internal_column.rust_field_name,
                doc::written_tokens(creation_default.to_token_stream()),
            ))
        })
        .collect();

    doc::section(
        "Defaults",
        Some("Fills these columns with their `#[creation_default]` instead of asking for them:"),
        &bullets,
    )
}
```

In `doc.rs`, add the token printer, and extend the import to `use {proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree}, std::borrow::Borrow};`:

```rust
/// Tokens the way a person writes them, for documentation.
///
/// `proc_macro2` puts a space between every two tokens, as in `String :: new ()`. This keeps
/// a space only between two words, around an operator between two operands, and after `,`,
/// `;` and a single `:`, so the tokens read `String::new()`, `vec![1, 2]`, `-1` or `a + b`.
/// A literal keeps its own text, a space inside a string literal included. Generic arguments
/// keep the spaces around their angle brackets, as in `Vec::< u8 >::new()`.
pub fn written_tokens(tokens: TokenStream) -> String {
    let mut written = String::new();
    write_tokens(tokens, &mut written, &mut Previous::Start);

    written
}

/// The kind of the token written last, which decides whether a space goes in front of the
/// next one.
#[derive(Clone, Copy, PartialEq)]
enum Previous {
    /// Nothing yet: the start of the tokens, or of the inside of a bracket.
    Start,
    /// A word, a literal, a closing bracket or a `?`: the end of an operand.
    Operand,
    /// A token the next one follows without a space: `.`, the second `:` of `::`, the `!` of
    /// a macro, or an operator in front of its operand, such as the `-` of `-1`.
    Glue,
    /// The first `:` of `::`.
    FirstColon,
    /// The first character of an operator of several, such as the first `=` of `==`.
    OperatorStart,
    /// A binary operator, `,`, `;` or a single `:`, which a space follows.
    Separator,
}

impl Previous {
    /// Whether a space separates this token from an operand after it.
    fn is_spaced_from_an_operand(self) -> bool {
        matches!(self, Previous::Operand | Previous::Separator)
    }
}

fn write_tokens(tokens: TokenStream, written: &mut String, previous: &mut Previous) {
    for token in tokens {
        match token {
            TokenTree::Ident(ident) => write_operand(&ident.to_string(), written, previous),
            TokenTree::Literal(literal) => write_operand(&literal.to_string(), written, previous),
            TokenTree::Punct(punct) => write_punctuation(&punct, written, previous),
            TokenTree::Group(group) => write_group(&group, written, previous),
        }
    }
}

fn write_operand(text: &str, written: &mut String, previous: &mut Previous) {
    if previous.is_spaced_from_an_operand() {
        written.push(' ');
    }

    written.push_str(text);
    *previous = Previous::Operand;
}

fn write_punctuation(punct: &Punct, written: &mut String, previous: &mut Previous) {
    let character = punct.as_char();
    let starts_an_operator = punct.spacing() == Spacing::Joint;
    let follows_an_operand = *previous == Previous::Operand;
    let operator = match starts_an_operator {
        true => Previous::OperatorStart,
        false => Previous::Separator,
    };

    let (is_spaced, next) = match character {
        ',' | ';' => (false, Previous::Separator),
        '.' => (false, Previous::Glue),
        '?' => (false, Previous::Operand),
        ':' if *previous == Previous::FirstColon => (false, Previous::Glue),
        ':' if starts_an_operator => (false, Previous::FirstColon),
        ':' => (false, Previous::Separator),
        _ if *previous == Previous::OperatorStart => (false, operator),
        '!' if follows_an_operand && !starts_an_operator => (false, Previous::Glue),
        '!' | '-' | '&' | '*' if !follows_an_operand => {
            (previous.is_spaced_from_an_operand(), Previous::Glue)
        }
        _ => (follows_an_operand || *previous == Previous::Separator, operator),
    };

    if is_spaced {
        written.push(' ');
    }

    written.push(character);
    *previous = next;
}

fn write_group(group: &Group, written: &mut String, previous: &mut Previous) {
    let delimiter = group.delimiter();

    if delimiter == Delimiter::None {
        write_tokens(group.stream(), written, previous);
        return;
    }

    // A call, an index and a macro take their brackets right after what they follow.
    let follows_its_callee = delimiter != Delimiter::Brace
        && matches!(*previous, Previous::Operand | Previous::Glue);

    if !follows_its_callee && previous.is_spaced_from_an_operand() {
        written.push(' ');
    }

    let mut inside = String::new();
    write_tokens(group.stream(), &mut inside, &mut Previous::Start);

    let bracketed = match delimiter {
        Delimiter::Parenthesis => format!("({inside})"),
        Delimiter::Bracket => format!("[{inside}]"),
        _ if inside.is_empty() => "{}".to_string(),
        _ => format!("{{ {inside} }}"),
    };

    written.push_str(&bracketed);
    *previous = Previous::Operand;
}
```

- [ ] **Step 7: Observe the green and read the diagnostics**

Run the unit gate. Expected: the contract test passes; the two compile tests still fail, now for want of their `.stderr`.

Regenerate the compile-test output as described in *Conventions*. Expected `creation_default_without_expression.stderr`:

```text
error: Expected an expression in `#[creation_default(...)]`, e.g. `#[creation_default(0)]` or `#[creation_default(String::new())]`: the value `create_<table>` fills this column with!
  --> tests/ui/creation_default_without_expression.rs:12:9
   |
12 |         #[creation_default]
   |         ^^^^^^^^^^^^^^^^^^^
```

and `creation_default_repeated.stderr` underlining `#[creation_default(200)]` in line 13 with *Only one `#[creation_default]` is allowed per column!*. Nothing else may change under `compile-tests/tests/ui`.

- [ ] **Step 8: Snapshot what the create method does with a default**

Create `derive/tests/fixtures/creation_default.rs`:

```rust
//! Covers `#[creation_default(...)]` on the column shapes a caller would otherwise supply: a
//! plain public column, a private one, a `#[create_wrapper]` column, a `#[use_wrapper]`
//! foreign key column and an `Option`. Each is left out of `CreateLoan`, filled with its
//! expression while `create_loan` builds the row, and listed under *Defaults* in its
//! documentation, which writes the expression without the spaces `proc_macro2` puts
//! between its tokens.
//!
//! The fixture is only expanded, never compiled, so `LoanState` does not have to exist.

#[spacetimedsl::dsl(plural_name = branches, method(update = false, delete = false))]
#[spacetimedb::table(accessor = branch, public)]
pub struct Branch {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = loan)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = loans, method(update = true))]
#[spacetimedb::table(accessor = loan, public)]
pub struct Loan {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub borrower: String,

    #[creation_default(-1)]
    pub priority: i32,

    #[creation_default(LoanState::Requested)]
    state: LoanState,

    #[index(btree)]
    #[create_wrapper]
    #[creation_default(String::from("general purpose"))]
    pub purpose: String,

    #[index(btree)]
    #[use_wrapper(BranchId)]
    #[foreign_key(path = self, table = branch, column = id)]
    #[creation_default(0)]
    pub branch_id: u64,

    #[creation_default(vec![1, 2])]
    pub reminder_days: Vec<u8>,

    #[creation_default(None)]
    pub note: Option<String>,
}
```

Register it in `derive/src/characterization_tests.rs`, after `every_field_attribute`:

```rust
#[test]
fn creation_default() {
    snapshot_fixture("creation_default");
}
```

In `derive/tests/fixtures/every_field_attribute.rs`, name `creation_default` in the list of the module comment, after `auto_gen`, and add to `Gadget` after `owner_id`:

```rust
    #[creation_default(1)]
    pub rating: u8,
```

Run the unit gate. Expected: `FAILED`, because the new snapshots do not exist yet and `every_field_attribute` moved. Regenerate the snapshots as described in *Conventions* and read them:

- `creation_default/Loan/table.snap`: `pub struct CreateLoan { pub borrower: String }`, and the accessors of every column.
- `creation_default/Loan/create_loan.snap`: its documentation has a *Defaults* section with the bullets ``- `priority`: `-1` ``, ``- `state`: `LoanState::Requested` ``, ``- `purpose`: `String::from("general purpose")` ``, ``- `branch_id`: `0` ``, ``- `reminder_days`: `vec![1, 2]` `` and ``- `note`: `None` ``, in front of the *Foreign keys* section. Its body binds `let priority: i32 = -1;`, `let state: LoanState = LoanState::Requested;`, `let purpose: String = String::from("general purpose");`, `let branch_id: u64 = 0;`, `let reminder_days: Vec<u8> = vec![1, 2];` and `let note: Option<String> = None;`, and the reference check of `branch_id` stays behind its `branch_id.ne(&0)` guard.
- `every_field_attribute/Gadget/create_gadget.snap`: the same for `rating`; `every_field_attribute/Gadget/table.snap`: `CreateGadget` without `rating`, and the three accessors of `rating`.

No other snapshot may change.

- [ ] **Step 9: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 10: Document the attribute**

In `docs/DOCUMENTATION.md`, under *Column Attributes*, add to the list of SpacetimeDSL attributes:

```rust
#[creation_default(0)]   // Fills the column on create instead of asking for it in Create{Table}
```

Under *What Fields are Excluded (Auto-Defaulted)*, add a row to the table:

```markdown
| `#[creation_default(<expression>)]` columns | The expression                       |
```

and add after the paragraph that ends *Generating a UUID fails outside reducers, like `ctx.timestamp` does.*:

````markdown
#### Defaults of Your Own: `#[creation_default(...)]`

`#[creation_default(<expression>)]` leaves a column out of `Create{Table}` and fills it with the expression instead, each time `create_<table>` builds a row:

```rust
#[spacetimedsl::dsl(plural_name = players, method(update = true))]
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub name: String,

    #[creation_default(100)]
    pub coins: u32,

    #[creation_default(Membership::Trial)]
    membership: Membership,
}

// CreatePlayer has: name
let player = dsl.create_player(CreatePlayer { name: "Ada".to_string() })?;
```

- The expression has the column's own type, also on a column with a wrapper type: `#[creation_default(0)]` on `#[use_wrapper(TeamId)] team_id: u64`. On a foreign key column, `0` and `Uuid::NIL` reference no row, so create skips the reference check for them.
- The column may be private or public. A public one keeps its setter, so an update can change it later.
- The `before_insert` hook receives `Create{Table}`, which does not hold the column; the row is built from the expression after the hook.
- The documentation of `create_<table>` lists the defaulted columns under *Defaults*.
````

In `README.md`, under *Smart Defaults & Automation*, add after the bullet about `#[set_on_create]`:

```markdown
- 🧩 `#[creation_default(...)]` fills a column on create, so `Create{Table}` doesn't ask for it.
```

In `docs/MIGRATION.md`, in the entry *`api::attribute::FIELD_ATTRIBUTE_NAMES` lists the field attributes*, add `creation_default` to the list in parentheses. Add under *For crates building on `spacetimedsl_derive-input`*:

```markdown
#### `SpacetimeDSLColumn::creation_default`

`SpacetimeDSLColumn` gained `creation_default: Option<syn::Expr>`, the expression of `#[creation_default(...)]`, which `create_<table>` fills the column with. Such a column is not a member of `CreateDSLMethodArg::struct_members`.
```

- [ ] **Step 11: Format, lint, commit**

Commit message:

```text
Fill a column on create with #[creation_default(...)]

create_<table> leaves a column with #[creation_default(<expression>)] out of
Create<Table> and binds it to the expression, typed as the field. Its documentation lists
the defaults under "Defaults", written by a small token printer. A missing expression and
a second attribute are rejected. The rustdoc helpers moved into method/doc.rs, and
derive-input's syn gained the full feature to parse any expression.

Tests: contract assertions for SpacetimeDSLColumn::creation_default; the fixture
creation_default; the compile tests creation_default_without_expression and
creation_default_repeated; the runtime group creation_default_test.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 3: Reject `#[creation_default]` where create fills the column or rows share it (#188, part 2)

**Files:**
- Modify: `derive-input/src/internal/dsl/creation_default.rs` (`try_parse` and two rejection functions)
- Modify: `derive-input/src/internal/dsl/column.rs` (the call)
- Modify: `derive-input/src/internal/error.rs` (eight diagnostics)
- Create: eight compile tests in `compile-tests/tests/ui` (+ `.stderr`): `creation_default_on_singleton_with_default`, `creation_default_on_auto_inc_column`, `creation_default_on_auto_gen_column`, `creation_default_on_set_on_create_column`, `creation_default_on_set_on_update_column`, `creation_default_on_marker_column`, `creation_default_on_primary_key_column`, `creation_default_on_unique_column`
- Modify: `docs/DOCUMENTATION.md` (*Defaults of Your Own*)

**Interfaces:**
- Consumes: `creation_default::try_parse` of Task 2.
- Produces: `creation_default::try_parse(field: &SatsField<'_>, column_name: &Ident, spacetimedb_column: &SpacetimeDBColumn, spacetimedsl_table: &SpacetimeDSLTable, auto_generated_uuid_version: Option<UUIDVersion>) -> syn::Result<Option<Expr>>`; the eight `error::creation_default_on_*` functions.

Every input of this task is accepted today and either ignored — create fills an `#[auto_inc]`, `#[auto_gen]`, timestamp or marker column before it looks at the default — or turned into a create method which fails at run time from the second row on. Each rejection is one compile test.

- [ ] **Step 1: Write the failing compile tests**

Create `compile-tests/tests/ui/creation_default_on_singleton_with_default.rs`:

```rust
//! A `singleton(with_default)` table has no create method: `upsert_settings` writes its row,
//! and `DefaultSingleton::get_default` supplies the whole default row.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so the
//! `impl DefaultSingleton` names a `Settings` which does not exist.

::spacetimedsl::spacetimedsl!();

pub mod settings {
    use crate::spacetimedsl::prelude::*;

    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(accessor = settings, public)]
    pub struct Settings {
        #[creation_default(8)]
        pub maximum_player_count: u32,
    }

    impl DefaultSingleton for Settings {
        fn get_default(
            _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
        ) -> Result<Settings, SpacetimeDSLError> {
            Ok(Settings {
                id: 0,
                maximum_player_count: 8,
            })
        }
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_auto_inc_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[creation_default(1)]
        id: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_auto_gen_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod session {
    #[spacetimedsl::dsl(plural_name = sessions, method(update = false))]
    #[spacetimedb::table(accessor = session, public)]
    pub struct Session {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[create_wrapper]
        #[auto_gen(v4)]
        #[creation_default(spacetimedb::Uuid::NIL)]
        token: spacetimedb::Uuid,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_set_on_create_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[creation_default(spacetimedb::Timestamp::UNIX_EPOCH)]
        created_at: spacetimedb::Timestamp,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_set_on_update_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = true))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,

        #[creation_default(None)]
        modified_at: Option<spacetimedb::Timestamp>,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_marker_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(
        plural_name = tickets,
        method(update = false, delete = true, soft_delete = true)
    )]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[creation_default(false)]
        deleted: bool,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_primary_key_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[create_wrapper]
        #[creation_default(1)]
        id: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/creation_default_on_unique_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[unique]
        #[creation_default(String::new())]
        code: String,
    }
}

fn main() {}
```

- [ ] **Step 2: Observe the red**

Run the unit gate.
Expected: `FAILED`; trybuild reports all eight files with *Expected test case to fail to compile, but it succeeded*.

- [ ] **Step 3: Write the diagnostics**

In `derive-input/src/internal/error.rs`, add to the `#[creation_default]` section:

```rust
pub fn creation_default_on_singleton_with_default(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on a `singleton(with_default)` table, because it has no create method! Its `DefaultSingleton::get_default` supplies the whole default row.",
    )
}

pub fn creation_default_on_auto_inc_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on an `#[auto_inc]` column, because SpacetimeDB fills it on create!",
    )
}

pub fn creation_default_on_auto_gen_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on an `#[auto_gen]` column, because the create method generates its UUID!",
    )
}

pub fn creation_default_on_set_on_create_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on the column with the `set_on_create` role, because the create method sets it to the current time!",
    )
}

pub fn creation_default_on_set_on_update_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on the column with the `set_on_update` role, because the create method fills it in!",
    )
}

pub fn creation_default_on_marker_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on the soft-delete marker column, because a new row always starts unmarked!",
    )
}

pub fn creation_default_on_primary_key_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on a primary key column! Every created row would get the same key, which SpacetimeDB rejects from the second row on.",
    )
}

pub fn creation_default_on_unique_column(creation_default_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        creation_default_attribute,
        "`#[creation_default]` is not allowed on a `#[unique]` column! Every created row would get the same value, which the unique constraint rejects from the second row on.",
    )
}
```

- [ ] **Step 4: Reject the eight shapes**

Replace `derive-input/src/internal/dsl/creation_default.rs` by:

```rust
//! `#[creation_default(<expression>)]`: the value `create_<table>` fills a column with,
//! instead of asking the caller for it in `Create<Table>`, and the columns it is not
//! allowed on.

use {
    super::creation_default,
    crate::{
        api::{
            db::column::SpacetimeDBColumn,
            dsl::{auto_gen::UUIDVersion, table::SpacetimeDSLTable},
        },
        internal::error,
    },
    spacetime_bindings_macro_input::sats::SatsField,
    syn::{Attribute, Expr, Ident},
};

/// Reads `#[creation_default(<expression>)]` from a column. A column may have one, where the
/// caller would otherwise supply the value and where two rows may hold the same one.
pub fn try_parse(
    field: &SatsField<'_>,
    column_name: &Ident,
    spacetimedb_column: &SpacetimeDBColumn,
    spacetimedsl_table: &SpacetimeDSLTable,
    auto_generated_uuid_version: Option<UUIDVersion>,
) -> syn::Result<Option<Expr>> {
    let mut creation_default_attributes = field
        .original_attrs
        .iter()
        .filter(|attribute| attribute.path() == creation_default);

    let Some(creation_default_attribute) = creation_default_attributes.next() else {
        return Ok(None);
    };

    if let Some(repeated_attribute) = creation_default_attributes.next() {
        return Err(error::multiple_creation_default_attributes(
            repeated_attribute,
        ));
    }

    let expression = parse_expression(creation_default_attribute)?;

    reject_where_create_fills_the_column(
        creation_default_attribute,
        column_name,
        spacetimedb_column,
        spacetimedsl_table,
        auto_generated_uuid_version,
    )?;

    reject_where_rows_would_share_the_value(creation_default_attribute, spacetimedb_column)?;

    Ok(Some(expression))
}

fn parse_expression(creation_default_attribute: &Attribute) -> syn::Result<Expr> {
    creation_default_attribute
        .meta
        .require_list()
        .and_then(|list| list.parse_args())
        .map_err(|_| error::creation_default_without_expression(creation_default_attribute))
}

/// A default would never be used where `create_<table>` fills the column itself, or where
/// the table has no create method at all.
fn reject_where_create_fills_the_column(
    creation_default_attribute: &Attribute,
    column_name: &Ident,
    spacetimedb_column: &SpacetimeDBColumn,
    spacetimedsl_table: &SpacetimeDSLTable,
    auto_generated_uuid_version: Option<UUIDVersion>,
) -> syn::Result<()> {
    let names_this_column =
        |role_column_name: &Option<Ident>| role_column_name.as_ref() == Some(column_name);

    if spacetimedsl_table.singleton_has_default() {
        return Err(error::creation_default_on_singleton_with_default(
            creation_default_attribute,
        ));
    }

    if spacetimedb_column.is_auto_inc {
        return Err(error::creation_default_on_auto_inc_column(
            creation_default_attribute,
        ));
    }

    if auto_generated_uuid_version.is_some() {
        return Err(error::creation_default_on_auto_gen_column(
            creation_default_attribute,
        ));
    }

    if names_this_column(&spacetimedsl_table.on_insert_set_current_timestamp_column_name) {
        return Err(error::creation_default_on_set_on_create_column(
            creation_default_attribute,
        ));
    }

    if names_this_column(&spacetimedsl_table.on_update_set_current_timestamp_column_name) {
        return Err(error::creation_default_on_set_on_update_column(
            creation_default_attribute,
        ));
    }

    if spacetimedsl_table
        .soft_delete_marker
        .as_ref()
        .is_some_and(|marker| marker.column_name == *column_name)
    {
        return Err(error::creation_default_on_marker_column(
            creation_default_attribute,
        ));
    }

    Ok(())
}

/// Every created row would get the same value, which a unique column holds only once.
fn reject_where_rows_would_share_the_value(
    creation_default_attribute: &Attribute,
    spacetimedb_column: &SpacetimeDBColumn,
) -> syn::Result<()> {
    if spacetimedb_column.is_primary_key {
        return Err(error::creation_default_on_primary_key_column(
            creation_default_attribute,
        ));
    }

    if spacetimedb_column
        .single_column_index
        .as_ref()
        .is_some_and(|index| index.is_unique)
    {
        return Err(error::creation_default_on_unique_column(
            creation_default_attribute,
        ));
    }

    Ok(())
}
```

In `derive-input/src/internal/dsl/column.rs`, replace the call by:

```rust
        let creation_default = creation_default::try_parse(
            field,
            &rust_field.name,
            spacetimedb_column,
            spacetimedsl_table,
            auto_generated_uuid_version,
        )?;
```

- [ ] **Step 5: Regenerate the compile-test output and read it**

Regenerate as described in *Conventions*. Expected: each new `.stderr` starts with its own diagnostic from Step 3, underlining the `#[creation_default(...)]` attribute. `creation_default_on_singleton_with_default.stderr` continues with the rustc errors the `//!` comment names, all of them about `Settings`; every other new file holds its diagnostic alone. No existing `.stderr` changes.

- [ ] **Step 6: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`. The snapshots do not move: no accepted input changed.

- [ ] **Step 7: Document the rejections**

In `docs/DOCUMENTATION.md`, append to the list in *Defaults of Your Own: `#[creation_default(...)]`*:

```markdown
- It is rejected on a `singleton(with_default)` table, which has no create method; on the columns create fills in already — `#[auto_inc]`, `#[auto_gen]`, the `set_on_create` and `set_on_update` columns and the soft-delete marker; and on a `#[primary_key]` or `#[unique]` column, where every created row would repeat the value.
```

- [ ] **Step 8: Format, lint, commit**

Commit message:

```text
Reject #[creation_default] where create fills the column or rows share it

A default is rejected on a singleton(with_default) table, which has no create method; on
the #[auto_inc], #[auto_gen], set_on_create, set_on_update and soft-delete marker
columns, which create fills in already; and on a #[primary_key] or #[unique] column,
where every created row would repeat the value.

Tests: one compile test per shape.

Closes #188

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 4: Classify signed integer and float columns (#186, preparation)

**Files:**
- Modify: `derive-input/src/internal/column.rs` (`ColumnTypeKind`)
- Modify: `derive-input/src/internal/dsl/method/reference_integrity.rs` (two exhaustive matches)
- Modify: `derive/tests/fixtures/foreign_keys_with_equivalent_spellings.rs`
- Modify: `docs/DOCUMENTATION.md` (*Column Type Spellings*), `docs/MIGRATION.md`
- Recorded output: the new `derive/tests/snapshots/foreign_keys_with_equivalent_spellings/Transfer/**`

**Interfaces:**
- Produces: `ColumnTypeKind::SignedInteger` (`i8`–`i128`) and `ColumnTypeKind::Float` (`f32`, `f64`), bare or as `core::primitive::X` / `std::primitive::X`. Tasks 5 and 6 decide the column types of `#[disallow]` by them.

`#[disallow(decreasing)]` accepts signed integers and floats, which `ColumnTypeKind::of` calls `Other` today. Giving them kinds has one effect a user sees before `#[disallow]` exists: `canonical_type` reduces `core::primitive::i64` to `i64`, so two foreign keys to one table which spell a signed type both ways stop being rejected as mismatched. That effect is this task's test.

- [ ] **Step 1: Write the failing fixture**

In `derive/tests/fixtures/foreign_keys_with_equivalent_spellings.rs`, replace the first sentence of the module comment by *Covers two foreign keys to the same table which spell the same type and the same path differently: `u64` and `core::primitive::u64` in `Shipment`, `i64` and `core::primitive::i64` in `Transfer`, `::other_crate::tables` and `other_crate::tables` in both.*, and add at the end:

```rust
#[spacetimedsl::dsl(plural_name = transfers, method(update = true))]
#[spacetimedb::table(accessor = transfer, public)]
pub struct Transfer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(::other_crate::tables::LedgerId)]
    #[foreign_key(path = ::other_crate::tables, table = ledger, column = id, on_delete = Delete)]
    pub source_ledger_id: i64,

    #[index(btree)]
    #[use_wrapper(other_crate::tables::LedgerId)]
    #[foreign_key(path = other_crate::tables, table = ledger, column = id, on_delete = Delete)]
    pub target_ledger_id: core::primitive::i64,
}
```

- [ ] **Step 2: Observe the red**

Run the unit gate.
Expected: `FAILED` in `foreign_keys_with_equivalent_spellings`, with *`foreign_keys_with_equivalent_spellings.rs` / `Transfer` should expand in pass 1: All foreign key columns which reference the same primary key of another table should have the same type*.

- [ ] **Step 3: Add the two kinds**

In `derive-input/src/internal/column.rs`, add to `ColumnTypeKind` after `UnsignedInteger`:

```rust
    SignedInteger,
    Float,
```

In `ColumnTypeKind::of`, add after the arm of `"u8" | "u16" | "u32" | "u64" | "u128"`:

```rust
            "i8" | "i16" | "i32" | "i64" | "i128" if is_bare || is_primitive_path => {
                ColumnTypeKind::SignedInteger
            }
            "f32" | "f64" if is_bare || is_primitive_path => ColumnTypeKind::Float,
```

and replace the doc line about `u8`–`u128` and `bool` by:

```rust
    /// - `u8`–`u128`, `i8`–`i128`, `f32`, `f64` and `bool` bare or as `core::primitive::X` /
    ///   `std::primitive::X`, which name the same primitive;
```

In `derive-input/src/internal/dsl/method/reference_integrity.rs`, add `| ColumnTypeKind::SignedInteger | ColumnTypeKind::Float` to the arm of `reference_integrity_checks` which emits the check without a guard, and to the arm of `documented_value_referencing_no_row` which returns `None`: every value of these kinds is checked, as before.

- [ ] **Step 4: Regenerate the snapshots and read them**

Regenerate as described in *Conventions*. Expected: `git diff` shows nothing; the new `foreign_keys_with_equivalent_spellings/Transfer/*.snap` have the shape of `Shipment`'s, with `i64` for the column type and one cascade function over both columns.

- [ ] **Step 5: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 6: Document the change**

In `docs/DOCUMENTATION.md`, *Column Type Spellings*, replace the row

```markdown
| `u8`–`u128`, `bool`   | bare, `core::primitive::u64`, `std::primitive::u64` (likewise for the others) |
```

by

```markdown
| `u8`–`u128`, `i8`–`i128`, `f32`, `f64`, `bool` | bare, `core::primitive::u64`, `std::primitive::u64` (likewise for the others) |
```

and let `.\x.ps1 format` or the markdown formatter of your editor align the table.

In `docs/MIGRATION.md`, append to the entry *Qualified spellings of the checked types are accepted*:

```markdown
Likewise `core::primitive::i64` / `std::primitive::f64` count as the signed integers and floats they name, so two foreign keys of one table to the same table which spell `i64` both ways are no longer rejected as mismatched.
```

- [ ] **Step 7: Format, lint, commit**

Commit message:

```text
Classify signed integer and float columns

ColumnTypeKind gains SignedInteger and Float, in every spelling of the primitive. Foreign
keys of one table to the same table which spell a signed type both ways, such as i64 and
core::primitive::i64, are accepted now. #[disallow] builds on the two kinds.

Tests: the fixture foreign_keys_with_equivalent_spellings gained Transfer.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 5: `#[disallow(zero)]` in create, update and upsert (#186, part 1)

**Files:**
- Modify: `derive-input/src/internal/dsl.rs` (module and two symbols), `derive-input/src/api/attribute.rs`, `derive/src/lib.rs` (helper attribute)
- Create: `derive-input/src/api/dsl/disallow.rs`; modify `derive-input/src/api/dsl.rs`
- Modify: `derive-input/src/api/dsl/column.rs` (field)
- Create: `derive-input/src/internal/dsl/disallow.rs` (parsing and rejections)
- Modify: `derive-input/src/internal/dsl/column.rs`, `derive-input/src/internal/column.rs`, `derive-input/src/internal/dsl/setter.rs`
- Create: `derive-input/src/internal/dsl/method/disallow.rs` (checks and documentation); modify `method.rs`
- Modify: `derive-input/src/internal/dsl/method/create.rs`, `update.rs`, `upsert.rs`
- Modify: `derive-input/src/internal/error.rs` (six diagnostics)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Create: nine compile tests in `compile-tests/tests/ui` (+ `.stderr`): `disallow_without_list`, `disallow_with_empty_list`, `disallow_rule_repeated`, `disallow_repeated`, `disallow_unknown_rule`, `disallow_zero_on_signed_column`, `disallow_zero_with_set_zero_strategy`, `disallow_zero_with_creation_default_zero`, `disallow_zero_with_creation_default_uuid_nil`
- Create: `derive/tests/fixtures/disallow_zero.rs`; register it
- Modify: `derive/tests/fixtures/every_field_attribute.rs`
- Create: `examples/test/src/disallow_zero_test.rs`; register it
- Modify: `docs/DOCUMENTATION.md` (*Column Attributes*, new *Disallowed Values*), `docs/MIGRATION.md`, `README.md`
- Recorded output: the new `derive/tests/snapshots/disallow_zero/**`; `every_field_attribute/Gadget/table.snap`, `create_gadget.snap` and `update_gadget_by_id.snap`

**Interfaces:**
- Consumes: `ColumnTypeKind` of Task 4; `SpacetimeDSLColumn::creation_default` and `method::doc` of Task 2.
- Produces: `api::dsl::disallow::Disallowed` (`Zero`; Task 6 adds `Decreasing` and `Increasing`); `SpacetimeDSLColumn::disallowed: BTreeSet<Disallowed>`; `InternalColumn::spacetimedsl_column_disallowed: BTreeSet<Disallowed>`; `Disallowed::keyword(self) -> &'static str` (`pub(crate)`); `internal::dsl::disallow::{try_parse, forbidden_zero}`; `method::disallow::{GuardedWrite, CheckedRules, checks, return_the_error, row_key, section, setter_doc}`, where `checks(context: &MethodGenerationContext, write: &GuardedWrite, row: &TokenStream, on_violation: impl Fn(&TokenStream) -> TokenStream) -> TokenStream`, `row_key(context: &MethodGenerationContext, row: &TokenStream) -> TokenStream`, `section(internal_columns: &[InternalColumn], checked: CheckedRules) -> String` and `setter_doc(disallowed: &BTreeSet<Disallowed>, column_type_kind: ColumnTypeKind) -> String`; `Setter::map` gains the parameter `disallowed: &BTreeSet<Disallowed>`.

- [ ] **Step 1: Declare the attribute**

In `derive-input/src/internal/dsl.rs`, add `pub mod disallow;` after `pub mod creation_default;`, and after `symbol!(creation_default);`:

```rust
symbol!(disallow);
symbol!(zero);
```

In `derive-input/src/api/attribute.rs`, make `FIELD_ATTRIBUTE_NAMES` ten long, with `dsl::disallow.0` after `dsl::creation_default.0`. In `derive/src/lib.rs`, add `disallow` after `creation_default` in the helper attributes of the `SpacetimeDSL` derive.

Create `derive-input/src/internal/dsl/disallow.rs` with its module comment alone:

```rust
//! `#[disallow(...)]`: what the value of a column must not be, and the columns a rule is not
//! allowed on.
```

Run the unit gate. Expected: all `ok`.

- [ ] **Step 2: Write the failing runtime group and compile tests**

Create `examples/test/src/disallow_zero_test.rs`:

```rust
//! `#[disallow(zero)]`: a column whose value must not be `0`, or `Uuid::NIL` for a `Uuid`.
//!
//! `create_<table>`, `update_<table>_by_<key>` and both paths of `upsert_<table>` refuse a row
//! which breaks the rule, after their before hook, so a hook may repair the value.
//! `create_<table>` skips an `#[auto_inc]` column, whose placeholder `0` SpacetimeDB replaces.

use {crate::spacetimedsl::prelude::*, spacetimedb::Uuid};

#[spacetimedsl::dsl(plural_name = disallow_zero_items, method(update = true))]
#[spacetimedb::table(accessor = disallow_zero_item)]
pub struct DisallowZeroItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[disallow(zero)]
    id: u64,

    #[disallow(zero)]
    pub stock: u32,

    #[disallow(zero)]
    pub serial: Uuid,
}

/// An item whose hooks turn a `stock` of `0` into `1` before the rule is checked.
#[spacetimedsl::dsl(
    plural_name = disallow_zero_repaired_items,
    method(update = true),
    hook(before(insert, update))
)]
#[spacetimedb::table(accessor = disallow_zero_repaired_item)]
pub struct DisallowZeroRepairedItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(zero)]
    pub stock: u32,
}

#[spacetimedsl::hook]
fn before_disallow_zero_repaired_item_insert(
    _dsl: &DSL<'_, T>,
    mut item: CreateDisallowZeroRepairedItem,
) -> Result<CreateDisallowZeroRepairedItem, SpacetimeDSLError> {
    if item.stock == 0 {
        item.stock = 1;
    }

    Ok(item)
}

#[spacetimedsl::hook]
fn before_disallow_zero_repaired_item_update(
    _dsl: &DSL<'_, T>,
    _old_item: &DisallowZeroRepairedItem,
    mut new_item: DisallowZeroRepairedItem,
) -> Result<DisallowZeroRepairedItem, SpacetimeDSLError> {
    if *new_item.get_stock() == 0 {
        new_item.set_stock(1);
    }

    Ok(new_item)
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = disallow_zero_limits)]
pub struct DisallowZeroLimits {
    #[disallow(zero)]
    pub maximum_stock: u32,
}

impl DefaultSingleton for DisallowZeroLimits {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<DisallowZeroLimits, SpacetimeDSLError> {
        Ok(DisallowZeroLimits {
            id: 0,
            maximum_stock: 100,
        })
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    create_skips_the_auto_inc_primary_key(dsl)?;
    create_refuses_a_zero(dsl)?;
    create_refuses_a_nil_uuid(dsl)?;
    update_refuses_a_zero(dsl)?;
    a_before_hook_can_repair_the_value(dsl)?;
    upsert_refuses_a_zero_on_both_paths(dsl)?;

    Ok(())
}

/// `id` is `#[auto_inc]` and `#[disallow(zero)]`: create writes the placeholder `0` into it,
/// which SpacetimeDB replaces, so the rule must not refuse the row.
fn create_skips_the_auto_inc_primary_key<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let item = dsl
        .create_disallow_zero_item(CreateDisallowZeroItem {
            stock: 5,
            serial: dsl.ctx().new_uuid_v4()?,
        })
        .map_err(|error| {
            format!(
                "Should create an item whose `#[auto_inc]` key has `#[disallow(zero)]`! Got:\n{error}"
            )
        })?;

    if item.get_id().value() == 0 {
        return Err(
            "SpacetimeDB should have replaced the placeholder 0 of the `#[auto_inc]` key!"
                .to_string(),
        );
    }

    Ok(())
}

fn create_refuses_a_zero<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let creation = dsl.create_disallow_zero_item(CreateDisallowZeroItem {
        stock: 0,
        serial: dsl.ctx().new_uuid_v4()?,
    });

    expect_disallowed(
        creation,
        "Disallowed Value Error while trying to create a row in the `disallow_zero_item` table because `stock` is `0`, which `#[disallow(zero)]` forbids!",
    )
}

fn create_refuses_a_nil_uuid<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let creation = dsl.create_disallow_zero_item(CreateDisallowZeroItem {
        stock: 5,
        serial: Uuid::NIL,
    });

    expect_disallowed(
        creation,
        "Disallowed Value Error while trying to create a row in the `disallow_zero_item` table because `serial` is `Uuid::NIL`, which `#[disallow(zero)]` forbids!",
    )
}

fn update_refuses_a_zero<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut item = dsl.create_disallow_zero_item(CreateDisallowZeroItem {
        stock: 5,
        serial: dsl.ctx().new_uuid_v4()?,
    })?;
    let expected_message = format!(
        "Disallowed Value Error while trying to update the row `{{ id : {} }}` in the `disallow_zero_item` table because `stock` is `0`, which `#[disallow(zero)]` forbids!",
        item.get_id().value()
    );
    item.set_stock(0);

    expect_disallowed(dsl.update_disallow_zero_item_by_id(item), &expected_message)
}

/// The rule is checked after the before hooks, which turn a `stock` of `0` into `1`.
fn a_before_hook_can_repair_the_value<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut item = dsl
        .create_disallow_zero_repaired_item(CreateDisallowZeroRepairedItem { stock: 0 })
        .map_err(|error| {
            format!("The before_insert hook should have repaired the stock of 0! Got:\n{error}")
        })?;

    if *item.get_stock() != 1 {
        return Err(format!(
            "The before_insert hook should have written a stock of 1! Got: {}",
            item.get_stock()
        ));
    }

    item.set_stock(0);

    let item = dsl
        .update_disallow_zero_repaired_item_by_id(item)
        .map_err(|error| {
            format!("The before_update hook should have repaired the stock of 0! Got:\n{error}")
        })?;

    if *item.get_stock() != 1 {
        return Err(format!(
            "The before_update hook should have written a stock of 1! Got: {}",
            item.get_stock()
        ));
    }

    Ok(())
}

/// The insert path of `upsert_disallow_zero_limits` checks the row it is about to create, the
/// update path the row it is about to write over the stored one.
fn upsert_refuses_a_zero_on_both_paths<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut limits = dsl.get_disallow_zero_limits()?;
    limits.set_maximum_stock(0);

    expect_disallowed(
        dsl.upsert_disallow_zero_limits(limits.clone()),
        "Disallowed Value Error while trying to create a row in the `disallow_zero_limits` table because `maximum_stock` is `0`, which `#[disallow(zero)]` forbids!",
    )?;

    limits.set_maximum_stock(50);
    let mut limits = dsl.upsert_disallow_zero_limits(limits)?;
    limits.set_maximum_stock(0);

    expect_disallowed(
        dsl.upsert_disallow_zero_limits(limits),
        "Disallowed Value Error while trying to update the row `{ id : 0 }` in the `disallow_zero_limits` table because `maximum_stock` is `0`, which `#[disallow(zero)]` forbids!",
    )
}

/// Checks that `write` failed with the message of a broken `#[disallow]` rule. The groups of
/// the other rules check their writes with it too.
pub(crate) fn expect_disallowed<Row: std::fmt::Debug>(
    write: Result<Row, SpacetimeDSLError>,
    expected_message: &str,
) -> Result<(), String> {
    match write {
        Err(SpacetimeDSLError::Error(message)) if message == expected_message => Ok(()),
        other => Err(format!(
            "The write should fail with \"{expected_message}\"! Got: {other:?}"
        )),
    }
}
```

In `examples/test/src/lib.rs`, add `pub mod disallow_zero_test;` after `pub mod creation_default_test;`, and append to `TEST_GROUPS`:

```rust
    ("disallow_zero_test", disallow_zero_test::run_tests),
```

Create the nine compile tests. The first five share this table, with the marked line replaced:

```rust
::spacetimedsl::spacetimedsl!();

pub mod item {
    #[spacetimedsl::dsl(plural_name = items, method(update = true))]
    #[spacetimedb::table(accessor = item, public)]
    pub struct Item {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[disallow] // the line each file varies
        pub stock: u32,
    }
}

fn main() {}
```

| File | The marked line |
| --- | --- |
| `disallow_without_list.rs` | `#[disallow]` |
| `disallow_with_empty_list.rs` | `#[disallow()]` |
| `disallow_rule_repeated.rs` | `#[disallow(zero, zero)]` |
| `disallow_repeated.rs` | `#[disallow(zero)]`, followed by a second line `#[disallow(zero)]` |
| `disallow_unknown_rule.rs` | `#[disallow(negative)]` |

Write each file out whole, without the trailing comment.

Create `compile-tests/tests/ui/disallow_zero_on_signed_column.rs` with the same table, the column being

```rust
        #[disallow(zero)]
        pub balance: i64,
```

Create `compile-tests/tests/ui/disallow_zero_with_creation_default_zero.rs` with the same table, the column being

```rust
        #[creation_default(0)]
        #[disallow(zero)]
        pub stock: u32,
```

Create `compile-tests/tests/ui/disallow_zero_with_creation_default_uuid_nil.rs` with the same table, the column being

```rust
        #[creation_default(spacetimedb::Uuid::NIL)]
        #[disallow(zero)]
        pub serial: spacetimedb::Uuid,
```

Create `compile-tests/tests/ui/disallow_zero_with_set_zero_strategy.rs`:

```rust
//! `on_delete = SetZero` writes `0` into `warehouse_id` when its warehouse is deleted, which
//! `#[disallow(zero)]` forbids.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so the
//! `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        #[disallow(zero)]
        pub warehouse_id: u64,
    }
}

fn main() {}
```

- [ ] **Step 3: Observe the red**

Run the unit gate.
Expected: `FAILED`; trybuild reports all nine files with *Expected test case to fail to compile, but it succeeded*: the attribute is known and ignored.

Run the runtime gate.
Expected: `FAILED`; the `tester` reducer reports `disallow_zero_test: The write should fail with "Disallowed Value Error while trying to create a row in the `disallow_zero_item` table because `stock` is `0`, …"! Got: Ok(DisallowZeroItem { … })`.

- [ ] **Step 4: Pin the model in the contract test (the red)**

In `derive/src/data_transfer_contract_tests.rs`, add `disallow::Disallowed,` to the `dsl::{…}` import. In the `Gadget` fixture, put `#[disallow(zero)]` below `#[creation_default(1)]` on `revision`. Add after the assertions on `revision`:

```rust
    assert_eq!(
        revision
            .spacetimedsl_column
            .disallowed
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [Disallowed::Zero]
    );
    assert_eq!(
        revision
            .spacetimedsl_column
            .setter
            .as_ref()
            .expect("a public column has a setter")
            .doc_comment,
        "Writing the row through the DSL fails with a *Disallowed Value Error* if this column is `0` (`#[disallow(zero)]`)."
    );
```

In `visit_spacetimedsl_column`, add `disallowed,` to the destructuring of `SpacetimeDSLColumn` after `creation_default: _,`, and after the `match auto_generated_uuid_version`:

```rust
    for rule in disallowed {
        match rule {
            Disallowed::Zero => {}
        }
    }
```

Run the unit gate.
Expected: `FAILED` with `error[E0432]: unresolved import `spacetimedsl_derive_input::api::dsl::disallow``.

- [ ] **Step 5: Add the rule to the public model**

Create `derive-input/src/api/dsl/disallow.rs`:

```rust
/// A rule of `#[disallow(...)]`: what the value of a column must not be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Disallowed {
    /// `zero`: the value `0`, or `Uuid::NIL` for a `Uuid` column.
    Zero,
}
```

In `derive-input/src/api/dsl.rs`, add `pub mod disallow;` after `pub mod soft_delete;`.

In `derive-input/src/api/dsl/column.rs`, add `disallow::Disallowed` to the `super::{…}` import and `std::collections::BTreeSet` to the imports, and add to `SpacetimeDSLColumn` after `creation_default`:

```rust
    /// The rules `#[disallow(...)]` states for the value of the field. Empty without the
    /// attribute.
    pub disallowed: BTreeSet<Disallowed>,
```

- [ ] **Step 6: Parse the rules and reject the columns `zero` cannot guard**

Fill `derive-input/src/internal/dsl/disallow.rs`:

```rust
//! `#[disallow(...)]`: what the value of a column must not be, and the columns a rule is not
//! allowed on.

use {
    super::{disallow, zero},
    crate::{
        api::{
            dsl::{
                disallow::Disallowed,
                foreign_key::{ForeignKey, OnDeleteStrategy},
            },
            rust::column::RustField,
        },
        internal::{column::ColumnTypeKind, dsl::method::doc, error},
    },
    quote::ToTokens,
    spacetime_bindings_macro_input::{match_meta, sats::SatsField},
    std::collections::BTreeSet,
    syn::{Attribute, Expr, ExprLit, Lit},
};

impl Disallowed {
    /// The word naming the rule inside `#[disallow(...)]`.
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Disallowed::Zero => zero.0,
        }
    }
}

/// How a message or the documentation writes the value `#[disallow(zero)]` forbids in a
/// column of this kind: `0`, or `Uuid::NIL`. `None` for a kind the rule is not allowed on.
pub fn forbidden_zero(column_type_kind: ColumnTypeKind) -> Option<&'static str> {
    match column_type_kind {
        ColumnTypeKind::UnsignedInteger => Some("0"),
        ColumnTypeKind::UUID => Some("Uuid::NIL"),
        _ => None,
    }
}

/// Reads `#[disallow(...)]` from a column, and rejects a rule the column cannot keep.
pub fn try_parse(
    field: &SatsField<'_>,
    rust_field: &RustField,
    foreign_key: Option<&ForeignKey>,
    creation_default: Option<&Expr>,
) -> syn::Result<BTreeSet<Disallowed>> {
    let mut disallow_attributes = field
        .original_attrs
        .iter()
        .filter(|attribute| attribute.path() == disallow);

    let Some(disallow_attribute) = disallow_attributes.next() else {
        return Ok(BTreeSet::new());
    };

    if let Some(repeated_attribute) = disallow_attributes.next() {
        return Err(error::multiple_disallow_attributes(repeated_attribute));
    }

    let disallowed = parse_rules(disallow_attribute)?;

    if disallowed.contains(&Disallowed::Zero) {
        reject_zero_the_column_cannot_keep(
            field,
            rust_field,
            disallow_attribute,
            foreign_key,
            creation_default,
        )?;
    }

    Ok(disallowed)
}

fn parse_rules(disallow_attribute: &Attribute) -> syn::Result<BTreeSet<Disallowed>> {
    if disallow_attribute.meta.require_list().is_err() {
        return Err(error::disallow_without_rules(disallow_attribute));
    }

    let mut disallowed = BTreeSet::new();

    disallow_attribute.parse_nested_meta(|meta| {
        let rule = match_meta!(match meta {
            zero => Disallowed::Zero,
        });

        if !disallowed.insert(rule) {
            return Err(error::repeated_disallow_rule(&meta.path));
        }

        Ok(())
    })?;

    if disallowed.is_empty() {
        return Err(error::disallow_without_rules(disallow_attribute));
    }

    Ok(disallowed)
}

/// `zero` needs a type with a value that references nothing, and a column nothing else
/// writes that value into.
fn reject_zero_the_column_cannot_keep(
    field: &SatsField<'_>,
    rust_field: &RustField,
    disallow_attribute: &Attribute,
    foreign_key: Option<&ForeignKey>,
    creation_default: Option<&Expr>,
) -> syn::Result<()> {
    let Some(zero_value) = forbidden_zero(ColumnTypeKind::of(&rust_field.type_name_or_path))
    else {
        return Err(error::disallow_zero_on_unsupported_type(field.ty));
    };

    let is_cleared_by_set_zero = foreign_key
        .and_then(|foreign_key| foreign_key.on_delete_strategy.as_ref())
        == Some(&OnDeleteStrategy::SetZero);

    if is_cleared_by_set_zero {
        return Err(error::disallow_zero_with_set_zero_strategy(
            disallow_attribute,
        ));
    }

    if let Some(creation_default) = creation_default
        && writes_zero(creation_default)
    {
        return Err(error::disallow_zero_with_zero_creation_default(
            creation_default,
            &doc::written_tokens(creation_default.to_token_stream()),
            zero_value,
        ));
    }

    Ok(())
}

/// Whether `expression` is written as the value `#[disallow(zero)]` forbids: an integer
/// literal `0`, with or without a suffix, or a path ending in `Uuid::NIL`. Any other
/// expression is left to the check that runs when the row is written.
fn writes_zero(expression: &Expr) -> bool {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Int(integer),
            ..
        }) => integer.base10_digits() == "0",
        Expr::Path(path) => {
            let segments: Vec<String> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();

            matches!(
                segments.as_slice(),
                [.., type_name, constant] if type_name == "Uuid" && constant == "NIL"
            )
        }
        Expr::Group(group) => writes_zero(&group.expr),
        Expr::Paren(parenthesized) => writes_zero(&parenthesized.expr),
        _ => false,
    }
}
```

In `derive-input/src/internal/error.rs`, add `Path` to the `syn::{…}` import and a section after `#[creation_default]`:

```rust
// `#[disallow]`

pub fn multiple_disallow_attributes(disallow_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        disallow_attribute,
        "Only one `#[disallow]` is allowed per column! Name all its rules in one list.",
    )
}

pub fn disallow_without_rules(disallow_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        disallow_attribute,
        "`#[disallow(...)]` has to name at least one rule, e.g. `#[disallow(zero)]`!",
    )
}

pub fn repeated_disallow_rule(rule: &Path) -> Error {
    Error::new_spanned(
        rule,
        format!(
            "`{}` is given twice in `#[disallow(...)]`! Remove this one.",
            rule.to_token_stream()
        ),
    )
}

pub fn disallow_zero_on_unsupported_type(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "`#[disallow(zero)]` is only allowed on unsigned integer and `Uuid` columns, whose `0` or `Uuid::NIL` it forbids! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn disallow_zero_with_set_zero_strategy(disallow_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        disallow_attribute,
        "`#[disallow(zero)]` is not allowed together with `on_delete = SetZero`, which writes `0` or `Uuid::NIL` into this column when the referenced row is deleted! Remove `zero`, or choose another strategy, such as `on_delete = Delete`.",
    )
}

pub fn disallow_zero_with_zero_creation_default(
    creation_default: &impl ToTokens,
    written_creation_default: &str,
    zero_value: &str,
) -> Error {
    Error::new_spanned(
        creation_default,
        format!(
            "`#[creation_default({written_creation_default})]` fills this column with `{zero_value}`, which `#[disallow(zero)]` forbids! Choose another default, or remove `zero`."
        ),
    )
}
```

In `derive-input/src/internal/dsl/column.rs`, add `dsl::disallow` to the `internal::{…}` import, parse the rules after `creation_default`:

```rust
        let disallowed = disallow::try_parse(
            field,
            rust_field,
            foreign_key.as_ref(),
            creation_default.as_ref(),
        )?;
```

pass `&disallowed` as the new last argument of `Setter::map`, and add `disallowed,` to the returned `SpacetimeDSLColumn` after `creation_default,`.

In `derive-input/src/internal/column.rs`, add to `InternalColumn` after `spacetimedsl_column_creation_default`:

```rust
    pub spacetimedsl_column_disallowed: BTreeSet<Disallowed>,
```

(import `api::dsl::disallow::Disallowed` and `std::collections::BTreeSet` next to `BTreeMap`), and fill it after `spacetimedsl_column_creation_default`:

```rust
            spacetimedsl_column_disallowed: spacetimedsl_column.disallowed.clone(),
```

- [ ] **Step 7: Generate the checks and their documentation**

Create `derive-input/src/internal/dsl/method/disallow.rs`:

```rust
//! The checks `#[disallow(...)]` adds to every DSL method which writes a row, and what the
//! documentation says about them.
//!
//! A write runs the checks after its before hook, so a hook may repair a value and cannot
//! slip a forbidden one past them, and before the framework writes the columns it owns.

use {
    super::{context::MethodGenerationContext, doc, message},
    crate::{
        api::{dsl::disallow::Disallowed, runtime},
        internal::{
            column::{ColumnTypeKind, InternalColumn},
            dsl::disallow::forbidden_zero,
            spacetimedb,
        },
    },
    proc_macro2::TokenStream,
    quote::quote,
    std::collections::BTreeSet,
};

/// Why a column with `zero` always has a value it forbids.
const ZERO_ONLY_WITH_A_FORBIDDEN_VALUE: &str =
    "`internal/dsl/disallow.rs` allows `zero` only on unsigned integer and `Uuid` columns";

/// The write a check guards, which its message names.
pub enum GuardedWrite {
    /// `create_<table>` and the insert path of `upsert_<table>`. The row has no key yet which
    /// a message could name: an `#[auto_inc]` key is still the placeholder `0`.
    Create,
    /// A write over a stored row. `row_key` renders the row's primary key as `{ id : 7 }`.
    Update { row_key: TokenStream },
}

/// Which of a column's rules a documented method checks.
#[derive(Clone, Copy)]
pub enum CheckedRules {
    /// `create_<table>`: every rule, but none on an `#[auto_inc]` column.
    OfANewRow,
    /// A write over a stored row: every rule.
    OfAWrittenRow,
}

/// `return Err(<error>);`, how a DSL method leaves on a broken rule.
pub fn return_the_error(error: &TokenStream) -> TokenStream {
    quote! {
        return Err(#error);
    }
}

/// The expression which renders the primary key of the row `row` as `{ id : 7 }`, or the
/// `{ id : 0 }` of a singleton, whose injected key has no getter.
pub fn row_key(context: &MethodGenerationContext, row: &TokenStream) -> TokenStream {
    let primary_key_column_name = &context.primary_key_column_name;

    match context.spacetimedsl_table.is_singleton() {
        true => message::singleton_primary_key(),
        false => message::single_column_and_value(
            primary_key_column_name,
            &quote! { #row.#primary_key_column_name },
        ),
    }
}

/// One `if` per rule of a column of the table which `write` checks, reading the row `row`
/// and ending in `on_violation` with the error the broken rule reports. Empty when no column
/// has such a rule.
pub fn checks(
    context: &MethodGenerationContext,
    write: &GuardedWrite,
    row: &TokenStream,
    on_violation: impl Fn(&TokenStream) -> TokenStream,
) -> TokenStream {
    let mut checks = TokenStream::new();

    for internal_column in context.internal_columns {
        for rule in &internal_column.spacetimedsl_column_disallowed {
            if !is_checked(*rule, internal_column, write) {
                continue;
            }

            let check = match rule {
                Disallowed::Zero => zero_check(internal_column, row),
            };

            let error = disallowed_value_error(context, write, internal_column, *rule, &check);
            let leave = on_violation(&error);
            let violated = check.violated;

            checks.extend(quote! {
                if #violated {
                    #leave
                }
            });
        }
    }

    checks
}

/// Whether `write` checks `rule` of `internal_column`. `create_<table>` writes the placeholder
/// `0` into an `#[auto_inc]` column, which SpacetimeDB replaces with a value of its sequence,
/// never `0`.
fn is_checked(rule: Disallowed, internal_column: &InternalColumn, write: &GuardedWrite) -> bool {
    match (rule, write) {
        (Disallowed::Zero, GuardedWrite::Create) => {
            !internal_column.spacetimedb_column_is_auto_inc
        }
        (Disallowed::Zero, GuardedWrite::Update { .. }) => true,
    }
}

/// A rule's condition, and the part of its message which says what the column is or did,
/// with the values the `{}` placeholders of that part show.
struct Check {
    violated: TokenStream,
    reason: String,
    reason_arguments: Vec<TokenStream>,
}

fn zero_check(internal_column: &InternalColumn, row: &TokenStream) -> Check {
    let column_name = &internal_column.rust_field_name;
    let zero_value = match internal_column.rust_field_type_kind {
        ColumnTypeKind::UUID => spacetimedb::uuid_nil(),
        _ => quote! { 0 },
    };

    Check {
        violated: quote! { #row.#column_name == #zero_value },
        reason: format!(
            "is `{}`",
            written_zero(internal_column.rust_field_type_kind)
        ),
        reason_arguments: vec![],
    }
}

fn written_zero(column_type_kind: ColumnTypeKind) -> &'static str {
    forbidden_zero(column_type_kind).expect(ZERO_ONLY_WITH_A_FORBIDDEN_VALUE)
}

/// `SpacetimeDSLError::Error(<message>)` for `rule` of `internal_column` broken by `write`,
/// such as *Disallowed Value Error while trying to update the row `{ id : 7 }` in the
/// `player` table because `level` is `0`, which `#[disallow(zero)]` forbids!*
fn disallowed_value_error(
    context: &MethodGenerationContext,
    write: &GuardedWrite,
    internal_column: &InternalColumn,
    rule: Disallowed,
    check: &Check,
) -> TokenStream {
    let (attempted, mut arguments) = match write {
        GuardedWrite::Create => ("create a row in", vec![]),
        GuardedWrite::Update { row_key } => ("update the row `{}` in", vec![row_key.clone()]),
    };
    arguments.extend(check.reason_arguments.iter().cloned());

    let message = format!(
        "Disallowed Value Error while trying to {attempted} the `{}` table because `{}` {}, which `#[disallow({})]` forbids!",
        context.singular_table_name,
        internal_column.rust_field_name,
        check.reason,
        rule.keyword(),
    );

    let message = match arguments.is_empty() {
        true => quote! { #message.to_string() },
        false => quote! { format!(#message, #(#arguments),*) },
    };

    runtime::generic_error(&message)
}

/// The `# Disallowed values` section of a method which checks `checked`. Empty when no column
/// of the table has a rule it checks.
pub fn section(internal_columns: &[InternalColumn], checked: CheckedRules) -> String {
    let bullets: Vec<String> = internal_columns
        .iter()
        .filter_map(|internal_column| {
            let forbidden: Vec<String> = internal_column
                .spacetimedsl_column_disallowed
                .iter()
                .filter(|rule| match checked {
                    CheckedRules::OfANewRow => {
                        is_checked(**rule, internal_column, &GuardedWrite::Create)
                    }
                    CheckedRules::OfAWrittenRow => true,
                })
                .map(|rule| forbidden_value(*rule, internal_column.rust_field_type_kind))
                .collect();

            (!forbidden.is_empty()).then(|| {
                format!(
                    "- `{}`: {}",
                    internal_column.rust_field_name,
                    forbidden.join(", ")
                )
            })
        })
        .collect();

    doc::section(
        "Disallowed values",
        Some("Fails with a *Disallowed Value Error* if a column holds what its `#[disallow]` forbids:"),
        &bullets,
    )
}

/// How the documentation names what `rule` forbids.
fn forbidden_value(rule: Disallowed, column_type_kind: ColumnTypeKind) -> String {
    match rule {
        Disallowed::Zero => format!("`{}`", written_zero(column_type_kind)),
    }
}

/// What the setter of a column says about the column's rules: which values a write through
/// the DSL refuses. Empty without rules.
pub fn setter_doc(disallowed: &BTreeSet<Disallowed>, column_type_kind: ColumnTypeKind) -> String {
    if disallowed.is_empty() {
        return String::new();
    }

    let refused: Vec<String> = disallowed
        .iter()
        .map(|rule| match rule {
            Disallowed::Zero => format!("is `{}`", written_zero(column_type_kind)),
        })
        .collect();

    let keywords: Vec<&str> = disallowed.iter().map(|rule| rule.keyword()).collect();

    format!(
        "Writing the row through the DSL fails with a *Disallowed Value Error* if this column {} (`#[disallow({})]`).",
        refused.join(" or "),
        keywords.join(", "),
    )
}
```

In `method.rs`, declare `pub mod disallow;` after `mod delete;`.

In `create.rs`, add `disallow::{self, CheckedRules, GuardedWrite}` to the `super::{…}` import. In `for_create`, add before `let insert = …`:

```rust
    let disallow_checks = disallow::checks(
        context,
        &GuardedWrite::Create,
        &quote! { #singular_table_name },
        disallow::return_the_error,
    );
```

In its `method_impl`, put `#disallow_checks` right after the row is built:

```rust
            let #singular_table_name = #struct_name {
                #(#constructor_arg_names),*
            };

            #disallow_checks

            #let_field_name_for_found_value
```

and add the section to its documentation, between the defaults and the foreign keys:

```rust
        doc_comment: doc::paragraphs([
            format!("Create a row in the `{singular_table_name}` table."),
            defaults_section(internal_columns),
            disallow::section(internal_columns, CheckedRules::OfANewRow),
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each column references a row:",
                &internal_columns.iter().collect_vec(),
            ),
        ]),
```

In `update.rs`, add the same import. In `for_update`, add before `SpacetimeDSLMethod { … }`:

```rust
    let row = quote! { #singular_table_name };
    let disallow_checks = disallow::checks(
        context,
        &GuardedWrite::Update {
            row_key: disallow::row_key(context, &row),
        },
        &row,
        disallow::return_the_error,
    );
```

put `#disallow_checks` in its `method_impl` right after `#before_update_hook`, and replace its `doc_comment` by:

```rust
        doc_comment: doc::paragraphs([
            match is_singleton_pk {
                true => format!(
                    "Try to update the `{struct_name}` row of the singleton `{singular_table_name}` table."
                ),
                false => format!(
                    "{unique_multi_column_index_hint}\n\nTry to update a `{struct_name}` row of the `{singular_table_name}` table {described_as}."
                ),
            },
            disallow::section(internal_columns, CheckedRules::OfAWrittenRow),
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each of these columns references a row whenever its value changes:",
                &columns_with_a_setter,
            ),
        ]),
```

In `upsert.rs`, add the same import. In `for_singleton_upsert`, add before `SpacetimeDSLMethod { … }`:

```rust
    let row = quote! { #singular_table_name };
    let disallow_checks_on_update = disallow::checks(
        context,
        &GuardedWrite::Update {
            row_key: disallow::row_key(context, &row),
        },
        &row,
        disallow::return_the_error,
    );
    let disallow_checks_on_insert = disallow::checks(
        context,
        &GuardedWrite::Create,
        &row,
        disallow::return_the_error,
    );
```

In its `method_impl`, put `#disallow_checks_on_update` right after `#before_update_hook_call` and `#disallow_checks_on_insert` right after `#before_insert_hook`, and replace its `doc_comment` by:

```rust
        doc_comment: doc::paragraphs([
            format!(
                "Write the `{struct_name}` row of the singleton `{singular_table_name}` table, whether or not it exists yet."
            ),
            disallow::section(internal_columns, CheckedRules::OfAWrittenRow),
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each column references a row. While the row exists, only the columns with a setter are checked, whenever their value changes:",
                &internal_columns.iter().collect_vec(),
            ),
        ]),
```

In `derive-input/src/internal/dsl/setter.rs`, add `api::dsl::disallow::Disallowed`, `internal::column::ColumnTypeKind`, `method::{disallow, doc}` and `std::collections::BTreeSet` to the imports, the parameter `disallowed: &BTreeSet<Disallowed>` after `foreign_key`, and replace its `doc_comment` by:

```rust
            doc_comment: doc::paragraphs([
                foreign_key
                    .map(relationship_doc::foreign_key)
                    .unwrap_or_default(),
                disallow::setter_doc(
                    disallowed,
                    ColumnTypeKind::of(&rust_field.type_name_or_path),
                ),
            ]),
```

- [ ] **Step 8: Observe the green and read the diagnostics**

Run the unit gate. Expected: the contract test passes, and no existing snapshot moves: `checks`, `section` and `setter_doc` are empty for a table without rules. The nine compile tests still fail, for want of their `.stderr`.

Regenerate the compile-test output and read every new file. Expected first errors:

| File | Message | Underlined |
| --- | --- | --- |
| `disallow_without_list` | *`#[disallow(...)]` has to name at least one rule, e.g. `#[disallow(zero)]`!* | `#[disallow]` |
| `disallow_with_empty_list` | the same | `#[disallow()]` |
| `disallow_rule_repeated` | *`zero` is given twice in `#[disallow(...)]`! Remove this one.* | the second `zero` |
| `disallow_repeated` | *Only one `#[disallow]` is allowed per column! Name all its rules in one list.* | the second attribute |
| `disallow_unknown_rule` | *expected `zero`* | `negative` |
| `disallow_zero_on_signed_column` | *`#[disallow(zero)]` is only allowed on unsigned integer and `Uuid` columns, whose `0` or `Uuid::NIL` it forbids! Found: i64* | `i64` |
| `disallow_zero_with_set_zero_strategy` | *`#[disallow(zero)]` is not allowed together with `on_delete = SetZero`, …*, followed by the follow-on errors its `//!` comment names | `#[disallow(zero)]` |
| `disallow_zero_with_creation_default_zero` | *`#[creation_default(0)]` fills this column with `0`, which `#[disallow(zero)]` forbids! …* | `0` |
| `disallow_zero_with_creation_default_uuid_nil` | *`#[creation_default(spacetimedb::Uuid::NIL)]` fills this column with `Uuid::NIL`, …* | `spacetimedb::Uuid::NIL` |

No existing `.stderr` changes.

- [ ] **Step 9: Snapshot the checks**

Create `derive/tests/fixtures/disallow_zero.rs`:

```rust
//! Covers `#[disallow(zero)]` on the column shapes it is allowed on: an unsigned integer, a
//! private `Uuid`, a `#[use_wrapper]` foreign key, where it forbids a reference to no row, and
//! an `#[auto_inc]` primary key, which `create_pallet` skips because SpacetimeDB replaces the
//! `0` it writes there. `create_pallet` and `update_pallet_by_id` check the row after their
//! before hooks, `upsert_pallet_limits` on both of its paths, and the setters and the write
//! methods document the rules.

#[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = false))]
#[spacetimedb::table(accessor = warehouse, public)]
pub struct Warehouse {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = pallet)]
    id: u64,
}

#[spacetimedsl::dsl(
    plural_name = pallets,
    method(update = true),
    hook(before(insert, update))
)]
#[spacetimedb::table(accessor = pallet, public)]
pub struct Pallet {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[disallow(zero)]
    id: u64,

    #[disallow(zero)]
    pub quantity: u32,

    #[disallow(zero)]
    tracking_code: spacetimedb::Uuid,

    #[index(btree)]
    #[use_wrapper(WarehouseId)]
    #[foreign_key(path = self, table = warehouse, column = id)]
    #[disallow(zero)]
    pub warehouse_id: u64,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = pallet_limits, public)]
pub struct PalletLimits {
    #[disallow(zero)]
    pub maximum_quantity: u32,
}
```

Register it after `creation_default` in `derive/src/characterization_tests.rs`. In `derive/tests/fixtures/every_field_attribute.rs`, name `disallow` in the module comment after `creation_default`, and put `#[disallow(zero)]` below `#[creation_default(1)]` on `rating`.

Regenerate the snapshots and read them:

- `disallow_zero/Pallet/create_pallet.snap`: after `let pallet = Pallet { … };`, three checks — `pallet.quantity == 0`, `pallet.tracking_code == ::spacetimedb::Uuid::NIL`, `pallet.warehouse_id == 0` — each returning `SpacetimeDSLError::Error("Disallowed Value Error while trying to create a row in the `pallet` table because … is …, which `#[disallow(zero)]` forbids!".to_string())`; none for the `#[auto_inc]` `id`. Its documentation lists `quantity`, `tracking_code` and `warehouse_id` under *Disallowed values*.
- `disallow_zero/Pallet/update_pallet_by_id.snap`: four checks, `id` included, right after the `before_update` hook, each message built with `format!(…, format!("{{ id : {} }}", pallet.id))`; the section lists all four.
- `disallow_zero/PalletLimits/upsert_pallet_limits.snap`: the check after the `before_update` hook call on the update path, with the literal `"{ id : 0 }"` as the key, and after the insert path's (absent) `before_insert` hook, as a create.
- `disallow_zero/Pallet/table.snap`: the setters of `quantity` and `warehouse_id` say *Writing the row through the DSL fails with a *Disallowed Value Error* if this column is `0` (`#[disallow(zero)]`).*, the latter after the paragraph about its foreign key.
- `every_field_attribute/Gadget/`: `create_gadget.snap` and `update_gadget_by_id.snap` gain the check of `rating` and the section; `table.snap` the setter documentation of `rating`.

No other snapshot may change.

- [ ] **Step 10: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 11: Document the rule**

In `docs/DOCUMENTATION.md`, under *Column Attributes*, add:

```rust
#[disallow(zero)]        // Refuses to write 0, or Uuid::NIL, into the column
```

Add a section after *Accessor Methods (Getters/Setters)*, before *Foreign Keys & Referential Integrity*:

````markdown
## Disallowed Values

`#[disallow(...)]` on a column names what its value must not be. Every DSL method which writes the row refuses a value a rule forbids and fails with a `SpacetimeDSLError::Error` whose message starts with *Disallowed Value Error*:

```rust
#[spacetimedsl::dsl(plural_name = players, method(update = true))]
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(zero)]
    pub level: u8,
}
```

| Rule   | Forbids                       | Column types        |
| ------ | ----------------------------- | ------------------- |
| `zero` | the value `0`, or `Uuid::NIL` | `u8`–`u128`, `Uuid` |

- `create_<table>`, `update_<table>_by_<key>` and both paths of `upsert_<table>` check the rules after their before hook, so a hook may repair a value, and a value a hook writes is checked as well.
- `create_<table>` skips `zero` on an `#[auto_inc]` column: it writes `0` there, which SpacetimeDB replaces with a value of its sequence, never `0`. A row which reaches `0` another way, such as a system user written through raw SpacetimeDB access in the table's module, cannot be written through the DSL afterwards.
- On a foreign key column, `zero` forbids a reference to no row.
- The setter of the column and the documentation of each write method name the rules.

A second `#[disallow]` on a column, a rule named twice and a `#[disallow]` without a rule are rejected, and so are `zero` on a column that is not `u8`–`u128` or `Uuid`, `zero` on a column whose foreign key has `on_delete = SetZero`, which writes `0` into it, and `zero` on a column whose `#[creation_default(...)]` is `0` or `Uuid::NIL`.

```txt
Disallowed Value Error while trying to create a row in the `player` table because `level` is `0`, which `#[disallow(zero)]` forbids!
Disallowed Value Error while trying to update the row `{ id : 7 }` in the `player` table because `level` is `0`, which `#[disallow(zero)]` forbids!
```
````

In `README.md`, under *Data Integrity by Construction*, add after the bullet about foreign-key validation:

```markdown
- 🚫 `#[disallow(zero)]` refuses to write `0` or `Uuid::NIL` into a column, whichever DSL method writes the row.
```

In `docs/MIGRATION.md`, add `disallow` to the list of *`api::attribute::FIELD_ATTRIBUTE_NAMES` lists the field attributes*. In *`Getter::doc_comment` and `Setter::doc_comment`*, append to the setter's description *, followed by the values its `#[disallow]` rules forbid*. Add under *For crates building on `spacetimedsl_derive-input`*:

```markdown
#### `SpacetimeDSLColumn::disallowed`

`SpacetimeDSLColumn` gained `disallowed: BTreeSet<Disallowed>`, the rules of `#[disallow(...)]`, from the new `api::dsl::disallow::Disallowed`. The checks they add live inside the `method_impl` of the write methods.
```

- [ ] **Step 12: Format, lint, commit**

Commit message:

```text
Refuse 0 and Uuid::NIL with #[disallow(zero)]

create_<table>, update_<table>_by_<key> and both paths of upsert_<table> refuse a row
whose column with #[disallow(zero)] holds 0 or Uuid::NIL, after their before hook, with a
Disallowed Value Error naming the table, the row, the column and the rule. Create skips
an #[auto_inc] column, whose placeholder 0 SpacetimeDB replaces. The setter and the write
methods document the rule. zero is rejected on other types, next to on_delete = SetZero
and next to a literal #[creation_default(0)] or Uuid::NIL.

Tests: contract assertions for SpacetimeDSLColumn::disallowed; the fixture disallow_zero;
nine compile tests; the runtime group disallow_zero_test.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 6: `#[disallow(decreasing)]` and `#[disallow(increasing)]` in update and upsert (#186, part 2)

**Files:**
- Modify: `derive-input/src/internal/dsl/method/reference_integrity.rs` (extract the stored-row lookup)
- Modify: `derive-input/src/api/dsl/disallow.rs` (two variants), `derive-input/src/internal/dsl.rs` (two symbols)
- Modify: `derive-input/src/internal/dsl/disallow.rs` (the two rules and their rejections), `derive-input/src/internal/dsl/column.rs` (the call)
- Modify: `derive-input/src/internal/dsl/method/disallow.rs` (the comparison), `update.rs`, `upsert.rs`
- Modify: `derive-input/src/internal/error.rs` (five diagnostics)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Create: five compile tests in `compile-tests/tests/ui` (+ `.stderr`): `disallow_decreasing_on_string_column`, `disallow_decreasing_and_increasing`, `disallow_increasing_on_primary_key_column`, `disallow_decreasing_on_private_column`, `disallow_decreasing_with_set_zero_strategy`; recorded output: `disallow_unknown_rule.stderr`
- Create: `derive/tests/fixtures/disallow_change.rs`; register it
- Create: `examples/test/src/disallow_change_test.rs`; register it
- Modify: `docs/DOCUMENTATION.md` (*Disallowed Values*), `docs/MIGRATION.md`, `README.md`
- Recorded output: the `expect` text of the stored-row lookup in every `update_*_by_*.snap` which checks a foreign key; the new `derive/tests/snapshots/disallow_change/**`

**Interfaces:**
- Consumes: `method::disallow` of Task 5.
- Produces: `Disallowed::Decreasing`, `Disallowed::Increasing`; `reference_integrity::look_up_the_stored_row(singular_table_name: &Ident, field_name_for_found_value: &Ident, primary_key_column: &InternalColumn, is_singleton: bool) -> TokenStream` and `reference_integrity::stored_row(field_name_for_found_value: &Ident) -> TokenStream`; `GuardedWrite::Update { row_key: TokenStream, stored_row: TokenStream }`; `CheckedRules::OfAnUpsertedRow`; `method::disallow::compares_with_the_stored_row(internal_columns: &[InternalColumn]) -> bool`; `internal::dsl::disallow::try_parse` gains `spacetimedb_column: &SpacetimeDBColumn` after `rust_field`.

- [ ] **Step 1: Extract the stored-row lookup**

`update_<table>_by_<key>` looks up the stored row today only to compare foreign key columns; the change rules compare with it too, so the lookup leaves the foreign key check. Its `expect` text named the foreign keys as the reason, so it becomes one that holds for both.

In `derive-input/src/internal/dsl/method/reference_integrity.rs`, replace the constant `STORED_ROW_LOOKED_UP` and its doc by:

```rust
/// Why the generated update may unwrap the stored row: `look_up_the_stored_row` looks it up by
/// its primary key and returns when it finds none, before anything is compared with it.
const STORED_ROW_LOOKED_UP: &str =
    "the stored row is looked up by its primary key before the row to write is compared with it";
```

Add after it:

```rust
/// `if <found>.is_none() { <found> = <the stored row, or return a NotFoundError>; }`: look up
/// the stored row of the row about to be written by its primary key, unless an earlier check
/// did so already.
///
/// `is_singleton` decides how the lookup finds the row. Every other table reads its primary key
/// off the row through the key's wrapper, but a singleton's injected `id: u8` has neither a
/// getter nor a wrapper, so the lookup names its only legal value instead.
pub fn look_up_the_stored_row(
    singular_table_name: &Ident,
    field_name_for_found_value: &Ident,
    primary_key_column: &InternalColumn,
    is_singleton: bool,
) -> TokenStream {
    let primary_key_column_name = &primary_key_column.rust_field_name;

    // The stored row is looked up by the primary key value of the row to write, so a missing
    // row is reported with that value.
    let (primary_key_value, missing_row) = match is_singleton {
        true => {
            let primary_key_value = singleton::primary_key_value();

            (
                quote! { &#primary_key_value },
                message::singleton_primary_key(),
            )
        }
        false => {
            let getter_name = naming::getter_name(primary_key_column_name);
            let primary_key_value = quote! { #singular_table_name.#getter_name().value() };
            let missing_row =
                message::single_column_and_value(primary_key_column_name, &primary_key_value);

            (primary_key_value, missing_row)
        }
    };

    let not_found_error =
        runtime::not_found_error(&singular_table_name.to_string(), &missing_row);

    quote! {
        if #field_name_for_found_value.is_none() {
            #field_name_for_found_value = match self.db().#singular_table_name().#primary_key_column_name().find(#primary_key_value) {
                Some(#singular_table_name) => Some(#singular_table_name),
                None => {
                    return Err(#not_found_error);
                }
            };
        }
    }
}

/// `<found>.as_ref().expect(…)`: the stored row `look_up_the_stored_row` bound.
pub fn stored_row(field_name_for_found_value: &Ident) -> TokenStream {
    quote! {
        #field_name_for_found_value.as_ref().expect(#STORED_ROW_LOOKED_UP)
    }
}
```

Replace `reference_integrity_checks_on_update` by:

```rust
/// The checks an update runs on each non-private foreign key column whose value changes: that
/// the new value references a row. The stored row it compares with is looked up by
/// `look_up_the_stored_row`.
pub fn reference_integrity_checks_on_update(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
    field_name_for_found_value: &Ident,
    primary_key_column: &InternalColumn,
    is_singleton: bool,
) -> Vec<TokenStream> {
    let referencing_table_name = &spacetimedb_table.singular_name;
    let stored_row_lookup = look_up_the_stored_row(
        referencing_table_name,
        field_name_for_found_value,
        primary_key_column,
        is_singleton,
    );
    let stored = stored_row(field_name_for_found_value);

    reference_integrity_checks(columns, true, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name =
            naming::get_by_index_method_name(
                referenced_table_name,
                primary_key_column_name_of_referenced_table,
            );

        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let referencing_table_column_getter_name =
            naming::getter_name(referencing_table_column_name);

        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Update },
                &message::single_column_and_value(
                    referencing_table_column_name,
                    referencing_table_column_name,
                ),
            );

        quote! {
            #stored_row_lookup
            if #stored.#referencing_table_column_getter_name().ne(&#referencing_table_name.#referencing_table_column_getter_name()) {
                match self.#get_row_of_referenced_table_by_primary_key_method_name(#referencing_table_name.#referencing_table_column_getter_name()) {
                    Ok(_) => {},
                    Err(_) => return Err(#reference_integrity_violation_error)
                };
            }
        }
    })
}
```

Regenerate the snapshots and read the diff.
Expected: every changed hunk replaces *the stored row is looked up by its primary key before its foreign key columns are compared* by *the stored row is looked up by its primary key before the row to write is compared with it*, and nothing else changes. Check with:

```powershell
git diff -U0 -- derive/tests/snapshots | Select-String "^[+-] " | Select-String -NotMatch "the stored row is looked up by its primary key"
```

Expected: no output.

- [ ] **Step 2: Write the failing runtime group and compile tests**

Create `examples/test/src/disallow_change_test.rs`:

```rust
//! `#[disallow(decreasing)]` and `#[disallow(increasing)]`: a column whose value an update must
//! not lower, or must not raise.
//!
//! `update_<table>_by_<key>` and the update path of `upsert_<table>` compare the value they
//! write with the stored one; the insert path of `upsert_<table>` has nothing to compare with.
//! A float compares through `partial_cmp`: a change to or from NaN breaks both rules, and an
//! unchanged value, NaN included, breaks neither.

use crate::{disallow_zero_test::expect_disallowed, spacetimedsl::prelude::*};

#[spacetimedsl::dsl(plural_name = disallow_change_scores, method(update = true))]
#[spacetimedb::table(accessor = disallow_change_score)]
pub struct DisallowChangeScore {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(decreasing)]
    pub points: i64,

    #[disallow(increasing)]
    pub remaining_attempts: u8,

    #[disallow(decreasing)]
    pub rating: f64,

    pub label: String,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = disallow_change_season)]
pub struct DisallowChangeSeason {
    #[disallow(decreasing)]
    pub number: u32,
}

impl DefaultSingleton for DisallowChangeSeason {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<DisallowChangeSeason, SpacetimeDSLError> {
        Ok(DisallowChangeSeason { id: 0, number: 1 })
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    update_refuses_a_decrease(dsl)?;
    update_refuses_an_increase(dsl)?;
    update_accepts_the_allowed_changes(dsl)?;
    an_unchanged_nan_is_no_change(dsl)?;
    a_change_to_or_from_nan_is_refused(dsl)?;
    upsert_compares_only_while_the_row_exists(dsl)?;

    Ok(())
}

fn create_score<T: WriteContext>(
    dsl: &DSL<'_, T>,
    rating: f64,
) -> Result<DisallowChangeScore, SpacetimeDSLError> {
    dsl.create_disallow_change_score(CreateDisallowChangeScore {
        points: 10,
        remaining_attempts: 3,
        rating,
        label: "first".to_string(),
    })
}

/// The message of an update of `score` which moved `column` from `stored` to `written`.
fn change_message(
    score: &DisallowChangeScore,
    column: &str,
    verb: &str,
    stored: impl std::fmt::Display,
    written: impl std::fmt::Display,
    rule: &str,
) -> String {
    format!(
        "Disallowed Value Error while trying to update the row `{{ id : {} }}` in the `disallow_change_score` table because `{column}` would {verb} from `{stored}` to `{written}`, which `#[disallow({rule})]` forbids!",
        score.get_id().value()
    )
}

fn update_refuses_a_decrease<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, 1.5)?;
    let expected_message = change_message(&score, "points", "decrease", 10, 5, "decreasing");
    score.set_points(5);

    expect_disallowed(dsl.update_disallow_change_score_by_id(score), &expected_message)
}

fn update_refuses_an_increase<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, 1.5)?;
    let expected_message = change_message(
        &score,
        "remaining_attempts",
        "increase",
        3,
        4,
        "increasing",
    );
    score.set_remaining_attempts(4);

    expect_disallowed(dsl.update_disallow_change_score_by_id(score), &expected_message)
}

fn update_accepts_the_allowed_changes<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, 1.5)?;
    score.set_points(15);
    score.set_remaining_attempts(2);
    score.set_rating(2.5);

    let mut score = dsl
        .update_disallow_change_score_by_id(score)
        .map_err(|error| {
            format!("Raising points and rating and lowering remaining_attempts should be allowed! Got:\n{error}")
        })?;

    score.set_label("second".to_string());

    dsl.update_disallow_change_score_by_id(score)
        .map_err(|error| {
            format!("An update which changes none of the guarded columns should be allowed! Got:\n{error}")
        })?;

    Ok(())
}

/// A NaN compares unordered with itself, but the stored and the written value are the same
/// bits, so the column did not change.
fn an_unchanged_nan_is_no_change<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, f64::NAN)?;
    score.set_label("second".to_string());

    dsl.update_disallow_change_score_by_id(score)
        .map_err(|error| {
            format!("An update which keeps a NaN rating should be allowed! Got:\n{error}")
        })?;

    Ok(())
}

fn a_change_to_or_from_nan_is_refused<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut from_nan = create_score(dsl, f64::NAN)?;
    let expected_message = change_message(&from_nan, "rating", "decrease", f64::NAN, 1.0, "decreasing");
    from_nan.set_rating(1.0);

    expect_disallowed(dsl.update_disallow_change_score_by_id(from_nan), &expected_message)?;

    let mut to_nan = create_score(dsl, 1.0)?;
    let expected_message = change_message(&to_nan, "rating", "decrease", 1.0, f64::NAN, "decreasing");
    to_nan.set_rating(f64::NAN);

    expect_disallowed(dsl.update_disallow_change_score_by_id(to_nan), &expected_message)
}

/// The insert path of `upsert_disallow_change_season` writes the first row, which no stored
/// row constrains; the update path compares with the stored one.
fn upsert_compares_only_while_the_row_exists<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let mut season = dsl.get_disallow_change_season()?;
    season.set_number(5);
    let mut season = dsl.upsert_disallow_change_season(season)?;

    season.set_number(3);
    expect_disallowed(
        dsl.upsert_disallow_change_season(season.clone()),
        "Disallowed Value Error while trying to update the row `{ id : 0 }` in the `disallow_change_season` table because `number` would decrease from `5` to `3`, which `#[disallow(decreasing)]` forbids!",
    )?;

    season.set_number(6);
    dsl.upsert_disallow_change_season(season)
        .map_err(|error| format!("Raising the season number should be allowed! Got:\n{error}"))?;

    Ok(())
}
```

In `examples/test/src/lib.rs`, add `pub mod disallow_change_test;` before `pub mod disallow_zero_test;`, and append to `TEST_GROUPS`:

```rust
    ("disallow_change_test", disallow_change_test::run_tests),
```

Create three compile tests on this table, with the marked column replaced:

```rust
::spacetimedsl::spacetimedsl!();

pub mod item {
    #[spacetimedsl::dsl(plural_name = items, method(update = true))]
    #[spacetimedb::table(accessor = item, public)]
    pub struct Item {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[disallow(decreasing)] // the column each file varies
        pub name: String,
    }
}

fn main() {}
```

| File | The marked column |
| --- | --- |
| `disallow_decreasing_on_string_column.rs` | `#[disallow(decreasing)]` above `pub name: String,` |
| `disallow_decreasing_and_increasing.rs` | `#[disallow(decreasing, increasing)]` above `pub score: i64,` |
| `disallow_decreasing_on_private_column.rs` | `#[disallow(decreasing)]` above `score: i64,`, followed by a second column `pub name: String,`, so the table keeps its update method |

Write each file out whole, without the trailing comment. Create `compile-tests/tests/ui/disallow_increasing_on_primary_key_column.rs`:

```rust
::spacetimedsl::spacetimedsl!();

pub mod item {
    #[spacetimedsl::dsl(plural_name = items, method(update = true))]
    #[spacetimedb::table(accessor = item, public)]
    pub struct Item {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[disallow(increasing)]
        id: u64,

        pub name: String,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/disallow_decreasing_with_set_zero_strategy.rs`:

```rust
//! `on_delete = SetZero` lowers `warehouse_id` to `0` when its warehouse is deleted, which
//! `#[disallow(decreasing)]` forbids.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so the
//! `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        #[disallow(decreasing)]
        pub warehouse_id: u64,
    }
}

fn main() {}
```

- [ ] **Step 3: Observe the red**

Run the unit gate.
Expected: `FAILED`; `disallow_unknown_rule` still passes, and the five new files fail with *expected `zero`* underlining `decreasing` or `increasing` — the rules do not exist yet, which is the reason under test.

Run the runtime gate.
Expected: `FAILED` with `error: expected `zero`` pointing at `#[disallow(decreasing)]` in `disallow_change_test.rs`.

- [ ] **Step 4: Pin the model in the contract test (the red)**

In `derive/src/data_transfer_contract_tests.rs`, change the attribute of `revision` to `#[disallow(zero, decreasing)]`, and its two assertions to:

```rust
    assert_eq!(
        revision
            .spacetimedsl_column
            .disallowed
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [Disallowed::Zero, Disallowed::Decreasing]
    );
    assert_eq!(
        revision
            .spacetimedsl_column
            .setter
            .as_ref()
            .expect("a public column has a setter")
            .doc_comment,
        "Writing the row through the DSL fails with a *Disallowed Value Error* if this column is `0` or decreases (`#[disallow(zero, decreasing)]`)."
    );
```

In `visit_spacetimedsl_column`, match the three variants:

```rust
    for rule in disallowed {
        match rule {
            Disallowed::Zero | Disallowed::Decreasing | Disallowed::Increasing => {}
        }
    }
```

Run the unit gate.
Expected: `FAILED` with `error[E0599]: no variant or associated item named `Decreasing` found for enum `Disallowed``.

- [ ] **Step 5: Add the two rules and reject the columns they cannot guard**

In `derive-input/src/api/dsl/disallow.rs`, replace the doc of the enum by *A rule of `#[disallow(...)]`: what the value of a column must not be or do.*, and add after `Zero`:

```rust
    /// `decreasing`: a write which makes the value smaller than the stored one.
    Decreasing,
    /// `increasing`: a write which makes the value larger than the stored one.
    Increasing,
```

In `derive-input/src/internal/dsl.rs`, add `symbol!(decreasing);` and `symbol!(increasing);` after `symbol!(zero);`.

In `derive-input/src/internal/dsl/disallow.rs`, import `super::{decreasing, disallow, increasing, zero}`, `api::db::column::SpacetimeDBColumn` and `api::rust::visibility::RustVisibility`, and add to `keyword`:

```rust
            Disallowed::Decreasing => decreasing.0,
            Disallowed::Increasing => increasing.0,
```

and to the `match_meta!` of `parse_rules`:

```rust
            decreasing => Disallowed::Decreasing,
            increasing => Disallowed::Increasing,
```

Give `try_parse` the parameter `spacetimedb_column: &SpacetimeDBColumn` after `rust_field`, and add before its `Ok(disallowed)`:

```rust
    let change_rules: Vec<Disallowed> = disallowed
        .iter()
        .copied()
        .filter(|rule| *rule != Disallowed::Zero)
        .collect();

    if !change_rules.is_empty() {
        reject_change_rules_the_column_cannot_keep(
            field,
            rust_field,
            spacetimedb_column,
            disallow_attribute,
            foreign_key,
            &change_rules,
        )?;
    }
```

Move the `SetZero` test of `reject_zero_the_column_cannot_keep` into a function both rules use:

```rust
/// Whether deleting the referenced row writes `0` or `Uuid::NIL` into the column.
fn is_cleared_by_set_zero(foreign_key: Option<&ForeignKey>) -> bool {
    foreign_key.and_then(|foreign_key| foreign_key.on_delete_strategy.as_ref())
        == Some(&OnDeleteStrategy::SetZero)
}
```

and add:

```rust
/// `decreasing` and `increasing` compare a written value with the stored one, so the column
/// needs an ordered number type and a value the DSL can change.
fn reject_change_rules_the_column_cannot_keep(
    field: &SatsField<'_>,
    rust_field: &RustField,
    spacetimedb_column: &SpacetimeDBColumn,
    disallow_attribute: &Attribute,
    foreign_key: Option<&ForeignKey>,
    change_rules: &[Disallowed],
) -> syn::Result<()> {
    let is_ordered_number = matches!(
        ColumnTypeKind::of(&rust_field.type_name_or_path),
        ColumnTypeKind::UnsignedInteger | ColumnTypeKind::SignedInteger | ColumnTypeKind::Float
    );

    if !is_ordered_number {
        return Err(error::disallow_change_on_unsupported_type(field.ty));
    }

    let [rule] = change_rules else {
        return Err(error::disallow_decreasing_and_increasing(
            disallow_attribute,
            &rust_field.name,
        ));
    };

    if spacetimedb_column.is_primary_key {
        return Err(error::disallow_change_on_primary_key_column(
            disallow_attribute,
            rule.keyword(),
        ));
    }

    if matches!(rust_field.visibility, RustVisibility::Private) {
        return Err(error::disallow_change_on_private_column(
            disallow_attribute,
            rule.keyword(),
            &rust_field.name,
        ));
    }

    if *rule == Disallowed::Decreasing && is_cleared_by_set_zero(foreign_key) {
        return Err(error::disallow_decreasing_with_set_zero_strategy(
            disallow_attribute,
        ));
    }

    Ok(())
}
```

The order puts the rejection whose fix removes the most first: a column with both rules is rejected for the pair before it could be rejected for being the private primary key, and removing both rules fixes it.

In `derive-input/src/internal/error.rs`, add to the `#[disallow]` section:

```rust
pub fn disallow_change_on_unsupported_type(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "`#[disallow(decreasing)]` and `#[disallow(increasing)]` are only allowed on the integer and float columns `u8`–`u128`, `i8`–`i128`, `f32` and `f64`, whose values are ordered! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn disallow_decreasing_and_increasing(
    disallow_attribute: &impl ToTokens,
    column_name: &Ident,
) -> Error {
    Error::new_spanned(
        disallow_attribute,
        format!(
            "`decreasing` and `increasing` together forbid every change of `{column_name}`! Remove both and make the column private (no visibility modifier), so it has no setter and does not change."
        ),
    )
}

pub fn disallow_change_on_primary_key_column(disallow_attribute: &impl ToTokens, rule: &str) -> Error {
    Error::new_spanned(
        disallow_attribute,
        format!(
            "`#[disallow({rule})]` is not allowed on a primary key column, which an update never changes, because it finds the row by it! Remove `{rule}`."
        ),
    )
}

pub fn disallow_change_on_private_column(
    disallow_attribute: &impl ToTokens,
    rule: &str,
    column_name: &Ident,
) -> Error {
    Error::new_spanned(
        disallow_attribute,
        format!(
            "`#[disallow({rule})]` needs a column with a setter, but `{column_name}` is private, so no DSL method changes it! Make the column `pub`, or remove `{rule}`."
        ),
    )
}

pub fn disallow_decreasing_with_set_zero_strategy(disallow_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        disallow_attribute,
        "`#[disallow(decreasing)]` is not allowed together with `on_delete = SetZero`, which lowers this column to `0` when the referenced row is deleted! Remove `decreasing`, or choose another strategy, such as `on_delete = Delete`.",
    )
}
```

In `derive-input/src/internal/dsl/column.rs`, pass `spacetimedb_column` as the new third argument of `disallow::try_parse`.

- [ ] **Step 6: Compare with the stored row**

In `derive-input/src/internal/dsl/method/disallow.rs`, give the update variant its stored row:

```rust
    /// A write over a stored row. `row_key` renders the row's primary key as `{ id : 7 }`, and
    /// `stored_row` is the row as it is stored, which `decreasing` and `increasing` compare
    /// with.
    Update {
        row_key: TokenStream,
        stored_row: TokenStream,
    },
```

and add after `CheckedRules::OfAWrittenRow`:

```rust
    /// `upsert_<table>`: every rule, `decreasing` and `increasing` while the row exists.
    OfAnUpsertedRow,
```

Add:

```rust
/// The change `decreasing` or `increasing` forbids.
#[derive(Clone, Copy)]
enum Change {
    Decrease,
    Increase,
}

/// The change `rule` forbids, `None` for `zero`, which reads the written value alone.
fn forbidden_change(rule: Disallowed) -> Option<Change> {
    match rule {
        Disallowed::Zero => None,
        Disallowed::Decreasing => Some(Change::Decrease),
        Disallowed::Increasing => Some(Change::Increase),
    }
}

/// Whether a write over a stored row has to look that row up for the table's rules: whether
/// a column states `decreasing` or `increasing`.
pub fn compares_with_the_stored_row(internal_columns: &[InternalColumn]) -> bool {
    internal_columns.iter().any(|internal_column| {
        internal_column
            .spacetimedsl_column_disallowed
            .iter()
            .any(|rule| forbidden_change(*rule).is_some())
    })
}

/// The condition under which `row` breaks `decreasing` or `increasing` of `internal_column`
/// against `stored_row`. An integer compares with `<` or `>`. A float compares through
/// `partial_cmp`, so a change to or from NaN breaks both rules, while an unchanged value —
/// the same bits, NaN included — breaks neither.
fn change_check(
    internal_column: &InternalColumn,
    change: Change,
    row: &TokenStream,
    stored_row: &TokenStream,
) -> Check {
    let column_name = &internal_column.rust_field_name;
    let written_value = quote! { #row.#column_name };
    let stored_value = quote! { #stored_row.#column_name };

    let (integer_violation, allowed_orderings, verb) = match change {
        Change::Decrease => (
            quote! { #written_value < #stored_value },
            quote! { ::core::cmp::Ordering::Greater | ::core::cmp::Ordering::Equal },
            "decrease",
        ),
        Change::Increase => (
            quote! { #written_value > #stored_value },
            quote! { ::core::cmp::Ordering::Less | ::core::cmp::Ordering::Equal },
            "increase",
        ),
    };

    let violated = match internal_column.rust_field_type_kind {
        ColumnTypeKind::Float => quote! {
            #written_value.to_bits() != #stored_value.to_bits()
                && !matches!(#written_value.partial_cmp(&#stored_value), Some(#allowed_orderings))
        },
        _ => integer_violation,
    };

    Check {
        violated,
        reason: format!("would {verb} from `{{}}` to `{{}}`"),
        reason_arguments: vec![stored_value, written_value],
    }
}
```

In `checks`, replace the `let check = match rule { … };` by:

```rust
            let check = match (forbidden_change(*rule), write) {
                (None, _) => zero_check(internal_column, row),
                (Some(change), GuardedWrite::Update { stored_row, .. }) => {
                    change_check(internal_column, change, row, stored_row)
                }
                (Some(_), GuardedWrite::Create) => continue,
            };
```

Replace `is_checked` by:

```rust
/// Whether `write` checks `rule` of `internal_column`. `create_<table>` writes the placeholder
/// `0` into an `#[auto_inc]` column, which SpacetimeDB replaces with a value of its sequence,
/// never `0`, and a new row has no stored one to compare with.
fn is_checked(rule: Disallowed, internal_column: &InternalColumn, write: &GuardedWrite) -> bool {
    match (rule, write) {
        (Disallowed::Zero, GuardedWrite::Create) => {
            !internal_column.spacetimedb_column_is_auto_inc
        }
        (Disallowed::Decreasing | Disallowed::Increasing, GuardedWrite::Create) => false,
        (_, GuardedWrite::Update { .. }) => true,
    }
}
```

In `disallowed_value_error`, bind the update variant's key with `GuardedWrite::Update { row_key, .. }`.

In `section`, cover the new variant and give it its lead:

```rust
                .filter(|rule| match checked {
                    CheckedRules::OfANewRow => {
                        is_checked(**rule, internal_column, &GuardedWrite::Create)
                    }
                    CheckedRules::OfAWrittenRow | CheckedRules::OfAnUpsertedRow => true,
                })
```

```rust
    let lead = match checked {
        CheckedRules::OfANewRow | CheckedRules::OfAWrittenRow => {
            "Fails with a *Disallowed Value Error* if a column holds what its `#[disallow]` forbids:"
        }
        CheckedRules::OfAnUpsertedRow => {
            "Fails with a *Disallowed Value Error* if a column holds what its `#[disallow]` forbids, a decrease or an increase only while the row exists:"
        }
    };

    doc::section("Disallowed values", Some(lead), &bullets)
```

Add to `forbidden_value`:

```rust
        Disallowed::Decreasing => "a decrease".to_string(),
        Disallowed::Increasing => "an increase".to_string(),
```

and to the `match` of `setter_doc`:

```rust
            Disallowed::Decreasing => "decreases".to_string(),
            Disallowed::Increasing => "increases".to_string(),
```

In `update.rs`, add `self` to the `reference_integrity::{…}` import. In `for_update`, compute before `let_field_name_for_found_value`:

```rust
    let compares_with_the_stored_row = disallow::compares_with_the_stored_row(internal_columns);
```

add `&& !compares_with_the_stored_row` to the condition of `let_field_name_for_found_value`, and replace the `disallow_checks` of Task 5 by:

```rust
    let look_up_the_stored_row = match compares_with_the_stored_row {
        false => TokenStream::default(),
        true => {
            let look_up = reference_integrity::look_up_the_stored_row(
                singular_table_name,
                field_name_for_found_value,
                primary_key_column,
                is_singleton_pk,
            );
            let stored_row = reference_integrity::stored_row(field_name_for_found_value);

            quote! {
                #look_up
                let stored_row = #stored_row;
            }
        }
    };

    let row = quote! { #singular_table_name };
    let disallow_checks = disallow::checks(
        context,
        &GuardedWrite::Update {
            row_key: disallow::row_key(context, &row),
            stored_row: quote! { stored_row },
        },
        &row,
        disallow::return_the_error,
    );
```

In its `method_impl`, put `#look_up_the_stored_row` right before `#disallow_checks`.

In `upsert.rs`, add a constant for the invariant `keep_created_at_on_update` spells out today, and use it there instead of the literal:

```rust
/// Why the update path may unwrap the row it looked up.
const ROW_FOUND_ON_THE_UPDATE_PATH: &str = "The update path only runs when the row was found";
```

In `for_singleton_upsert`, add:

```rust
    let stored_row_on_update = match disallow::compares_with_the_stored_row(internal_columns) {
        false => TokenStream::default(),
        true => quote! {
            let stored_row = #field_name_for_found_value
                .as_ref()
                .expect(#ROW_FOUND_ON_THE_UPDATE_PATH);
        },
    };
```

give `GuardedWrite::Update` of `disallow_checks_on_update` the field `stored_row: quote! { stored_row }`, put `#stored_row_on_update` right before `#disallow_checks_on_update`, and document the method with `CheckedRules::OfAnUpsertedRow`.

- [ ] **Step 7: Observe the green and read the diagnostics**

Run the unit gate. Expected: the contract test passes; the five new compile tests and `disallow_unknown_rule` fail on their `.stderr`.

Regenerate the compile-test output and read it:

| File | First error | Underlined |
| --- | --- | --- |
| `disallow_unknown_rule` | *expected one of: `zero`, `decreasing`, `increasing`* | `negative` |
| `disallow_decreasing_on_string_column` | *`#[disallow(decreasing)]` and `#[disallow(increasing)]` are only allowed on the integer and float columns …! Found: String* | `String` |
| `disallow_decreasing_and_increasing` | *`decreasing` and `increasing` together forbid every change of `score`! …* | the attribute |
| `disallow_increasing_on_primary_key_column` | *`#[disallow(increasing)]` is not allowed on a primary key column, …* | the attribute |
| `disallow_decreasing_on_private_column` | *`#[disallow(decreasing)]` needs a column with a setter, but `score` is private, …* | the attribute |
| `disallow_decreasing_with_set_zero_strategy` | *`#[disallow(decreasing)]` is not allowed together with `on_delete = SetZero`, …*, then the follow-on errors its comment names | the attribute |

- [ ] **Step 8: Snapshot the comparison**

Create `derive/tests/fixtures/disallow_change.rs`:

```rust
//! Covers `#[disallow(decreasing)]` and `#[disallow(increasing)]` on an unsigned integer, a
//! signed integer and a float, one of them next to `zero`. `update_score_by_id` compares each
//! with the stored row, which it looks up by the primary key itself because no other check
//! does; `update_season_by_id` reuses the row its before hook looked up. The update path of
//! `upsert_league` compares with the row it found, and its insert path compares nothing. A
//! float compares through `partial_cmp`.

#[spacetimedsl::dsl(plural_name = scores, method(update = true))]
#[spacetimedb::table(accessor = score, public)]
pub struct Score {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(zero, decreasing)]
    pub points: u64,

    #[disallow(increasing)]
    pub remaining_attempts: i8,

    #[disallow(decreasing)]
    pub rating: f64,
}

#[spacetimedsl::dsl(plural_name = seasons, method(update = true), hook(before(update)))]
#[spacetimedb::table(accessor = season, public)]
pub struct Season {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(decreasing)]
    pub number: u32,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = league, public)]
pub struct League {
    #[disallow(increasing)]
    pub remaining_rounds: u16,
}
```

Register it after `disallow_zero`. Regenerate the snapshots and read them:

- `disallow_change/Score/update_score_by_id.snap`: `let mut the_same_or_another_score: Option<Score> = None;`, the lookup by `score.get_id().value()` returning a `NotFoundError` for `{ id : … }`, `let stored_row = the_same_or_another_score.as_ref().expect("the stored row is looked up by its primary key before the row to write is compared with it");`, then four checks: `points == 0`, `points < stored_row.points`, `remaining_attempts > stored_row.remaining_attempts` and, for `rating`, `score.rating.to_bits() != stored_row.rating.to_bits() && !matches!(score.rating.partial_cmp(&stored_row.rating), Some(::core::cmp::Ordering::Greater | ::core::cmp::Ordering::Equal))`, each message a `format!` over the key, the stored and the written value.
- `disallow_change/Score/create_score.snap`: only the `zero` check of `points`; the section lists `points: `0``.
- `disallow_change/Season/update_season_by_id.snap`: the hook's prelude fills the binding, the lookup finds it filled, `stored_row` is bound after the hook.
- `disallow_change/League/upsert_league.snap`: `let stored_row = the_same_or_another_league.as_ref().expect("The update path only runs when the row was found");` and the check on the update path only; the section's lead says *a decrease or an increase only while the row exists*.

No other snapshot may change.

- [ ] **Step 9: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 10: Document the rules**

In `docs/DOCUMENTATION.md`, *Disallowed Values*: add to the example struct

```rust
    #[disallow(decreasing)]
    pub experience: u64,

    #[disallow(increasing)]
    pub remaining_lives: u8,
```

add two rows to the table:

```markdown
| `decreasing` | a write which makes the value smaller | `u8`–`u128`, `i8`–`i128`, `f32`, `f64` |
| `increasing` | a write which makes the value larger  | `u8`–`u128`, `i8`–`i128`, `f32`, `f64` |
```

add after the table *Name several rules in one attribute, such as `#[disallow(zero, decreasing)]`.*, and add to the list:

```markdown
- `decreasing` and `increasing` compare the written value with the stored one, so `create_<table>` and the insert path of `upsert_<table>`, which have no stored row, do not check them.
- `f32` and `f64` compare through `partial_cmp`: a change to or from NaN breaks both rules, while an unchanged value, NaN included, breaks neither.
```

Extend the paragraph of rejections by *, `decreasing` or `increasing` on a column that is not an integer or a float, on the primary key, which an update never changes, or on a private column, which has no setter, `decreasing` together with `increasing`, which forbid every change — remove both and make the column private instead — and `decreasing` on a column whose foreign key has `on_delete = SetZero`*, and add to the example messages:

```txt
Disallowed Value Error while trying to update the row `{ id : 7 }` in the `player` table because `experience` would decrease from `10` to `5`, which `#[disallow(decreasing)]` forbids!
```

In `README.md`, replace the bullet *🚫 `#[disallow(zero)]` refuses to write `0` or `Uuid::NIL` into a column, whichever DSL method writes the row.* by:

```markdown
- 🚫 `#[disallow(zero)]`, `#[disallow(decreasing)]` and `#[disallow(increasing)]` refuse a forbidden value or change, whichever DSL method writes the row.
```

In `docs/MIGRATION.md`, in the entry *`SpacetimeDSLColumn::disallowed`*, name the variants: *…from the new `api::dsl::disallow::Disallowed` (`Zero`, `Decreasing`, `Increasing`).*

- [ ] **Step 11: Format, lint, commit**

Commit message:

```text
Refuse lower or higher values with #[disallow(decreasing)] and #[disallow(increasing)]

update_<table>_by_<key> and the update path of upsert_<table> compare a column with
#[disallow(decreasing)] or #[disallow(increasing)] with the stored row, which they look up
by the primary key when no other check did, after the before hook. A float compares
through partial_cmp, so a change to or from NaN is refused and an unchanged value never
is. The rules are rejected on non-number types, together, on the primary key, on a
private column, and decreasing next to on_delete = SetZero. The stored-row lookup moved
out of the foreign key check, and its expect text now names both of its users.

Tests: contract assertions for Disallowed::Decreasing; the fixture disallow_change; five
compile tests; the runtime group disallow_change_test.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 7: `#[disallow]` in soft deletions and cascades (#186, part 3)

**Files:**
- Modify: `derive-input/src/internal/dsl/method/disallow.rs` (`GuardedWrite::SoftDelete`, `stop_the_cascade`)
- Modify: `derive-input/src/internal/dsl/method/removal.rs` (`retire_row`, the documentation of the soft-delete methods)
- Modify: `derive-input/src/internal/dsl/method/on_delete_strategy.rs` (the `SetZero` and `SoftDelete` arms)
- Modify: `derive-input/src/internal/dsl/method/naming.rs` (doc of `cascade_binding::error_from_hook`)
- Modify: `src/delete.rs` (`Display for DeletionResult`, two doc comments)
- Create: `derive/tests/fixtures/disallow_soft_delete_and_cascade.rs`; register it
- Create: `examples/test/src/disallow_soft_delete_and_cascade_test.rs`; register it
- Modify: `docs/DOCUMENTATION.md` (*DeletionResult*, *During a Cascading Delete*, *Example Error Messages*, *Disallowed Values*), `docs/MIGRATION.md`
- Recorded output: the new `derive/tests/snapshots/disallow_soft_delete_and_cascade/**`; `every_field_attribute/Gadget/soft_delete_gadget_by_id.snap` and `soft_delete_gadgets_by_owner_id.snap`

**Interfaces:**
- Consumes: `method::disallow` of Tasks 5 and 6.
- Produces: `GuardedWrite::SoftDelete { row_key: TokenStream, stored_row: TokenStream }`; `method::disallow::stop_the_cascade(error: &TokenStream) -> TokenStream`; `retire_row(row: &Ident, context: &MethodGenerationContext) -> TokenStream` (private to `removal.rs`).

- [ ] **Step 1: Write the failing runtime group**

Create `examples/test/src/disallow_soft_delete_and_cascade_test.rs`:

```rust
//! `#[disallow(...)]` in the writes which are neither a create nor an update: a soft deletion,
//! and the rows an `on_delete = SetZero` or an `on_soft_delete = SoftDelete` cascade writes.
//!
//! The before hooks of `disallow_member` and `disallow_guest` lower `experience` when the row
//! asks for it, which `#[disallow(decreasing)]` forbids. `soft_delete_*` then fails with the
//! rule's error, and a cascade stops with it as the error which stopped the cascade. Badges
//! reference members, so a member's soft deletion cascades further; nothing references a
//! guest.

use crate::{disallow_zero_test::expect_disallowed, spacetimedsl::prelude::*};

#[spacetimedsl::dsl(
    plural_name = disallow_guilds,
    method(update = false, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = disallow_guild)]
pub struct DisallowGuild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(DisallowGuildId)]
    #[referenced_by(path = crate::disallow_soft_delete_and_cascade_test, table = disallow_member)]
    #[referenced_by(path = crate::disallow_soft_delete_and_cascade_test, table = disallow_guest)]
    id: u64,

    deleted: bool,
}

#[spacetimedsl::dsl(
    plural_name = disallow_members,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete))
)]
#[spacetimedb::table(accessor = disallow_member)]
pub struct DisallowMember {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(DisallowMemberId)]
    #[referenced_by(path = crate::disallow_soft_delete_and_cascade_test, table = disallow_badge)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DisallowGuildId)]
    #[foreign_key(
        path = crate::disallow_soft_delete_and_cascade_test,
        table = disallow_guild,
        column = id,
        on_delete = SetZero,
        on_soft_delete = SoftDelete
    )]
    pub guild_id: u64,

    #[disallow(decreasing)]
    pub experience: u32,

    /// Whether the before hooks lower `experience`.
    lowers_experience: bool,

    deleted: bool,
}

#[spacetimedsl::dsl(plural_name = disallow_badges, method(update = false, delete = true))]
#[spacetimedb::table(accessor = disallow_badge)]
pub struct DisallowBadge {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DisallowMemberId)]
    #[foreign_key(
        path = crate::disallow_soft_delete_and_cascade_test,
        table = disallow_member,
        column = id,
        on_soft_delete = Ignore
    )]
    member_id: u64,
}

#[spacetimedsl::dsl(
    plural_name = disallow_guests,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete))
)]
#[spacetimedb::table(accessor = disallow_guest)]
pub struct DisallowGuest {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DisallowGuildId)]
    #[foreign_key(
        path = crate::disallow_soft_delete_and_cascade_test,
        table = disallow_guild,
        column = id,
        on_delete = SetZero,
        on_soft_delete = SoftDelete
    )]
    pub guild_id: u64,

    #[disallow(decreasing)]
    pub experience: u32,

    /// Whether the before hooks lower `experience`.
    lowers_experience: bool,

    deleted: bool,
}

#[spacetimedsl::hook]
fn before_disallow_member_update(
    _dsl: &DSL<'_, T>,
    old_member: &DisallowMember,
    mut new_member: DisallowMember,
) -> Result<DisallowMember, SpacetimeDSLError> {
    if *old_member.get_lowers_experience() {
        new_member.set_experience(*old_member.get_experience() - 1);
    }

    Ok(new_member)
}

#[spacetimedsl::hook]
fn before_disallow_member_soft_delete(
    _dsl: &DSL<'_, T>,
    old_member: &DisallowMember,
    mut new_member: DisallowMember,
) -> Result<DisallowMember, SpacetimeDSLError> {
    if *old_member.get_lowers_experience() {
        new_member.set_experience(*old_member.get_experience() - 1);
    }

    Ok(new_member)
}

#[spacetimedsl::hook]
fn before_disallow_guest_update(
    _dsl: &DSL<'_, T>,
    old_guest: &DisallowGuest,
    mut new_guest: DisallowGuest,
) -> Result<DisallowGuest, SpacetimeDSLError> {
    if *old_guest.get_lowers_experience() {
        new_guest.set_experience(*old_guest.get_experience() - 1);
    }

    Ok(new_guest)
}

#[spacetimedsl::hook]
fn before_disallow_guest_soft_delete(
    _dsl: &DSL<'_, T>,
    old_guest: &DisallowGuest,
    mut new_guest: DisallowGuest,
) -> Result<DisallowGuest, SpacetimeDSLError> {
    if *old_guest.get_lowers_experience() {
        new_guest.set_experience(*old_guest.get_experience() - 1);
    }

    Ok(new_guest)
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    soft_delete_refuses_what_its_hook_breaks(dsl)?;
    set_zero_cascade_stops_at_a_broken_rule(dsl)?;
    soft_delete_cascade_stops_at_a_broken_rule(dsl)?;
    writes_which_keep_the_rules_pass(dsl)?;

    Ok(())
}

fn create_member<T: WriteContext>(
    dsl: &DSL<'_, T>,
    guild: &DisallowGuild,
    lowers_experience: bool,
) -> Result<DisallowMember, SpacetimeDSLError> {
    dsl.create_disallow_member(CreateDisallowMember {
        guild_id: guild.get_id(),
        experience: 10,
        lowers_experience,
    })
}

fn create_guest<T: WriteContext>(
    dsl: &DSL<'_, T>,
    guild: &DisallowGuild,
    lowers_experience: bool,
) -> Result<DisallowGuest, SpacetimeDSLError> {
    dsl.create_disallow_guest(CreateDisallowGuest {
        guild_id: guild.get_id(),
        experience: 10,
        lowers_experience,
    })
}

/// The message of the broken rule, written by `write` into the row `row_id` of `table`.
fn decrease_message(write: &str, row_id: u64, table: &str) -> String {
    format!(
        "Disallowed Value Error while trying to {write} the row `{{ id : {row_id} }}` in the `{table}` table because `experience` would decrease from `10` to `9`, which `#[disallow(decreasing)]` forbids!"
    )
}

/// Checks that `removal` failed with an error which contains `expected_part`.
fn expect_failure_containing(
    removal: Result<DeletionResult, SpacetimeDSLError>,
    expected_part: &str,
) -> Result<(), String> {
    match removal {
        Ok(deletion_result) => Err(format!(
            "The removal should have failed with \"{expected_part}\"! Got:\n{deletion_result}"
        )),
        Err(error) if error.to_string().contains(expected_part) => Ok(()),
        Err(error) => Err(format!(
            "The error should contain \"{expected_part}\"! Got:\n{error}"
        )),
    }
}

fn soft_delete_refuses_what_its_hook_breaks<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let guild = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild, true)?;

    expect_disallowed(
        dsl.soft_delete_disallow_member_by_id(&member),
        &decrease_message("soft delete", member.get_id().value(), "disallow_member"),
    )
}

/// Deleting a guild clears `guild_id` of its members, an update of each member row.
fn set_zero_cascade_stops_at_a_broken_rule<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let guild = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild, true)?;

    expect_failure_containing(
        dsl.delete_disallow_guild_by_id(&guild),
        &format!(
            "Error which stopped the cascade: {}",
            decrease_message("update", member.get_id().value(), "disallow_member")
        ),
    )
}

/// Retiring a guild retires its members and guests; a member's retirement cascades further,
/// a guest's does not.
fn soft_delete_cascade_stops_at_a_broken_rule<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let guild_of_a_member = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild_of_a_member, true)?;

    expect_failure_containing(
        dsl.soft_delete_disallow_guild_by_id(&guild_of_a_member),
        &format!(
            "Error which stopped the cascade: {}",
            decrease_message("soft delete", member.get_id().value(), "disallow_member")
        ),
    )?;

    let guild_of_a_guest = dsl.create_disallow_guild()?;
    let guest = create_guest(dsl, &guild_of_a_guest, true)?;

    expect_failure_containing(
        dsl.soft_delete_disallow_guild_by_id(&guild_of_a_guest),
        &format!(
            "Error which stopped the cascade: {}",
            decrease_message("soft delete", guest.get_id().value(), "disallow_guest")
        ),
    )
}

fn writes_which_keep_the_rules_pass<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let guild = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild, false)?;
    dsl.soft_delete_disallow_member_by_id(&member)
        .map_err(|error| format!("A soft deletion which keeps the rules should pass! Got:\n{error}"))?;

    let deleted_guild = dsl.create_disallow_guild()?;
    create_member(dsl, &deleted_guild, false)?;
    dsl.delete_disallow_guild_by_id(&deleted_guild)
        .map_err(|error| format!("A SetZero cascade which keeps the rules should pass! Got:\n{error}"))?;

    let retired_guild = dsl.create_disallow_guild()?;
    create_guest(dsl, &retired_guild, false)?;
    dsl.soft_delete_disallow_guild_by_id(&retired_guild)
        .map_err(|error| format!("A SoftDelete cascade which keeps the rules should pass! Got:\n{error}"))?;

    Ok(())
}
```

In `examples/test/src/lib.rs`, add `pub mod disallow_soft_delete_and_cascade_test;` after `pub mod disallow_change_test;`, and append to `TEST_GROUPS`:

```rust
    (
        "disallow_soft_delete_and_cascade_test",
        disallow_soft_delete_and_cascade_test::run_tests,
    ),
```

- [ ] **Step 2: Observe the red**

Run the runtime gate.
Expected: `FAILED`; the `tester` reducer reports `disallow_soft_delete_and_cascade_test: The write should fail with "Disallowed Value Error while trying to soft delete the row …"! Got: Ok(DeletionResult { … })`: soft deletions do not check the rules yet.

- [ ] **Step 3: Check the rules in a soft deletion**

In `derive-input/src/internal/dsl/method/disallow.rs`, import `super::naming::cascade_binding`, and add the variant:

```rust
    /// A soft deletion of a stored row, named and compared like an update.
    SoftDelete {
        row_key: TokenStream,
        stored_row: TokenStream,
    },
```

Extend the arms that read a stored row: in `checks`, `(Some(change), GuardedWrite::Update { stored_row, .. } | GuardedWrite::SoftDelete { stored_row, .. })`; in `is_checked`, `(_, GuardedWrite::Update { .. } | GuardedWrite::SoftDelete { .. }) => true`; in `disallowed_value_error`:

```rust
        GuardedWrite::SoftDelete { row_key, .. } => {
            ("soft delete the row `{}` in", vec![row_key.clone()])
        }
```

Add:

```rust
/// `error = true; error_from_hook = Some(Box::new(<error>)); break 'outer;`, how a cascade
/// stops on a broken rule: the way it stops on an error a hook raised.
pub fn stop_the_cascade(error: &TokenStream) -> TokenStream {
    let error_flag = cascade_binding::error();
    let error_from_hook = cascade_binding::error_from_hook();
    let outer = cascade_binding::outer();

    quote! {
        #error_flag = true;
        #error_from_hook = Some(Box::new(#error));
        break #outer;
    }
}
```

In `derive-input/src/internal/dsl/method/removal.rs`, add `context::MethodGenerationContext` and `disallow::{self, CheckedRules, GuardedWrite}` to the `super::{…}` import. Replace the signature of `retire_row` by `fn retire_row(row: &syn::Ident, context: &MethodGenerationContext) -> TokenStream`, starting its body with:

```rust
    let MethodGenerationContext {
        spacetimedsl_table,
        singular_table_name,
        primary_key_column_name,
        ..
    } = context;
```

Build the checks after `let new_row = format_ident!("new_row");`:

```rust
    let disallow_checks = disallow::checks(
        context,
        &GuardedWrite::SoftDelete {
            row_key: disallow::row_key(context, &quote! { #new_row }),
            stored_row: quote! { #row },
        },
        &quote! { #new_row },
        disallow::return_the_error,
    );
```

and emit them right after the before hook:

```rust
    quote! {
        #clone_row

        #before_hook

        #disallow_checks

        #rebind_row

        #set_marker

        #store_row

        #after_hook
    }
```

In `removal_method`, destructure `internal_columns` from the context too, call `retire_row(&old_row, context)`, and replace the final `let doc_comment = doc::with_section(…);` by:

```rust
    let doc_comment = doc::paragraphs([
        doc_comment,
        match removal {
            Removal::Hard => String::new(),
            Removal::Soft => disallow::section(internal_columns, CheckedRules::OfAWrittenRow),
        },
        relationship_doc::cascade(removal, &spacetimedsl_table.referencing_tables),
    ]);
```

- [ ] **Step 4: Check the rules in the cascades**

In `derive-input/src/internal/dsl/method/on_delete_strategy.rs`, add `disallow::{self, GuardedWrite}` to the `super::{…}` import, and compute after `let is_singleton = …;`:

```rust
    let compares_with_the_stored_row = disallow::compares_with_the_stored_row(internal_columns);
```

In the `SetZero` arm, replace the comment and the binding `clone_old_row` by:

```rust
                // The hooks and the change rules see the row as it was before the column was
                // cleared.
                let clone_old_row = match before_update_hook.is_empty()
                    && after_update_hook.is_empty()
                    && !compares_with_the_stored_row
                {
                    true => TokenStream::default(),
                    false => quote! { let old_row = row.clone(); },
                };

                let disallow_checks = disallow::checks(
                    context,
                    &GuardedWrite::Update {
                        row_key: disallow::row_key(context, &quote! { row }),
                        stored_row: quote! { old_row },
                    },
                    &quote! { row },
                    disallow::stop_the_cascade,
                );
```

and emit `#disallow_checks` right after `#before_update_hook` in its per-row body.

In the `SoftDelete` arm, build the checks once, after `let set_marker = …;`:

```rust
                let disallow_checks = disallow::checks(
                    context,
                    &GuardedWrite::SoftDelete {
                        row_key: disallow::row_key(context, &quote! { row }),
                        stored_row: quote! { old_row },
                    },
                    &quote! { row },
                    disallow::stop_the_cascade,
                );
```

In its `ReferencingTables::Absent` branch, clone the row for the change rules too:

```rust
                        let clone_old_row = match a_hook_runs || compares_with_the_stored_row {
                            true => quote! { let old_row = row.clone(); },
                            false => TokenStream::default(),
                        };
```

and emit `#disallow_checks` right after `#before_soft_delete_hook` in both branches — in the `Present` branch inside `soft_delete_many_impl`, where `old_row` is the recorded row.

In `derive-input/src/internal/dsl/method/naming.rs`, replace the doc of `cascade_binding::error_from_hook` by:

```rust
    /// The error which stopped the cascade, if one did: one a hook raised, or a broken
    /// `#[disallow]` rule of a row the cascade wrote.
```

- [ ] **Step 5: Name the error which stopped the cascade**

In `src/delete.rs`, replace the doc of `OnDeleteStrategyFailure` by:

```rust
/// What a cascade returns when it refused: the entries it built before it stopped, and the
/// error which stopped it, if one did.
```

the doc of `DeletionResult::error_from_hook` by:

```rust
    /// The error which stopped the cascade, if one did: one a hook of a referencing table
    /// raised, or a `#[disallow]` rule which a row the cascade wrote broke.
```

keeping its paragraph about the `Box`, and in `Display for DeletionResult` replace `"Error from a hook: {error_from_hook}\n\n{}"` by `"Error which stopped the cascade: {error_from_hook}\n\n{}"`.

- [ ] **Step 6: Snapshot the checks**

Create `derive/tests/fixtures/disallow_soft_delete_and_cascade.rs`:

```rust
//! Covers `#[disallow(...)]` in the writes which are neither a create nor an update:
//! `soft_delete_member_by_id` and `soft_delete_members_by_guild_id` check the row their before
//! hook hands back, and the cascades of `member` and `guest` check each row they write — the
//! `SetZero` arm as an update, the `SoftDelete` arm as a soft deletion — in the shape of a
//! table other tables reference (`member`, referenced by `badge`) and of one nothing
//! references (`guest`). `guest` has no hooks, so only its change rule makes the cascade keep
//! the stored row. A broken rule stops a cascade the way an error of a hook does.

#[spacetimedsl::dsl(
    plural_name = guilds,
    method(update = false, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = guild, public)]
pub struct Guild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = member)]
    #[referenced_by(path = self, table = guest)]
    id: u64,

    deleted: bool,
}

#[spacetimedsl::dsl(
    plural_name = members,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete))
)]
#[spacetimedb::table(accessor = member, public)]
pub struct Member {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = badge)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(GuildId)]
    #[foreign_key(path = self, table = guild, column = id, on_delete = SetZero, on_soft_delete = SoftDelete)]
    pub guild_id: u64,

    #[disallow(zero, decreasing)]
    pub experience: u32,

    deleted: bool,
}

#[spacetimedsl::dsl(plural_name = badges, method(update = false, delete = true))]
#[spacetimedb::table(accessor = badge, public)]
pub struct Badge {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(MemberId)]
    #[foreign_key(path = self, table = member, column = id, on_soft_delete = Ignore)]
    member_id: u64,
}

#[spacetimedsl::dsl(
    plural_name = guests,
    method(update = true, delete = false, soft_delete = true)
)]
#[spacetimedb::table(accessor = guest, public)]
pub struct Guest {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(GuildId)]
    #[foreign_key(path = self, table = guild, column = id, on_delete = SetZero, on_soft_delete = SoftDelete)]
    pub guild_id: u64,

    #[disallow(increasing)]
    pub remaining_visits: u8,

    deleted: bool,
}
```

Register it after `disallow_change`. Regenerate the snapshots and read them:

- `Member/soft_delete_member_by_id.snap` and `soft_delete_members_by_guild_id.snap`: right after the `before_member_soft_delete` call, `if new_row.experience == 0 { return Err(…) }` and `if new_row.experience < old_row.experience { return Err(…) }`, each message starting *Disallowed Value Error while trying to soft delete the row `{}` in the `member` table* with `format!("{{ id : {} }}", new_row.id)`; both methods document the rules under *Disallowed values*.
- `Member/internal_methods.snap`: in the `SetZero` arm, the two checks after `before_member_update`, comparing `row` with `old_row`, each ending in `error = true; error_from_hook = Some(Box::new(…)); break 'outer;`, the message saying *update the row*; in the `SoftDelete` arm, which cascades further through `badge`, the same checks after `before_member_soft_delete`, the message saying *soft delete the row*.
- `Guest/internal_methods.snap`: `let old_row = row.clone();` in the `SetZero` and in the `SoftDelete` arm although `guest` has no hooks, followed by the check `row.remaining_visits > old_row.remaining_visits`.
- `every_field_attribute/Gadget/soft_delete_gadget_by_id.snap` and `soft_delete_gadgets_by_owner_id.snap`: the check of `rating` after the soft-delete write's clone of the row, and the section.

No other snapshot may change.

- [ ] **Step 7: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`. `cascade_hook_error_test` still passes: it looks for the hook's message, not for the prefix.

- [ ] **Step 8: Document the change**

In `docs/DOCUMENTATION.md`:

*DeletionResult*: replace the comment above `error_from_hook` by

```rust
    // The error which stopped the cascade: one a delete hook raised, or a broken
    // `#[disallow]` rule of a row the cascade wrote. Boxed because
    // `SpacetimeDSLError::ReferenceIntegrityViolation` holds a `DeletionResult`, so an
    // unboxed field would make both types infinitely sized.
```

and in the paragraph below the CSV, `Error from a hook: <error>` by `Error which stopped the cascade: <error>`.

*During a Cascading Delete*: replace `Error from a hook: this lock holder is locked` by `Error which stopped the cascade: this lock holder is locked` in the example, and add after the paragraph about the hook's own error: *A `#[disallow]` rule which a row written by the cascade breaks stops it the same way; see [Disallowed Values](#disallowed-values).*

*Example Error Messages*, *Delete with a hook that refused during the cascade*: replace `Error from a hook:` by `Error which stopped the cascade:`.

*Disallowed Values*: replace the first bullet by:

```markdown
- Every write checks the rules after its before hook: `create_<table>`, `update_<table>_by_<key>`, both paths of `upsert_<table>`, `soft_delete_*`, and each row an `on_delete = SetZero` or `SoftDelete` cascade writes. So a hook may repair a value, and a value a hook writes is checked as well.
- In a cascade, a broken rule stops the cascade like an error a hook raised: the delete or soft-delete method fails, and the `DeletionResult` in its error carries the rule's error in `error_from_hook`.
```

In `docs/MIGRATION.md`, add under *Changed messages and generated code*:

```markdown
#### A `DeletionResult` prints *Error which stopped the cascade*

`Display` of a `DeletionResult` whose `error_from_hook` is `Some` starts with *Error which stopped the cascade:* instead of *Error from a hook:*, because a `#[disallow]` rule which a row written by the cascade breaks stops the cascade the same way. The field keeps its name.
```

- [ ] **Step 9: Format, lint, commit**

Commit message:

```text
Check #[disallow] in soft deletions and cascades

soft_delete_<table>_by_<index> checks the row its before hook hands back, and the SetZero
and SoftDelete arms of a cascade check every row they write, the former as an update, the
latter as a soft deletion. A broken rule stops a cascade like an error a hook raised; the
DeletionResult carries it in error_from_hook, and its Display now says "Error which
stopped the cascade".

Tests: the fixture disallow_soft_delete_and_cascade; the runtime group
disallow_soft_delete_and_cascade_test.

Closes #186

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 8: Wrapper-type methods which look up the referenced row (#190, part 1)

**Files:**
- Modify: `derive-input/src/api/dsl/table.rs` (field), `derive-input/src/api/dsl/wrapper.rs` (doc of `WrapperMethod`)
- Modify: `derive-input/src/internal/spacetimedb.rs` (`table_row_type`)
- Modify: `derive-input/src/internal/dsl/method/wrapper_method.rs` (`for_referenced_row_methods`, `column_stem`), `derive-input/src/internal/dsl/method.rs`
- Modify: `derive/src/output.rs` (`DSLAttributePass`), `derive/src/lib.rs`
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Create: `derive/tests/fixtures/referenced_row_methods.rs`; register it
- Create: `examples/test/src/referenced_row_method_test.rs`; register it
- Modify: `docs/DOCUMENTATION.md` (new *Look Up the Referenced Row From a Wrapper*), `README.md`, `docs/MIGRATION.md`, `CODE_QUALITY_REPORT.md`
- Recorded output: every `wrapper_methods.snap` of a table with a foreign key column besides its primary key — 27 files, among them those of `Transfer`, `Loan`, `Pallet`, `Member`, `Badge` and `Guest` from Tasks 2–7; `compile-tests/tests/ui/referenced_by_repeated.stderr`; the new `derive/tests/snapshots/referenced_row_methods/**`

**Interfaces:**
- Produces: `SpacetimeDSLTableMethods::referenced_row_methods: Vec<WrapperMethod>`; `wrapper_method::for_referenced_row_methods(context: &MethodGenerationContext, foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>) -> Vec<WrapperMethod>` (Task 10 makes it return `syn::Result`); `internal::spacetimedb::table_row_type(module_path: &Path, table_accessor: &Ident) -> TokenStream`; `output::DSLAttributePass { Only, First, Later }` with `of(is_first: bool, is_last: bool)`, replacing the parameter `first_dsl_attribute: bool` of `output::build`.

The method follows a foreign key from the referencing row, so it is added to the wrapper type which names that row, the one of its table's primary key. `#[foreign_key]` names the referenced table by its accessor, not by its struct, and the method has to spell the struct in its return type. SpacetimeDB 2.10.1 generates `<accessor>__TableHandle` next to every table, with `impl Table for <accessor>__TableHandle { type Row = <struct>; }`, so the return type is `<path::<accessor>__TableHandle as ::spacetimedb::Table>::Row`, and the runtime gate breaks the day SpacetimeDB renames it.

- [ ] **Step 1: Write the failing runtime group**

Create `examples/test/src/referenced_row_method_test.rs`:

```rust
//! The methods a table adds to the wrapper type of its primary key, which look up the row a
//! foreign key column of the row with that key references: `alliance.get_server_id()
//! .get_lookup_season(dsl)` follows `lookup_server.season_id` without naming the server row.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = lookup_seasons, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_season)]
pub struct LookupSeason {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_server)]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_tournament)]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_active_membership)]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_expired_membership)]
    id: u64,

    max_alliance_level: u32,
}

#[spacetimedsl::dsl(plural_name = lookup_servers, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_server)]
pub struct LookupServer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_alliance)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    season_id: u64,
}

#[spacetimedsl::dsl(plural_name = lookup_alliances, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_alliance)]
pub struct LookupAlliance {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupServerId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_server, column = id)]
    server_id: u64,
}

/// A tournament runs from one season into another, so it references `lookup_season` twice,
/// and each of its methods takes the name of its column.
#[spacetimedsl::dsl(plural_name = lookup_tournaments, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_tournament)]
pub struct LookupTournament {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    opening_season_id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    closing_season_id: u64,
}

/// A chain of stages, each naming the one after it through a unique foreign key to its own
/// table: `LookupStageId` gets `get_lookup_stage_by_next_stage_id`, the stage before, and
/// `get_next_stage`, the stage after.
#[spacetimedsl::dsl(plural_name = lookup_stages, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_stage)]
pub struct LookupStage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_stage)]
    id: u64,

    #[unique]
    #[use_wrapper(LookupStageId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_stage, column = id)]
    next_stage_id: u64,
}

/// A membership is active or expired, one table each. `LookupMembershipId` names a row in
/// both, so the struct adds no method for the referenced row; it compiles only while neither
/// of its two expansions adds one.
#[spacetimedsl::dsl(
    plural_name = lookup_active_memberships,
    table = lookup_active_membership,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = lookup_active_membership)]
#[spacetimedsl::dsl(
    plural_name = lookup_expired_memberships,
    table = lookup_expired_membership,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = lookup_expired_membership)]
pub struct LookupMembership {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    season_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    a_chain_of_foreign_keys_is_followed_from_a_wrapper(dsl)?;
    a_column_referencing_no_row_finds_no_row(dsl)?;
    each_of_several_foreign_keys_to_one_table_is_followed(dsl)?;
    a_unique_foreign_key_to_its_own_table_is_followed_both_ways(dsl)?;
    a_struct_with_several_tables_keeps_its_methods_for_referencing_rows(dsl)?;

    Ok(())
}

fn a_chain_of_foreign_keys_is_followed_from_a_wrapper<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let season = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 30,
    })?;
    let server = dsl.create_lookup_server(CreateLookupServer {
        season_id: season.get_id(),
    })?;
    let alliance = dsl.create_lookup_alliance(CreateLookupAlliance {
        server_id: server.get_id(),
    })?;

    let max_alliance_level = *alliance
        .get_server_id()
        .get_lookup_season(dsl)?
        .get_max_alliance_level();

    if max_alliance_level != 30 {
        return Err(format!(
            "LookupServerId::get_lookup_season should find the season the server references! Got a max_alliance_level of {max_alliance_level}"
        ));
    }

    if alliance.get_id().get_lookup_server(dsl)?.get_id() != server.get_id() {
        return Err(
            "LookupAllianceId::get_lookup_server should find the server the alliance references!"
                .to_string(),
        );
    }

    Ok(())
}

/// `0` references no row, so the lookup finds none.
fn a_column_referencing_no_row_finds_no_row<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let alliance = dsl.create_lookup_alliance(CreateLookupAlliance {
        server_id: LookupServerId::new(0),
    })?;

    match alliance.get_id().get_lookup_server(dsl) {
        Err(SpacetimeDSLError::NotFoundError { table_name, .. })
            if &*table_name == "lookup_server" =>
        {
            Ok(())
        }
        other => Err(format!(
            "An alliance whose server_id is 0 should find no server! Got: {other:?}"
        )),
    }
}

fn each_of_several_foreign_keys_to_one_table_is_followed<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let opening = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 10,
    })?;
    let closing = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 20,
    })?;
    let tournament = dsl.create_lookup_tournament(CreateLookupTournament {
        opening_season_id: opening.get_id(),
        closing_season_id: closing.get_id(),
    })?;

    if tournament.get_id().get_opening_season(dsl)?.get_id() != opening.get_id() {
        return Err(
            "LookupTournamentId::get_opening_season should find the season opening_season_id references!"
                .to_string(),
        );
    }

    if tournament.get_id().get_closing_season(dsl)?.get_id() != closing.get_id() {
        return Err(
            "LookupTournamentId::get_closing_season should find the season closing_season_id references!"
                .to_string(),
        );
    }

    Ok(())
}

fn a_unique_foreign_key_to_its_own_table_is_followed_both_ways<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let last = dsl.create_lookup_stage(CreateLookupStage {
        next_stage_id: LookupStageId::new(0),
    })?;
    let middle = dsl.create_lookup_stage(CreateLookupStage {
        next_stage_id: last.get_id(),
    })?;
    let first = dsl.create_lookup_stage(CreateLookupStage {
        next_stage_id: middle.get_id(),
    })?;

    if first.get_id().get_next_stage(dsl)?.get_id() != middle.get_id() {
        return Err(
            "LookupStageId::get_next_stage should find the stage the first one names!".to_string(),
        );
    }

    if middle
        .get_id()
        .get_lookup_stage_by_next_stage_id(dsl)?
        .get_id()
        != first.get_id()
    {
        return Err(
            "LookupStageId::get_lookup_stage_by_next_stage_id should find the stage which names the middle one!"
                .to_string(),
        );
    }

    Ok(())
}

fn a_struct_with_several_tables_keeps_its_methods_for_referencing_rows<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let season = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 5,
    })?;
    dsl.create_lookup_active_membership(CreateLookupActiveMembership {
        season_id: season.get_id(),
    })?;

    if season.get_id().get_lookup_active_memberships(dsl).len() != 1 {
        return Err(
            "LookupSeasonId::get_lookup_active_memberships should find the one active membership!"
                .to_string(),
        );
    }

    Ok(())
}
```

In `examples/test/src/lib.rs`, add `pub mod referenced_row_method_test;` after `pub mod reference_integrity_message_test;`, and append to `TEST_GROUPS`:

```rust
    (
        "referenced_row_method_test",
        referenced_row_method_test::run_tests,
    ),
```

- [ ] **Step 2: Observe the red**

Run the runtime gate.
Expected: `FAILED` with `error[E0599]: no method named `get_lookup_season` found for struct `LookupServerId` in the current scope`, and the same for the other methods of Step 1.

- [ ] **Step 3: Pin the model in the contract test (the red)**

In `derive/src/data_transfer_contract_tests.rs`, after the assertion on `wrapper_method.method_name`, add:

```rust
    let [referenced_row_method] = spacetimedsl_methods.referenced_row_methods.as_slice() else {
        panic!("the one foreign key column besides the primary key adds one method to the primary key's wrapper type");
    };
    assert_eq!(referenced_row_method.method_name, "get_owner");
    assert_eq!(
        referenced_row_method.wrapper_type.to_token_stream().to_string(),
        "GadgetId"
    );
```

In `visit_spacetimedsl_table_methods`, add `referenced_row_methods,` to the destructuring, and after the loop over `wrapper_methods`:

```rust
    for WrapperMethod {
        wrapper_type: _,
        doc_comment: _,
        method_name: _,
        return_type: _,
        method_impl: _,
    } in referenced_row_methods
    {}
```

Run the unit gate.
Expected: `FAILED` with `error[E0026]: struct `SpacetimeDSLTableMethods` does not have a field named `referenced_row_methods``.

- [ ] **Step 4: Add the methods to the public model**

In `derive-input/src/api/dsl/table.rs`, add to `SpacetimeDSLTableMethods` after `wrapper_methods`:

```rust
    /// Methods this table adds to the wrapper type of its primary key, one per foreign key
    /// column besides the primary key, which look up the row that column of the row with a key
    /// references, such as `server_id.get_season(&dsl)`.
    ///
    /// Every table of a struct shares the struct's wrapper types, so the key names a row in
    /// each of them and the lookup would be ambiguous: `spacetimedsl_derive` emits these methods
    /// only for a struct with a single `#[dsl]` attribute. Empty for a singleton, whose
    /// injected primary key has no wrapper type.
    pub referenced_row_methods: Vec<WrapperMethod>,
```

In `derive-input/src/api/dsl/wrapper.rs`, replace the doc of `WrapperMethod` by:

```rust
/// A method a table adds to a wrapper type, which looks rows up through a foreign key: the
/// rows which reference one value of the wrapper type of a foreign key column, like
/// `entity_id.get_position(&dsl)`, or the row a foreign key column of the row with one value
/// of the primary key's wrapper type references, like `position_id.get_entity(&dsl)`.
///
/// `method_impl` reads the DSL from the argument `dsl`, which the code that renders this
/// method declares.
```

- [ ] **Step 5: Emit them for a struct with a single `#[dsl]`**

In `derive/src/output.rs`, import `dsl::wrapper::WrapperMethod` next to `WrapperType`, and add before `build`:

```rust
/// Which of the struct's `#[dsl]` attributes an expansion is.
///
/// Every table of a struct shares the struct's wrapper types and accessors, so only its first
/// expansion emits them. A method on the primary key's wrapper type which reads a row of the
/// table would be ambiguous on a struct with several tables, because the key names a row in
/// each of them, so only the expansion of a struct's only `#[dsl]` emits those.
#[derive(Clone, Copy)]
pub enum DSLAttributePass {
    /// The struct's only `#[dsl]` attribute.
    Only,
    /// The first of several.
    First,
    /// A later one of several.
    Later,
}

impl DSLAttributePass {
    pub fn of(is_first: bool, is_last: bool) -> DSLAttributePass {
        match (is_first, is_last) {
            (true, true) => DSLAttributePass::Only,
            (true, false) => DSLAttributePass::First,
            (false, _) => DSLAttributePass::Later,
        }
    }

    fn emits_the_struct_items(self) -> bool {
        matches!(self, DSLAttributePass::Only | DSLAttributePass::First)
    }

    fn emits_the_referenced_row_methods(self) -> bool {
        matches!(self, DSLAttributePass::Only)
    }
}
```

Change the signature of `build` to `pub fn build(input: &Table, pass: DSLAttributePass) -> syn::Result<GeneratedOutput>`. Replace the comment *Only generate wrapper types if this is the last DSL attribute to avoid conflicts* and its condition by:

```rust
    // Every table of the struct shares its wrapper types, so the first expansion emits them.
    if pass.emits_the_struct_items() {
```

replace `if first_dsl_attribute {` in the loop over the columns by `if pass.emits_the_struct_items() {`, and build the wrapper methods as:

```rust
    let referenced_row_methods: &[WrapperMethod] = match pass.emits_the_referenced_row_methods() {
        true => &input.spacetimedsl_methods.referenced_row_methods,
        false => &[],
    };

    let wrapper_methods = input
        .spacetimedsl_methods
        .wrapper_methods
        .iter()
        .chain(referenced_row_methods)
        .map(wrapper_method::build)
        .collect();
```

In `derive/src/lib.rs`, delete the two lines

```rust
    // Check if this is the last #[dsl] attribute by counting remaining ones
    let _is_last_dsl_attribute = is_last_dsl_attribute(&derive_input);
```

and replace `let generated_output = output::build(&input, first_dsl_attribute)?;` by:

```rust
    let pass = output::DSLAttributePass::of(
        first_dsl_attribute,
        is_last_dsl_attribute(&derive_input),
    );
    let generated_output = output::build(&input, pass)?;
```

- [ ] **Step 6: Generate the methods**

In `derive-input/src/internal/spacetimedb.rs`, import `quote::format_ident` and `syn::{Ident, Path}`, and add:

```rust
/// `<#module_path::<accessor>__TableHandle as ::spacetimedb::Table>::Row`: the struct of the
/// table `table_accessor` in `module_path`, named without knowing it. SpacetimeDB generates the
/// handle type `<accessor>__TableHandle` next to every table and implements `Table` for it with
/// the struct as `Row`.
pub fn table_row_type(module_path: &Path, table_accessor: &Ident) -> TokenStream {
    let table_handle = format_ident!("{table_accessor}__TableHandle");

    quote! {
        <#module_path::#table_handle as ::spacetimedb::Table>::Row
    }
}
```

In `derive-input/src/internal/dsl/method/wrapper_method.rs`, extend the module comment by *It also adds the methods in the other direction to the wrapper type of the table's primary key, which look up the row a foreign key column references.*, import `super::{foreign_key::foreign_key_of, naming}`, `crate::{api::runtime, internal::spacetimedb}`, `std::collections::BTreeMap` and `syn::Ident`, and add:

```rust
/// One method per foreign key column besides the primary key, added to the wrapper type of
/// the primary key, which looks up the row the column of the row with that key references.
///
/// It takes the name of the referenced table, `get_<table>`. Where the table references that
/// table through several columns, or references itself, the table's name would not say which
/// column the method follows, so it takes the column's stem instead: `parent_folder_id` adds
/// `get_parent_folder`.
pub fn for_referenced_row_methods(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> Vec<WrapperMethod> {
    let MethodGenerationContext {
        spacetimedsl_table,
        primary_key_column,
        struct_name,
        singular_table_name,
        primary_key_column_name,
        ..
    } = context;

    // A singleton's injected primary key has no wrapper type to add a method to.
    if spacetimedsl_table.is_singleton() {
        return vec![];
    }

    let primary_key_wrapper = primary_key_column
        .spacetimedsl_column_wrapper_type
        .as_ref()
        .expect("`internal/dsl/column.rs` rejects a primary key without a wrapper outside singletons");
    let primary_key_wrapper_struct_name = primary_key_wrapper.struct_name();
    let primary_key_wrapper_variable_name =
        RenameRule::SnakeCase.apply_to_variant(primary_key_wrapper_struct_name.to_string());
    let get_this_row =
        naming::get_by_index_method_name(singular_table_name, primary_key_column_name);

    let mut methods = vec![];

    for (referenced_table_name, columns_with_foreign_key) in
        foreign_key_columns_by_referenced_table
    {
        let takes_the_column_stem =
            *referenced_table_name == singular_table_name || columns_with_foreign_key.len() > 1;

        // A foreign key on the primary key references the row its own value names.
        let columns = columns_with_foreign_key
            .iter()
            .filter(|column| !column.spacetimedb_column.is_primary_key);

        for column in columns {
            let foreign_key = foreign_key_of(column);
            let column_name = &column.rust_field.name;

            let method_name = match takes_the_column_stem {
                false => format_ident!("get_{referenced_table_name}"),
                true => format_ident!(
                    "get_{}",
                    column_stem(column_name, &foreign_key.primary_key_column_name)
                ),
            };

            let get_referenced_row = naming::get_by_index_method_name(
                &foreign_key.table_name,
                &foreign_key.primary_key_column_name,
            );
            let getter_name = naming::getter_name(column_name);
            let referenced_row_type =
                spacetimedb::table_row_type(&foreign_key.path, &foreign_key.table_name);

            methods.push(WrapperMethod {
                wrapper_type: wrapper_type_of(primary_key_wrapper),
                doc_comment: format!(
                    "Get the row of the `{referenced_table_name}` table which the `{column_name}` column of the `{struct_name}` row with this `{primary_key_wrapper_struct_name}` references.\n\nUse it like `{primary_key_wrapper_variable_name}.{method_name}(&dsl)`."
                ),
                method_name,
                return_type: runtime::error_result_type(&referenced_row_type),
                method_impl: quote! {
                    let dsl = dsl.into();
                    let #singular_table_name = dsl.#get_this_row(self)?;
                    dsl.#get_referenced_row(#singular_table_name.#getter_name())
                },
            });
        }
    }

    methods
}

/// The name of a foreign key column without the suffix which names the key it holds: without
/// `_<primary key of the referenced table>`, or else without `_id`, or else whole.
fn column_stem(column_name: &Ident, referenced_primary_key_column_name: &Ident) -> String {
    let column_name = column_name.to_string();
    let key_suffix = format!("_{referenced_primary_key_column_name}");

    column_name
        .strip_suffix(key_suffix.as_str())
        .or_else(|| column_name.strip_suffix("_id"))
        .unwrap_or(&column_name)
        .to_string()
}
```

In `derive-input/src/internal/dsl/method.rs`, import `wrapper_method::{for_referenced_row_methods, for_wrapper_methods}`, and add to the `SpacetimeDSLTableMethods` built in `generate`:

```rust
            referenced_row_methods: for_referenced_row_methods(
                context,
                &foreign_key_columns_by_referenced_table,
            ),
```

- [ ] **Step 7: Observe the green, regenerate the snapshots and the compile-test output, and read both diffs**

Run the unit gate. Expected: the contract test passes; snapshots and `referenced_by_repeated` fail.

Regenerate the snapshots, then:

```powershell
git diff --name-only -- derive/tests/snapshots | Where-Object { $_ -notlike "*/wrapper_methods.snap" }
(git diff --name-only -- derive/tests/snapshots | Measure-Object).Count
```

Expected: no output, then `27`. Each diff adds, after the existing methods, one `impl <PrimaryKeyWrapper>` block per foreign key column besides the primary key, such as in `on_delete_delete/Book/wrapper_methods.snap`:

```rust
impl BookId {
    /**Get the row of the `author` table which the `author_id` column of the `Book` row with this `BookId` references.

Use it like `book_id.get_author(&dsl)`.*/
    pub fn get_author<'a, T: 'a + crate::spacetimedsl::ReadContext>(
        &self,
        dsl: impl Into<crate::spacetimedsl::ReadOnlyDSL<'a, T>>,
    ) -> Result<
        <self::author__TableHandle as ::spacetimedb::Table>::Row,
        crate::spacetimedsl::error::SpacetimeDSLError,
    > {
        let dsl = dsl.into();
        let book = dsl.get_book_by_id(self)?;
        dsl.get_author_by_id(book.get_author_id())
    }
}
```

The two foreign keys of `foreign_key_and_referenced_by/Shipment`, `foreign_keys_with_equivalent_spellings/Shipment` and `Transfer`, and the self-reference of `self_referencing_cascade/Folder` and `wrapper_methods/Category`, take column stems. `wrapper_methods/Membership/pass_1` and `pass_2` and `wrapper_methods/Profile` do not change.

Regenerate the compile-test output. Expected: only `referenced_by_repeated.stderr` changes, gaining errors about `warehouse__TableHandle` and `get_warehouse_by_id` for the two methods of `shipment`; its comment already names *the items the `warehouse` table would have generated for it* as the cause.

- [ ] **Step 8: Snapshot every shape of the name**

Create `derive/tests/fixtures/referenced_row_methods.rs`:

```rust
//! Covers the methods a table adds to the wrapper type of its primary key, which look up the
//! row a foreign key column of the row with that key references:
//!
//! - `Server` and `Alliance` reference one table through one column each, so their methods
//!   take the name of the referenced table: `server_id.get_season(&dsl)`,
//!   `alliance_id.get_server(&dsl)`.
//! - `Tournament` references `Season` through two columns, so each method takes the name of
//!   its column without the key suffix: `get_opening_season`, `get_closing_season`.
//! - `Stage` references its own table through a unique column, which adds
//!   `get_stage_by_next_stage_id` to `StageId` for the referencing row; its method for the
//!   referenced row is `get_next_stage`.
//! - `Circle` has a foreign key on its primary key, which adds no such method, and one on
//!   `player_id`, which adds `get_player` to `EntityId`, the wrapper its key uses.
//! - `Referee` is a singleton, whose injected primary key has no wrapper type: it adds none.
//!
//! A struct with several `#[dsl]` attributes adds none either; the `Membership` snapshots of
//! the fixture `wrapper_methods` pin that.

#[spacetimedsl::dsl(plural_name = seasons, method(update = false, delete = false))]
#[spacetimedb::table(accessor = season, public)]
pub struct Season {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = server)]
    #[referenced_by(path = self, table = tournament)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = servers, method(update = false, delete = false))]
#[spacetimedb::table(accessor = server, public)]
pub struct Server {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = alliance)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(SeasonId)]
    #[foreign_key(path = self, table = season, column = id)]
    season_id: u64,
}

#[spacetimedsl::dsl(plural_name = alliances, method(update = false, delete = false))]
#[spacetimedb::table(accessor = alliance, public)]
pub struct Alliance {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ServerId)]
    #[foreign_key(path = self, table = server, column = id)]
    server_id: u64,
}

#[spacetimedsl::dsl(plural_name = tournaments, method(update = false, delete = false))]
#[spacetimedb::table(accessor = tournament, public)]
pub struct Tournament {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(SeasonId)]
    #[foreign_key(path = self, table = season, column = id)]
    opening_season_id: u64,

    #[index(btree)]
    #[use_wrapper(SeasonId)]
    #[foreign_key(path = self, table = season, column = id)]
    closing_season_id: u64,
}

#[spacetimedsl::dsl(plural_name = stages, method(update = false, delete = false))]
#[spacetimedb::table(accessor = stage, public)]
pub struct Stage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = stage)]
    id: u64,

    #[unique]
    #[use_wrapper(StageId)]
    #[foreign_key(path = self, table = stage, column = id)]
    next_stage_id: u64,
}

#[spacetimedsl::dsl(plural_name = entities, method(update = false, delete = false))]
#[spacetimedb::table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = circle)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = players, method(update = false, delete = false))]
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = circle)]
    #[referenced_by(path = self, table = referee)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = circles, method(update = true, delete = false))]
#[spacetimedb::table(accessor = circle, public)]
pub struct Circle {
    #[primary_key]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id)]
    entity_id: u64,

    #[index(btree)]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id)]
    pub player_id: u64,
}

#[spacetimedsl::dsl(singleton, method(update = true))]
#[spacetimedb::table(accessor = referee, public)]
pub struct Referee {
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id)]
    pub player_id: u64,
}
```

Register it after `wrapper_methods` in `derive/src/characterization_tests.rs`. Regenerate the snapshots and read the new `wrapper_methods.snap` files: `Server` adds `SeasonId::get_servers` and `ServerId::get_season`; `Alliance` `ServerId::get_alliances` and `AllianceId::get_server`; `Tournament` `SeasonId::get_tournaments_by_opening_season_id`, `SeasonId::get_tournaments_by_closing_season_id`, `TournamentId::get_opening_season` and `TournamentId::get_closing_season`; `Stage` `StageId::get_stage_by_next_stage_id` and `StageId::get_next_stage`; `Circle` `EntityId::get_circle`, `PlayerId::get_circles` and `EntityId::get_player`, whose body looks the circle up with `dsl.get_circle_by_entity_id(self)?`. `Referee` has no `wrapper_methods.snap`.

- [ ] **Step 9: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`. The runtime gate also builds `blackholio`, whose `Circle` now adds `get_player` to `EntityId`.

- [ ] **Step 10: Document the methods**

In `docs/DOCUMENTATION.md`, add after the section *Look Up Referencing Rows From a Wrapper*:

````markdown
### Look Up the Referenced Row From a Wrapper

Every `#[foreign_key]` column besides the primary key also adds a method to the wrapper type of its own table's primary key. The method looks up the row the column of the row with that key references:

```rust
// Alliance has `#[use_wrapper(ServerId)] #[foreign_key(… table = server …)] server_id`,
// Server has `#[use_wrapper(SeasonId)] #[foreign_key(… table = season …)] season_id`:
let max_alliance_level = alliance.get_server_id().get_season(&dsl)?.get_max_alliance_level();

// instead of
let server = dsl.get_server_by_id(alliance.get_server_id())?;
let max_alliance_level = dsl.get_season_by_id(server.get_season_id())?.get_max_alliance_level();
```

- The method is `get_<referenced table>` and returns `Result<Row, SpacetimeDSLError>`: a `NotFoundError` when no row has the key, or when the column holds `0` or `Uuid::NIL`, which reference no row.
- When a table references the same table through several columns, or references itself, each method takes the name of its column without its `_<primary key>` or `_id` suffix: `parent_entity_id` adds `get_parent_entity`. A unique foreign key to the own table therefore gets both `get_<table>_by_<column>`, the row which references this one, and `get_<column stem>`, the row this one references.
- The return type names the row as `<path::<table>__TableHandle as ::spacetimedb::Table>::Row`, the type SpacetimeDB generates for the referenced table: the foreign key names the table, not its struct. Its value is the table's struct.
- A foreign key on the primary key adds no such method, and neither does a singleton, whose injected primary key has no wrapper type, nor a struct with several `#[dsl]` attributes, whose tables share the wrapper type.
- The documentation of each method says which column it follows, so its direction is clear although both directions share the `get_` prefix.
````

and replace the first sentence of *Look Up Referencing Rows From a Wrapper* by *Every `#[foreign_key]` column with a single-column index adds a method to its `#[use_wrapper]` type, which looks up the rows referencing one value of that wrapper; the next section describes the method for the other direction.*

In `README.md`, replace the bullet *Foreign-key columns add lookups to their wrapper types, like `entity_id.get_position(&dsl)`.* by:

```markdown
- 🧭 Foreign-key columns add lookups to wrapper types in both directions, like `entity_id.get_position(&dsl)` and `position_id.get_entity(&dsl)`.
```

In `docs/MIGRATION.md`, add under *For crates building on `spacetimedsl_derive-input`*:

```markdown
#### `SpacetimeDSLTableMethods::referenced_row_methods`

`SpacetimeDSLTableMethods` gained `referenced_row_methods: Vec<WrapperMethod>`: the methods a table adds to the wrapper type of its primary key, which look up the row each foreign key column references. Emit them only for a struct with a single `#[dsl]` attribute, as `spacetimedsl_derive` does: the tables of a struct with several share its wrapper types, and each would add the same methods. Their return type names the referenced struct through SpacetimeDB's `<accessor>__TableHandle`.
```

In `CODE_QUALITY_REPORT.md`:

- Rename the entry *`lib.rs`: `fn is_last_dsl_attribute(derive_input: &syn::DeriveInput) -> bool` and the commented-out `make_struct_fields_private`* to *`lib.rs`: the commented-out `make_struct_fields_private`*, delete its sentence about `_is_last_dsl_attribute`, which is used now, and let its recommendation delete *the commented-out block and the TODO* only.
- Rename the entry *`output.rs`: `pub fn build(input: &Table, first_dsl_attribute: bool) -> syn::Result<GeneratedOutput>`* to *`output.rs`: `pub fn build(input: &Table, pass: DSLAttributePass) -> syn::Result<GeneratedOutput>`*, and delete its sentence about the comment which contradicted `if first_dsl_attribute`, and *Correct the comment and* from its recommendation: both are done.

- [ ] **Step 11: Format, lint, commit**

Commit message:

```text
Add wrapper-type methods which look up the referenced row

Every foreign key column besides the primary key adds get_<referenced table> to the
wrapper type of its table's primary key, which looks the row up and follows the column:
alliance.get_server_id().get_season(&dsl). With several foreign keys to one table or a
self-reference, the method takes the column's stem, such as get_parent_folder. The return
type names the referenced struct through SpacetimeDB's <accessor>__TableHandle.
spacetimedsl_derive emits the methods only for a struct with a single #[dsl] attribute;
output::build takes a DSLAttributePass instead of a bool.

Tests: contract assertions for referenced_row_methods; the fixture
referenced_row_methods; the runtime group referenced_row_method_test.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 9: Switch a referenced-row method off with `referenced_row_method = false` (#190, part 2)

**Files:**
- Modify: `derive-input/src/internal/dsl.rs` (symbol), `derive-input/src/api/dsl/foreign_key.rs` (field)
- Modify: `derive-input/src/internal/dsl/foreign_key.rs` (the argument and its two rejections)
- Modify: `derive-input/src/internal/dsl/method/wrapper_method.rs` (the filter)
- Modify: `derive-input/src/internal/error.rs` (two diagnostics)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Create: `compile-tests/tests/ui/referenced_row_method_on_singleton.rs`, `compile-tests/tests/ui/referenced_row_method_on_primary_key_foreign_key.rs` (+ `.stderr`)
- Modify: `derive/tests/fixtures/referenced_row_methods.rs` (`Banner`)
- Create: `examples/test/src/referenced_row_method_opt_out_test.rs`; register it
- Modify: `docs/DOCUMENTATION.md` (*Declaration*, *Look Up the Referenced Row From a Wrapper*), `docs/MIGRATION.md`
- Recorded output: `derive/tests/snapshots/referenced_row_methods/Server/**` (the new `#[referenced_by]`) and the new `Banner/**`

**Interfaces:**
- Consumes: `for_referenced_row_methods` of Task 8.
- Produces: `ForeignKey::referenced_row_method: bool`; the symbol `internal::dsl::referenced_row_method`; `error::referenced_row_method_on_singleton`, `error::referenced_row_method_on_primary_key_column`.

- [ ] **Step 1: Write the failing runtime group and compile tests**

Create `examples/test/src/referenced_row_method_opt_out_test.rs`:

```rust
//! `#[foreign_key(..., referenced_row_method = false)]` keeps a table from adding the method
//! which looks up the referenced row to the wrapper type of its primary key.
//!
//! An account and a character reference each other through unique foreign keys, so the method
//! each table adds for the referenced row would take the name of the method the other table
//! adds for the referencing row, on the same wrapper type. With both switched off the module
//! compiles, and the methods for the referencing rows keep working.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = opt_out_accounts, method(update = true, delete = false))]
#[spacetimedb::table(accessor = opt_out_account)]
pub struct OptOutAccount {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_opt_out_test, table = opt_out_character)]
    id: u64,

    #[unique]
    #[use_wrapper(OptOutCharacterId)]
    #[foreign_key(
        path = crate::referenced_row_method_opt_out_test,
        table = opt_out_character,
        column = id,
        referenced_row_method = false
    )]
    pub character_id: u64,
}

#[spacetimedsl::dsl(plural_name = opt_out_characters, method(update = true, delete = false))]
#[spacetimedb::table(accessor = opt_out_character)]
pub struct OptOutCharacter {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_opt_out_test, table = opt_out_account)]
    id: u64,

    #[unique]
    #[use_wrapper(OptOutAccountId)]
    #[foreign_key(
        path = crate::referenced_row_method_opt_out_test,
        table = opt_out_account,
        column = id,
        referenced_row_method = false
    )]
    pub account_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut account = dsl.create_opt_out_account(CreateOptOutAccount {
        character_id: OptOutCharacterId::new(0),
    })?;
    let character = dsl.create_opt_out_character(CreateOptOutCharacter {
        account_id: account.get_id(),
    })?;
    account.set_character_id(&character);
    let account = dsl.update_opt_out_account_by_id(account)?;

    if character.get_id().get_opt_out_account(dsl)?.get_id() != account.get_id() {
        return Err(
            "OptOutCharacterId::get_opt_out_account should find the account whose character_id references the character!"
                .to_string(),
        );
    }

    if account.get_id().get_opt_out_character(dsl)?.get_id() != character.get_id() {
        return Err(
            "OptOutAccountId::get_opt_out_character should find the character whose account_id references the account!"
                .to_string(),
        );
    }

    Ok(())
}
```

In `examples/test/src/lib.rs`, add `pub mod referenced_row_method_opt_out_test;` before `pub mod referenced_row_method_test;`, and append to `TEST_GROUPS`:

```rust
    (
        "referenced_row_method_opt_out_test",
        referenced_row_method_opt_out_test::run_tests,
    ),
```

Create `compile-tests/tests/ui/referenced_row_method_on_singleton.rs`:

```rust
//! A singleton's injected primary key has no wrapper type, so a singleton adds no method for
//! the referenced row, and `referenced_row_method` would switch off nothing.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so the
//! `region` table's expansion misses the trait the `server_binding` table would have declared
//! for it.

::spacetimedsl::spacetimedsl!();

pub mod region {
    #[spacetimedsl::dsl(plural_name = regions, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = region, public)]
    pub struct Region {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(RegionId)]
        #[referenced_by(path = crate::binding, table = server_binding)]
        id: u64,
    }
}

pub mod binding {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = server_binding, public)]
    pub struct ServerBinding {
        #[use_wrapper(crate::region::RegionId)]
        #[foreign_key(path = crate::region, table = region, column = id, referenced_row_method = false)]
        pub region_id: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/referenced_row_method_on_primary_key_foreign_key.rs`:

```rust
//! A foreign key on the primary key adds no method for the referenced row: the row it
//! references has the key's own value, so `referenced_row_method` would switch off nothing.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so the
//! `account` table's expansion misses the trait the `profile` table would have declared for it.

::spacetimedsl::spacetimedsl!();

pub mod account {
    #[spacetimedsl::dsl(plural_name = accounts, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = account, public)]
    pub struct Account {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(AccountId)]
        #[referenced_by(path = crate::profile, table = profile)]
        id: u64,
    }
}

pub mod profile {
    #[spacetimedsl::dsl(plural_name = profiles, method(update = true, delete = false))]
    #[spacetimedb::table(accessor = profile, public)]
    pub struct Profile {
        #[primary_key]
        #[use_wrapper(crate::account::AccountId)]
        #[foreign_key(path = crate::account, table = account, column = id, referenced_row_method = false)]
        account_id: u64,

        pub display_name: String,
    }
}

fn main() {}
```

- [ ] **Step 2: Observe the red**

Run the unit gate.
Expected: `FAILED`; both new compile tests fail with *expected one of: `path`, `table`, `column`, `on_delete`, `on_soft_delete`*, underlining `referenced_row_method`: the argument does not exist yet.

Run the runtime gate.
Expected: `FAILED` with the same message pointing into `referenced_row_method_opt_out_test.rs`.

- [ ] **Step 3: Pin the model in the contract test (the red)**

In `derive/src/data_transfer_contract_tests.rs`, add `assert!(foreign_key.referenced_row_method);` after the assertion on `foreign_key.on_delete_strategy`. After the block which parses `price`, add:

```rust
    let invoice = parse_table(
        quote! { plural_name = invoices, method(update = false, delete = false) },
        quote! {
            #[spacetimedb::table(accessor = invoice, public)]
            pub struct Invoice {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u64,

                #[index(btree)]
                #[use_wrapper(crate::currency::CurrencyId)]
                #[foreign_key(path = crate::currency, table = currency, column = id, referenced_row_method = false)]
                currency_id: u64,
            }
        },
    );
    visit_table(&invoice);

    assert!(
        !column(&invoice, "currency_id")
            .spacetimedsl_column
            .foreign_key
            .as_ref()
            .expect("`currency_id` has `#[foreign_key]`")
            .referenced_row_method
    );
    assert!(
        invoice
            .spacetimedsl_methods
            .referenced_row_methods
            .is_empty(),
        "`referenced_row_method = false` switches the method off"
    );
```

In `visit_spacetimedsl_column`, add `referenced_row_method: _,` to the destructuring of `ForeignKey`.

Run the unit gate.
Expected: `FAILED` with `error[E0609]: no field `referenced_row_method` on type `&ForeignKey``.

- [ ] **Step 4: Parse the argument, reject it where it says nothing, and honour it**

In `derive-input/src/internal/dsl.rs`, add `symbol!(referenced_row_method);` after `symbol!(on_soft_delete);`.

In `derive-input/src/api/dsl/foreign_key.rs`, add to `ForeignKey` after `on_soft_delete_strategy`:

```rust
    /// Whether the table adds the method which looks up the referenced row to the wrapper type
    /// of its primary key: `false` with `referenced_row_method = false`.
    pub referenced_row_method: bool,
```

In `derive-input/src/internal/dsl/foreign_key.rs`, import the symbol `referenced_row_method` from `internal::dsl`, declare next to the other arguments

```rust
            let mut referenced_row_method_argument: Option<(Path, bool)> = None;
```

and add to the `match_meta!`:

```rust
                    referenced_row_method => {
                        check_duplicate(&referenced_row_method_argument, &meta)?;
                        let argument = meta.path.clone();
                        let value = meta.value()?.parse::<syn::LitBool>()?.value;
                        referenced_row_method_argument = Some((argument, value));
                    }
```

Add before `foreign_key_value = Some(ForeignKey { … });`:

```rust
            // Where the table adds no such method, the argument would switch off nothing.
            if let Some((argument, _)) = &referenced_row_method_argument {
                if is_singleton {
                    return Err(error::referenced_row_method_on_singleton(argument));
                }

                if spacetimedb_column.is_primary_key {
                    return Err(error::referenced_row_method_on_primary_key_column(
                        argument,
                    ));
                }
            }
```

and fill the field:

```rust
                referenced_row_method: referenced_row_method_argument
                    .is_none_or(|(_, is_generated)| is_generated),
```

In `derive-input/src/internal/error.rs`, add to the `#[foreign_key]` section:

```rust
pub fn referenced_row_method_on_singleton(argument: &impl ToTokens) -> Error {
    Error::new_spanned(
        argument,
        "`referenced_row_method` has no effect on a singleton table, whose injected primary key has no wrapper type to add the method to! Remove it.",
    )
}

pub fn referenced_row_method_on_primary_key_column(argument: &impl ToTokens) -> Error {
    Error::new_spanned(
        argument,
        "`referenced_row_method` has no effect on a foreign key on the primary key, which adds no such method: the row it references has the key's own value! Remove it.",
    )
}
```

In `derive-input/src/internal/dsl/method/wrapper_method.rs`, replace the comment and the filter over the columns of `for_referenced_row_methods` by:

```rust
        // A foreign key on the primary key references the row its own value names, and
        // `referenced_row_method = false` switches the method off.
        let columns = columns_with_foreign_key.iter().filter(|column| {
            !column.spacetimedb_column.is_primary_key && foreign_key_of(column).referenced_row_method
        });
```

A column which switches its method off still counts when the method name of another foreign key to the same table is decided, so switching one off does not rename the other.

- [ ] **Step 5: Observe the green and regenerate**

Run the unit gate. Expected: the contract test passes; the two compile tests fail on their `.stderr`.

Regenerate the compile-test output. Expected: each new file starts with its diagnostic, underlining `referenced_row_method`, followed by the one unresolved import its comment names.

In `derive/tests/fixtures/referenced_row_methods.rs`, add `#[referenced_by(path = self, table = banner)]` to the primary key of `Server`, add to the list of the module comment *`Banner` switches its method off with `referenced_row_method = false`, which leaves only `ServerId::get_banners`.*, and add after `Alliance`:

```rust
#[spacetimedsl::dsl(plural_name = banners, method(update = false, delete = false))]
#[spacetimedb::table(accessor = banner, public)]
pub struct Banner {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ServerId)]
    #[foreign_key(path = self, table = server, column = id, referenced_row_method = false)]
    server_id: u64,
}
```

Regenerate the snapshots and read them. Expected: `Banner/wrapper_methods.snap` holds `ServerId::get_banners` alone; `Server/table.snap` gains the pairing trait for `banner` and names it under *Tables referencing the `server` table*. Nothing else changes.

- [ ] **Step 6: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`: with both methods for the referenced rows switched off, the two tables which reference each other compile.

- [ ] **Step 7: Document the argument**

In `docs/DOCUMENTATION.md`, *Foreign Keys & Referential Integrity*, *Declaration*, add after the paragraph about a foreign key to a table which removes no rows:

```markdown
`referenced_row_method = false` keeps the column from adding the method which looks up the row it references; see [Look Up the Referenced Row From a Wrapper](#look-up-the-referenced-row-from-a-wrapper).
```

In *Look Up the Referenced Row From a Wrapper*, add to the list:

```markdown
- `#[foreign_key(..., referenced_row_method = false)]` switches the method of one column off. It is rejected where the table adds none: on a foreign key on the primary key and on a singleton.
```

In `docs/MIGRATION.md`, add under *For crates building on `spacetimedsl_derive-input`*:

```markdown
#### `ForeignKey::referenced_row_method`

`ForeignKey` gained `referenced_row_method: bool`, `false` when `#[foreign_key(..., referenced_row_method = false)]` switches off the method which looks up the referenced row. Such a column adds nothing to `SpacetimeDSLTableMethods::referenced_row_methods`.
```

- [ ] **Step 8: Format, lint, commit**

Commit message:

```text
Switch a referenced-row method off with referenced_row_method = false

#[foreign_key(..., referenced_row_method = false)] keeps the column from adding the method
which looks up the referenced row, for where it would take a name another method of the
wrapper type has. The argument is rejected where the table adds no such method: on a
foreign key on the primary key and on a singleton.

Tests: contract assertions for ForeignKey::referenced_row_method; the fixture
referenced_row_methods gained Banner; the compile tests referenced_row_method_on_singleton
and referenced_row_method_on_primary_key_foreign_key; the runtime group
referenced_row_method_opt_out_test.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 10: Report and pin the name collisions of referenced-row methods (#190, part 3)

**Files:**
- Modify: `derive-input/src/internal/dsl/method/wrapper_method.rs` (duplicate names), `derive-input/src/internal/dsl/method.rs` (the `?`)
- Modify: `derive-input/src/internal/error.rs` (one diagnostic)
- Create: `compile-tests/tests/ui/referenced_row_methods_with_the_same_name.rs`, `compile-tests/tests/ui/referenced_row_methods_of_tables_referencing_each_other.rs`, `compile-tests/tests/ui/referenced_row_methods_of_tables_sharing_a_primary_key_wrapper.rs` (+ `.stderr`)
- Modify: `docs/DOCUMENTATION.md` (*Look Up the Referenced Row From a Wrapper*), `docs/MIGRATION.md`

**Interfaces:**
- Consumes: `for_referenced_row_methods` of Tasks 8 and 9.
- Produces: `for_referenced_row_methods(…) -> syn::Result<Vec<WrapperMethod>>`; `error::referenced_row_methods_with_the_same_name(column_name: &Ident, other_column_name: &Ident, method_name: &Ident, wrapper_struct_name: &Ident) -> Error`.

Two collisions happen inside one table, where `derive-input` sees both columns and can say what to change. The other two need a second table, which no `#[dsl]` sees; as the issue decided, a compile test pins each of them as an UNSUPPORTED COMBINATION, the way `wrapper_optional_unique_index` pins a SpacetimeDB limit.

- [ ] **Step 1: Write the three compile tests**

Create `compile-tests/tests/ui/referenced_row_methods_with_the_same_name.rs`:

```rust
//! `owner_id` and `owner` both reference `player`, so their methods for the referenced row
//! take the stems of their columns, which are the same: both would add `get_owner` to
//! `GameId`.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so the
//! `player` table's expansion misses the trait the `game` table would have declared for it.

::spacetimedsl::spacetimedsl!();

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerId)]
        #[referenced_by(path = crate::game, table = game)]
        id: u64,
    }
}

pub mod game {
    #[spacetimedsl::dsl(plural_name = games, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = game, public)]
    pub struct Game {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        owner_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        owner: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/referenced_row_methods_of_tables_referencing_each_other.rs`:

```rust
//! UNSUPPORTED COMBINATION - this file pins a limit of the methods SpacetimeDSL adds to wrapper
//! types, not a rule it enforces: no single `#[dsl]` sees both tables, so the macro cannot
//! report it itself.
//!
//! `player_account` and `player_character` reference each other through unique foreign keys.
//! `player_account.player_character_id` adds `get_player_account` to `PlayerCharacterId` for
//! the account which references a character, and `player_character.player_account_id` adds
//! `get_player_account` to `PlayerCharacterId` for the account a character references; the
//! same happens to `get_player_character` on `PlayerAccountId`. rustc rejects both as
//! duplicate definitions. Adding `referenced_row_method = false` to both foreign keys keeps
//! the methods for the referencing rows and leaves a program that compiles.

::spacetimedsl::spacetimedsl!();

pub mod account {
    #[spacetimedsl::dsl(plural_name = player_accounts, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player_account, public)]
    pub struct PlayerAccount {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerAccountId)]
        #[referenced_by(path = crate::character, table = player_character)]
        id: u64,

        #[unique]
        #[use_wrapper(crate::character::PlayerCharacterId)]
        #[foreign_key(path = crate::character, table = player_character, column = id)]
        player_character_id: u64,
    }
}

pub mod character {
    #[spacetimedsl::dsl(plural_name = player_characters, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player_character, public)]
    pub struct PlayerCharacter {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerCharacterId)]
        #[referenced_by(path = crate::account, table = player_account)]
        id: u64,

        #[unique]
        #[use_wrapper(crate::account::PlayerAccountId)]
        #[foreign_key(path = crate::account, table = player_account, column = id)]
        player_account_id: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/referenced_row_methods_of_tables_sharing_a_primary_key_wrapper.rs`:

```rust
//! UNSUPPORTED COMBINATION - this file pins a limit of the methods SpacetimeDSL adds to wrapper
//! types, not a rule it enforces: no single `#[dsl]` sees both tables, so the macro cannot
//! report it itself.
//!
//! `circle` and `food` both use `EntityId` for their primary key, and both reference `player`
//! through `player_id`. Each adds `get_player` to `EntityId` for the player its row
//! references, so rustc rejects the second as a duplicate definition. Adding
//! `referenced_row_method = false` to one of the two foreign keys to `player` leaves a program
//! that compiles.

::spacetimedsl::spacetimedsl!();

pub mod entity {
    #[spacetimedsl::dsl(plural_name = entities, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = entity, public)]
    pub struct Entity {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(EntityId)]
        #[referenced_by(path = crate::circle, table = circle)]
        #[referenced_by(path = crate::food, table = food)]
        id: u64,
    }
}

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(PlayerId)]
        #[referenced_by(path = crate::circle, table = circle)]
        #[referenced_by(path = crate::food, table = food)]
        id: u64,
    }
}

pub mod circle {
    #[spacetimedsl::dsl(plural_name = circles, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = circle, public)]
    pub struct Circle {
        #[primary_key]
        #[use_wrapper(crate::entity::EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = id)]
        entity_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        player_id: u64,
    }
}

pub mod food {
    #[spacetimedsl::dsl(plural_name = foods, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = food, public)]
    pub struct Food {
        #[primary_key]
        #[use_wrapper(crate::entity::EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = id)]
        entity_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::player::PlayerId)]
        #[foreign_key(path = crate::player, table = player, column = id)]
        player_id: u64,
    }
}

fn main() {}
```

- [ ] **Step 2: Observe the red**

Run the unit gate.
Expected: `FAILED`; all three files fail to compile, with rustc's `error[E0592]: duplicate definitions with name `get_owner`` for the first, `get_player_account` and `get_player_character` for the second, and `get_player` for the third. The first is the one SpacetimeDSL has to report itself; the other two are what the files pin.

- [ ] **Step 3: Report two columns whose methods would take the same name**

In `derive-input/src/internal/error.rs`, add to the `#[foreign_key]` section:

```rust
pub fn referenced_row_methods_with_the_same_name(
    column_name: &Ident,
    other_column_name: &Ident,
    method_name: &Ident,
    wrapper_struct_name: &Ident,
) -> Error {
    Error::new_spanned(
        column_name,
        format!(
            "The foreign key columns `{other_column_name}` and `{column_name}` would both add `{method_name}` to `{wrapper_struct_name}`! Rename one of them, or add `referenced_row_method = false` to the `#[foreign_key]` of one."
        ),
    )
}
```

In `derive-input/src/internal/dsl/method/wrapper_method.rs`, import `crate::internal::error`, let `for_referenced_row_methods` return `syn::Result<Vec<WrapperMethod>>`, end it with `Ok(methods)`, return `Ok(vec![])` for a singleton, and declare after `let mut methods = vec![];`:

```rust
    let mut column_by_method_name: BTreeMap<String, &Ident> = BTreeMap::new();
```

Right after `method_name` is computed, add:

```rust
            if let Some(other_column_name) =
                column_by_method_name.insert(method_name.to_string(), column_name)
            {
                return Err(error::referenced_row_methods_with_the_same_name(
                    column_name,
                    other_column_name,
                    &method_name,
                    &primary_key_wrapper_struct_name,
                ));
            }
```

In `derive-input/src/internal/dsl/method.rs`, add `?` after the call of `for_referenced_row_methods`.

- [ ] **Step 4: Regenerate the compile-test output and read it**

Regenerate. Expected:

- `referenced_row_methods_with_the_same_name.stderr` starts with *The foreign key columns `owner_id` and `owner` would both add `get_owner` to `GameId`! …*, underlining `owner` in line 39, followed by the unresolved import its comment names.
- `referenced_row_methods_of_tables_referencing_each_other.stderr` holds the two `E0592` errors, and `referenced_row_methods_of_tables_sharing_a_primary_key_wrapper.stderr` the one, each pointing at the `#[spacetimedsl::dsl]` attributes whose expansions define the method twice, and nothing else.

No other `.stderr` changes. Run the unit gate and `git diff --stat -- derive/tests/snapshots`: all `ok`, no file listed.

- [ ] **Step 5: Run the runtime gate**

Expected: `PASSED`.

- [ ] **Step 6: Document the collisions**

In `docs/DOCUMENTATION.md`, add at the end of *Look Up the Referenced Row From a Wrapper*:

````markdown
#### When Two Methods Take the Same Name

The methods of both directions share the wrapper types, so two of them can take the same name, which rustc rejects as *duplicate definitions with name `get_…`* (E0592):

- Two tables reference each other through unique foreign keys: `player_account.player_character_id` and `player_character.player_account_id` add `get_player_account` to `PlayerCharacterId` and `get_player_character` to `PlayerAccountId`, once in each direction.
- Two tables share a primary key wrapper and reference the same table, such as `circle` and `food`, both keyed by `EntityId`, with a `player_id` each: both add `get_player` to `EntityId`.
- A method you wrote on the wrapper type yourself has the name already.

Add `referenced_row_method = false` to the `#[foreign_key]` whose method for the referenced row you do not need:

```rust
#[foreign_key(path = crate::character, table = player_character, column = id, referenced_row_method = false)]
```

Within one table, SpacetimeDSL reports two foreign key columns whose methods would take the same name itself, such as `owner_id` and `owner` referencing the same table.
````

In `docs/MIGRATION.md`, add at the end of *Newly rejected inputs*:

```markdown
#### Two wrapper-type methods of the same name

Every `#[foreign_key]` column besides the primary key now adds a method to the wrapper type of its table's primary key, which looks up the row it references (*Look Up the Referenced Row From a Wrapper* in the documentation). Where that method takes a name another method of the wrapper type already has, the build fails:

- Two tables which reference each other through unique foreign keys, and two tables which share a primary key wrapper and reference the same table: rustc's *duplicate definitions with name `get_…`* (E0592).
- A method of your own on a wrapper type with the name of such a method: E0592.
- Two foreign keys of one table to the same table whose columns differ only in the key suffix, such as `owner_id` and `owner`: *The foreign key columns `owner_id` and `owner` would both add `get_owner` to `GameId`!*

Add `referenced_row_method = false` to the `#[foreign_key]` whose method you do not need, or rename the column.
```

- [ ] **Step 7: Format, lint, commit**

Commit message:

```text
Report and pin the name collisions of referenced-row methods

Two foreign key columns of one table whose referenced-row methods would take the same name
are rejected with a diagnostic naming both and the fix. Tables which reference each other
through unique foreign keys, and tables sharing a primary key wrapper which reference the
same table, collide across two expansions; two UNSUPPORTED COMBINATION compile tests pin
rustc's E0592 for them, whose fix is referenced_row_method = false.

Tests: the compile tests referenced_row_methods_with_the_same_name,
referenced_row_methods_of_tables_referencing_each_other and
referenced_row_methods_of_tables_sharing_a_primary_key_wrapper.

Closes #190

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 11: Verify the whole branch

**Files:** none changed, unless a check below finds something.

- [ ] **Step 1: Run every gate on a clean tree**

Run the unit gate, the runtime gate, `.\x.ps1 format` twice (the second run changes nothing) and `.\x.ps1 lint`. Expected: all `ok`, `PASSED`, no file changed, exit code `0`.

- [ ] **Step 2: Check the history**

Run: `git log --oneline main..HEAD`
Expected: 11 commits, Task 0 to Task 10, in plan order; the commits of Tasks 1, 3, 7 and 10 close #187, #188, #186 and #190.

- [ ] **Step 3: Read the documentation once more**

Read `## 0.23 → 0.24` of `docs/MIGRATION.md` from top to bottom: every change of Tasks 1–10 which affects existing code or a public `derive-input` type has exactly one entry, and no entry contradicts a later one. Read *Create Structs & Create Methods*, *Disallowed Values*, *Look Up the Referenced Row From a Wrapper*, *DeletionResult* and *Error Handling* in `docs/DOCUMENTATION.md`, and the feature list of `README.md`, for the same.

- [ ] **Step 4: Hand over**

Use superpowers:finishing-a-development-branch. The pull request body lists `Closes #187`, `Closes #188`, `Closes #186` and `Closes #190`, and names the three findings of *Found while planning and left out of scope* for the developer to file. Open it only after the developer agrees.

