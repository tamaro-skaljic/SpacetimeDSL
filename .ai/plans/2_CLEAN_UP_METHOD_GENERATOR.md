# Plan 02 — Clean up the DSL method generator

Addresses items 2–8 of the Recommended Resolution Order in
[`CODE_QUALITY_REPORT.md`](../../CODE_QUALITY_REPORT.md).

---

## Context

`derive-input/src/internal/dsl/method.rs` is 3454 lines and generates every DSL
method SpacetimeDSL emits. Items 9–13 of the resolution order are the structural
work — breaking up the god function `for_method`, introducing a generation
context, consolidating the drifted duplication, splitting the module along domain
seams. None of that is safe or even legible while the module still carries dead
scaffolding, three disagreeing copies of one user-facing message, sixty inlined
runtime paths and a dozen hand-copied hook emissions.

This plan removes that noise. It fixes the two defects the characterization
suite exposed, deletes what is provably unused, extracts the copies that are
genuinely identical, resolves column-type classification to one authoritative
answer, and repairs the signatures and the error strategy. After it, `for_method`
is still one function — but it is a function whose body is readable, whose
helpers have honest signatures, and whose failure modes reach the user as
compiler diagnostics instead of proc-macro panics.

The safety net is already in place:
[`1_ADD_CHARACTERIZATION_TESTS.md`](1_ADD_CHARACTERIZATION_TESTS.md) landed 318
snapshots over the generated output and 14 `trybuild` cases over the rejection
diagnostics. Every step below states up front whether it may move a snapshot,
and which ones.

### Two findings this plan records that the report does not

**The in-`quote!` markers are not emitted into generated code.** The report states
that `//TODO … #set_none_strategy` inside a `quote!` body reaches the source users
read. It does not: `quote!` consumes a token stream, and the Rust tokenizer drops
ordinary comments before `quote!` ever sees them. Verified — `TODO|FIXME` matches
0 of the 318 snapshots and 0 lines of `debug-helper/output/lib.expanded.rs`. Step 3
is therefore source hygiene only, with no user-visible effect. The
`#set_none_strategy` interpolations inside those comments are likewise inert.

**A user-reachable panic the report does not list.** `internal/db/column.rs:40`
rejects `#[index]`/`#[unique]` on a singleton's columns, but its loop only matches
the three single-column `IndexType` variants. A singleton carrying a *multi-column*
index is accepted, reaches the `multi_column_indices` loop in
`SpacetimeDSLTableMethods::try_parse`, and panics inside `for_method` on
`primary_key_column.spacetimedsl_column_wrapper_type.expect("Should have a wrapper
type")` — because a singleton's injected `id: u8` primary key deliberately has no
wrapper (`internal/dsl/column.rs:31` exempts it). The result is `custom attribute
panicked` with no span. Step 8 rejects it upstream instead.

---

## Locked decisions

Made explicitly by the developer during planning; do not re-litigate during
implementation.

| # | Decision |
| --- | --- |
| Scope | One plan file for resolution-order items 2–8. Items 9–17 are untouched and keep their numbers. |
| Report | Items 2–8 in the report's Recommended Resolution Order become one-line pointers to this file, exactly as item 1 points at plan 1. The violation sections they cover move into this file. |
| Commits | One commit per sub-change (the `### N.M` sections below), not one per step. |
| Gate | `./x.sh unit-test`, `./x.sh format` and `./x.sh test` all green before a step counts as done. `./x.sh test` needs a locally running `spacetime start`. |
| `additional_paths_to_use` | Delete the whole round trip, including the public field on `SpacetimeDSLMethod`. Accepted API break at 0.22, recorded here; no version bump inside this plan. |
| `get_referencing_table_trait_name` | Delete the helper, its discarded call, its inline comment and the PascalCase values that only feed it. Rationale goes in the commit message. |
| `strategy_before_all` | Keep. Add one line of comment naming it a deliberate empty slot symmetric with `strategy_after_all`. |
| Step 3 scope | Only the `set_none_strategy` blocks and their `//TODO … issues/32` markers. The other in-`quote!` FIXMEs stay for item 14. |
| Message format | `{ id : {} }` for one column, `{ a : {}, b : {} }` for several — the shapes `docs/DOCUMENTATION.md` already documents. Both builders are corrected in step 4; merging them stays item 11. |
| Option-wrapper defect | Fix the double-`Option` only, by splitting the branch on wrapper kind. Do **not** reject indices on `Option<T>` columns. |
| `Used` + `Option` coverage | Extend `wrapper_optional_index.rs` with a `#[use_wrapper]` `Option<T>` column rather than adding a second fixture. |
| Hook helper | One helper taking a call-body closure for the five `?`-style sites; the on-delete-strategy hooks keep a separate small helper because their `use` must escape the loop. |
| Wrapper path helper | `WrapperType::struct_name_or_path_tokens`, beside the existing impls in `internal/dsl/wrapper.rs`. One `expect` message: *"A non-singleton primary key column must have a wrapper type"*. |
| Runtime paths | Extract the recurring *constructions* (error variants, deletion-result values, internals calls), not bare path constants. |
| Type classification | One enum `ColumnTypeKind { String, UnsignedInteger, Optional, Other }` on `InternalColumn`, resolved once in `internal/column.rs`. |
| Classification rule | Match on `syn::Path`'s last segment, but only when the leading segments are empty or `std`/`core`/`alloc`. Unsigned integers stay bare-only. |
| Qualified spellings | `std::string::String`, `core::option::Option<T>` and friends are **accepted** (classified as their bare forms), not rejected. A fixture pins today's wrong output first. |
| Inner type | The `Optional` variant carries no payload. Issue 32 adds it when something reads it. |
| `SpacetimeDSLColumn.is_option` | Stays in the public API, but is fed from the classifier so the `starts_with("Option <")` rule disappears. |
| Clippy | `gen-x.sh` is changed so `format` runs `cargo clippy --workspace --all-targets --all-features`; `derive-input` has never been linted. Land this before step 7. |
| Split names | `create_method_column_parts` (returning a named struct) and `update_method_row_value_getter`. |
| Bool enums | `RowBinding`, `IndexUniqueness`, `ReferencingTables`; `strategy_by_row`'s last parameter renamed to `strategy_for_each_row`. |
| `try_parse` rename | `SpacetimeDSLTableMethods::generate`. |
| `syn::Result` | Kept, and finally used. |
| Error scope | Convert the two foreign-key `panic!`s; reject the singleton multi-column-index case upstream in `internal/db/column.rs`; `for_method` and `SpacetimeDSLColumnMethods::map` stay infallible. |
| Spans | New and converted errors use `syn::Error::new_spanned` on the offending column's field name. Existing `Span::call_site()` errors are left alone. |
| Reference checks | Two per-mode functions plus one shared private helper carrying the loop, the foreign-key lookup and the guard. |
| `CreateOrUpdate` / `Action` | `CreateOrUpdate` is deleted in step 8 — the two per-mode splits leave it with no callers. `Action` keeps its role with `#[derive(strum::Display)]` replacing its hand-written impl. |
| Ownership | `generate` keeps taking `SpacetimeDSLTable` by value and handing it back. Removing the `&mut` mutation is item 10. |

---

## Expected snapshot movement

A step that is output-preserving must finish with `cargo test -p spacetimedsl_derive`
green and **zero** pending snapshots. If a snapshot moves in one of those steps,
the change is wrong — do not accept the snapshot.

| Step | Output | May move |
| --- | --- | --- |
| 2 | preserving | nothing |
| 3 | preserving | nothing |
| 4 | **changing** | `wrapper_optional_index/**` (new column + the fix), `wrapper_optional_unique_index.stderr`, and every snapshot containing a `column_names_and_row_values` format string |
| 5 | preserving | nothing |
| 6 | **changing** | only the new `qualified_type_spellings` fixture, and only in its own second commit |
| 7 | preserving | nothing |
| 8 | **changing** | `foreign_keys_with_mismatched_types.stderr`, `foreign_keys_with_mismatched_paths.stderr`, plus one new `.stderr` for the singleton case |

---

## Step 2 — Resolve the dead code

### 2.1 Delete `additional_paths_to_use`

**Violates:** Unused scaffolding removed immediately; YAGNI; Optimize for Deletion

`additional_paths_to_use` is declared as an empty vector in `for_method`, passed
into `reference_integrity_checks_on_create_or_update`, returned from it completely
unchanged, reassigned from the returned tuple, and finally stored on the generated
method — where the downstream output stage faithfully emits `use <path> as _;` for
each entry. It is never pushed to, anywhere in the codebase. `for_referenced_by`
and `for_foreign_key` simply declare it as an empty vector and store it. The entire
round trip is a no-op that complicates two signatures and one return type.

The report asked whether the disuse is itself a bug — whether
`reference_integrity_checks_on_create_or_update` was supposed to collect the paths
of referenced tables so the generated code can `use` them. It is not. The
referenced-table calls that function generates go through inherent methods on the
DSL (`self.get_<table>_by_<pk>(…)`), which need no import, and the one import the
referential-integrity machinery genuinely needs — the referenced table's
compile-error check — is emitted explicitly and separately as
`use #referenced_table_path::#compile_error_check;` inside `for_referenced_by`
(today's `method.rs:2645`) and `for_foreign_key` (today's `method.rs:2925`). The
mechanism was replaced; the vector is its residue.

**Change**

- `derive-input/src/api/dsl/method.rs:7` — remove the `additional_paths_to_use`
  field from `SpacetimeDSLMethod`. Public API break, accepted.
- `derive-input/src/internal/dsl/method.rs` — remove the local at `660`, the two
  argument/reassignment pairs at `780`/`785` and `1175`/`1180`, the struct field at
  `2145`, the parameter at `2207`, the tuple element of the return type at `2211`,
  the tuple construction at `2354`, and the `let additional_paths_to_use = vec![];`
  at `2550` and `2815` with their struct fields at `2716` and `2950`.
  `reference_integrity_checks_on_create_or_update` then returns a plain
  `Vec<TokenStream>`, so the `let res = …; … = res.0; let … = res.1;` dance at both
  call sites collapses to one `let`.
- `derive/src/output/function.rs:122` and `:129` — remove the binding and the
  `#(use #additional_paths_to_use as _;)*` interpolation.

**Verify:** no snapshot moves. The interpolation expanded to nothing for every
fixture, so the emitted tokens are identical.

### 2.2 Delete `get_referencing_table_trait_name` and its discarded call

**Violates:** Unused scaffolding removed immediately; Optimize for Deletion; Boy Scout Rule

`for_referenced_by` computes `referencing_table_trait_name`, then discards it with
`let _ = &referencing_table_trait_name;` and an inline comment stating the trait is
no longer generated and an inherent `impl` is used instead. The helper that builds
the name is otherwise unreferenced, and the PascalCase conversions of the table
names exist only to feed it.

The comment is accurate, and `derive/src/output/function.rs` proves it: the
`MethodImplTarget::InternalDslInternals` arm emits
`impl crate::spacetimedsl::internal::DSLInternals { … }`, an inherent impl. No
per-table trait is generated anywhere.

**Change** (all in `derive-input/src/internal/dsl/method.rs`)

- Delete `get_referencing_table_trait_name` (`3418`–`3435`).
- Delete the call and the discard at `2648`–`2652` and `2660`, including the
  trailing comment — the rationale belongs in the commit message.
- Delete `singular_table_name_pascal_case` (`2540`) and
  `referencing_table_name_pascal_case` (`2629`), whose only consumer was that call.
  Note `singular_table_name_pascal_case` in `for_method` (`643`) is a *different*
  binding with a live consumer (`Create{…}` arg-struct naming) — leave it.

**Verify:** no snapshot moves.

### 2.3 Keep `strategy_before_all`, and say so

**Violates:** Unused scaffolding removed immediately; YAGNI; KISS

`strategy_before_all` is bound to an empty `quote! {}` and never reassigned, yet it
is interpolated into both output arms of `get_on_delete_strategy_implementation` as
a placeholder. It emits nothing.

**Developer decision: keep**, even though its presence violates YAGNI and KISS.
It is a deliberate slot, symmetric with `strategy_after_all`, which *is* populated.

**Change:** add one line above `method.rs:2978`:

```rust
// Deliberate empty slot, symmetric with strategy_after_all.
let strategy_before_all = quote! {};
```

**Verify:** no snapshot moves.

---

## Step 3 — Delete the commented-out `set_none_strategy` blocks

**Violates:** Unused scaffolding removed immediately; Optimize for Deletion;
Documentation & Communication Clarity (*Future ideas captured outside codebase*,
*Log blockers to future cleanups for retrospectives*)

The `SetNone` on-delete strategy is present as commented-out code in three
generator functions, plus `//TODO … #set_none_strategy` markers inside `quote!`
bodies. The work is tracked in
[issue 32](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32), and
`api/dsl/foreign_key.rs:31`–`38` already carries the full design note — what
`SetNone` means, and why it is blocked ("Because Option is currently not allowed on
primary_key and unique/btree indices this strategy isn't used and implemented
yet"). The commented code adds nothing that the enum's own doc comment and the
issue do not already carry, while making every surrounding function longer.

Correcting the report: these markers are *not* emitted into the generated source.
Comments never survive tokenization, so `quote!` cannot reproduce them. This is
source hygiene, not a user-visible change.

**Developer Decision:** Keep as is, no work to do in step 3, continue with step 4.

---

## Step 4 — Fix the defects the tests expose

### 4.1 One message format, in both builders

**Violates:** Don't Repeat Yourself (*One authoritative source for each business
rule*); Duplication Control & Reuse; Connascence of Algorithm

The format string that describes "these columns had these values", used in
`NotFoundError` and `UniqueConstraintViolation` messages, is built by imperative
string pushes in two places: once inside `for_method` (with a branch per
`IndexType`) and again inside `multi_column_index_checks`. The placeholder-to-
argument correspondence is maintained by hand, in parallel with a separately built
list of row-value getters.

The copies have drifted into three different shapes, and `docs/DOCUMENTATION.md`
documents a fourth — which is to say, it documents the intended one:

| | `docs/DOCUMENTATION.md` | `for_method` | `multi_column_index_checks` |
| --- | --- | --- | --- |
| one column | `{ entity_id : 1 }` (lines 1485 and 1512) | `{ , id : {}  }` | — |
| several columns | `{ parent_entity_id : 1, child_entity_id : 2 }` (line 1499) | `{ a : {} , b : {} }` | `a : {} , b : {}` |

In `for_method`, the opening brace is pushed first and then the single-column
branches push a *separator-prefixed* column segment, so a single-column index
produces a stray leading comma — and the trailing space from `"{} "` meets the
leading space of `" }}"`, so it also produces a double space. The multi-column
branch pushes its first column without the separator and is correct on that count,
but puts a space before every comma, which the documentation does not. And
`multi_column_index_checks` omits the braces entirely.

**Developer decision:** the documented shapes are authoritative. Both builders are
corrected here. Merging them into one builder that emits the format string *and*
the matching getter list as a single value stays item 11 — this step only makes
the two agree on what they emit.

**Change** (all in `derive-input/src/internal/dsl/method.rs`)

- `916`–`979`, the `for_method` builder:
  - `BTreeSingleColumn` / `HashSingleColumn` (`927`–`928`) and `Direct`
    (`975`–`976`): push `format!("{column} : ")` then `"{}"` — drop the leading
    `", "` and the trailing space.
  - `BTreeMultiColumn` / `HashMultiColumn` (`951`–`965`): push `"{}"` instead of
    `"{} "` for the first column and for each middle column. The last column
    already pushes `"{}"`. The `", {column} : "` separators stay.
- `2394`–`2425`, the `multi_column_index_checks` builder: push `"{{ "` before the
  first column and `" }}"` after the last, and make the same `"{} "` → `"{}"`
  change.

Resulting messages, for every shape:

```txt
{ id : {} }
{ row_id : {}, number : {} }
```

`reference_integrity_checks_on_create_or_update` already emits
`format!("{{ {} : {} }}", …)`, and the singleton paths already hard-code
`"{ id : 0 }"` — both are already in the target shape and need no change.

**Snapshot movement:** every snapshot carrying a `column_names_and_row_values`
format string. Read each diff: the only permitted changes are the removal of the
stray comma-and-space prefix on a single-column message, the removal of the space
before each separating comma, the removal of the doubled space before the closing
brace, and the addition of the surrounding braces on the
`multi_column_index_checks` messages. Nothing else.

### 4.2 Fix the doubled `Option` on wrapped optional index columns

**Violates:** Robustness & Reliability; Principle of Least Astonishment

`compile-tests/tests/ui/wrapper_optional_unique_index.rs` pins a column typed
`Option<T>` carrying both `#[create_wrapper]` and `#[unique]` as a compile failure.
Its `.stderr` shows the generator asking for
`Option<Option<spacetimedb::Timestamp>>: Display` and
`Option<Option<…>>: Borrow<Option<…>>`.

The cause is in `method.rs:1342`–`1362`, the `spacetimedsl_column_is_option` arm of
the index path, which treats both wrapper kinds alike although they do not wrap the
same thing:

- `#[create_wrapper]` on `Option<T>` generates `struct W { value: Option<T> }` —
  the whole field type, `Option` included (`internal/dsl/wrapper.rs:71`). So
  `W::value()` already returns `Option<T>`, and the `Some(…)` around it in the
  mapper produces `Option<Option<T>>`.
- `#[use_wrapper(W)]` on `Option<T>` reuses a wrapper the user wrote around `T`,
  so `W::value()` returns `T` and the `Some(…)` is exactly right.

`getter.rs` and `setter.rs` are unaffected: the `Created` arms there take and
return the `Option` directly rather than round-tripping through `.value()`.

The defect is not limited to unique indices — the non-unique path runs through the
same arm, and the `wrapper_optional_index` snapshot shows `Option<Option<Timestamp>>`
being handed to `.filter(…)`.

**Developer decision:** fix the doubling only. Do **not** reject indices on
`Option<T>` columns, and do **not** change what a created wrapper wraps. Both
fixtures will still fail to compile against SpacetimeDB, because `&Option<T>` does
not implement `FilterableValue` — that is a SpacetimeDB limitation, noted in commit
`81ada87`'s message, not something SpacetimeDSL can generate its way out of.
`wrapper_optional_unique_index` therefore stays a `compile_fail` case, with a
shorter `.stderr`.

**Change, in two commits**

1. Extend `derive/tests/fixtures/wrapper_optional_index.rs` with a second column —
   a `#[use_wrapper(…)]` `Option<T>` carrying an index — and snapshot it. This pins
   the `Used` arm, which the fix must leave untouched, before the branch is split.
   Update the fixture's module doc to say it now covers both wrapper kinds.
2. Split `method.rs:1342` on the wrapper kind:

   ```rust
   // WrapperType::Created wraps the whole Option, so value() already yields it.
   WrapperType::Created(_) => quote! {
       let #column_name = match #column_name.into() {
           None => None,
           Some(#column_name) => Into::<#wrapper_type_ty>::into(#column_name).value(),
       };
   },
   WrapperType::Used(_) => quote! {
       let #column_name = match #column_name.into() {
           None => None,
           Some(#column_name) => Some(Into::<#wrapper_type_ty>::into(#column_name).value()),
       };
   },
   ```

**Snapshot movement:** commit 1 adds `wrapper_optional_index` snapshots for the new
column. Commit 2 changes only the `Created` column's mapper in that fixture, and
regenerates `wrapper_optional_unique_index.stderr` with `TRYBUILD=overwrite`. The
`Used` column's snapshots from commit 1 must be byte-identical after commit 2 —
that is the point of doing them in this order. Update the `KNOWN DEFECT` header of
`wrapper_optional_unique_index.rs` to describe what now remains: the SpacetimeDB
`FilterableValue` limitation and `Option<T>: !Display`, not the doubled `Option`.

---

## Step 5 — Extract the mechanical duplication

Nothing in this step may change a single emitted token. Each sub-change is a pure
extraction, and the snapshots are the proof.

### 5.1 One hook emitter

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse

Every hook — before/after insert, before/after update, before/after delete — is
emitted by the identical shape: match the `Option` on the table's hooks, return
`TokenStream::default()` for `None`, and for `Some` destructure `trait_name` and
`function_name` and build a `quote!` that emits a `use self::<trait>;` followed by
a `DSLMethodHooks::<function>` call. The delete hooks in particular are re-written
verbatim in each delete-generating branch, including the singleton branch. There
are twelve such sites in the module.

They differ only in the call they wrap: `before_insert` rebinds the row,
`after_insert` takes `&entity`, the update hooks take the found value and the row,
the `DeleteMany` hooks wrap the call in a `for` loop, the `DeleteOne` and singleton
hooks call it once, and the on-delete-strategy hooks test `.is_err()` and
`break 'outer` instead of using `?` — and emit their `use self::<trait>;` *outside*
the per-row loop, into `strategy_for_before_hook` / `strategy_for_after_hook`.

**Change** (in `derive-input/src/internal/dsl/method.rs`)

```rust
/// `use self::<trait>;` followed by the call `build_call` produces, or nothing
/// when the table declares no such hook.
fn hook_tokens(
    hook: &Option<SpacetimeDSLMethodHook>,
    build_call: impl FnOnce(&Ident) -> TokenStream,
) -> TokenStream
```

covering the ten `?`-style sites (`797`, `810`, `1200`, `1225`, `1559`, `1574`,
`1796`, `1810`, `1980`, `1996`), and a second small helper for the two
strategy sites (`3081`, `3101`) returning the `use` and the call separately, so the
caller can keep hoisting the `use` out of the loop:

```rust
/// The `use self::<trait>;` and the `.is_err()` guard, separately, because the
/// import must escape the per-row loop the guard sits in.
fn strategy_hook_tokens(
    hook: &Option<SpacetimeDSLMethodHook>,
    build_guard: impl FnOnce(&Ident) -> TokenStream,
) -> (TokenStream, TokenStream)
```

The two `before_update`/`after_update` sites keep their found-value prelude in the
caller — only the `use` + call pair moves into the helper.

### 5.2 One wrapper-struct-path resolution

**Violates:** Don't Repeat Yourself; Law of Demeter; Duplication Control & Reuse

Resolving a `WrapperType` to its struct name or path — matching `Created` to
`wrapper_struct_name` and `Used` to `wrapper_struct_name_or_path`, then
`to_token_stream()` — is written out inline at three sites (`1529`, `1940`, `3029`),
each preceded by an `expect` on the primary key column's optional wrapper type, and
each with a differently worded expect message for the same condition: `"Should have
a wrapper type"`, `"should have a wrapper type"`, `"Wrapper Type should exist"`.

The inline matching also reaches through the primary key column into its wrapper
type into that wrapper's fields — a Law of Demeter violation that couples this
module to `WrapperType`'s internal variant layout, so adding a variant forces edits
here.

**Change**

- `derive-input/src/internal/dsl/wrapper.rs`, beside the existing `map` and
  `map_to_wrapped_type` impls:

  ```rust
  pub(in crate::internal) fn struct_name_or_path_tokens(&self) -> TokenStream
  ```

- The three sites become
  `primary_key_column.spacetimedsl_column_wrapper_type.as_ref().expect(…)
  .struct_name_or_path_tokens()`, with one message:
  `"A non-singleton primary key column must have a wrapper type"` — which is the
  invariant `internal/dsl/column.rs:31` actually enforces. Step 8 makes the one
  remaining way to reach that `expect` unrepresentable.

### 5.3 Constructors for the generated runtime surface

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse

Generated code references `crate::spacetimedsl::error::SpacetimeDSLError` (27×),
`crate::spacetimedsl::delete::DeletionResult` and `DeletionResultEntry` (20×),
`crate::spacetimedsl::DSLMethodHooks` (12×), `crate::spacetimedsl::internal::DSLInternals`
(5×), plus `error::Action`, `error::ErrorFrom` and `delete::OnDeleteStrategy`, as
literal paths written out at each use inside `quote!` bodies. Renaming or
relocating any of these runtime items means a text hunt through this module, with
no compiler assistance, because the paths only exist as tokens.

**Developer decision:** extract the recurring *constructions*, not bare path
constants — a helper that returns a whole `SpacetimeDSLError::NotFoundError { … }`
value carries more of the contract than one that returns a path.

**Change:** a new `derive-input/src/internal/dsl/generated_runtime.rs`, holding
`pub(in crate::internal) fn`s for each construction that recurs, with the path
literal appearing exactly once per item:

| Helper | Replaces |
| --- | --- |
| `not_found_error(table_name, column_names_and_row_values)` | 5 sites |
| `unique_constraint_violation(table_name, action, error_from, one_or_multiple, column_names_and_row_values)` | 3 sites |
| `generic_error(message_tokens)` | 4 sites |
| `reference_integrity_violation_on_delete(result_tokens)` | 2 sites |
| `reference_integrity_violation_on_create_or_update(table_name, action, column_names_and_row_values)` | 2 sites |
| `auto_inc_overflow(table_name)` | 1 site |
| `deletion_result(table_name, one_or_multiple, entries_tokens)` | 6 sites |
| `deletion_result_entry(table_name, column_name, strategy, row_value, child_entries)` | 4 sites |
| `dsl_internals_call(function_name, args)` | 3 call shapes |
| `dsl_method_hooks_call(function_name, args)` | used by 5.1's helpers |
| `error_result_type(ok_tokens)` | 4 return types |

Keep these as plain token constructors: they take already-built token streams and
splice them. They must not grow branching, or they become a second generator.

Scope is `method.rs`. The same literals in `internal/dsl/hook.rs`,
`internal/dsl/wrapper.rs`, `api/dsl/foreign_key.rs` and the separate `derive`
crate's `output/function.rs` and `output/hook.rs` are left alone — converting them
means widening `derive-input`'s public API, which is a decision for a later step.
Record it in the report as a remaining finding.

---

## Step 6 — Classify column types once

**Violates:** Connascence of Meaning; Robustness & Reliability; Code For The
Maintainer; Don't Repeat Yourself

Column types are classified throughout by rendering them to a token stream,
converting to `String`, and comparing text: equality against `"String"` at
`method.rs:504`, `591`, `1291`, `1902` and `2494`; `starts_with("Option")` at `475`,
`1146` and `2339`; and a match over `"u8" | "u16" | "u32" | "u64" | "u128"` at
`2333`. One of these sites calls `.trim()` first; the others do not. The same
classification question is asked repeatedly at different sites, each re-deriving
the answer. A tenth site,
`internal/dsl/column.rs:23`, asks the `Option` question a *fourth* way —
`starts_with("Option <")`, with the space `proc_macro2` inserts — and its answer is
what reaches `SpacetimeDSLColumn.is_option`, the getter and the setter.

This silently misclassifies any type the user writes differently from the expected
spelling — a fully qualified path, or a spacing the token renderer does not produce
identically — and the failure is not a diagnostic but wrong generated code.

The `.trim()` at `2332` is a red herring: `ToTokens::to_string` on a `syn::Path`
never produces leading or trailing whitespace, so it is and always was a no-op.
Dropping it changes nothing.

### 6.1 Pin today's behaviour for qualified spellings

New fixture `derive/tests/fixtures/qualified_type_spellings.rs`, with columns typed
`std::string::String` and `core::option::Option<T>` alongside their bare
equivalents, snapshotted **before** the classifier lands. Its snapshots record
today's wrong output — the qualified `String` reaching the generator as a plain
type rather than as `&str`, the qualified `Option` missing the `is_option`
handling — so 6.2 shows the fix as a reviewable diff rather than as a new file
nobody can compare against.

### 6.2 Resolve the classification onto `InternalColumn`

**Change**

- `derive-input/src/internal/column.rs`:

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub(in crate::internal) enum ColumnTypeKind {
      String,
      UnsignedInteger,
      Optional,
      Other,
  }
  ```

  resolved once where `InternalColumn` is built, and stored on it as
  `rust_field_type_kind`. The four kinds are mutually exclusive because every
  question the generator asks is asked of the *whole* type: `Option<String>` is
  `Optional`, not `String`, exactly as today.

  Resolution matches on the `syn::Path` rather than on rendered text: take
  `path.segments.last()`, and accept it only when the leading segments are empty or
  spell `std`, `core` or `alloc`. So `String`, `std::string::String`,
  `alloc::string::String`, `Option<…>`, `std::option::Option<…>` and
  `core::option::Option<…>` all classify; a user's own `my_crate::String` stays
  `Other`, as it does today. Unsigned integers match the bare
  `u8`/`u16`/`u32`/`u64`/`u128` only.

- The ten classification sites read `internal_column.rust_field_type_kind`.
  `multi_column_index_checks`'s `column_type_by_name: HashMap<String, String>`
  (`2379`–`2389`) and `get_row_value_getter` (`2486`) disappear with it:
  `get_row_value_getter` takes the `InternalColumn` and reads the kind.

- `derive-input/src/internal/dsl/column.rs:23` — `SpacetimeDSLColumn.is_option`
  keeps its place in the public API but is fed from the same classifier, so the
  `starts_with("Option <")` rule is deleted rather than duplicated. The getter and
  setter keep their `is_option: bool` parameter; they need one bit, not the kind.

- The `Optional` variant carries no inner type. The TODO at today's `method.rs:2761`
  (issue 32) wants one — *"the type of the primary key values needs to be without
  option"* — but nothing reads an inner type yet, and issue 32 is also the place
  where the stripping rule can be tested.

**Snapshot movement:** only `qualified_type_spellings`, whose snapshots move from
the wrong output pinned in 6.1 to the right one. Every other fixture uses bare
spellings and must be untouched — if one moves, the classifier changed an answer
it should not have.

---

## Step 7 — Clean up signatures

Output-preserving throughout.

### 7.1 Lint `derive-input` at all

`./x.sh format` runs `cargo clippy --fix` in `derive/`, `examples/test/` and
`examples/blackholio/` — never in `derive-input/`. Clippy does not lint
dependencies, so `clippy::ptr_arg` has never fired on this crate, which is exactly
why eleven `&Vec<…>` / `&String` / nested-reference parameters accumulated in one
module unnoticed.

**Change:** `gen-x.sh` (which generates `x.sh` and `x.ps1` — do not edit those
directly) so the `format` case runs
`cargo clippy --workspace --all-targets --all-features --fix --allow-dirty` from
the repository root, replacing the three per-directory invocations. Regenerate
`x.sh`/`x.ps1`, run it, and fix everything it reports in this same commit. Land
this before 7.2 so the lint finds the signatures rather than the implementer
transcribing them from a list.

### 7.2 Split `process_columns_for_create_and_update_method`

**Violates:** Self-Documenting Code (descriptive identifiers, no abbreviations);
Principle of Least Astonishment; Single Responsibility Principle; Interface
Segregation Principle

The name is plural but the function handles exactly one column. It returns an
unlabeled four-element tuple of mostly-`Option` token streams, so every call site
must know the positional meaning of each slot — and the Update call site at
`method.rs:1118` destructures it as `(_, _, column_getter, _)`, discarding three
quarters of the work the function just did. Internally it serves two different
callers through a mode flag, with early returns that apply to only one mode, so
reading the Create path means skipping over Update concerns and vice versa.

**Developer decision: split into separate functions.** They share the wrapper-type
handling in this isolated function, but the Update path discards it anyway, so it
is relevant only to Create and can be dropped from the Update half. Note that
`wrapper_type_option_to_wrapped_type_option_mapper` is irrelevant for update paths
*only in this one function* — other functions have local variables with the same
name that are assigned in several ways, so this change is isolated to this function
and its two call sites.

The Update half also never reads `spacetimedsl_table` — the only uses of it are in
the Create arm (`is_singleton`, the two timestamp column names) — so it drops that
parameter too, and always produces a getter, so it returns `TokenStream` rather
than `Option<TokenStream>`.

**Change**

```rust
struct CreateMethodColumnParts {
    arg: Option<SpacetimeDSLArg>,
    wrapper_option_mapper: Option<TokenStream>,
    constructor_arg: Option<TokenStream>,
    constructor_arg_name: TokenStream,
}

fn create_method_column_parts(
    spacetimedsl_table: &SpacetimeDSLTable,
    internal_column: &InternalColumn,
) -> CreateMethodColumnParts

fn update_method_row_value_getter(internal_column: &InternalColumn) -> TokenStream
```

The `CreateOrUpdate` parameter disappears from both.

### 7.3 Named enums instead of boolean parameters

**Violates:** Connascence of Position; Principle of Least Astonishment; Code For
The Maintainer

`strategy_by_row(mut_row, is_unique_index, row_finder, strategy_by_row)` is called
with bare boolean literals at every site, so the call sites read as two unexplained
booleans followed by token streams. `for_foreign_key` similarly takes a bare
`has_referenced_bys` boolean positioned among several references. A transposition
of two booleans compiles and produces subtly wrong generated code. The final
parameter of `strategy_by_row` also shares its name with the function itself, so
the body reads ambiguously.

**Change**

```rust
enum RowBinding { Immutable, Mutable }
enum IndexUniqueness { Unique, NonUnique }
enum ReferencingTables { Present, Absent }

fn strategy_by_row(
    row_binding: RowBinding,
    index_uniqueness: IndexUniqueness,
    row_finder: &TokenStream,
    strategy_for_each_row: TokenStream,
) -> TokenStream
```

and `for_foreign_key`'s `has_referenced_bys: bool` becomes
`referencing_tables: ReferencingTables`, threaded on to
`get_on_delete_strategy_implementation`. The five `strategy_by_row` call sites and
the two `for_foreign_key` call sites in `try_parse` update accordingly.

### 7.4 Honest signature types

**Violates:** Hide Implementation Details; Principle of Least Astonishment; Code
For The Maintainer

Several signatures over-constrain their callers or leak representation:

- `&Vec<InternalColumn>` (`105`, `203`, `637`, `2206`, `2361`), `&Vec<TokenStream>`
  (`2507`) and `&Vec<Ident>` (`2208`) appear where a slice would do, forcing callers
  to own a `Vec`.
- `&String` (`2208`, `2506`) appears where `&str` would do.
- `columns_with_foreign_key: &Vec<&&Column>` (`2730`) and
  `columns_by_on_delete_strategy: Vec<&&&Column>` (`2964`) expose multiple levels of
  reference nesting that exist only as an artifact of how the collections were
  built upstream, not because the domain has three levels of indirection.
- `get_unique_multi_column_index_check` is declared
  `pub(in crate::internal::dsl::method)` (`2502`) — a visibility scoped to the
  module that defines it, which is exactly private, written in a way that implies
  otherwise.

**Change:** take `&[InternalColumn]`, `&[TokenStream]`, `&[Ident]` and `&str`;
flatten the nesting to `&[&Column]` and `Vec<&Column>` by collecting owned
references once at the point of construction in `try_parse` and in
`for_foreign_key`'s `columns_by_on_delete_strategies`; make
`get_unique_multi_column_index_check` a plain private `fn`. Clippy from 7.1 should
already be pointing at most of these.

### 7.5 Derive `Display` instead of hand-writing it

**Violates:** Don't Repeat Yourself; Simplicity & Right-Sized Solutions

`CreateOrUpdate` and `Action` overlap — `Action` contains `Create` and `Update`
alongside `Get` and `Delete`, so the smaller enum is a subset of the larger one
used for a narrower purpose. Both carry hand-written `Display` implementations that
do nothing but echo the variant name, even though `strum` is already a dependency
of this crate (with its `derive` feature enabled at the workspace root) and derives
exactly that. `CreateOrUpdate` is compared with `.eq(…)` at `method.rs:2216` and
destructured with `match` at `434` and `2253`, for no reason.

**Change:** replace both hand-written `impl Display` blocks with
`#[derive(strum::Display)]`, whose default is the variant name — identical output,
so `format_ident!("{action}")` at `2442` and `2513` is unaffected. Use `match`
consistently at the remaining `CreateOrUpdate` sites.

**On unifying the two enums:** `CreateOrUpdate` exists only to serve
`process_columns_for_create_and_update_method` and
`reference_integrity_checks_on_create_or_update`. 7.2 splits the first per mode and
8.1 splits the second, which leaves `CreateOrUpdate` with no callers.
**Developer decision:** delete it in step 8, keep `Action`. The per-mode splits
answer the unification question by deletion, without collapsing `Action`'s four
variants into a slot that only ever holds two of them.

---

## Step 8 — Split the reference-integrity checks and fix the error strategy

These belong together: the error-strategy decision determines whether the
`syn::Result` stays, and the entry point that would carry the errors is the one
being renamed.

### 8.1 Split `reference_integrity_checks_on_create_or_update` per mode

**Violates:** Interface Segregation Principle; Principle of Least Astonishment;
Command Query Separation

Beyond the `additional_paths_to_use` pass-through removed in 2.1, this function
takes `column_names_and_row_values_and_column_names: Option<(&String, &Vec<Ident>)>`
— a parameter that must be `Some` for the Update mode and is `None` for Create,
enforced only by an `expect` at runtime (`2274`). The signature therefore
advertises a contract it does not encode: callers must know which mode requires
which arguments. The parameter name itself concatenates two descriptions, and the
body immediately re-splits the tuple into locals with better names. `one_or_multiple`
is likewise read only by the Update arm.

**Change**

```rust
fn reference_integrity_checks_on_create(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
) -> Vec<TokenStream>

fn reference_integrity_checks_on_update(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
    column_names_and_row_values: &str,
    index_columns: &[Ident],
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> Vec<TokenStream>
```

`primary_key_column` is read only by the Update arm too, so it moves with it.

The per-column loop, the private-column skip, the foreign-key lookup and the
`if x != 0` / `if x.is_some()` guard wrapper are substantial and identical, so they
move into one private helper both call with their own check-body builder:

```rust
fn reference_integrity_checks(
    columns: &[InternalColumn],
    skip_private_columns: bool,
    build_check: impl Fn(&InternalColumn, &ForeignKey) -> TokenStream,
) -> Vec<TokenStream>
```

The guard wrapper reads `rust_field_type_kind` from step 6 rather than re-deriving
the unsigned/`Option` question. The runtime `expect` is gone: neither signature can
express the invalid combination.

With both call sites split, `CreateOrUpdate` has no users left — delete the enum
and its `Display` (see 7.5).

### 8.2 Rename `try_parse` and use the `syn::Result`

**Violates:** Principle of Least Astonishment; Robustness Principle;
Self-Documenting Code (descriptive identifiers); Hide Implementation Details

Three separate problems in one signature:

- The name says `try_parse`, but the function parses nothing. It consumes
  already-parsed models and generates methods. A reader looking for the parsing
  stage will land here and be misled.
- It returns `syn::Result`, promising fallibility, but no path in it ever produces
  an `Err`. Meanwhile the module's genuine error cases — a user writing foreign keys
  with mismatched types or mismatched paths — `panic!` instead of returning `Err`.
  The error channel that exists is unused, and the cases that need it bypass it.
- It takes `SpacetimeDSLTable` by value and returns it, purely to work around the
  `&mut` aliasing. This ownership shuffle is not expressing anything about the
  domain.

**Developer decision:** keep `syn::Result` and start using it. **The by-value
parameter stays for now** — removing it depends on removing the `&mut` mutation,
which is resolution-order item 10.

**Change:** rename `SpacetimeDSLTableMethods::try_parse` to
`SpacetimeDSLTableMethods::generate`, and update its one call site in
`derive-input/src/internal/table.rs:39`.

### 8.3 Separate user-input errors from internal invariants

**Violates:** Robustness & Reliability (*Log and surface malformed partner payloads
immediately*); Principle of Least Astonishment; Code For The Maintainer

The module panics in two distinct situations that it does not distinguish, across
46 `panic!`/`expect` sites:

- **Internal invariants** — `"… should already be processed!"`, `"When this code is
  called, it should be a single column index!"`, and the many `expect` calls on
  collection lookups. These are the generator asserting its own consistency.
- **User input errors** — mismatched foreign key column types (`2762`) and
  mismatched foreign key paths (`2777`) both `panic!` with a prose message. These
  are reachable purely by a user writing a valid-looking but unsupported attribute
  combination, and the result is a proc-macro panic with no span, rather than a
  compiler error pointing at the offending attribute.

Both are pinned by `compile-tests/tests/ui/foreign_keys_with_mismatched_types.rs`
and `foreign_keys_with_mismatched_paths.rs`. Their `.stderr` files show what the
user sees today: `custom attribute panicked`, the prose message demoted to a
`help:` note, and a span covering the whole `#[dsl]` attribute rather than the
offending column.

The audit the report asked for found one further user-reachable case, described in
the Context section above: a singleton carrying a multi-column index panics on the
missing primary-key wrapper.

**Developer decision:** convert the two foreign-key panics; reject the singleton
case upstream so `for_method` and `SpacetimeDSLColumnMethods::map` stay infallible;
leave the remaining `expect`s as panics but give each a message naming the invariant
it protects.

**Change**

- `for_foreign_key` (`2725`) returns `syn::Result<SpacetimeDSLMethod>`. Its two
  `panic!`s become
  `syn::Error::new_spanned(&column_with_foreign_key.rust_field.name, …)`, so the
  diagnostic underlines the column carrying the bad `#[foreign_key]` rather than
  the whole attribute. Its two call sites in `generate` (`303`, `314`) propagate
  with `?`; the surrounding `for_each` becomes a `for` loop so `?` is available.
  Note this diverges from the `Span::call_site()` used by every other diagnostic in
  the crate — deliberately, and only for the new ones. Migrating the existing ones
  is out of scope.
- `derive-input/src/internal/db/column.rs:40`–`59` — extend the singleton index
  rejection to the multi-column variants, so
  `#[index(btree(name = …, columns = [a, b]))]` on a singleton is refused with the
  same kind of message the single-column case already produces, spanned on the
  offending column. Add
  `compile-tests/tests/ui/multi_column_index_on_singleton.rs` with its `.stderr`.
- Restate the remaining `expect` messages to name their invariant — for example the
  wrapper-type `expect` unified in 5.2, which after this change is unreachable
  except through a generator bug.

**Snapshot movement:** regenerate `foreign_keys_with_mismatched_types.stderr` and
`foreign_keys_with_mismatched_paths.stderr` with `TRYBUILD=overwrite` and read both
diffs — each should turn from `custom attribute panicked` into a pointed
`error: …` on the offending field. The new singleton case gets a fresh `.stderr`.
No `insta` snapshot may move: no accepted table definition changes.

---

## Verification

Per sub-change commit:

```sh
./x.sh unit-test   # insta snapshots + trybuild diagnostics
./x.sh format      # cargo fmt --all + workspace clippy (from 7.1 onwards)
./x.sh test        # publishes both example modules against a local spacetime start
```

For an output-preserving sub-change, `cargo test -p spacetimedsl_derive` must leave
**zero** pending snapshots. Do not run `cargo insta accept`. If a snapshot moves,
the change is wrong.

For an output-changing sub-change, run `cargo insta review` and read every diff
against the "may move" list in the table above before accepting. Regenerate
`.stderr` files with `TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests`
and read those diffs too.

## What this plan does not cover

- Breaking up `for_method` into one generator per variant (item 9).
- The generation-context struct and removing the `&mut` mutation (item 10) — which
  is also what lets `generate` stop taking the table by value.
- Merging the two `column_names_and_row_values` builders, and the rest of the
  drifted duplication (item 11). Step 4 makes the two agree; it does not merge them.
- Consolidating singleton handling (item 12), splitting the module (item 13),
  triaging the remaining `TODO`/`FIXME` markers (item 14), and items 15–17.
- Converting the runtime-path literals outside `method.rs` — see 5.3.
- Migrating the crate's existing `Span::call_site()` diagnostics to spanned ones —
  see 8.3.
