# Plan 01 — Characterization tests for the DSL method generator

Addresses item 1 of the Recommended Resolution Order in
[`CODE_QUALITY_REPORT.md`](../../CODE_QUALITY_REPORT.md).

---

## Context

`derive-input/src/internal/dsl/method.rs` is 3461 lines and generates every DSL
method SpacetimeDSL emits. The code quality report lists 20 violations in it,
and items 2–14 of its resolution order are all refactorings of that file —
including breaking up the god function `for_method` and splitting the module
along domain seams.

`AGENTS.md` requires tests before refactoring (_Add tests before refactoring or
when missing_). The crate has none. `examples/test` is a `cdylib` compiled
against the macro: it proves the paths those examples happen to exercise produce
compilable output, and asserts nothing about the output's content. Any generated
branch no example triggers is entirely unverified — which is how the emission bug
fixed in commit `81ada87` survived undetected.

The intended outcome of this plan is a safety net that makes every later
refactoring step provable: a snapshot of the exact tokens the generator emits for
every table shape the codebase supports, plus compile tests pinning the
diagnostics users get for invalid attribute usage. After this plan, any change to
`method.rs` that alters generated output shows up as a reviewable diff instead of
a silent behavioural change.

Two production changes must land **before** the first snapshot, because without
them the tests cannot satisfy F.I.R.S.T (_Stabilize tests across environments and
runs_) and cannot avoid duplicating generator logic. They are isolated in their
own commit and verified against the two example modules.

---

## Locked decisions

Made explicitly by the developer during planning; do not re-litigate during
implementation.

| # | Decision |
| --- | --- |
| Seam | Snapshot tests are `#[cfg(test)]` units inside the `derive` crate, driving a new `proc_macro2`-level entry point. |
| Determinism | Fixed in production code (hash collections → ordered collections), not normalized in tests. |
| Tooling | `insta` for snapshots, `trybuild` for compile tests. |
| Granularity | Both: one table-level snapshot per struct (everything _except_ DSL methods) **and** one snapshot per generated DSL method. |
| No duplication | `derive/src/output.rs` is split so the test and the macro share one walk of the `Table`. |
| Doc comments | Textual doc comments are pinned; the pretty-printed implementation block is suppressed under `cfg(test)` — but `PrettyPlease` still runs, so `malformed_code_generation_result` still panics on malformed tokens. Empty doc attributes are kept as `#[doc = ""]`. |
| Compile tests | Live in a new non-published workspace member `compile-tests/`, because generated code needs `crate::spacetimedsl::…` and `derive` cannot depend on `spacetimedsl`. |
| Known defect | The `Option<Option<Timestamp>>` + `#[unique]` failure is pinned as a passing `trybuild` **compile_fail** case, so it turns red when step 4 fixes it. No `#[ignore]`d tests. |
| `compile_error_checks` | `HashSet<Ident>` → `BTreeSet<Ident>` (semver break, accepted at 0.22). A `Vec` was rejected: both `for_referenced_by` and `for_foreign_key` insert the same ident twice, so a `Vec` would emit duplicate `pub trait X {}`. |
| Toolchain | Repo-wide `rust-toolchain.toml` pin, so `trybuild` `.stderr` files are stable. |
| CI | `spacetime start &`, then `./x.sh unit-test`, then `./x.sh test`. No `sleep` — the unit tests give SpacetimeDB time to boot. Re-add the sleep only if publishing actually fails. |

---

## Step 0 — Spike (do this first, ~15 minutes)

**Unverified assumption:** `spacetime_bindings_macro_input::table::TableArgs::parse`
(`derive-input/src/internal/integration.rs:59`) has never been run outside a real
proc-macro invocation in this repo. `derive-input` contains no `proc_macro::` calls,
so it should work under `proc_macro2`'s fallback — but prove it before building
anything on top.

Write a throwaway `#[test]` in `derive/src/lib.rs`:

```rust
let args: proc_macro2::TokenStream = syn::parse_str("plural_name = things").unwrap();
let item: syn::DeriveInput = syn::parse_str(
    "#[spacetimedb::table(accessor = thing, public)] pub struct Thing { #[primary_key] id: u64 }"
).unwrap();
let table = spacetimedsl_derive_input::api::Table::try_parse(args, &item).unwrap();
assert_eq!(table.rust_struct.name.to_string(), "Thing");
```

- **Passes** → continue with this plan unchanged.
- **Panics with "procedural macro API is used outside of a procedural macro"** →
  stop and re-plan. Fallback is moving the snapshot suite into `derive-input`
  (which loses coverage of `derive/src/output.rs`) or moving the `output` module
  into `derive-input`.

---

## Step 1 — Production prerequisites (one commit)

### 1.1 Make generated output deterministic

Four hash collections currently reach the emitted token stream, so the same input
produces different output text between runs.

| File / line | Change |
| --- | --- |
| `derive-input/src/api/dsl/table.rs:22` | `compile_error_checks: HashSet<Ident>` → `BTreeSet<Ident>`. Update the `HashSet::new()` at `derive-input/src/internal/dsl/table.rs:199`. Public API break, accepted. |
| `derive-input/src/api/dsl/foreign_key.rs:13` | Add `PartialOrd, Ord` to the derive list on `OnDeleteStrategy`. Additive, non-breaking. |
| `derive-input/src/internal/dsl/method.rs:279` | `columns_with_foreign_keys_by_table`: `HashMap` → `BTreeMap`, keyed by the table name's `String` so ordering is by name. Fixes the order of the generated `execute_on_delete_strategies_of_this_table_after_*` methods. |
| `derive-input/src/internal/dsl/method.rs:2749` | `columns_by_on_delete_strategies`: `HashMap` → `BTreeMap`. |
| `derive-input/src/internal/dsl/method.rs:2889` | `strategy_implementations`: drop the `HashMap` entirely. Build a `Vec<TokenStream>` by iterating `OnDeleteStrategy::iter()` (strum already yields declaration order) and looking each strategy up in `columns_by_on_delete_strategies`, emitting `TokenStream::default()` when absent. This removes the insert-then-overwrite dance at 2891–2908 as well. |

`method.rs:1543`, `2599`, `2614`, `2866`, `2881`, `3204`–`3206` are `HashMap`s
_inside_ `quote!` bodies — runtime data structures in the generated code, not
generator state. Leave them alone.

### 1.2 Split `derive/src/output.rs` so the test and the macro share one walk

Without this, the per-method snapshot test must re-implement the walk that
`output()` already performs — the exact drift `AGENTS.md` forbids (_One
authoritative source_).

```rust
pub(crate) struct GeneratedOutput {
    /// Compile-error checks, wrapper types, the accessor `impl`, the create-argument
    /// struct, and the hook traits — everything the macro emits that is not a DSL method.
    pub items_outside_dsl_methods: TokenStream,
    pub dsl_methods: Vec<GeneratedDslMethod>,
}

pub(crate) struct GeneratedDslMethod {
    pub method_name: Ident,
    pub tokens: TokenStream,
}

pub(crate) fn build(input: &Table, first_dsl_attribute: bool) -> syn::Result<GeneratedOutput>;

pub(crate) fn output(input: &Table, first_dsl_attribute: bool) -> syn::Result<TokenStream>;
// now just: build(..) then concatenate items_outside_dsl_methods and every dsl_methods entry,
// preserving today's emission order exactly.
```

Every `dsl_methods.push(...)` site in today's `output()` (`output.rs:32`–`88`)
gains the method name alongside the tokens. `get_column_dsl_methods`
(`output.rs:133`) currently merges two or three methods into one `TokenStream`;
change it to return `Vec<GeneratedDslMethod>` so each method stays separable.

Emission order must not change — verify by diffing `./x.sh debug` output before
and after.

### 1.3 Suppress the implementation block in doc comments under `cfg(test)`

`derive/src/output/doc_comment.rs`, in `implementation_section`, immediately after
the `PrettyPlease` call and before the `format!`:

```rust
    if cfg!(test) {
        return String::default();
    }
```

`if cfg!(test)` rather than `#[cfg(test)] return …;` — the attribute form makes
the trailing `format!` unreachable under the test cfg and trips rustc's
`unreachable_code` lint, which `cargo clippy --all-targets` compiles.

The `PrettyPlease::format_tokens(...).unwrap_or_else(|_| panic!(…))` above stays
untouched, so malformed emissions still panic through
`malformed_code_generation_result` during tests. `accessor.rs:17` and
`create_method_arg.rs:7` route through `implementation_doc_comment`, whose whole
string _is_ the implementation section — under test they emit `#[doc = ""]`,
which the snapshots record deliberately (a lost doc attribute stays visible).

### 1.4 Extract a `proc_macro2`-level entry point

`derive/src/lib.rs:10` `dsl()` takes `proc_macro::TokenStream`, which cannot be
constructed outside macro expansion. Move its body into:

```rust
fn expand_dsl_attribute(
    args: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream>
```

`dsl()` becomes: convert both inputs to `proc_macro2`, call
`ok_or_compile_error(|| expand_dsl_attribute(args, item))`. All existing logic —
the singleton detection at `lib.rs:20`, `inject_singleton_primary_key`, the
`derive_table_helper` push, `Table::try_parse`, `output::output`, the
`#derive_input` echo — moves verbatim into `expand_dsl_attribute`.

### 1.5 Fix the `x.sh` working-directory bug

`gen-x.sh:145` and `gen-x.sh:161` emit `cd ../..` only for PowerShell
(`[ "$shell" = "powershell" ] && cmd_cd ...`). The bash `test` case therefore
runs `cd examples/blackholio` while already inside `examples/test`, which fails,
and the following `spacetime publish --server local blackholio` then publishes the
**spacetimedsl** module under the name `blackholio`.

Nobody has hit this because CI inlines the commands today. Step 4 makes CI call
`./x.sh test`, so fix it: emit the `cd ../..` for both shells at both sites.

### 1.6 Pin the toolchain

New `rust-toolchain.toml` at the repo root:

```toml
[toolchain]
channel = "1.XX.0"          # whatever stable is current at implementation time
components = ["clippy", "rustfmt"]
targets = ["wasm32-unknown-unknown"]
```

Then remove `toolchain: stable` from the `actions-rust-lang/setup-rust-toolchain@v1`
step in `.github/workflows/test.yml`, so the file governs CI and local development
identically. Must be ≥ 1.85 (the workspace is `edition = "2024"`).

### Verification for step 1

- `cargo build --workspace` clean.
- `./x.sh debug` output identical to before, modulo the deliberate ordering changes.
- `spacetime start &` then `./x.sh test` — both modules publish, the `tester`
  reducer runs, `blackholio` publishes from the right directory.

---

## Step 2 — Snapshot tests (`derive` crate)

### Layout

```
derive/
  Cargo.toml                     # + [dev-dependencies] insta = "1"
  src/
    lib.rs                       # + #[cfg(test)] mod characterization_tests;
    characterization_tests.rs    # the harness
  tests/
    fixtures/<shape>.rs          # inert: cargo only auto-compiles tests/*.rs
    snapshots/<shape>/<StructName>/table.snap
    snapshots/<shape>/<StructName>/<method_name>.snap
```

`derive/Cargo.toml` already has `include = ["/src/**"]`, so `tests/` never ships
in the published package.

### Harness

For each fixture file:

1. Read it, parse as `syn::File`.
2. For each top-level `struct` item that carries a `dsl` / `spacetimedsl::dsl`
   attribute:
   - Take the **first** such attribute, extract its args, and remove it from the
     item — this is exactly what the item looks like when the real attribute macro
     receives it (attribute macros strip only themselves).
   - Call `expand_dsl_attribute(args, item_tokens)`.
   - Re-run `Table::try_parse` + `output::build` on the same input to obtain the
     `GeneratedOutput` halves for snapshotting.
3. Snapshot `items_outside_dsl_methods` as `table.snap`, formatted with
   `PrettyPlease` — plus, appended as a trailing comment block, the sorted list of
   generated DSL method names. The manifest means adding or removing a method
   fails the table snapshot, not just orphans a file.
4. Snapshot each `GeneratedDslMethod` as `<method_name>.snap`, formatted with
   `PrettyPlease`.

`insta::with_settings!` with `snapshot_path => "../tests/snapshots/<shape>/<Struct>"`,
`prepend_module_to_snapshot => false`, `omit_expression => true`.

**Multi-`#[dsl]` shape:** call `expand_dsl_attribute` a second time, feeding the
first call's echoed `#derive_input` back in as `item`. The second pass sees the
`derive(SpacetimeDSL)` attribute already present, so `first_dsl_attribute` is
`false` and wrapper types and accessors are suppressed. Snapshot both passes
(`pass_1/`, `pass_2/`).

### Determinism test

One extra `#[test]` that expands the `foreign_keys_*` fixture 50 times in the same
process and asserts every result string is identical. This proves step 1.1 rather
than assuming it — process hash seeds are randomized per run, so a single
comparison would not catch a regression.

### Fixtures (~20 shapes)

Hand-written, minimal, one feature isolated per file. Each names the branch it
covers in a leading comment.

| Fixture | Covers |
| --- | --- |
| `plain_table` | primary key + `#[auto_inc]`, no indices |
| `unique_single_column_index` | `#[unique]`, drives `get_one_option` / `update` / `delete_one` |
| `non_unique_single_column_index` | `#[index(btree)]`, drives `get_many` / `delete_many` |
| `unique_multi_column_index` | `#[dsl(unique_index(name = …))]` — the experimental path with the warning at `method.rs:988` |
| `non_unique_multi_column_index` | multi-column `index(accessor = …, btree(columns = […]))` |
| `direct_index` | `#[index(direct)]` + `#[unique]` |
| `string_index_column` | `String` classification path, `&str` argument shaping |
| `wrapper_created_unnamed` | `#[create_wrapper]` |
| `wrapper_created_named` | `#[create_wrapper(Name)]` |
| `wrapper_used` | `#[use_wrapper(X)]` |
| `wrapper_optional_index` | `Option<Timestamp>` + `#[create_wrapper]` + `#[index(btree)]` — the branch fixed in `81ada87`, unreachable through `examples/test` |
| `singleton` | `#[dsl(singleton)]`, injected `id: u8`, suppressed `get_all`/`get_count` |
| `foreign_key_and_referenced_by` | referenced + referencing struct in one file |
| `on_delete_error` / `on_delete_delete` / `on_delete_set_zero` / `on_delete_ignore` | one fixture per `OnDeleteStrategy`, each with its referenced table |
| `hooks_all_six` | before/after × insert/update/delete |
| `methods_disabled` | `method(update = false, delete = false)` |
| `timestamps` | `created_at` / `modified_at` auto-set columns |
| `scheduled_table` | `ScheduleAt` column + `scheduled(...)` |
| `multiple_dsl_attributes` | `first_dsl_attribute == false` path |
| `multiple_table_attributes` | the heuristic selector at `internal/integration.rs:66` |
| `delete_hooks_with_foreign_key_on_unique_index` | regression guard for [#138](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/138) (closed): delete hooks + `#[foreign_key]` on a `#[unique]` index previously generated invalid Rust |

Two closed bugs get permanent guards this way: [#138](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/138)
via the fixture above, and [#20](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/20)
(`#[index(direct)]` with `#[unique]` produced compilation errors) via the
`direct_index` fixture. The `compile_error_checks` emitted into every
`table.snap` also guard [#116](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/116).

Expect roughly 150–250 snapshot files and 10k–20k committed lines. That is normal
for generator characterization tests; `cargo insta review` makes the diffs
navigable.

---

## Step 3 — Compile tests (`compile-tests` crate)

New workspace member, added to `members` in the root `Cargo.toml`.

```
compile-tests/
  Cargo.toml           # name = "spacetimedsl-compile-tests", publish = false
                       # [dependencies] spacetimedsl (path = ".."), spacetimedb
                       # [dev-dependencies] trybuild = "1"
  src/lib.rs           # empty
  tests/
    compile_tests.rs   # trybuild driver
    ui/*.rs + *.stderr
```

Each `ui/*.rs` fixture is standalone: it starts with `::spacetimedsl::spacetimedsl!();`
and ends with `fn main() {}`.

### `compile_fail` cases

Errors already emitted as `syn::Error` (stable diagnostics):

- missing `plural_name` (`internal.rs:212`)
- `plural_name` on a singleton (`internal.rs:193`)
- `unique_index` on a singleton (`internal.rs:199`)
- singleton with a manually declared `id` field (`derive/src/lib.rs:147`)
- `before_update` / `after_update` hook with `method(update = false)` (`internal.rs:160`)
- `before_delete` / `after_delete` hook with `method(delete = false)` (`internal.rs:176`)
- missing `#[table]` attribute (`integration.rs:51`)
- singleton with more than one `#[table]` attribute (`integration.rs:16`)

Panics reachable from user input:

- foreign key columns referencing the same table with mismatched types (`method.rs:2762`)
- foreign key columns referencing the same table with mismatched paths (`method.rs:2777`)

`method.rs:160` ("should be a single column index") is treated as an internal
invariant with no fixture. If an attempt to reach it succeeds, record that
finding — it changes step 8's audit.

The two panic fixtures pin `custom attribute panicked` output. Step 8 converts
these to spanned `syn::Error`s; their `.stderr` files are expected to change then,
and that change is the point.

### The known defect (pinned as `compile_fail`)

`ui/wrapper_optional_unique_index.rs` — a column typed `Option<Timestamp>` with
`#[create_wrapper]` and `#[unique]`. It currently fails with:

```
`Option<Option<spacetimedb::Timestamp>>` doesn't implement `std::fmt::Display`
```

Pinned as a **passing** `compile_fail` case with a header comment stating that
this is a known SpacetimeDSL defect, that step 4 must fix it, and that fixing it
turns this test red — at which point it is converted to `t.pass()` and moved to
`ui/pass/`.

**Maintenance note to record in the file:** this `.stderr` pins a source location
inside `spacetimedb`'s own `macros.rs`, so it churns on every SpacetimeDB upgrade
independently of the rustc pin. Regenerating it with `TRYBUILD=overwrite` becomes
part of the SpacetimeDB upgrade routine.

---

## Step 4 — Wiring

### `gen-x.sh`

Add a `generate_unit_test` function emitting a `unit-test` case:

```
cargo test -p spacetimedsl_derive
cargo test -p spacetimedsl-compile-tests
```

Register it in `generate()` (after `generate_test`), add `"unit-test"` to the
PowerShell `ValidateSet` in `generate_header`, and add a line to
`generate_usage`. Regenerate both scripts by running `./gen-x.sh` — never edit
`x.sh` / `x.ps1` directly.

Apply the `cd ../..` fix from step 1.5 in the same pass.

### `.github/workflows/test.yml`

Replace the inlined spacetime block with:

```yaml
- name: Build & test SpacetimeDSL
  run: |
    spacetime start &
    ./x.sh unit-test
    ./x.sh test
```

No `sleep` — the unit tests give SpacetimeDB ample time to boot. Re-add
`sleep 2` only if publishing actually fails in CI.

Also remove `toolchain: stable` from the setup-rust-toolchain step (step 1.6).

Note this gives CI **blackholio** coverage it does not have today, since
`x.sh test` publishes and deletes both example modules.

CI currently swallows every failure with `|| echo "…"`. Leave that as-is —
tightening it is out of scope and belongs with the report's own follow-ups.

---

## Step 5 — Report surgery

### 5.1 Remove from `CODE_QUALITY_REPORT.md`

Delete the entire section **`### method.rs: unverified branches and an apparently
non-compiling emission`** (lines 280–331, including the `Note from developer
START` / `END` block and both error transcripts). Its verbatim content is
preserved in the appendix below.

### 5.2 Amend the intro

Lines 16–17 currently read:

> Neither of these could have been caught, because the crate has no automated
> tests at all. `examples/test` is a `cdylib` that exercises the macro by
> compiling against it; it proves the happy paths compile, it asserts nothing,
> and it does not cover branches that no example happens to trigger.

Replace with a statement that this was true when the report was written and that
[`.ai/plans/01-characterization-tests.md`](.ai/plans/01-characterization-tests.md)
closes it — keeping the historical explanation of _why_ the two defects survived.

### 5.3 Amend the `column_names_and_row_values` section

Delete its final sentence (line 124):

> Add a test pinning the exact message text for a single-column index, a
> multi-column index, and a direct index before changing anything, so the fix is
> provably a fix.

The snapshots produced by this plan already pin those three message texts. The
violation itself (the drifted builders, the stray leading comma) stays in the
report — step 4 of the resolution order fixes it.

### 5.4 Rewrite resolution-order item 1

Line 347 currently reads:

> 1. **Add characterization tests for the generator.** Snapshot the generated
>    output for the table shapes listed above, plus compile-failure tests for
>    invalid attribute usage. Nothing else in this list can be done safely first
>    — `AGENTS.md` requires tests before refactoring, and this module currently
>    has none.

Replace with a one-line pointer to this plan. Keep the numbering 1–14 intact so
items 2–14 keep their existing references.

---

## Commit sequence

1. **Production prerequisites** — step 1 in full. Verified by `./x.sh test`
   against a local SpacetimeDB and by a before/after diff of `./x.sh debug`.
2. **Snapshot harness + first fixture** — step 2 infrastructure plus
   `plain_table`, proving the pipeline end to end.
3. **Remaining fixtures** — the other ~19 shapes, plus the determinism test.
4. **Compile tests** — step 3, the new workspace member.
5. **Wiring** — step 4, `gen-x.sh` + CI.
6. **Report surgery** — step 5.

---

## Verification

```bash
cargo build --workspace
cargo test -p spacetimedsl_derive          # snapshots
cargo test -p spacetimedsl-compile-tests   # trybuild
cargo insta pending-snapshots              # must be empty
cargo insta test --unreferenced=reject     # no orphaned snapshot files
./x.sh unit-test
spacetime start &
./x.sh test                                # both example modules publish, tester runs
cargo fmt --all -- --check
```

Manual smoke test proving the net actually catches regressions:

1. Change one character inside any `quote!` in `method.rs` — e.g. rename a
   generated local in the `DeleteOne` body.
2. `cargo test -p spacetimedsl_derive` must fail with a diff naming exactly the
   affected method snapshots and nothing else.
3. `git checkout derive-input/` and confirm the suite is green again.

Determinism proof: `cargo test -p spacetimedsl_derive` five times in a row must
produce identical results, and the in-process 50-iteration determinism test must
pass.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| `TableArgs::parse` may require a real proc-macro context | Step 0 spike settles it before anything is built on top. Fallback: relocate the suite to `derive-input`. |
| Steps 1.1 and 1.2 are production changes made before any test exists | Both are behaviour-preserving by construction (collection ordering; a pure function split). The guard is `./x.sh debug` diffing plus `./x.sh test` against a live server. Isolated in commit 1 so they can be reverted alone. |
| `.stderr` churn | `rust-toolchain.toml` handles rustc. The SpacetimeDB-originated `.stderr` is documented as part of the upgrade routine. |
| Snapshot volume (~200 files) | Accepted. `cargo insta review` keeps diffs navigable; the alternative is losing coverage of exactly the branches steps 9–13 rewrite. |
| `compile_error_checks` semver break | Accepted at 0.22. Note it in the changelog for the next release. |

---

## Related GitHub issues

None are fixed by this plan — it adds tests, it does not change behaviour. Listed
so the fixtures are chosen with them in mind.

| Issue | Relation |
| --- | --- |
| [#138](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/138) (closed) | Regression guard added: `delete_hooks_with_foreign_key_on_unique_index` fixture. |
| [#20](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/20) (closed) | Regression guard added: `direct_index` fixture. |
| [#116](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/116) (closed) | Regression guard added: `compile_error_checks` appear in every `table.snap`. |
| [#22](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/22) (open) | `#[index(btree)]` on `Timestamp` fields — same area as the `wrapper_optional_index` fixture and the pinned `Option<Option<Timestamp>>` defect. Check whether the snapshots settle it. |
| [#98](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/98) (open) | Wants `prettyplease` instead of `rust-format` in doc formatting. Step 1.3 edits exactly that function; do **not** fold the swap in — it would change every snapshot at once. |
| [#44](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/44) (open) | Proposes removing unique multi-column indices. The `unique_multi_column_index` fixture pins a feature that may be deleted; its snapshots go with it. |
| [#43](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/43) (open) | Proposes removing the foreign-key / referenced-by feature. Six fixtures cover it; same caveat. |
| [#114](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/114) (open) | Renaming code concepts — overlaps report step 7 and the `try_parse` rename in step 8. |
| [#32](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32), [#60](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/60), [#35](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/35) (open) | Already referenced by `TODO`s in `method.rs`; report step 14 handles them. |

Repository discussions: only the default "Welcome to SpacetimeDSL Discussions!"
thread exists — nothing related.

---

## Explicitly out of scope

- Fixing the stray leading comma in `column_names_and_row_values` — report step 4.
- Fixing the `Option<Option<Timestamp>>` `Display` defect — report step 4. This
  plan only pins it.
- Converting user-input panics to spanned `syn::Error`s — report step 8.
- Any restructuring of `method.rs` itself — report steps 5–13.
- Tightening CI's `|| echo "…"` failure swallowing.

---

# Appendix — content moved verbatim out of `CODE_QUALITY_REPORT.md`

Preserved here per step 5.1 so nothing is lost when the section is deleted.

### `method.rs`: unverified branches and an apparently non-compiling emission

**Violates:** Testing & Verification (entire checklist); F.I.R.S.T Principles of Testing; Arrange, Act, Assert

The crate contains no unit tests. `examples/test` is a `cdylib` compiled against the macro; it demonstrates that the paths those examples exercise produce compilable output, and asserts nothing about the output's content. Any generated-code branch not reached by an example is entirely unverified.

At least one such branch appears to be broken. The wrapper-type option mapper generated for an optional, wrapper-typed index column emits a `match` whose final arm is terminated with a semicolon inside the braces and whose overall expression is not terminated — tokens that would not compile if a user ever triggered that path. This branch cannot have been exercised.

```rust
wrapper_type_option_to_wrapped_type_option_mapper = quote! {
    let #column_name = match #column_name.into() {
        None => None,
        Some(#column_name) => Some(Into::<#wrapper_type_ty>::into(#column_name).value()); // wrong ;
    } // missing ;
};
```

Note from developer START: The reason why this path was never emitted is that Spacetime **DB** currently doesn't support indices on types wrapped in an `Option<T>`. I have added a column to the "Test" table in the example which is an wrapped non-String Option and when I've added a index, it produced the currently implemented panic handler for such a case:

```rust
custom attribute panicked
message: 

Congratulations, you have found a bug in SpacetimeDSL!

We would be very pleased if you can create an issue in our GitHub repository: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/new

Please include your table definition as well as the following, malformed, code generation result - thank you very much!

impl < T : crate :: spacetimedsl :: WriteContext > crate :: spacetimedsl :: DSL < '_, T > { #[allow(clippy :: needless_lifetimes, clippy :: too_many_arguments)] pub fn get_tests_by_wrapped_timestamp_option < 'a > (& 'a self, wrapped_timestamp_option : & impl Into < Option < TestWrappedTimestampOption >>) -> impl Iterator < Item = Test > { use :: spacetimedsl :: Wrapper; use spacetimedb :: { CtxDbRead, CtxDbWrite, Table as _ }; let wrapped_timestamp_option = match wrapped_timestamp_option.into() { None => None, Some(wrapped_timestamp_option) => Some(Into :: < TestWrappedTimestampOption > :: into(wrapped_timestamp_option).value()), } self.db().test().wrapped_timestamp_option().filter(wrapped_timestamp_option) } }
```

Note that there is another bug which only appears when adding a `#[unique]` rather than a `#[index(btree)]` to the column:

```txt
`Option<Option<spacetimedb::Timestamp>>` doesn't implement `std::fmt::Display`
the trait `std::fmt::Display` is not implemented for `Option<Option<spacetimedb::Timestamp>>`
in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
required for `&Option<Option<spacetimedb::Timestamp>>` to implement `std::fmt::Display`
macros.rs(114, 33): Actual error occurred here
macros.rs(114, 33): Error originated from macro call here
```

This is a defect of SpacetimeDSL and must be traced to its origin, so that it will work flawlessly when SpacetimeDB implements indices for option types.

Note from developer END.

The `Update` path also carries an unanswered correctness question in a comment, noting that the `String` handling was written for single-column indices and asking whether it works for multi-column indices. That question has been left in the source rather than answered.

Recommendation: this is the highest-leverage item in the report, because every other recommendation here is a refactoring, and `AGENTS.md` requires tests before refactoring. Add tests that generate token streams for representative table shapes and assert on the output — snapshot tests over the formatted generated code are the usual fit for a proc-macro and make assertions binary without manual inspection. Cover at minimum: a plain table, a table with a unique single-column index, a unique multi-column index, a `String` index column, a wrapper-typed index column, an optional wrapper-typed index column (the suspected-broken branch), a singleton, and a foreign-key/referenced-by pair. Complement these with compile-failure tests for the user-input error cases so the diagnostics themselves are pinned.

Once the optional-wrapper branch has a test, confirm whether it is broken; if it is, the developers must decide whether to fix the emission or remove the branch as an unsupported combination that should instead produce a clear compiler error.

### Status update on the above (as of planning)

The non-compiling emission was **already fixed** in commit `81ada87`
(`method.rs:1343`) — the arm now ends with `,` and the `match` with `};`. The
fixture `wrapper_optional_index` pins that fixed output.

The **second** defect described in the developer note — `Option<Option<Timestamp>>`
not implementing `Display` when the column carries `#[unique]` rather than
`#[index(btree)]` — is still open. This plan pins it as a passing `trybuild`
`compile_fail` case; report step 4 fixes it.

### Moved from `method.rs`: `column_names_and_row_values` format-string construction

> Add a test pinning the exact message text for a single-column index, a
> multi-column index, and a direct index before changing anything, so the fix is
> provably a fix.

Satisfied by the `unique_single_column_index`, `unique_multi_column_index` and
`direct_index` fixtures.

### Moved from Recommended Resolution Order, item 1

> 1. **Add characterization tests for the generator.** Snapshot the generated
>    output for the table shapes listed above, plus compile-failure tests for
>    invalid attribute usage. Nothing else in this list can be done safely first
>    — `AGENTS.md` requires tests before refactoring, and this module currently
>    has none.
