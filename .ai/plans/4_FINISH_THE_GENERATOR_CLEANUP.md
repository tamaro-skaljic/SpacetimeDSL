# Plan 04 — Finish the generator cleanup

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development`
> (recommended) or `superpowers:executing-plans` to implement this plan sub-change by
> sub-change. Every action is a checkbox (`- [ ]`).

**Goal:** Resolve the last five findings the code quality review raised. The review's
report, `CODE_QUALITY_REPORT.md`, held nothing else, so it was deleted when this plan was
written and its remaining content lives here.

**Architecture:** Two behaviour changes first, on today's file layout, so their diffs read
as behaviour: a delete hook's error stops being swallowed during a cascade, and every
compiler diagnostic points at the token it names instead of at the whole attribute. Then
the generated-code/runtime contract becomes one published module. Then, strictly
output-preserving, the 3 565-line `method.rs` becomes a wiring module over fourteen domain
modules, and the four paired long field names on `SpacetimeDSLTableMethods` regroup behind
two small structs. Last, every comment that describes what the code used to be is rewritten
to describe what it is.

**Tech Stack:** Rust 2024, `syn` 2 / `quote` / `proc-macro2`, `insta` snapshots,
`trybuild` compile tests, `spacetimedb` 2.7.0.

**Spec:** the five finding sections and the developer decisions attached to them, moved out
of `CODE_QUALITY_REPORT.md`. Each is reproduced in full as the `**Violates:**` preamble of
the step that resolves it. The report is gone; this file is the record.

## Global Constraints

- Rust toolchain is pinned exactly in [`rust-toolchain.toml`](../../rust-toolchain.toml).
  The `.stderr` files are only reproducible against it.
- `spacetimedb` is pinned to `=2.7.0`; `spacetimedsl` and `spacetimedsl_derive` to `=0.22.0`.
- **No abbreviations** anywhere, in source or in generated identifiers.
  `InputOutput`, not `Io`. This applies to every name this plan introduces.
- Generated method names are the public API users call. This plan does not change a single
  one of them.
- `derive-input` must not depend on the `spacetimedsl` runtime crate — the dependency runs
  the other way. Everything the generated code refers to is emitted as tokens.
- `./x.sh test` needs a locally running `spacetime start`.

---

## Context

Three plans preceded this one.

- [`1_ADD_CHARACTERIZATION_TESTS.md`](1_ADD_CHARACTERIZATION_TESTS.md) built the safety net:
  329 `insta` snapshots over the generated output and 14 `trybuild` cases over the rejection
  diagnostics, all run by `./x.sh unit-test`.
- [`2_CLEAN_UP_METHOD_GENERATOR.md`](2_CLEAN_UP_METHOD_GENERATOR.md) landed in `0923f2c`:
  dead scaffolding, message formats, column-type classification, signatures, error strategy.
- [`3_SPLIT_THE_METHOD_GENERATOR.md`](3_SPLIT_THE_METHOD_GENERATOR.md) landed in
  `dc0d1f4`..`26f36a8`: the god function `for_method` became ten per-variant generators, the
  positional parameter bundle became `MethodGenerationContext`, and the `&mut` mutation of
  the table became returned `TableContributions`. It deliberately did **not** shorten the
  file.

What is left is what neither of them did, and it is exactly five things:

1. A referencing table's before/after-delete hook can fail during a cascade. Its error is
   discarded and the caller reports "An unknown error occurred after changing the database
   state!". The report classified this as a behavioural defect, not a cleanup.
2. Every `syn::Error` the workspace raises is built with `Span::call_site()`, so a message
   that names one offending column underlines the whole `#[dsl(…)]` attribute.
3. The fully qualified `crate::spacetimedsl::…` paths the generators emit are written as
   literal tokens in six files across two crates, and two of them are spelled two different
   ways for the same item.
4. `method.rs` is one 3 565-line file holding nine separate responsibilities.
5. Four fields of `SpacetimeDSLTableMethods` are long enough that `rustfmt` breaks `let mut`
   onto its own line, and two of them are parallel `Vec`s that are built pairwise.

Plan 3 unblocked 4 and 5 by separating the responsibilities inside the file. This plan moves
them into files.

---

## Locked decisions

Made explicitly by the developer during planning; do not re-litigate during implementation.

| # | Decision |
| --- | --- |
| Scope | The five findings above, in that order. Items 13 and 14 of the report's Recommended Resolution Order, plus the two "Beyond `method.rs`" findings. |
| Report | The remaining findings are woven into the steps that resolve them, as plan 3 did, not copied into an appendix. The Recommended Resolution Order is **dropped**, not moved: each plan already states which items it addresses. `CODE_QUALITY_REPORT.md` was deleted when this plan was written — see [step 7](#step-7--the-code-quality-report-is-already-retired). |
| Dangling links | Plans 1, 2 and 3 each linked to the report in their opening lines. Each link now points at this plan, with one sentence saying the report was deleted once empty. Nothing else in those files was touched. |
| Commits | One commit per sub-change (the `### N.M` sections), not one per action. |
| Order | Output-changing sub-changes first (steps 1–3), then the output-preserving work (steps 4–6), then the report (step 7). |
| Gate | `./x.sh unit-test`, `./x.sh format`, `./x.sh test` all green **and** `cargo insta pending-snapshots` empty before a sub-change counts as done. |
| Hook error transport | The cascade functions' `Err` payload becomes `OnDeleteStrategyFailure<Entries>` — the entries plus `error_from_hook`. The caller records it on the `DeletionResult` it builds. |
| Hook error type | `Option<Box<SpacetimeDSLError>>`, in both places. The `Box` is required: `DeletionResult` is reachable from `SpacetimeDSLError`, so an unboxed field makes the type infinitely sized. |
| Hook error display | `Display for DeletionResult` prints `Error from a hook: <error>`, a blank line, then today's CSV. With no hook error the output is byte-identical to today's. |
| Returned error | The existing wrapper stays — `SpacetimeDSLError::Error("Delete One Error: …")` — with the word **unknown** removed from its message. The hook's error reaches the user through the embedded `DeletionResult`. |
| Error priority | The hook's error wins: the first one raised is the one carried. |
| Span scope | **Every** `Span::call_site()` diagnostic in the workspace: `internal/column.rs`, `internal/db/column.rs`, `internal/db/table.rs`, `internal/dsl/column.rs`, `internal/dsl/table.rs`, `internal/integration.rs`, `internal.rs`, `derive/src/lib.rs`. |
| Table-wide spans | Diagnostics that name no column underline the **struct name**. |
| Flag spans | `internal.rs` changes its `Option<()>` flags to `Option<Span>` so the `#[dsl(…)]`-argument diagnostics underline the conflicting argument. |
| `.stderr` | Regenerated with `TRYBUILD=overwrite`, then every diff read. |
| Runtime module | `derive-input/src/api/runtime.rs`, public as `spacetimedsl_derive_input::api::runtime`. `internal/dsl/generated_runtime.rs` moves into it. |
| Runtime coverage | Every `crate::spacetimedsl::…` path any generator emits, in either crate. An acceptance grep proves no literal survives outside that file. |
| Runtime spelling | **Module-qualified**: `crate::spacetimedsl::error::SpacetimeDSLError`, `crate::spacetimedsl::delete::OnDeleteStrategy`. The two short spellings move to it, which changes 22 snapshots. |
| `derive` crate | Rewritten to call the published constructors in the same plan. |
| Paired fields | Two named pair structs, `OnDeleteStrategiesOfReferencingTables` and `OnDeleteStrategiesOfTheReferencedTable`. The second pair becomes `Vec<Pair>` instead of two parallel `Vec`s. |
| Split layout | `internal/dsl/method/` subdirectory. `method.rs` stays and becomes the wiring module. |
| Split granularity | Fourteen modules: one per DSL method family plus the support modules. Names in [step 5](#step-5--split-methodrs-into-domain-modules). |
| CRUD names | `create.rs`, `get.rs`, `update.rs`, `delete.rs` — `get`, not `read`, because every generator in it emits a method named `get_*` or `count_of_all_*`. |
| `OneOrMultiple` | Moves to `internal/dsl/one_or_multiple.rs`, a sibling of `method`, not into the split. |
| Entry points | `SpacetimeDSLColumnMethods::map`, `SpacetimeDSLTableMethods::generate`, `column_methods_for` and `update_method_for` stay in `method.rs`. |
| Renames | The split drops the `get_` prefix from the functions it moves, where the module name already carries the sense. Full list in [5.9](#59-drop-the-get_-prefixes). Nothing else is renamed. |
| Primary key invariant | The three `.expect(PRIMARY_KEY_WRAPPER_TYPE_INVARIANT)` sites collapse into one function in `method/context.rs`. |
| Module docs | `//!` comments on the modules whose boundary is not obvious: `context`, `index`, `hook_call`, `reference_integrity`, `referenced_by`, `foreign_key`, `on_delete_strategy`, `naming`. Not on `create`, `get`, `update`, `delete`, `singleton_table`, `one_or_multiple`. |
| Comment cleanup | Rewrite in present tense and state what must stay true; do not simply delete. Full list in [step 6](#step-6--remove-the-comments-that-describe-the-codes-past). |
| Markers | Only two are touched: the swallowed hook error is fixed (step 1), and the answered multi-column `String` question is **removed** (step 6.5). Every other `TODO`/`FIXME` and every commented-out `SetNone` block stays untouched. |
| Line references | Every `file.rs:NNN` reference in a comment is replaced by the symbol it means. |
| Docs | `docs/DOCUMENTATION.md` is updated for the new field, the CSV, the error examples, **and** gains a section on a hook failing during a cascade. |
| Versioning | Left to the release workflow. This plan does not touch `version = "0.22.0"`. |

---

## What this plan does not cover

An implementer who takes any of these on has widened the plan.

- **The `OnDeleteStrategy` copy between crates.** `derive-input/src/api/dsl/foreign_key.rs`
  and `src/delete.rs` hold hand-synchronised copies of the same enum, each with a comment
  telling you to update the other. The dependency direction forbids sharing one definition:
  `spacetimedsl` depends on `spacetimedsl_derive` depends on `spacetimedsl_derive_input`.
  Leave both copies and both comments.
- **Merging the delete paths.** Plan 3 settled `for_delete_one` / `for_delete_many`, the
  `OneOrMultiple` branch pairs in the four cascade builders, and `OneOrMultiple` serving both
  row arity and column arity, as "no change — every difference is intentional, required and
  deliberate". Step 5 puts `for_delete_one` and `for_delete_many` in one file for the first
  time. That is not an invitation.
- **The remaining `TODO`/`FIXME` markers.** The `try_update` markers linked to issue 60, the
  commented-out `SetNone` blocks linked to issue 32, the unnecessary clone in the create
  path, the create error that shows all columns where only the unique ones are relevant, the
  wrapper-type row-value getter shape, the `CtxDbRead`/`CtxDbWrite` import marker — all stay
  exactly as they are, inside `quote!` bodies and out.
- **Splitting `SpacetimeDSLTableMethods` further.** It keeps its fields; only the four long
  ones regroup.
- **Renaming generated methods or generated identifiers.** The long names users see and call
  are the public API. Only the Rust-side field names change.
- **Renaming anything the split does not move**, and any rename beyond the `get_` prefixes
  listed in [5.9](#59-drop-the-get_-prefixes). `naming.rs`'s functions keep their remaining
  words; `internal/column.rs`'s `get_primary_key_column_name` is not touched.
- **A version bump.**

---

## Expected snapshot and diagnostic movement

A sub-change marked *preserving* must finish with `./x.sh unit-test` green and **zero**
pending snapshots. If a snapshot moves there, the change is wrong — do not accept it.

Step 7 is already done and appears in no row.

| Sub-change | Output | May move |
| --- | --- | --- |
| 1.1 | changing | no `.snap` — the runtime crate only (`src/delete.rs`, `src/error.rs`, `src/lib.rs`) |
| 1.2 | **changing** | the 45 `.snap` naming `execute_on_delete_strategies_*`, and the `delete_*` method snapshots that build a `DeletionResult` — 66 files contain `DeletionResult {` |
| 1.3 | preserving | nothing — `examples/test` only |
| 2.1–2.3 | changing (diagnostics) | up to 14 `.stderr`; **no** `.snap` |
| 2.4 | — | the `.stderr` regeneration itself |
| 3.1 | preserving | nothing |
| 3.2 | **changing** | 20 `.snap` carrying `crate::spacetimedsl::OnDeleteStrategy`, 2 `table.snap` carrying `crate::spacetimedsl::SpacetimeDSLError` |
| 3.3 | preserving | nothing |
| 4 | preserving | nothing |
| 5.1–5.9 | preserving | nothing |
| 6.1–6.6 | preserving | nothing |

---

## Acceptance criteria

Each is one command, so "done" is not a judgement call. Run from the repository root after
step 7.

```sh
# 1. The report is gone and no link points at it.
#    (Plans 1 to 3 still name it in prose, as the record of where their items came from.)
test ! -e CODE_QUALITY_REPORT.md
rg -n '\]\([^)]*CODE_QUALITY_REPORT' .                                # 0 matches

# 2. No diagnostic is spanned at the call site any more.
rg -nU 'Error::new\(\s*(proc_macro2::)?Span::call_site\(\)' \
   derive-input/src derive/src                                        # 0 matches

# 3. No generator writes a runtime path as a literal.
rg -n 'crate::spacetimedsl' derive-input/src derive/src               # only api/runtime.rs

# 4. `method.rs` is the wiring module and nothing else.
wc -l derive-input/src/internal/dsl/method.rs                         # under 250

# 5. No module of the split is oversized.
wc -l derive-input/src/internal/dsl/method/*.rs                       # each under 600

# 6. The paired field names are gone from the Rust side.
#    (`method/naming.rs` still builds the *generated* identifiers, which do not change.
#     The generated doc comments spell the same words with spaces and capitals, so they
#     do not match this pattern.)
rg -n 'execute_on_delete_strategies_of_' derive-input/src derive/src \
   | rg -v 'method/naming.rs'                                         # 0 matches

# 7. No comment describes the code's past.
rg -n 'used to |this plan|was a SpacetimeDSL bug|today it does not|81ada87|is set later in' \
   derive-input/src derive/src derive/tests compile-tests src         # 0 matches

# 8. No comment points at a line number.
rg -n '\.rs:[0-9]+' derive-input/src derive/src derive/tests compile-tests src  # 0 matches

# 9. The hook error is no longer discarded.
rg -n 'error supplied by the hook being ignored' derive-input/src     # 0 matches
rg -n 'An unknown error occurred' derive-input/src                    # 0 matches

# 10. The suite is green with nothing pending.
./x.sh unit-test && ./x.sh format && ./x.sh test && cargo insta pending-snapshots
```

---

## Step 1 — Propagate the error a delete hook raises during a cascade

**Violates:** Robustness & Reliability (*Log and surface malformed partner payloads
immediately*); Code For The Maintainer; Principle of Least Astonishment

Moved out of `CODE_QUALITY_REPORT.md`, section *`method.rs`: unresolved `FIXME` and `TODO`
markers*, together with its developer decision:

> **Violates:** Documentation & Communication Clarity (*Future ideas captured outside
> codebase*, *Log blockers to future cleanups for retrospectives*); Refactoring & Change
> Containment
>
> The module carries a mix of markers. Some are linked to tracked issues — the `try_update`
> replacement, doc comments influenced by foreign-key attributes. Others are not linked to
> anything: an unnecessary clone in the create path, an error message that shows all columns
> where only the unique ones are relevant, row-value getters for wrapper types described as
> being built in the wrong shape, a hook error that is swallowed because propagating it would
> require a signature change, and an unanswered correctness question in the `Update` path
> asking whether the `String` handling, written for single-column indices, also holds for
> multi-column ones. That last question has an answer now: `string_index_column` pins the
> single-column shapes and `multiple_dsl_attributes` pins a unique multi-column index over
> `[database_id, name]` where `name` is a `String`. Its snapshots show the column arriving as
> `&str` and reaching `filter` as part of the tuple, so the question can be settled by
> reading them rather than by reasoning about the code.
>
> Several of these markers sit inside `quote!` bodies. They are *not* emitted into the code
> users read — the Rust tokenizer drops ordinary comments before `quote!` ever sees them, and
> `TODO|FIXME` matches none of the 335 snapshots nor `debug-helper/output/lib.expanded.rs`.
> They are still misplaced: a marker inside a `quote!` body reads as if it described the
> generated code, when it describes the generator.
>
> Recommendation: open issues for the unlinked markers and reduce each in-source marker to a
> one-line reference to its issue, so the rationale lives in the tracker where it can be
> prioritised. Move markers out of `quote!` bodies to the generator statement they actually
> concern. Note that the swallowed hook error is not merely a cleanup: it silently discards a
> user-supplied error, which is a behavioural defect and should be triaged as one rather than
> left as a comment.
>
> **Developer decision:** Only address the swallowed hook error by fixing it now. Do not
> remove any other `TODO|FIXME` markers or code around it which got commented out.

So the recommendation's first two sentences are declined: no issues are opened, no marker is
shortened, and no marker moves out of a `quote!` body. Only the defect is fixed, here, and
the one question the report itself settles is deleted in [6.5](#65-the-answered-marker).

The defect sits twice in `get_on_delete_strategy_implementation`, in the
`OnDeleteStrategy::Delete` arm, around the before- and after-delete hook calls of the
*referencing* table:

```rust
quote! {
    if #hook_call.is_err() {
        error = true;
        // FIXME: This results in the error supplied by the hook being ignored, we should propagate it back to the caller but that requires changing the function signature.
        break 'outer;
    }
}
```

`error = true` makes the generated cascade function return `Err(entries)`. The entries say
which rows were involved; nothing says why. The caller — `for_delete_one` or
`for_delete_many` — then builds:

```rust
SpacetimeDSLError::Error(format!(
    "Delete One Error: An unknown error occurred after changing the database state! \
     If the reducer running this doesn't return an error, the state changes are persisted \
     and you have problems now! Here is the deletion result: {error}"
))
```

The user wrote the error. The user does not get it back.

### 1.1 Carry the hook error on the deletion result

Runtime crate only. No generated code changes yet, so no snapshot moves.

- [x] **Add the failure type to `src/delete.rs`.**

```rust
/// What a cascade returns when it refused: the entries it built before it stopped, and the
/// error a delete hook raised, if one did.
///
/// `Entries` is a `Vec<DeletionResultEntry>` when one row of the referenced table was
/// deleted and a `HashMap<&PrimaryKeyValue, Vec<DeletionResultEntry>>` when several were.
#[derive(Debug)]
pub struct OnDeleteStrategyFailure<Entries> {
    pub entries: Entries,
    pub error_from_hook: Option<Box<SpacetimeDSLError>>,
}
```

Add `use crate::error::SpacetimeDSLError;` to the existing imports.

- [x] **Add the field to `DeletionResult` in `src/delete.rs`.**

```rust
#[derive(Debug)]
pub struct DeletionResult {
    pub table_name: Box<str>,
    pub one_or_multiple: OneOrMultiple,
    pub entries: Vec<DeletionResultEntry>,
    /// The error a delete hook of a referencing table raised while the cascade ran.
    ///
    /// Boxed because `SpacetimeDSLError::ReferenceIntegrityViolation` holds a
    /// `DeletionResult`, so an unboxed field would make both types infinitely sized.
    pub error_from_hook: Option<Box<SpacetimeDSLError>>,
}
```

- [x] **Show it in `Display for DeletionResult`.**

```rust
impl Display for DeletionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_from_hook {
            None => write!(f, "{}", self.to_csv()),
            Some(error_from_hook) => {
                write!(f, "Error from a hook: {error_from_hook}\n\n{}", self.to_csv())
            }
        }
    }
}
```

- [x] **Make `SpacetimeDSLError`'s own message go through that `Display`.**

`src/error.rs` formats the `ReferenceIntegrityViolationError::OnDelete` arm with
`deletion_result.to_csv()`, which bypasses the new line. Change that one expression to
`deletion_result`:

```rust
ReferenceIntegrityViolationError::OnDelete(deletion_result) => {
    let one_or_multiple_rows = match deletion_result.one_or_multiple {
        OneOrMultiple::One => "a row",
        OneOrMultiple::Multiple => "multiple rows",
    };

    format!(
        "Reference Integrity Violation Error while trying to delete {one_or_multiple_rows} in the `{}` table because of:\n\n{}",
        &deletion_result.table_name, deletion_result
    )
}
```

With no hook error this is byte-identical to today's output.

- [x] **Re-export the new type** in the two flat lists inside the `spacetimedsl!` macro in
  `src/lib.rs`, beside `DeletionResult` and `DeletionResultEntry` — once in the module body
  and once in `prelude`:

```rust
pub use ::spacetimedsl::delete::{
    DeletionResult, DeletionResultEntry, OnDeleteStrategy, OnDeleteStrategyFailure,
};
```

- [x] **Run the gate.** `./x.sh unit-test && ./x.sh format && ./x.sh test` and
  `cargo insta pending-snapshots`. Nothing may be pending: this sub-change generates nothing.

- [x] **Commit.**

```sh
git add src/delete.rs src/error.rs src/lib.rs
git commit -m "feat: carry a cascade's hook error on the deletion result"
```

### 1.2 Raise it, forward it, and return it

All in `derive-input/src/internal/dsl/method.rs` and
`derive-input/src/internal/dsl/generated_runtime.rs`, on today's layout.

- [x] **Add the runtime constructors** to `internal/dsl/generated_runtime.rs`:

```rust
/// `error::SpacetimeDSLError`, as a type rather than a value.
pub(in crate::internal) fn spacetimedsl_error_type() -> TokenStream {
    quote! {
        crate::spacetimedsl::error::SpacetimeDSLError
    }
}

/// `delete::OnDeleteStrategyFailure { entries, error_from_hook }`, the `Err` payload of
/// every generated cascade function.
pub(in crate::internal) fn on_delete_strategy_failure(
    entries: &impl ToTokens,
    error_from_hook: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::OnDeleteStrategyFailure {
            entries: #entries,
            error_from_hook: #error_from_hook,
        }
    }
}

/// `delete::OnDeleteStrategyFailure<#entries_type>`, as a type rather than a value.
pub(in crate::internal) fn on_delete_strategy_failure_type(
    entries_type: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::OnDeleteStrategyFailure<#entries_type>
    }
}

/// `let mut error_from_hook: Option<Box<SpacetimeDSLError>> = None;`
///
/// Annotated rather than inferred: a table whose strategies never assign to it would
/// otherwise leave the type ambiguous.
pub(in crate::internal) fn error_from_hook_declaration() -> TokenStream {
    let error_type = spacetimedsl_error_type();

    quote! {
        let mut error_from_hook: Option<Box<#error_type>> = None;
    }
}
```

- [x] **Give `deletion_result` an error argument.** `generated_runtime::deletion_result`
  gains a fourth parameter, spliced as the new field:

```rust
pub(in crate::internal) fn deletion_result(
    table_name: &impl ToTokens,
    one_or_multiple: &impl ToTokens,
    entries: &impl ToTokens,
    error_from_hook: &impl ToTokens,
) -> TokenStream {
    quote! {
        crate::spacetimedsl::delete::DeletionResult {
            table_name: #table_name.into(),
            one_or_multiple: #one_or_multiple,
            entries: #entries,
            error_from_hook: #error_from_hook,
        }
    }
}
```

- [x] **Raise it.** In `get_on_delete_strategy_implementation`, replace both hook guards.
  The `FIXME` comment goes with them:

```rust
let (use_before_delete_hook_trait, before_delete_hook) = hook_use_and_call(
    &spacetimedsl_table.hooks.before_delete,
    |hook_function_name| {
        let hook_call =
            runtime::dsl_method_hooks_call(hook_function_name, &quote! { &dsl, &row });

        quote! {
            if let Err(error_raised_by_the_hook) = #hook_call {
                error = true;
                error_from_hook = Some(Box::new(error_raised_by_the_hook));
                break 'outer;
            }
        }
    },
);
```

The after-delete guard is the same shape with `hooks.after_delete`.

- [x] **Declare it in the two generated cascade functions.** In `for_referenced_by` and in
  `for_foreign_key`, emit `runtime::error_from_hook_declaration()` immediately before the
  existing `let mut error = false;`, and change the tail:

```rust
let error_from_hook_declaration = runtime::error_from_hook_declaration();
let failure = runtime::on_delete_strategy_failure(
    &quote! { entries },
    &quote! { error_from_hook },
);

// ... inside the emitted body:
quote! {
    #error_from_hook_declaration
    let mut error = false;

    // ...

    match error {
        false => Ok(entries),
        true => Err(#failure),
    }
}
```

- [x] **Widen both return types.** In `for_referenced_by`:

```rust
OneOrMultiple::One => {
    let deletion_result_entry_type = runtime::deletion_result_entry_type();
    let entries_type = quote! { Vec<#deletion_result_entry_type> };
    let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);
    return_type = quote! { Result<#entries_type, #failure_type> };
}
OneOrMultiple::Multiple => {
    let deletion_result_entry_type = runtime::deletion_result_entry_type();
    let entries_type = quote! {
        std::collections::HashMap<&'a #primary_key_column_type, Vec<#deletion_result_entry_type>>
    };
    let failure_type = runtime::on_delete_strategy_failure_type(&entries_type);
    return_type = quote! { Result<#entries_type, #failure_type> };
}
```

`for_foreign_key` gets the same treatment with
`referenced_table_primary_key_column_type` in place of `primary_key_column_type`.

- [x] **Forward it where one cascade calls another.** Three call builders unwrap the `Err`
  payload today and have to unwrap the new struct instead. The rule is the same in all
  three: take the entries out of the failure, append them as before, and adopt the failure's
  `error_from_hook` **only if none is held yet** — the first error raised wins.

  In `for_referenced_by`'s `strategy_calls`, `OneOrMultiple::One` arm:

```rust
quote! {
    match #referencing_table_call {
        Err(failure) => {
            let mut child_entries = failure.entries;
            entries.append(&mut child_entries);

            if error_from_hook.is_none() {
                error_from_hook = failure.error_from_hook;
            }

            error = true;
        },
        Ok(mut child_entries) => {
            entries.append(&mut child_entries);
        },
    };
}
```

  The `OneOrMultiple::Multiple` arm keeps its loop and binds `failure.entries` to the name
  the loop reads:

```rust
quote! {
    match #referencing_table_call {
        Err(failure) => {
            for (primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                entries.get_mut(&primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in entries.")).append(&mut child_entries);
            }

            if error_from_hook.is_none() {
                error_from_hook = failure.error_from_hook;
            }

            error = true;
        },
        Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
            for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                entries.get_mut(&primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in entries.")).append(&mut child_entries);
            }
        },
    };
}
```

  `get_referenced_table_function_call_for_strategy_implementation` gets the same `Err(failure)`
  shape, and its `on_error_handler` — built in the `ReferencingTables::Present` branch of
  `get_on_delete_strategy_implementation` — returns the failure instead of the entries:

```rust
let failure = runtime::on_delete_strategy_failure(
    &quote! { entries },
    &quote! { error_from_hook },
);

let on_error_handler = quote! {
    #create_entries_and_add_them_to_entries
    return Err(#failure);
};
```

- [x] **Return it from the DSL methods.** In
  `get_referenced_table_function_call_for_dsl_method`, bind the hook error as a local so the
  spliced `on_error_handler` can read it. `OneOrMultiple::One`:

```rust
quote! {
    match #referenced_table_call {
        Err(failure) => {
            let mut child_entries = failure.entries;
            deletion_result_entry.child_entries.append(&mut child_entries);

            let error_from_hook = failure.error_from_hook;

            #on_error_handler
        },
        Ok(mut child_entries) => {
            deletion_result_entry.child_entries.append(&mut child_entries);
        }
    };
}
```

  `OneOrMultiple::Multiple` keeps its loop over `failure.entries` and binds
  `let error_from_hook = failure.error_from_hook;` after it, before `#on_error_handler`.

- [x] **Build two deletion results per delete generator.** `for_delete_one` and
  `for_delete_many` each construct their `DeletionResult` for both a success and a failure
  path; only the failure path has an `error_from_hook` in scope. In `for_delete_one`:

```rust
let single_entry_deletion_result = runtime::deletion_result(
    singular_table_name_as_string,
    &OneOrMultiple::One,
    &quote! { vec![deletion_result_entry] },
    &quote! { None },
);

let single_entry_deletion_result_with_error_from_hook = runtime::deletion_result(
    singular_table_name_as_string,
    &OneOrMultiple::One,
    &quote! { vec![deletion_result_entry] },
    &quote! { error_from_hook },
);
```

  The success `return Ok(#single_entry_deletion_result);` keeps the first. The
  `on_error_handler` and the `error_strategy` handler use the second. `for_delete_many` does
  the same for `deletion_result_from_entries`; its `empty_deletion_result` keeps `None`.

- [x] **Drop the word `unknown`.** Both messages become, verbatim:

```text
Delete One Error: An error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}
```

```text
Delete Many Error: An error occurred after changing the database state! If the reducer running this doesn't return an error, the state changes are persisted and you have problems now! Here is the deletion result: {error}
```

  The singleton delete generator's `count_mismatch_error` is a different message and does not
  change.

- [x] **Run `./x.sh unit-test`** and expect failures in the fixtures listed in the movement
  table. Review every diff with `cargo insta review`: a cascade function's signature, the
  `Err(failure)` destructuring, `error_from_hook: None` in every `DeletionResult` literal, and
  the two reworded messages. Nothing else.

- [x] **Run the rest of the gate.** `./x.sh format && ./x.sh test`.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/generated_runtime.rs \
        derive-input/src/internal/dsl/method.rs \
        derive/tests/snapshots
git commit -m "fix: return the error a delete hook raises during a cascade"
```

### 1.3 Prove it at runtime

Snapshots pin tokens, not behaviour. `examples/test` compiles and runs against a real
database, so it is where "the hook's error reaches the caller" is actually checked. Both
cascade arities get a case, because the one-row and many-row paths carry the error through
different containers — a `Vec` and a `HashMap`.

- [x] **Add the module** to `examples/test/src/lib.rs`, beside
  `spacetimedsl_cascade_delete_hook_repro`:

```rust
pub mod cascade_hook_error_test {
    use crate::spacetimedsl::SpacetimeDSLError;

    /// What the child's before-delete hook says when it refuses.
    pub const LOCKED_MESSAGE: &str = "this lock holder is locked";

    #[spacetimedsl::dsl(plural_name = lock_groups, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = lock_group)]
    pub struct LockGroup {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(LockGroupId)]
        #[referenced_by(path = crate::cascade_hook_error_test, table = lock_holder)]
        id: u64,

        /// A non-unique index, so the many-row delete method exists.
        #[index(btree)]
        batch: u64,
    }

    #[spacetimedsl::dsl(
        plural_name = lock_holders,
        method(update = false, delete = true),
        hook(before(delete))
    )]
    #[spacetimedb::table(accessor = lock_holder)]
    pub struct LockHolder {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(LockHolderId)]
        id: u64,

        #[index(btree)]
        #[use_wrapper(LockGroupId)]
        #[foreign_key(
            path = crate::cascade_hook_error_test,
            table = lock_group,
            column = id,
            on_delete = Delete
        )]
        group_id: u64,

        locked: bool,
    }

    #[spacetimedsl::hook]
    fn before_lock_holder_delete(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        row: &LockHolder,
    ) -> Result<(), SpacetimeDSLError> {
        match row.get_locked() {
            false => Ok(()),
            true => Err(SpacetimeDSLError::Error(LOCKED_MESSAGE.to_string())),
        }
    }
}
```

- [x] **Assert both arities in the `tester` reducer**, at the end of the existing body:

```rust
use crate::cascade_hook_error_test::{CreateLockGroup, CreateLockHolder, LOCKED_MESSAGE};

let one_row_group = dsl.create_lock_group(CreateLockGroup { batch: 1 })?;
dsl.create_lock_holder(CreateLockHolder {
    group_id: one_row_group.get_id(),
    locked: true,
})?;

match dsl.delete_lock_group_by_id(&one_row_group) {
    Ok(_) => {
        return Err(
            "Deleting a lock group whose holder's before_delete hook fails should fail!"
                .to_string(),
        );
    }
    Err(error) => {
        let error = error.to_string();
        if !error.contains(LOCKED_MESSAGE) {
            return Err(format!(
                "The error the before_delete hook raised should reach the caller! Got:\n{error}"
            ));
        }
    }
};

let many_rows_group = dsl.create_lock_group(CreateLockGroup { batch: 2 })?;
dsl.create_lock_holder(CreateLockHolder {
    group_id: many_rows_group.get_id(),
    locked: true,
})?;

match dsl.delete_lock_groups_by_batch(&2) {
    Ok(_) => {
        return Err(
            "Deleting lock groups whose holders' before_delete hook fails should fail!"
                .to_string(),
        );
    }
    Err(error) => {
        let error = error.to_string();
        if !error.contains(LOCKED_MESSAGE) {
            return Err(format!(
                "The error the before_delete hook raised should reach the caller of the many-row delete! Got:\n{error}"
            ));
        }
    }
};
```

  **Do not** add an assertion that nothing was deleted. The parent row is deleted *before*
  the cascade runs its `Delete` strategy — that is exactly what "an error occurred after
  changing the database state" means. Use a fresh group for the second case, as above,
  because the first one is gone.

- [x] **Run the gate.** `./x.sh unit-test` must show **no** snapshot movement: `examples/test`
  is not a fixture. `./x.sh test` must publish, call `tester` and report no error.

- [x] **Commit.**

```sh
git add examples/test/src/lib.rs
git commit -m "test: check a cascade hook's error reaches the caller"
```

### 1.4 Document it

- [x] **Update `docs/DOCUMENTATION.md`:**
  - `#### DeletionResult` — add `pub error_from_hook: Option<Box<SpacetimeDSLError>>` to the
    shown struct, with the one-line explanation of why it is boxed, and note that the CSV is
    preceded by `Error from a hook: …` when it is `Some`.
  - `### Explicit Matching for ReferenceIntegrityViolation` — the surrounding prose says what
    the error carries; add the hook error to it.
  - `### Example Error Messages` — add a `Delete` example whose output starts with the
    `Error from a hook:` line, and correct the existing "unknown error" wording if it appears.
  - `### Error Handling in Hooks` — add a subsection:

```markdown
#### During a Cascading Delete

A `before_delete` or `after_delete` hook also runs when the row is deleted by a cascade, that
is when a referenced row is deleted and this table's `#[foreign_key(… on_delete = Delete)]`
removes the referencing rows.

If the hook returns an error there, the cascade stops and the delete method that started it
returns an error. The hook's own error is carried on the `DeletionResult` as
`error_from_hook`, and `Display` prints it above the CSV:

    Error from a hook: this lock holder is locked

    entry_id, parent_entry_id, table_name, column_name, strategy, row_value,
    1, 0, lock_holder, group_id, Delete, 7,

Rows deleted before the hook refused are not rolled back by SpacetimeDSL. Return the error
from your reducer so SpacetimeDB rolls the transaction back.
```

- [x] **Commit.**

```sh
git add docs/DOCUMENTATION.md
git commit -m "docs: document a hook failing during a cascading delete"
```

---

## Step 2 — Span every diagnostic on the thing it names

**Violates:** Code For The Maintainer; Principle of Least Astonishment

Moved out of `CODE_QUALITY_REPORT.md`, section *Beyond `method.rs`: every diagnostic is
spanned at the call site*, with its developer decision:

> Every `syn::Error` the crate raises today is built with `Span::call_site()` — in
> `internal/column.rs`, `internal/db/column.rs`, `internal/dsl/column.rs` and
> `internal/dsl/table.rs` — so each of the 14 `compile-tests/tests/ui/*.stderr` files
> underlines the whole `#[dsl(…)]` attribute, even when the message names one offending
> column. `syn::Error::new_spanned` on the field would point at it.
>
> [`2_CLEAN_UP_METHOD_GENERATOR.md`](2_CLEAN_UP_METHOD_GENERATOR.md) uses spanned errors for
> the diagnostics it adds, which makes the two conventions coexist until this is resolved.
>
> Recommendation: migrate the existing diagnostics to spanned errors wherever a span is
> available, in one pass, so every `.stderr` moves together and the resulting convention is
> uniform. (**developer decision:** Approved, should be done!)

The scope is wider than the four files the report names: `internal/db/table.rs`,
`internal/integration.rs`, `internal.rs` and `derive/src/lib.rs` carry the same convention.
All of them move.

Spanning an error on user tokens also removes the trailing
`= note: this error originates in the attribute macro …` line from the `.stderr`, so every
file changes shape, not only its underline.

### 2.1 Keep the spans of the `#[dsl(…)]` arguments

`derive-input/src/internal.rs` throws its spans away at parse time: `is_singleton`,
`before_insert_hook` and their eight siblings are `Option<()>`, so the checks that run
afterwards have nothing to point at.

- [x] **Change the six hook flags to `Option<Span>`.** In `try_parse_dsl`,
  `before_insert_hook`, `before_update_hook`, `before_delete_hook`, `after_insert_hook`,
  `after_update_hook` and `after_delete_hook` change from
  `x = Some(());` to `x = Some(meta.path.span());`, and their declarations become
  `let mut before_insert_hook: Option<Span> = None;` and so on. `check_duplicate` takes
  `&Option<T>` and is unaffected.

  `is_singleton`, `hooks`, `before_hooks`, `after_hooks` and `methods` keep `Option<()>`:
  their spans are never read — the two singleton diagnostics point at the argument they
  reject, not at `singleton` — and an unused binding would fail `./x.sh format`.

- [x] **Span the six checks on the argument that conflicts.**

```rust
if !update_method.unwrap_or(true) {
    if let Some(span) = before_update_hook {
        return Err(syn::Error::new(
            span,
            "Cannot have a `before_update` hook when the `update` method is disabled with `#[dsl(method(update = false))]`",
        ));
    }
    if let Some(span) = after_update_hook {
        return Err(syn::Error::new(
            span,
            "Cannot have an `after_update` hook when the `update` method is disabled with `#[dsl(method(update = false))]`",
        ));
    }
}
```

  The `delete` pair is the same shape. The two singleton checks use the span of the argument
  they reject — `name_plural`'s ident for `plural_name`, and the first unique index's name
  for `unique_index` — not the `singleton` span, because the offending argument is the one
  the message names.

- [x] **Span "PluralName must be set" on the arguments.** There is no `plural_name` token to
  point at, so use the whole argument list:

```rust
name_plural.ok_or_else(|| {
    syn::Error::new_spanned(
        args,
        "PluralName must be set in `#[dsl(plural_name = PluralName)]`",
    )
})?
```

  `__singleton_placeholder` keeps `Span::call_site()`: it is a synthesised identifier, not a
  diagnostic, and acceptance criterion 2 only matches `Error::new(… call_site …)`.

- [x] **Run `./x.sh unit-test`.** Four `.stderr` files fail. Leave them failing until 2.4.

- [x] **Commit.**

```sh
git add derive-input/src/internal.rs
git commit -m "refactor: keep the spans of the dsl attribute arguments"
```

### 2.2 Span the column and index diagnostics

Every one of these already has a user token in hand.

- [x] **`internal/db/column.rs`** — both errors name a column and hold `rust_field`:

```rust
return Err(Error::new_spanned(
    &rust_field.name,
    format!(
        "A #[primary_key] column must not be prefixed with the table's name! Use `{}` instead of `{}`.",
        // unchanged
    ),
));
```

  and, in the singleton check, the same `new_spanned(&rust_field.name, …)` for
  ``"`#[index]` and `#[unique]` are not allowed on singleton tables! Found index on column `{column_name}`."``.

- [x] **`internal/db/table.rs`** — the multi-column-index error names an index.
  `Index::map` copies `IndexArg::accessor`, which is a real identifier from the
  `#[spacetimedb::table(index(accessor = …))]` attribute, so:

```rust
return Err(Error::new_spanned(&index.name, format!(/* unchanged */)));
```

- [x] **`internal/dsl/column.rs`** — all three errors name a column and hold `rust_field`:
  `new_spanned(&rust_field.name, …)` for the `#[primary_key]`-needs-a-wrapper message and
  both `#[foreign_key]`-needs-`use_wrapper` messages.

- [x] **`internal/dsl/table.rs`** — eleven of the thirteen errors are raised inside
  `for field in &column_args.fields`, which holds a `SatsField` with
  `ident: Option<&syn::Ident>`, `vis: &syn::Visibility` and `ty: &syn::Type`. Span each on the
  part the message is about:

  | Message names | Count | Span on |
  | --- | --- | --- |
  | the column's visibility — "All columns in a table with disabled `update` DSL method should be private!", plus the four "should have `Visibility::Inherited`! Found: …" messages | 5 | `field.vis` |
  | the column's type — the two "should have the type `spacetimedb::Timestamp`…" messages | 2 | `field.ty` |
  | the column by name — the two "Multiple columns for …" messages, and "A column with name `modified_at` or `updated_at` requires the `update` method to be enabled…" | 3 | `field.ident` |

  For `field.ident`, unwrap with the existing expectation:
  `field.ident.expect("a named field has an identifier")`.

- [x] **Run `./x.sh unit-test`.** More `.stderr` files fail. Leave them.

- [x] **Commit.**

```sh
git add derive-input/src/internal/db/column.rs \
        derive-input/src/internal/db/table.rs \
        derive-input/src/internal/dsl/column.rs \
        derive-input/src/internal/dsl/table.rs
git commit -m "refactor: span the column and index diagnostics on the column they name"
```

### 2.3 Span the table-wide diagnostics on the struct name

Five diagnostics name no column: they reject the table as a whole. They underline the struct
name, which is where the fix goes.

- [x] **`internal/column.rs`** — "Your table should have a `#[primary_key]` column!".
  `try_parse` holds `rust_struct: &RustStruct`:

```rust
return Err(syn::Error::new_spanned(
    &rust_struct.name,
    "Your table should have a `#[primary_key]` column!",
));
```

- [x] **`internal/dsl/table.rs`** — the two `has_update_method == None` messages and the
  trailing `modified_at`/`updated_at` message are raised outside the field loop.
  `ColumnArgs` carries `original_struct_name: Ident`; span all three on
  `&column_args.original_struct_name`.

- [x] **`internal/integration.rs`** — the three messages hold the `&DeriveInput`:

```rust
return Err(Error::new_spanned(&item.ident, /* unchanged message */));
```

  `get_all_table_attributes` and `select_table_with_heuristics` each take the input under a
  different name; use `&input.ident` there.

- [x] **`derive/src/lib.rs`** — `"Singleton tables must be structs with named fields!"` holds
  the `&mut syn::DeriveInput`: `syn::Error::new_spanned(&derive_input.ident, …)`. The
  injected `id` field's `Ident::new(… Span::call_site())` is a synthesised identifier and
  stays.

- [x] **Commit.**

```sh
git add derive-input/src/internal/column.rs \
        derive-input/src/internal/dsl/table.rs \
        derive-input/src/internal/integration.rs \
        derive/src/lib.rs
git commit -m "refactor: span the table-wide diagnostics on the struct name"
```

### 2.4 Regenerate and read every `.stderr`

- [x] **Regenerate.** `TRYBUILD=overwrite cargo test -p spacetimedsl-compile-tests`.

- [x] **Read every diff.** For each file, confirm the new underline is on the thing the
  message names, and that the `= note: this error originates in the attribute macro …` line
  disappeared. Expected targets:

  | `.stderr` | should now underline |
  | --- | --- |
  | `after_delete_hook_without_delete_method` | the `delete` inside `hook(after(…))` |
  | `after_update_hook_without_update_method` | the `update` inside `hook(after(…))` |
  | `before_delete_hook_without_delete_method` | the `delete` inside `hook(before(…))` |
  | `before_update_hook_without_update_method` | the `update` inside `hook(before(…))` |
  | `foreign_keys_with_mismatched_paths` | unchanged — already spanned on the second column |
  | `foreign_keys_with_mismatched_types` | unchanged — already spanned on the second column |
  | `missing_plural_name` | the `#[dsl(…)]` argument list |
  | `missing_table_attribute` | the struct name |
  | `multi_column_index_on_singleton` | the index accessor in `#[table(index(…))]` |
  | `plural_name_on_singleton` | the `plural_name = …` argument |
  | `referenced_by_without_delete_method` | unchanged — raised by `ReferencingTable::try_parse`, already spanned |
  | `singleton_with_manual_id_field` | unchanged — already spanned on the field |
  | `singleton_with_multiple_table_attributes` | the struct name |
  | `unique_index_on_singleton` | the `unique_index(…)` argument, then the two indexed columns |
  | `wrapper_optional_unique_index` | unchanged — the errors come from SpacetimeDB, not from SpacetimeDSL |

  A file in the "unchanged" rows that moved anyway means something was spanned that should
  not have been. Investigate before accepting.

- [x] **Run the whole gate.** `./x.sh unit-test && ./x.sh format && ./x.sh test`, and
  `cargo insta pending-snapshots` empty — **no** `.snap` may have moved in this step.

- [x] **Commit.**

```sh
git add compile-tests/tests/ui
git commit -m "test: regenerate the diagnostics after spanning them"
```

---

## Step 3 — One definition of the generated-code / runtime contract

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse

Moved out of `CODE_QUALITY_REPORT.md`, section *Beyond `method.rs`: runtime paths in the
other generators*, with its developer decision:

> The fully qualified runtime paths that `method.rs` inlines — `crate::spacetimedsl::error::…`,
> `crate::spacetimedsl::delete::…`, `crate::spacetimedsl::DSLMethodHooks`,
> `crate::spacetimedsl::internal::DSLInternals` — are also written out as literal tokens in
> `derive-input/src/internal/dsl/hook.rs`, `derive-input/src/internal/dsl/wrapper.rs`,
> `derive-input/src/api/dsl/foreign_key.rs`, and in the separate `derive` crate's
> `output/function.rs` and `output/hook.rs`. `OnDeleteStrategy`'s `ToTokens` impl emits
> `crate::spacetimedsl::OnDeleteStrategy::…` while `method.rs` writes
> `crate::spacetimedsl::delete::OnDeleteStrategy::…` for the same type, so even the spelling
> is not consistent.
>
> [`2_CLEAN_UP_METHOD_GENERATOR.md`](2_CLEAN_UP_METHOD_GENERATOR.md) gives `method.rs` a
> single set of constructors for this surface, but deliberately stops at the module
> boundary: sharing them with the `derive` crate means widening `derive-input`'s public API.
>
> Recommendation: once the `method.rs` constructors exist and have settled, decide whether
> the generated-code/runtime-crate contract should be a published part of `derive-input`'s
> API so every generator targets one definition of it.
>
> **Developer decision:** It should be part of the `derive-input` crate's API.

### 3.1 Publish `api::runtime`

Output-preserving: a move plus a visibility change.

- [x] **Move the file.**
  `git mv derive-input/src/internal/dsl/generated_runtime.rs derive-input/src/api/runtime.rs`

- [x] **Declare it.** Add `pub mod runtime;` to the `pub mod api { … }` block in
  `derive-input/src/lib.rs`, and remove `pub mod generated_runtime;` from
  `derive-input/src/internal/dsl.rs`.

- [x] **Widen the visibility.** Every `pub(in crate::internal) fn` in the moved file becomes
  `pub fn`.

- [x] **Rewrite the module comment** so it addresses a generator author rather than this
  crate's internals:

```rust
//! The contract between the code SpacetimeDSL generates and the `spacetimedsl` runtime
//! crate it runs against.
//!
//! Every `crate::spacetimedsl::…` path any generator emits is written here exactly once, so
//! renaming or relocating a runtime item is a change to this file rather than a text hunt
//! through `quote!` bodies that no compiler checks. A crate building on
//! [`crate::api::Table`] emits the same paths by calling these, instead of spelling them
//! out and drifting from them.
//!
//! These are plain token constructors: they splice already-built token streams and take no
//! decisions. They must not grow branching, or they become a second generator.
```

- [x] **Fix the imports.** `internal/dsl/method.rs` imports it as
  `use crate::internal::dsl::generated_runtime as runtime;`. Change to
  `use crate::api::runtime;` and drop the `generated_runtime` entry from the `internal::dsl`
  import list.

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src
git commit -m "refactor: publish the runtime contract as api::runtime"
```

### 3.2 Cover every remaining runtime path

Output-changing: the two short spellings become module-qualified.

- [x] **Add the missing constructors** to `derive-input/src/api/runtime.rs`. Each is one
  `quote!`, no branching:

```rust
/// `error::OneOrMultiple::#variant`
pub fn one_or_multiple(variant: &impl ToTokens) -> TokenStream

/// `WriteContext`, the bound on every DSL method that writes.
pub fn write_context() -> TokenStream

/// `ReadContext`, the bound on every DSL method that only reads.
pub fn read_context() -> TokenStream

/// `DSL<'_, T>`, the receiver of every public DSL method.
pub fn dsl_type() -> TokenStream

/// `DSL<'_, T>`, behind a reference, as a cascade function's first argument takes it.
pub fn dsl_reference_type() -> TokenStream

/// `ReadOnlyDSL<'_, T>`, the second receiver a read-compatible method is emitted on.
pub fn read_only_dsl_type() -> TokenStream

/// `internal::DSLInternals`, as a type rather than a call.
pub fn dsl_internals_type() -> TokenStream

/// `DSLMethodHooks`, as a type rather than a call.
pub fn dsl_method_hooks_type() -> TokenStream

/// `Wrapper<#wrapped_type, #wrapper_type>`, the trait a generated wrapper implements.
pub fn wrapper_trait(wrapped_type: &impl ToTokens, wrapper_type: &impl ToTokens) -> TokenStream

/// `::spacetimedsl::Wrapper`, the `use` every method body opens with.
pub fn wrapper_trait_path() -> TokenStream

/// `use ::spacetimedsl::itertools::Itertools;`, the import every body that calls
/// `at_most_one`, `collect_vec` or `into_values().collect_vec()` opens with.
pub fn itertools_import() -> TokenStream
```

  `itertools_import` and `wrapper_trait_path` emit `::spacetimedsl::…`, not
  `crate::spacetimedsl::…`, so they fall outside the decision's literal wording and outside
  acceptance criterion 3. They are the same contract with a different prefix, written out at
  seven and two sites respectively, so they are included. If that is unwanted, drop these two
  constructors and leave those `quote!` bodies alone — nothing else in the step depends on
  them.

  plus `spacetimedsl_error_type`, `on_delete_strategy_failure`,
  `on_delete_strategy_failure_type` and `error_from_hook_declaration` from 1.2, which are
  already there.

- [x] **Adopt them in `derive-input`.**
  - `api/dsl/foreign_key.rs`: `OnDeleteStrategy::to_tokens` calls
    `crate::api::runtime::on_delete_strategy(&quote! { Error })` and so on for the four
    variants. **This changes the emitted spelling** from
    `crate::spacetimedsl::OnDeleteStrategy::Error` to
    `crate::spacetimedsl::delete::OnDeleteStrategy::Error`.
  - `internal/dsl/hook.rs`: the three `Result<…, crate::spacetimedsl::SpacetimeDSLError>`
    sites call `runtime::spacetimedsl_error_type()`, and the
    `&crate::spacetimedsl::DSL<'_, T>` site calls `runtime::dsl_reference_type()`. **This
    changes the emitted spelling** to `crate::spacetimedsl::error::SpacetimeDSLError`.
  - `internal/dsl/wrapper.rs`: the `impl crate::spacetimedsl::Wrapper<…>` site calls
    `runtime::wrapper_trait(…)`.
  - `internal/dsl/method.rs`: `OneOrMultiple::to_tokens` calls `runtime::one_or_multiple(…)`.
    It moves to `internal/dsl/one_or_multiple.rs` in
    [5.1](#51-the-shared-vocabulary-and-the-context) and keeps that call.
  - `internal/dsl/method.rs`: the seven `use ::spacetimedsl::itertools::Itertools;` sites
    call `runtime::itertools_import()`.

- [x] **Run `./x.sh unit-test`.** Exactly 22 snapshots move: 20 carrying an
  `OnDeleteStrategy` value and the two `table.snap` of
  `delete_hooks_with_foreign_key_on_unique_index/ChildMarker` and `hooks_all_six/Potion`
  carrying a hook trait's return type. Review each: the only change in a file is a path
  gaining `delete::` or `error::`.

- [x] **Run the rest of the gate.**

- [x] **Commit.**

```sh
git add derive-input/src derive/tests/snapshots
git commit -m "refactor: emit every runtime path through api::runtime"
```

### 3.3 Adopt it in the `derive` crate

Output-preserving: the constructors emit the tokens those files already write.

- [x] **Add the dependency edge if it is missing.** `derive/Cargo.toml` already depends on
  `spacetimedsl_derive_input`; nothing to add.

- [x] **Rewrite `derive/src/output/function.rs`.** The five `quote! { crate::spacetimedsl::… }`
  literals become calls:

```rust
use spacetimedsl_derive_input::api::runtime;

// build_public
let mut output_variants = vec![MethodImplVariant::associated(
    runtime::write_context(),
    runtime::dsl_type(),
)];

if method.read_context_compatible {
    output_variants.push(MethodImplVariant::associated(
        runtime::read_context(),
        runtime::read_only_dsl_type(),
    ));
}
```

  and in `render_impl`:

```rust
let wrapper_trait_path = runtime::wrapper_trait_path();
let method_impl = quote! {
    use #wrapper_trait_path;
    use spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};
    #method_impl
};
```

  and the `MethodImplTarget::InternalDslInternals` arm:

```rust
let dsl_internals_type = runtime::dsl_internals_type();
let write_context = runtime::write_context();

quote! {
    impl #dsl_internals_type {
        #doc_comment
        pub fn #method_name<'a, T: #write_context>(
            #(#method_args),*
        ) -> #return_type {
            #method_impl
        }
    }
}
```

  The `// FIXME: We should probably only import one of CtxDbRead or CtxDbWrite …` marker
  stays exactly where it is.

- [x] **Rewrite `derive/src/output/hook.rs`** to build its trait bound from
  `runtime::write_context()`.

- [x] **Rewrite `derive/src/lib.rs`'s hook `impl`** to build
  `impl<T: #write_context> #trait_name<T> for #dsl_method_hooks_type` from
  `runtime::write_context()` and `runtime::dsl_method_hooks_type()`.

- [x] **Run the gate.** Acceptance criterion 3 must pass. **No** snapshot may move: the
  constructors emit the same tokens.

- [x] **Commit.**

```sh
git add derive/src
git commit -m "refactor: emit the derive crate's runtime paths through api::runtime"
```

---

## Step 4 — Regroup the paired method fields

**Violates:** Code For The Maintainer; *Prefer clarity over cleverness*; Simplicity &
Right-Sized Solutions

Moved out of `CODE_QUALITY_REPORT.md`, section *`method.rs`: generated identifiers and the
field names built from them*, with its developer decision:

> Field and local names such as
> `execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted`
> are long enough that the formatter is forced to break `let mut` onto its own line,
> separating the binding keyword from the name. Assignments to these names dominate the
> visual weight of the function that builds them.
>
> `AGENTS.md` forbids abbreviation, so shortening these by truncating words is not an option
> and should not be attempted.
>
> Recommendation: shorten by *namespacing*, not abbreviating. Group the related pair into a
> small struct — for example a type holding `after_one_row_was_deleted` and
> `after_multiple_rows_were_deleted` — so the shared prefix lives in the type name and each
> field keeps a fully spelled, unabbreviated name. This satisfies both the no-abbreviation
> rule and readability. Note that the *generated* function names (which users see and call)
> are a separate question and should not be changed without considering the effect on the
> public generated API. (**developer decision:** Approved, should be done!)

Taken before the split so the split moves the final shape rather than the old one.

- [x] **Add the two pair structs** to `derive-input/src/api/dsl/table.rs`:

```rust
/// The two cascade entry points a table earns when another table references it.
///
/// Both exist or neither does: a referenced table needs the one-row and the many-row entry
/// point, because a referencing table's foreign key does not know which delete method will
/// reach it.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfReferencingTables {
    pub after_one_row_of_this_table_was_deleted: SpacetimeDSLMethod,
    pub after_multiple_rows_of_this_table_were_deleted: SpacetimeDSLMethod,
}

/// The two strategy implementations a table earns for one table it references.
///
/// One pair per referenced table, which is why `SpacetimeDSLTableMethods` holds a `Vec` of
/// these rather than two parallel `Vec`s that could go out of step.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfTheReferencedTable {
    pub after_one_row_was_deleted: SpacetimeDSLMethod,
    pub after_multiple_rows_were_deleted: SpacetimeDSLMethod,
}
```

- [x] **Replace the four fields** on `SpacetimeDSLTableMethods`:

```rust
#[derive(Clone)]
pub struct SpacetimeDSLTableMethods {
    pub create: SpacetimeDSLMethod,
    pub get_all: Option<SpacetimeDSLMethod>,
    pub get_count: Option<SpacetimeDSLMethod>,
    pub on_delete_strategies_of_referencing_tables: Option<OnDeleteStrategiesOfReferencingTables>,
    pub on_delete_strategies_of_this_table: Vec<OnDeleteStrategiesOfTheReferencedTable>,
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
}
```

- [x] **Rewrite the producer.** In `SpacetimeDSLTableMethods::generate`, the two
  declare-then-assign-in-a-branch locals become one `match`, and the two `push`es become one:

```rust
let on_delete_strategies_of_referencing_tables = match spacetimedsl_table
    .referencing_tables
    .is_empty()
{
    true => None,
    false => {
        let (after_one_row, after_one_row_contributions) = for_referenced_by(
            &OneOrMultiple::One,
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column,
        );
        contributions.merge(after_one_row_contributions);

        let (after_multiple_rows, after_multiple_rows_contributions) = for_referenced_by(
            &OneOrMultiple::Multiple,
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column,
        );
        contributions.merge(after_multiple_rows_contributions);

        Some(OnDeleteStrategiesOfReferencingTables {
            after_one_row_of_this_table_was_deleted: after_one_row,
            after_multiple_rows_of_this_table_were_deleted: after_multiple_rows,
        })
    }
};

let mut on_delete_strategies_of_this_table = vec![];
```

  and, inside the loop over `columns_with_foreign_keys_by_table`:

```rust
on_delete_strategies_of_this_table.push(OnDeleteStrategiesOfTheReferencedTable {
    after_one_row_was_deleted: after_one_row,
    after_multiple_rows_were_deleted: after_multiple_rows,
});
```

- [x] **Rewrite the consumer.** In `derive/src/output.rs`, the four blocks collapse to two:

```rust
if let Some(strategies) = &input
    .spacetimedsl_methods
    .on_delete_strategies_of_referencing_tables
{
    dsl_methods.push(build_internal_dsl_method(
        &strategies.after_one_row_of_this_table_was_deleted,
    )?);
    dsl_methods.push(build_internal_dsl_method(
        &strategies.after_multiple_rows_of_this_table_were_deleted,
    )?);
}

// Two loops, not one: today every one-row method is emitted before any many-row method, and
// a single loop over the pairs would interleave them.
for strategies in &input.spacetimedsl_methods.on_delete_strategies_of_this_table {
    dsl_methods.push(build_internal_dsl_method(&strategies.after_one_row_was_deleted)?);
}

for strategies in &input.spacetimedsl_methods.on_delete_strategies_of_this_table {
    dsl_methods.push(build_internal_dsl_method(
        &strategies.after_multiple_rows_were_deleted,
    )?);
}
```

  The emission order is unchanged: one-row before many-rows, the referencing-tables pair
  before the referenced-table pairs. `table.snap`'s manifest is a sorted `BTreeSet`, so a
  reordering would not fail a snapshot — check it by reading, not by running the suite.

- [x] **Run the gate.** **No** snapshot may move: the methods, their names and their order
  are the same.

- [x] **Commit.**

```sh
git add derive-input/src/api/dsl/table.rs \
        derive-input/src/internal/dsl/method.rs \
        derive/src/output.rs
git commit -m "refactor: group the paired cascade methods behind two structs"
```

---

## Step 5 — Split `method.rs` into domain modules

**Violates:** Maximize Cohesion; Single Responsibility Principle; Optimize for Deletion
(*Keep modules small enough to rewrite in a week*); Separation of Concerns

Moved out of `CODE_QUALITY_REPORT.md`, section *`method.rs`: module size and placement*:

> The module holds, in one file: the `OneOrMultiple` / `Action` type definitions, the two
> public entry points (`SpacetimeDSLColumnMethods::map` and
> `SpacetimeDSLTableMethods::generate`), the method generators, foreign-key and
> referenced-by generators, on-delete strategy generation, multi-column-index uniqueness
> checks, and a family of identifier-naming helpers. These are separate responsibilities
> with separate change drivers — a change to naming conventions, a change to the delete
> protocol, and a change to argument typing all land in the same file.
>
> Recommendation: split along the domain axis that `AGENTS.md` prescribes for the Maximize
> Cohesion / Separation of Concerns conflict ("split in domain modules wiring together
> technical modules"). Natural seams: one module per DSL method family (create, read,
> update, delete), one module for referential integrity / on-delete strategies, one module
> for identifier naming, and a small module for the shared generation context.
>
> This was blocked by the god function `for_method`, which owned every variant's logic and
> could not be distributed across files while it existed. Plan 3 removed that blocker — the
> module now holds ten per-variant generators, a named generation context and a shared index
> analysis, which are the pieces this split moves — but it deliberately did not shorten the
> file, which is still about 3 570 lines. Take this next, together with the paired field
> names below.

**Every sub-change below is strictly output-preserving.** `./x.sh unit-test` must stay green
with zero pending snapshots throughout. Sub-change 5.9 is the only one that changes a name,
and every name it changes is private to the crate.

The target layout, with the items each module receives. The names below are the ones after
[5.9](#59-drop-the-get_-prefixes); sub-changes 5.1 to 5.8 move them under their current
names.

```text
derive-input/src/internal/dsl/one_or_multiple.rs   OneOrMultiple + ToTokens
derive-input/src/internal/dsl/method.rs            wiring: map, generate,
                                                   column_methods_for, update_method_for
derive-input/src/internal/dsl/method/
    context.rs              MethodGenerationContext, TableContributions,
                            primary_key_wrapper_type
    index.rs                IndexShape, IndexColumnArguments, index_column_arguments,
                            index_accessor, column_names_and_row_values,
                            documentation_on_columns, index_kind
    create.rs               for_create, CreateMethodColumnParts,
                            create_method_column_parts
    get.rs                  for_get_all, for_get_count, for_get_many, for_get_one
    update.rs               for_update, update_method_row_value_getter
    delete.rs               for_delete_one, for_delete_many
    singleton_table.rs      for_singleton_get, for_singleton_delete
    hook_call.rs            hook_use_and_call, hook_tokens
    reference_integrity.rs  Action, reference_integrity_checks,
                            reference_integrity_checks_on_create,
                            reference_integrity_checks_on_update,
                            multi_column_index_checks, row_value_getter,
                            unique_multi_column_index_check
    referenced_by.rs        for_referenced_by,
                            referenced_table_function_call_for_dsl_method
    foreign_key.rs          for_foreign_key
    on_delete_strategy.rs   on_delete_strategy_implementation, strategy_by_row,
                            referenced_table_function_call_for_strategy_implementation,
                            RowBinding, IndexUniqueness, ReferencingTables
    naming.rs               referenced_table_function_name,
                            referencing_table_function_name,
                            referenced_table_compile_error_check,
                            referencing_table_compile_error_check
```

`singleton_table.rs`, not `singleton.rs`: `internal/dsl/singleton.rs` already exists and
holds the injected-primary-key contract. Two modules named `singleton` in the same subtree,
one of which imports the other, would be a reading hazard.

**Visibility rule for the whole step:** an item used outside its own module gets
`pub(in crate::internal)`; an item used only inside it stays private. `method.rs` re-exports
what `internal` already imports from it, so `internal/column.rs` and `internal/table.rs` keep
their existing import paths:

```rust
pub(in crate::internal) use context::{MethodGenerationContext, TableContributions};
```

**Import rule:** a sibling module is reached as `super::<module>`, for example
`use super::hook_call::hook_tokens;`.

### 5.1 The shared vocabulary and the context

- [x] **Create `derive-input/src/internal/dsl/one_or_multiple.rs`** with `OneOrMultiple` and
  its `ToTokens` impl, the impl calling `crate::api::runtime::one_or_multiple`:

```rust
#[derive(Debug)]
pub(in crate::internal) enum OneOrMultiple {
    One,
    Multiple,
}

impl quote::ToTokens for OneOrMultiple {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            OneOrMultiple::One => crate::api::runtime::one_or_multiple(&quote! { One }),
            OneOrMultiple::Multiple => crate::api::runtime::one_or_multiple(&quote! { Multiple }),
        };
        tokens.extend(variant);
    }
}
```

  Declare it with `pub mod one_or_multiple;` in `internal/dsl.rs`.

- [x] **Create `derive-input/src/internal/dsl/method/context.rs`** with
  `MethodGenerationContext`, its `new`, and `TableContributions` with `merge` and `apply_to`,
  moved unchanged apart from the doc comment rewrite in [6.1](#61-the-source-doc-comments).
  Add the module comment:

```rust
//! What every method generator is handed, and what it hands back.
//!
//! [`MethodGenerationContext`] is a plain data carrier: the references a generator needs
//! plus the names it would otherwise re-derive. It must not grow generation methods.
//! [`TableContributions`] is the other direction — what a generator wants recorded on the
//! table, returned rather than written, so reordering two generator calls cannot change the
//! table.
```

- [x] **Collapse the primary-key invariant into one function** in the same file. Three sites
  — `for_delete_one`, `for_delete_many` and `on_delete_strategy_implementation` — write the
  same chain today:

```rust
/// The wrapper type of the primary key column, which `internal/dsl/column.rs` guarantees
/// exists: it rejects a `#[primary_key]` column that carries neither `#[create_wrapper]`
/// nor `#[use_wrapper(…)]`. The one exception, a singleton's injected `id: u8`, never
/// reaches a caller of this.
pub(in crate::internal) fn primary_key_wrapper_type(
    primary_key_column: &InternalColumn,
) -> TokenStream {
    primary_key_column
        .spacetimedsl_column_wrapper_type
        .as_ref()
        .expect(
            "A primary key column must be accompanied by `#[create_wrapper]` or `#[use_wrapper(crate::path::to::MyIdType)]`",
        )
        .struct_name_or_path_tokens()
}
```

  `PRIMARY_KEY_WRAPPER_TYPE_INVARIANT` disappears; the string lives inside the function.

- [x] **Create `derive-input/src/internal/dsl/method/` and `method.rs`'s `mod` block.** At
  this point `method.rs` declares `mod context;` and imports from
  `crate::internal::dsl::one_or_multiple::OneOrMultiple`.

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl
git commit -m "refactor: move the generation context and OneOrMultiple into their own modules"
```

### 5.2 The index analysis

- [x] **Create `method/index.rs`** with `IndexShape` and its `of`, `IndexColumnArguments`,
  `index_column_arguments`, `index_accessor`, `column_names_and_row_values`,
  `documentation_on_columns` and `index_kind`.

  `IndexShape`'s fields and `IndexColumnArguments`' fields become `pub(in crate::internal)`,
  because the four generator modules destructure them. `documentation_on_columns` and
  `index_kind` are read only by `IndexShape::of` and stay private.

  Module comment:

```rust
//! What an index tells the generators that look rows up through it.
//!
//! [`IndexShape`] is the per-index analysis: its name, its columns, whether it is the
//! primary key, and the prose fragments every generated doc comment ends with.
//! [`index_column_arguments`] is the per-column analysis: the method argument, the row-value
//! getter and the wrapper unwrapping for each column, built from one walk so the n-th of
//! each belongs to the same column.
```

- [x] **Update `internal/dsl/singleton.rs`'s doc comment**, which names
  `internal::dsl::method::column_names_and_row_values`. It becomes
  `internal::dsl::method::index::column_names_and_row_values`.

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl
git commit -m "refactor: move the index analysis into method/index.rs"
```

### 5.3 The hook-call helper and referential integrity

- [x] **Create `method/hook_call.rs`** with `hook_use_and_call` and `hook_tokens`, both
  `pub(in crate::internal)`. Module comment:

```rust
//! The `use self::<trait>;` and the call that every emitted hook is made of.
//!
//! [`hook_tokens`] joins them, which is what almost every site wants. [`hook_use_and_call`]
//! keeps them apart for the two sites that have to place the import themselves — before a
//! prelude that has to run first, or outside the loop the call sits in.
```

- [x] **Create `method/reference_integrity.rs`** with `Action`,
  `reference_integrity_checks`, `reference_integrity_checks_on_create`,
  `reference_integrity_checks_on_update`, `multi_column_index_checks`,
  `get_row_value_getter` and `get_unique_multi_column_index_check`.

  `Action`, the two `_on_create` / `_on_update` builders, `multi_column_index_checks` and
  `get_unique_multi_column_index_check` become `pub(in crate::internal)`.
  `reference_integrity_checks` and `get_row_value_getter` stay private.

  Module comment:

```rust
//! The checks a generated method runs before it writes: that every foreign key still points
//! at a row that exists, and that no unique multi-column index is about to be violated.
//!
//! SpacetimeDB enforces neither. A unique multi-column index is not a SpacetimeDB feature at
//! all, so its uniqueness is checked in generated code, and referential integrity is checked
//! on create and on update because the delete side is handled by the on-delete strategies.
```

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/method
git commit -m "refactor: move the hook-call helper and the integrity checks into their own modules"
```

### 5.4 The four DSL method families

One commit, because the four modules are one decomposition and each is a plain move.

- [x] **Create `method/create.rs`** with `CreateMethodColumnParts`,
  `create_method_column_parts` and `for_create`. Only `for_create` is
  `pub(in crate::internal)`.

- [x] **Create `method/get.rs`** with `for_get_all`, `for_get_count`, `for_get_many` and
  `for_get_one`, all `pub(in crate::internal)`.

- [x] **Create `method/update.rs`** with `update_method_row_value_getter` (private) and
  `for_update` (`pub(in crate::internal)`).

- [x] **Create `method/delete.rs`** with `for_delete_one` and `for_delete_many`, both
  `pub(in crate::internal)`.

  Do **not** merge the two, do **not** factor out their shared stage sequence, and do not
  reorder a statement inside either. Plan 3 settled every difference between them as
  intentional; putting them in one file is not a reason to revisit that.

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/method
git commit -m "refactor: move the create, get, update and delete generators into their own modules"
```

### 5.5 The singleton generators

- [x] **Create `method/singleton_table.rs`** with `for_singleton_get` and
  `for_singleton_delete`, both `pub(in crate::internal)`. Both read the injected-key contract
  from `crate::internal::dsl::singleton`.

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/method
git commit -m "refactor: move the singleton generators into method/singleton_table.rs"
```

### 5.6 The cascade

The largest piece, split by which side of the relationship it generates for.

- [x] **Create `method/naming.rs`** with the four generated-identifier builders, all
  `pub(in crate::internal)`. Module comment:

```rust
//! The identifiers that two tables in a foreign key relationship agree on by name.
//!
//! A referencing table calls a function it does not see the definition of, and imports a
//! trait the other table defines, so both sides have to build the same identifier from the
//! same two table names. Both sides build it here.
//!
//! These names are part of the generated API. Changing one breaks every module generated
//! against the previous name until it is regenerated.
```

- [x] **Create `method/referenced_by.rs`** with `for_referenced_by` and
  `get_referenced_table_function_call_for_dsl_method`, both `pub(in crate::internal)`.
  Module comment:

```rust
//! The referenced side of a foreign key: the two entry points a table earns when another
//! table declares `#[referenced_by]` on it, and the call its own delete methods make into
//! them.
//!
//! These fan out to every referencing table. The referencing side is
//! [`super::foreign_key`].
```

- [x] **Create `method/foreign_key.rs`** with `for_foreign_key`, `pub(in crate::internal)`.
  Module comment:

```rust
//! The referencing side of a foreign key: the function a table generates for each table it
//! references, which the referenced side calls when one of its rows is deleted.
//!
//! One function per referenced table, holding a match arm per on-delete strategy. The
//! referenced side is [`super::referenced_by`].
```

- [x] **Create `method/on_delete_strategy.rs`** with
  `get_on_delete_strategy_implementation`, `strategy_by_row`,
  `get_referenced_table_function_call_for_strategy_implementation`, `RowBinding`,
  `IndexUniqueness` and `ReferencingTables`.

  `get_on_delete_strategy_implementation` and `ReferencingTables` become
  `pub(in crate::internal)`; the other four stay private. Module comment:

```rust
//! What one on-delete strategy does to the rows that reference a deleted row.
//!
//! `Error` refuses, `Delete` removes them — and cascades further if they are themselves
//! referenced — `SetZero` clears the column, `Ignore` does nothing. Each is generated into
//! one arm of the match in [`super::foreign_key`].
```

  Do **not** merge the `match one_or_multiple` arm pairs here or in the two modules above.
  Plan 3 settled them as intentional.

- [x] **Run the gate.** No snapshot may move.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/method
git commit -m "refactor: move the cascade generators into their own modules"
```

### 5.7 The wiring module

- [x] **Reduce `method.rs`** to the `mod` declarations, the re-exports, the two entry-point
  `impl` blocks, `column_methods_for` and `update_method_for`, and the imports those need:

```rust
mod context;
mod create;
mod delete;
mod foreign_key;
mod get;
mod hook_call;
mod index;
mod naming;
mod on_delete_strategy;
mod reference_integrity;
mod referenced_by;
mod singleton_table;
mod update;

pub(in crate::internal) use context::{MethodGenerationContext, TableContributions};
```

- [ ] **Confirm the size.** `wc -l derive-input/src/internal/dsl/method.rs` under 250, and
  every file in `method/` under 600.

- [x] **Run the gate.** No snapshot may move.

- [ ] **Commit.**

```sh
git add derive-input/src/internal/dsl/method.rs
git commit -m "refactor: reduce method.rs to the wiring between the generator modules"
```

### 5.8 Update the call sites outside `method`

- [x] **Check the two importers.** `internal/column.rs` and `internal/table.rs` import
  `crate::internal::dsl::method::MethodGenerationContext`. The re-export in 5.7 keeps that
  path working; confirm with `cargo check -p spacetimedsl_derive_input` and leave both files
  untouched if it passes. If a path did change, fix it here and nowhere else.

- [x] **Commit** only if a file changed.

### 5.9 Drop the `get_` prefixes

The last sub-change of the split, and the only one that renames. All six names are private to
`derive-input`; none is a generated identifier.

- [x] **Rename, in `method/naming.rs`:**

| Before | After |
| --- | --- |
| `get_referenced_table_function_name` | `referenced_table_function_name` |
| `get_referencing_table_function_name` | `referencing_table_function_name` |
| `get_referenced_table_compile_error_check` | `referenced_table_compile_error_check` |
| `get_referencing_table_compile_error_check` | `referencing_table_compile_error_check` |

- [x] **Rename, elsewhere in the split:**

| Module | Before | After |
| --- | --- | --- |
| `reference_integrity.rs` | `get_row_value_getter` | `row_value_getter` |
| `reference_integrity.rs` | `get_unique_multi_column_index_check` | `unique_multi_column_index_check` |
| `referenced_by.rs` | `get_referenced_table_function_call_for_dsl_method` | `referenced_table_function_call_for_dsl_method` |
| `on_delete_strategy.rs` | `get_referenced_table_function_call_for_strategy_implementation` | `referenced_table_function_call_for_strategy_implementation` |
| `on_delete_strategy.rs` | `get_on_delete_strategy_implementation` | `on_delete_strategy_implementation` |

  Nothing else is renamed. `internal/column.rs`'s `get_primary_key_column_name` and
  `get_auto_inc_column_names` are outside the split and stay.

- [x] **Run the gate.** No snapshot may move — these names never reach the output.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/method
git commit -m "refactor: drop the get_ prefix from the moved generator helpers"
```

---

## Step 6 — Remove the comments that describe the code's past

Plans 1 to 3 left comments that explain the code by what it used to be. A reader who never
saw the old code is told about a defect that no longer exists, in the present tense in three
places. Rewrite each in the present tense, and where the history existed to stop someone
undoing the change, replace it with the rule that must stay true.

Nothing in this step changes behaviour or output.

### 6.1 The source doc comments

- [x] **`method/context.rs`, `MethodGenerationContext`** — three paragraphs of history become
  the rule:

```rust
/// Everything every method generator needs, under one name.
///
/// The derived names are resolved once here rather than in each generator, so a rule about
/// a generated identifier — `field_name_for_found_value` in particular — has one place that
/// states it.
///
/// This is a plain data carrier. It must not grow generation methods: a generator produces
/// one method whole, and a context that generates would take that back.
pub(in crate::internal) struct MethodGenerationContext<'a> {
```

- [x] **`method.rs`, `column_methods_for`** — drop the trailing clause:

```rust
/// Which DSL methods an index earns.
///
/// A non-unique index yields many rows, so it earns `get_many` and `delete_many`. A unique
/// index yields at most one, so it earns `get_one_option` and `delete_one`, plus `update`
/// when it is the primary key. The `method(...)` flags suppress the delete and update
/// methods on top of that.
///
/// Single-column and multi-column indices read this rule from here. It must stay stated
/// once: two copies of it disagreed.
```

- [x] **`method/index.rs`, `IndexShape`** — drop the `for_method` sentence:

```rust
/// Everything the five index-based generators derive from the index they are given.
///
/// The four prose fragments the doc comments are built from are assembled here into the one
/// phrase all five of them end with, so a wording change is one edit.
```

- [x] **`internal/dsl/table.rs`** — `// is set later in method.rs.` above
  `create_dsl_method_arg: None` names a file that no longer sets it. Since plan 3 the value
  is returned by the create generator and applied by `internal/table.rs`:

```rust
// `TableContributions::apply_to` fills this in, after the create method is generated.
create_dsl_method_arg: None,
```

- [x] **Commit.**

```sh
git add derive-input/src
git commit -m "docs: describe the generator as it is, not as it was"
```

### 6.2 The fixture headers that state a fixed defect as current

Three headers describe behaviour that plans 2 and 3 changed. They are wrong, not merely
historical.

- [x] **`derive/tests/fixtures/hash_index.rs`** — plan 3 routed single-column hash indices
  through the single-column path and took `update_session_by_device_id` away. Replace lines
  4 to 11:

```rust
//! Covers `#[index(hash)]` and the multi-column `hash(columns = [...])` form - the third
//! index kind beside btree and direct, and the only one no example module uses.
//!
//! A single-column hash index is extracted onto its column like a btree or a direct one, so
//! its snapshots read like their btree counterparts. A multi-column hash index stays on the
//! table and takes the multi-column path.
```

- [x] **`derive/tests/fixtures/qualified_type_spellings.rs`** — plan 2 replaced the
  text-comparison classification with `ColumnTypeKind::of`, so the pairs agree now. Verified:
  the two `get_documents_by_*_title` snapshots are identical after renaming, and the two
  `get_documents_by_*_note` snapshots differ only in where `prettyplease` wraps a line.
  Replace lines 4 to 11:

```rust
//! `ColumnTypeKind::of` classifies by the path's last segment and accepts a path that is
//! bare or rooted in `std`, `core` or `alloc`, so `std::string::String` classifies as
//! `String` and `core::option::Option<T>` as `Optional`.
//!
//! Each pair below is one type written two ways, and each pair generates the same code -
//! apart from where `prettyplease` wraps a line, because the two spellings are different
//! lengths.
```

- [x] **`compile-tests/tests/ui/before_delete_hook_without_delete_method.rs`** — plan 3 made
  `delete = false` remove the delete methods. Replace lines 3 to 5:

```rust
//! `method(delete = false)` removes every delete method, which the snapshot fixture
//! `methods_disabled` pins, and rejects the three things that would have needed one: a
//! delete hook, an `on_delete = Delete` foreign key, and a `#[referenced_by]` attribute.
```

- [x] **`derive/tests/fixtures/methods_disabled.rs`** — drop the "which is now the same kind
  of thing for both" clause; state the two flags directly.

- [x] **Commit.**

```sh
git add derive/tests/fixtures compile-tests/tests/ui
git commit -m "docs: correct the fixture headers that describe fixed defects as current"
```

### 6.3 The headers that name a fixing commit or a closed issue

These describe a past bug as the reason a guard exists. Keep the reason; drop the history.

- [x] **`derive/tests/fixtures/wrapper_optional_index.rs`** — replace "the branch fixed in
  `81ada87`, which no table in `examples/test` reaches because the index there is commented
  out" with what the fixture guards:

```rust
//! Covers an `Option<Timestamp>` column carrying an index under both wrapper kinds. No table
//! in `examples/test` reaches this branch, because the index there is commented out, so
//! these snapshots are the only thing pinning it.
```

- [x] **`compile-tests/tests/ui/wrapper_optional_unique_index.rs`** — delete the paragraph at
  lines 10 to 13 entirely. The paragraphs above it say why the case is a `compile_fail`, and
  the maintenance note below it is still true.

- [x] **`derive/tests/fixtures/delete_hooks_with_foreign_key_on_unique_index.rs`** — state
  the shape and the rule rather than the bug:

```rust
//! Covers delete hooks on a table whose `#[foreign_key]` sits on a `#[unique]` column that
//! is not the primary key.
//!
//! The cascade path emits the hook calls per row, inside the scope that binds the row being
//! deleted. The shape matters here, not any single method: delete hooks plus a cascading
//! foreign key plus a unique - not primary key - index on the referencing column.
```

- [x] **Commit.**

```sh
git add derive/tests/fixtures compile-tests/tests/ui
git commit -m "docs: state what the regression fixtures guard instead of the bugs they came from"
```

### 6.4 The comments this plan invalidates

- [x] **`compile-tests/tests/ui/foreign_keys_with_mismatched_types.rs`** — after step 2 there
  is no `Span::call_site()` diagnostic left to be more precise than. Replace lines 5 to 7:

```rust
//! The diagnostic is spanned on the second of the two columns, the one whose type
//! contradicts the first.
```

- [x] **`compile-tests/tests/ui/foreign_keys_with_mismatched_paths.rs`** — the reference to
  the sibling file stays; only confirm it still reads correctly after the edit above.

- [x] **Commit.**

```sh
git add compile-tests/tests/ui
git commit -m "docs: drop the call-site contrast from the compile-test headers"
```

### 6.5 The answered marker

One `TODO` asks a question the snapshots answer. Every other marker stays.

- [x] **Delete the marker** at the `None` arm of `index_column_arguments`, in
  `method/index.rs`:

```rust
// TODO: string stuff was only in the single column index implementation, does that work for multi column indices?
```

  `string_index_column` pins the single-column shapes and `multiple_dsl_attributes` pins a
  unique multi-column index over `[database_id, name]` where `name` is a `String`. The
  snapshots show the column arriving as `&str` and reaching `filter` as part of the tuple.
  The line below it, `let column_type = if column_is_string { … }`, stays exactly as it is.

- [x] **Confirm nothing else went.** `rg -n 'TODO|FIXME' derive-input/src derive/src src`
  should list every marker it listed before this plan, minus the two the hook fix removed in
  1.2 and this one.

- [x] **Commit.**

```sh
git add derive-input/src/internal/dsl/method/index.rs
git commit -m "docs: delete the multi-column string question the snapshots answer"
```

### 6.6 The line-number references

A comment that points at `file.rs:123` is wrong the next time anything above line 123 moves.
Step 5 moved most of this code, so all of them are stale now.

- [x] **Replace each with the symbol it means:**

| File | Before | After |
| --- | --- | --- |
| `derive/tests/fixtures/unique_multi_column_index.rs` | "the warning at `method.rs:988`" | "the experimental-feature warning `IndexShape::of` attaches to a unique multi-column index" |
| `derive/tests/fixtures/multiple_table_attributes.rs` | "the heuristic table selector at `internal/integration.rs:66`" | "the heuristic table selector `integration::select_table_with_heuristics`" |
| `derive/tests/fixtures/hash_index.rs` | "the extraction loop at `db/column.rs:63`" | removed with the rewrite in 6.2 |

- [x] **Sweep for the rest.** `rg -n '\.rs:[0-9]+' derive-input/src derive/src derive/tests compile-tests src`
  must come back empty. Acceptance criterion 8.

- [x] **Commit.**

```sh
git add derive/tests/fixtures
git commit -m "docs: name the symbols the comments point at instead of line numbers"
```

---

## Step 7 — The code quality report is already retired

**Done when this plan was written, not during execution.** Every finding the report still
held is quoted verbatim in the `**Violates:**` preamble of the step that resolves it, above.
Nothing it said is lost, so it was deleted rather than left describing work that is planned.

What was done, for the record:

- `CODE_QUALITY_REPORT.md` was deleted.
- The opening link in each of plans 1, 2 and 3 was rewritten to point here instead of at the
  deleted file, keeping the item numbers each of them already named.
- Plan 1's two further mentions — its step 5.1 heading ("Remove from
  `CODE_QUALITY_REPORT.md`") and its appendix heading ("content moved verbatim out of
  `CODE_QUALITY_REPORT.md`") — were left alone. They record what that plan did to a file that
  existed then, and they are plain code spans, not links.
- The Recommended Resolution Order was **not** carried over. Each plan states which of its
  items it addresses; the index itself was redundant once the report was gone.

The only thing left for an implementer here:

- [x] **Confirm nothing regressed.** Acceptance criterion 1:

```sh
test ! -e CODE_QUALITY_REPORT.md
rg -n '\]\([^)]*CODE_QUALITY_REPORT' .                                # 0 matches
```

- [x] **Run the full gate one last time.**

```sh
./x.sh unit-test && ./x.sh format && ./x.sh test && cargo insta pending-snapshots
```

---

## Verification

Run all ten acceptance criteria from the repository root. Then, by hand:

1. **The hook error is visible.** `./x.sh test` publishes `examples/test` and calls `tester`.
   If the two `cascade_hook_error_test` assertions pass, a hook's error reached the caller
   through both the one-row and the many-row cascade.
2. **The diagnostics point at something.** Open three `.stderr` files — one column-level, one
   argument-level, one table-level — and confirm the caret is under the thing the message
   names, not under the whole attribute.
3. **The split holds.** `wc -l derive-input/src/internal/dsl/method.rs` and
   `wc -l derive-input/src/internal/dsl/method/*.rs`. If a module is over 600 lines, say which
   and why rather than splitting it further without a decision.
4. **Nothing regressed.** `git diff <base>..HEAD -- derive/tests/snapshots` should contain
   exactly three kinds of change: the cascade signatures and deletion results from 1.2, the
   two path spellings from 3.2, and nothing else.

---

## Execution record

Written after the fact, by auditing the twenty-six commits in `main..HEAD` against the
repository as it now stands. The oldest of them, `bc27238`, only adds this file; the other
twenty-five are the implementation. No source file was changed while writing this section.

**Verdict:** the plan was carried out as written, with one substantive miss —
`derive-input/src/internal/dsl/method.rs` finished at 291 lines rather than the under-250 that
acceptance criterion 4 and [5.7](#57-the-wiring-module) require — plus a small number of
cosmetic divergences recorded below. Everything else holds: the suite is green, no snapshot is
pending, every sub-change the movement table marks *preserving* moved nothing, and none of the
work in [What this plan does not cover](#what-this-plan-does-not-cover) was done anyway.

### Deviations from the plan

#### `method.rs` did not reach 250 lines

`wc -l derive-input/src/internal/dsl/method.rs` reports **291**. The file holds exactly what
[5.7](#57-the-wiring-module) prescribes and nothing more: the thirteen `mod` declarations, the
`pub(in crate::internal) use context::{MethodGenerationContext, TableContributions};`
re-export, the nine `use` lines that pull the generators back in, `update_method_for`,
`column_methods_for`, `impl SpacetimeDSLColumnMethods` and `impl SpacetimeDSLTableMethods`.
Nothing was left behind that another module could take: the two `impl` blocks are the entry
points the plan names as staying, and `SpacetimeDSLTableMethods::generate` alone is about 150
lines because it carries the foreign-key grouping loop. The 250-line target was set below what
the prescribed contents occupy. Every module under `method/` is well inside its own limit, the
largest being `delete.rs` at 587 lines.

#### Sub-change 5.7 has no commit of its own

The "Commits" row of [Locked decisions](#locked-decisions) asks for one commit per `### N.M`
section. There is no `refactor: reduce method.rs to the wiring between the generator modules`
commit. The reduction happened inside `7a4664e`, the commit for [5.6](#56-the-cascade): moving
`for_referenced_by`, `for_foreign_key`, `on_delete_strategy_implementation` and the naming
helpers out was the last thing left in the file, so `method.rs` fell from 1 330 lines to 292
in that one commit and nothing remained for a separate 5.7 to do beyond the size check. The
end state is the one 5.7 describes.

#### Two enums in `on_delete_strategy.rs` are wider than the visibility rule allows

[5.6](#56-the-cascade) says `on_delete_strategy_implementation` and `ReferencingTables` become
`pub(in crate::internal)` and "the other four stay private", and the step-wide visibility rule
says an item used only inside its own module stays private. `RowBinding` and `IndexUniqueness`
are declared `pub(in crate::internal)` in
`derive-input/src/internal/dsl/method/on_delete_strategy.rs` although both are used only
inside that file. The effect is nil — `method.rs` declares `mod on_delete_strategy;`
privately, so neither type can be named from anywhere else — which is also why no lint caught
it. `strategy_by_row` and `referenced_table_function_call_for_strategy_implementation` are
private, as asked.

#### `IndexShape` and `IndexColumnArguments` have `pub` fields

[5.2](#52-the-index-analysis) asks for `pub(in crate::internal)` on the fields of both
structs. They are plain `pub` in `derive-input/src/internal/dsl/method/index.rs`. Because both
structs are themselves `pub(in crate::internal)`, the reach is identical, and it matches
`MethodGenerationContext` in `method/context.rs`, whose fields were already plain `pub` before
this plan.

#### The singleton delete generator also gained the fourth `deletion_result` argument

[1.2](#12-raise-it-forward-it-and-return-it) names only `for_delete_one` and `for_delete_many`
under "Build two deletion results per delete generator". `for_singleton_delete` had to change
too, because `runtime::deletion_result` grew a fourth parameter that every call site must
pass. It passes `&quote! { None }`, which is right: a singleton table has no cascade, so no
hook error can reach its deletion result.

#### Three verbatim quotations differ from the tree in wording or layout

- [Step 4](#step-4--regroup-the-paired-method-fields) quotes the comment in
  `derive/src/output.rs` as "Two loops, not one: today every one-row method is emitted before
  any many-row method". The tree drops "today". That is consistent with
  [step 6](#step-6--remove-the-comments-that-describe-the-codes-past), which forbids exactly
  that kind of time-stamped phrasing, so the divergence reads as deliberate.
- [1.1](#11-carry-the-hook-error-on-the-deletion-result) shows `Display for DeletionResult`
  with its `write!` on one line. `src/delete.rs` wraps the same call across four lines. That
  is `rustfmt`; the tokens are identical.
- [1.4](#14-document-it) shows the cascading-delete example as an indented block whose CSV row
  ends in a comma. `docs/DOCUMENTATION.md` renders it as a fenced `txt` block and the row has
  no trailing comma. A rendering choice, not a content change.

#### Two `.stderr` files moved in step 6, which no movement-table row anticipates

The [movement table](#expected-snapshot-and-diagnostic-movement) attributes all `.stderr`
motion to [2.4](#24-regenerate-and-read-every-stderr). Two more moved later: `a381717`
([6.3](#63-the-headers-that-name-a-fixing-commit-or-a-closed-issue)) updated
`compile-tests/tests/ui/wrapper_optional_unique_index.stderr`, and `e7b92cf`
([6.4](#64-the-comments-this-plan-invalidates)) updated
`compile-tests/tests/ui/foreign_keys_with_mismatched_types.stderr`. Both are unavoidable:
those sub-changes delete lines from the corresponding `.rs` headers, and `trybuild` records
the offending token as `--> tests/ui/<file>.rs:LINE:COL`, so every line number below the edit
shifts. Both files sit inside the `git add compile-tests/tests/ui` those sub-changes already
list. [6.2](#62-the-fixture-headers-that-state-a-fixed-defect-as-current) replaced three
header lines with three, so its `.stderr` did not move.

#### `debug-helper/output/lib.expanded.rs` is now stale

The checked-in macro expansion still contains `Delete One Error: An unknown error occurred …`
and the pre-1.2 `Err(entries)` cascade shape. No action in this plan covers that file — it is
regenerated by `./x.sh debug`, not by the gate — so leaving it is not a scope violation, but
it no longer matches the generator and would mislead anyone reading it.

#### An arithmetic slip in the plan itself

[2.2](#22-span-the-column-and-index-diagnostics) says "eleven of the thirteen errors" in
`internal/dsl/table.rs` are raised inside `for field in &column_args.fields`, while the table
directly underneath it sums to ten: 5 on `field.vis`, 2 on `field.ty`, 3 on `field.ident`. The
implementation followed the table. `a824d92` spans exactly those ten and `92df820` spans the
remaining three on `&column_args.original_struct_name`, thirteen in total, as expected.

### What this plan does not cover: nothing was widened

Each guard was checked against the tree rather than assumed.

| Excluded item | State |
| --- | --- |
| The `OnDeleteStrategy` copy between crates | Both copies and both warning comments survive, at `derive-input/src/api/dsl/foreign_key.rs:11` and `src/delete.rs:7`. |
| Merging the delete paths | `for_delete_one` and `for_delete_many` share `method/delete.rs`, but neither was merged, reordered nor given a shared stage sequence, and the `match one_or_multiple` arm pairs in the cascade builders are intact. |
| The remaining `TODO`/`FIXME` markers | `method.rs` carried 14; the split modules carry 11. The three that went are the two hook `FIXME`s removed in 1.2 and the one `TODO` removed in 6.5. Every other file's count is unchanged, including the four in `derive/src/lib.rs` and the two in `src/delete.rs`. |
| The commented-out `SetNone` blocks | `method.rs` held 3; `method/delete.rs` holds 2 and `method/on_delete_strategy.rs` holds 1. The counts in `api/dsl/foreign_key.rs`, `internal/dsl/foreign_key.rs` and `src/delete.rs` are untouched. |
| Splitting `SpacetimeDSLTableMethods` further | It still holds `create`, `get_all`, `get_count` and `multi_column_indices`; only the four long fields regrouped. |
| Renaming generated methods or identifiers | No `.snap` file was added, removed or renamed, and no snapshot hunk touches a method name or a generated doc comment. |
| Renames beyond [5.9](#59-drop-the-get_-prefixes) | `internal/column.rs` still spells `get_primary_key_column_name` and `get_auto_inc_column_names`. |
| A version bump | `git diff main..HEAD -- '*Cargo.toml'` is empty. |

### The ten acceptance criteria, as run

Run from the repository root. `rg` is not on `PATH` in this environment, so the greps were run
as `grep -rnE` with equivalent patterns, with a ripgrep-backed search used for the one pattern
that has to span lines. The substitution changes no result.

| # | Result | Note |
| --- | --- | --- |
| 1 | pass | `CODE_QUALITY_REPORT.md` is absent and no `](…)` link names it. |
| 2 | pass | No `Error::new(Span::call_site())` in `derive-input/src` or `derive/src`. The three surviving `Span::call_site()` calls are the synthesised identifiers the plan protects: `__singleton_placeholder`, the injected `id` field and `internal/dsl/singleton.rs`'s `PRIMARY_KEY_NAME`. |
| 3 | **does not pass literally** | Two matches outside `api/runtime.rs`, both inside the issue-60 `FIXME` that [What this plan does not cover](#what-this-plan-does-not-cover) protects: `method/on_delete_strategy.rs:378` and `method/update.rs:257`, each reading `… on error return Err(crate::spacetimedsl::error::SpacetimeDSLError);`. No generator *emits* a literal path. |
| 4 | **fail** | 291 lines against a limit of 250. See the first deviation above. |
| 5 | pass | Thirteen modules, 44 to 587 lines. |
| 6 | pass | The only file naming `execute_on_delete_strategies_of_` is `method/naming.rs`. |
| 7 | pass | Zero matches. |
| 8 | **does not pass literally** | The pattern also matches `rustc`'s own `--> tests/ui/<file>.rs:LINE:COL` locations inside the 14 `.stderr` fixtures, which `trybuild` writes and which predate this plan. Restricted to `*.rs` sources — the comments the criterion is about — it returns zero matches. The criterion was never satisfiable as written. |
| 9 | pass | Both patterns return zero matches in `derive-input/src`. |
| 10 | pass, with a substitution | `./x.sh unit-test` green: 27 snapshot tests and 15 `trybuild` cases. `cargo insta pending-snapshots --workspace` reports none. `./x.sh test` publishes both example modules, calls `tester` and logs `Test executed successfully!`. For `./x.sh format` the audit ran `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --all-features` **without** `--fix`, so that checking the plan could not rewrite the tree; both exit 0, which is the same evidence with none of the risk. |

### Verification, by hand

1. **The hook error is visible.** `./x.sh test` created `lock_group` and `lock_holder` and the
   reducer returned no error, so both `cascade_hook_error_test` assertions passed: the hook's
   message reached the caller through the one-row `Vec` path and the many-row `HashMap` path.
   That is also the only coverage the many-row branch has. `Delete Many Error` appears in no
   snapshot, because no fixture reaches `for_delete_many`'s referencing-tables branch.
2. **The diagnostics point at something.** `plural_name_on_singleton.stderr` underlines
   `plural_name = configurations`, `unique_index_on_singleton.stderr` underlines
   `world_name_and_seed` and then the two indexed columns, and `missing_table_attribute.stderr`
   underlines the struct name `Thing`. None of the ten regenerated files still carries the
   `= note: this error originates in the attribute macro …` line; the three that do are the
   three the plan marks unchanged.
3. **The split holds.** `method.rs` 291, over the 250 target as recorded above. `method/`:
   `context` 122, `create` 397, `delete` 587, `foreign_key` 283, `get` 209, `hook_call` 44,
   `index` 356, `naming` 67, `on_delete_strategy` 488, `reference_integrity` 334,
   `referenced_by` 294, `singleton_table` 148, `update` 270. All under 600.
4. **Nothing regressed.** `git diff main..HEAD -- derive/tests/snapshots` holds only the two
   expected kinds of change — the cascade signatures, the `OnDeleteStrategyFailure`
   destructuring, the `error_from_hook` fields and the two reworded messages from 1.2, and
   `delete::` or `error::` appearing in the two path spellings from 3.2 — plus the line
   re-wrapping `prettyplease` does around them. Per commit: `1849737` moved 92 snapshots,
   `372b16a` moved 22 and matched the twenty-plus-two prediction exactly, and every other
   commit moved none.

### Checkboxes left unticked

| Sub-change | Action | Why |
| --- | --- | --- |
| [5.7](#57-the-wiring-module) | **Confirm the size.** | `method.rs` is 291 lines, not under 250. The `method/*.rs` half of the same check passes. |
| [5.7](#57-the-wiring-module) | **Commit.** | No commit carries the message this block specifies; the work landed inside `7a4664e` with 5.6. |

The other 131 are ticked. Two of those are worth naming, because their stated outcome is not
literally reproducible and they were judged on the end state the plan was steering towards:

- 2.1's and 2.2's "Run `./x.sh unit-test`, leave the `.stderr` failing until 2.4" cannot be
  observed from the finished tree. What can be observed is that `8a1dcbe`, `a824d92` and
  `92df820` changed no `.stderr` at all and `c729374` changed exactly ten, which is the
  sequence those checkboxes were arranging.
- 6.6's "Sweep for the rest … must come back empty" is ticked on the action rather than on the
  literal command, for the reason given against acceptance criterion 8.

### Commit per sub-change

| Sub-change | Commit | Subject |
| --- | --- | --- |
| — | `bc27238` | Add plan to finish generator clean up |
| [1.1](#11-carry-the-hook-error-on-the-deletion-result) | `dbb1918` | feat: carry a cascade's hook error on the deletion result |
| [1.2](#12-raise-it-forward-it-and-return-it) | `1849737` | fix: return the error a delete hook raises during a cascade |
| [1.3](#13-prove-it-at-runtime) | `5b256eb` | test: check a cascade hook's error reaches the caller |
| [1.4](#14-document-it) | `53876a7` | docs: document a hook failing during a cascading delete |
| [2.1](#21-keep-the-spans-of-the-dsl-arguments) | `8a1dcbe` | refactor: keep the spans of the dsl attribute arguments |
| [2.2](#22-span-the-column-and-index-diagnostics) | `a824d92` | refactor: span the column and index diagnostics on the column they name |
| [2.3](#23-span-the-table-wide-diagnostics-on-the-struct-name) | `92df820` | refactor: span the table-wide diagnostics on the struct name |
| [2.4](#24-regenerate-and-read-every-stderr) | `c729374` | test: regenerate the diagnostics after spanning them |
| [3.1](#31-publish-apiruntime) | `0033345` | refactor: publish the runtime contract as api::runtime |
| [3.2](#32-cover-every-remaining-runtime-path) | `372b16a` | refactor: emit every runtime path through api::runtime |
| [3.3](#33-adopt-it-in-the-derive-crate) | `1ec2a30` | refactor: emit the derive crate's runtime paths through api::runtime |
| [4](#step-4--regroup-the-paired-method-fields) | `df96335` | refactor: group the paired cascade methods behind two structs |
| [5.1](#51-the-shared-vocabulary-and-the-context) | `be7d1d8` | refactor: move the generation context and OneOrMultiple into their own modules |
| [5.2](#52-the-index-analysis) | `6372521` | refactor: move the index analysis into method/index.rs |
| [5.3](#53-the-hook-call-helper-and-referential-integrity) | `af10b4a` | refactor: move the hook-call helper and the integrity checks into their own modules |
| [5.4](#54-the-four-dsl-method-families) | `ebf0609` | refactor: move the create, get, update and delete generators into their own modules |
| [5.5](#55-the-singleton-generators) | `f84f07f` | refactor: move the singleton generators into method/singleton_table.rs |
| [5.6](#56-the-cascade) | `7a4664e` | refactor: move the cascade generators into their own modules |
| [5.7](#57-the-wiring-module) | `7a4664e` | folded into 5.6; no commit of its own |
| [5.8](#58-update-the-call-sites-outside-method) | — | no commit, correctly: the re-export kept both importers' paths working and neither file changed |
| [5.9](#59-drop-the-get_-prefixes) | `7ef7b6f` | refactor: drop the get_ prefix from the moved generator helpers |
| [6.1](#61-the-source-doc-comments) | `500eca0` | docs: describe the generator as it is, not as it was |
| [6.2](#62-the-fixture-headers-that-state-a-fixed-defect-as-current) | `768027b` | docs: correct the fixture headers that describe fixed defects as current |
| [6.3](#63-the-headers-that-name-a-fixing-commit-or-a-closed-issue) | `a381717` | docs: state what the regression fixtures guard instead of the bugs they came from |
| [6.4](#64-the-comments-this-plan-invalidates) | `e7b92cf` | docs: drop the call-site contrast from the compile-test headers |
| [6.5](#65-the-answered-marker) | `bfc2f00` | docs: delete the multi-column string question the snapshots answer |
| [6.6](#66-the-line-number-references) | `7b7e94e` | docs: name the symbols the comments point at instead of line numbers |
| [7](#step-7--the-code-quality-report-is-already-retired) | `bc27238` | done when the plan was written: the report was deleted and plans 1 to 3 relinked here |
