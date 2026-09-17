# Code Quality Report

Scope: `derive-input/src/internal/dsl/method.rs`, reviewed against the programming principles in [`AGENTS.md`](AGENTS.md). Two findings that reach beyond that file are marked as such.

## `derive-input/src/internal/dsl/method.rs`

This module is the code generator for every DSL method SpacetimeDSL emits: `create`, `get_all`, `get_count`, index-based `get`/`update`/`delete`, referential-integrity checks, and on-delete strategy execution. It is by far the largest module in the crate — larger than every other module in `internal/dsl` combined.

The central problem is that one module, and inside it one function, owns every decision for every generated method: naming, doc comments, argument shapes, return types, and body assembly. Because there is no seam between "which method are we generating" and "how is that method assembled", every variant's logic is expressed as nesting inside a single control-flow tree, and every shared concept (hook emission, wrapper-type unwrapping, format-string construction, one-row-versus-many-rows handling) is re-implemented at each site where it is needed rather than expressed once.

Two consequences are already visible in the code rather than merely predicted:

- Knowledge that exists in more than one place had **drifted apart** — the two `column_names_and_row_values` builders constructed the same user-facing message independently, and one business rule, which indices may update a row, was stated twice in two places that disagreed. Both moved to [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md) and are resolved there.
- Branches existed that cannot ever have been executed, because the tokens they emitted were not valid Rust. The wrapper-type option mapper was one; `81ada87` has since fixed it, and the `wrapper_optional_index` fixture now pins the corrected emission so it cannot regress unnoticed.

Neither of these could have been caught when this report was written, because the crate then had no automated tests at all. `examples/test` is a `cdylib` that exercises the macro by compiling against it; it proves the happy paths compile, it asserts nothing, and it does not cover branches that no example happens to trigger — which is exactly how both defects survived.

That gap is now closed by [`.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md`](.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md): snapshot tests pin the generated output for every table shape the generator supports, and compile tests pin the diagnostics it emits for the definitions it rejects. Both run with `./x.sh unit-test`. The three behaviors that suite exposed — `method(delete = false)` suppressing no delete method, the two disagreeing update rules, and the single-column hash index routed through the multi-column path — moved to plan 3 with the decisions that settled them.

Two rounds of the work this report raised are done, and their violation sections have moved out of it:

- **Items 2 to 8** — the dead scaffolding, the commented-out `SetNone` blocks, the two user-visible defects, the mechanical duplication, the string-based type classification, the dishonest signatures and the error strategy — moved to [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md) and landed in `0923f2c`.
- **Items 9, 10, 12, 15, 16 and 17**, plus the `column_names_and_row_values` half of item 11 — breaking up `for_method`, the generation context and the `&mut` mutation, the singleton contract, and the three defects above — moved to [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md) and landed in `dc0d1f4`..`26f36a8`. The three findings that plan is _constrained_ by rather than fixing — the `DeleteOne`/`DeleteMany` assembly, the `OneOrMultiple` branch pairs, and `OneOrMultiple` serving two concepts, all settled as "no change" — moved there with it, so the constraint sits beside the work it binds.

What remains below is the work neither plan does: the module's size and placement, the long paired field names, the unresolved markers, and the two findings that reach beyond `method.rs`. They are described individually, but they share a single root: the module never paid the abstraction cost that its duplication had already earned.

---

### `method.rs`: module size and placement

**Violates:** Maximize Cohesion; Single Responsibility Principle; Optimize for Deletion (_Keep modules small enough to rewrite in a week_); Separation of Concerns

The module holds, in one file: the `OneOrMultiple` / `Action` type definitions, the two public entry points (`SpacetimeDSLColumnMethods::map` and `SpacetimeDSLTableMethods::generate`), the method generators, foreign-key and referenced-by generators, on-delete strategy generation, multi-column-index uniqueness checks, and a family of identifier-naming helpers. These are separate responsibilities with separate change drivers — a change to naming conventions, a change to the delete protocol, and a change to argument typing all land in the same file.

Recommendation: split along the domain axis that `AGENTS.md` prescribes for the Maximize Cohesion / Separation of Concerns conflict ("split in domain modules wiring together technical modules"). Natural seams: one module per DSL method family (create, read, update, delete), one module for referential integrity / on-delete strategies, one module for identifier naming, and a small module for the shared generation context.

This was blocked by the god function `for_method`, which owned every variant's logic and could not be distributed across files while it existed. Plan 3 removed that blocker — the module now holds ten per-variant generators, a named generation context and a shared index analysis, which are the pieces this split moves — but it deliberately did not shorten the file, which is still about 3 570 lines. Take this next, together with the paired field names below.

### `method.rs`: generated identifiers and the field names built from them

**Violates:** Code For The Maintainer; Prefer clarity over cleverness; Simplicity & Right-Sized Solutions

Field and local names such as `execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted` are long enough that the formatter is forced to break `let mut` onto its own line, separating the binding keyword from the name. Assignments to these names dominate the visual weight of the function that builds them.

`AGENTS.md` forbids abbreviation, so shortening these by truncating words is not an option and should not be attempted.

Recommendation: shorten by _namespacing_, not abbreviating. Group the related pair into a small struct — for example a type holding `after_one_row_was_deleted` and `after_multiple_rows_were_deleted` — so the shared prefix lives in the type name and each field keeps a fully spelled, unabbreviated name. This satisfies both the no-abbreviation rule and readability. Note that the _generated_ function names (which users see and call) are a separate question and should not be changed without considering the effect on the public generated API.

### `method.rs`: unresolved `FIXME` and `TODO` markers

**Violates:** Documentation & Communication Clarity (_Future ideas captured outside codebase_, _Log blockers to future cleanups for retrospectives_); Refactoring & Change Containment

The module carries a mix of markers. Some are linked to tracked issues — the `try_update` replacement, doc comments influenced by foreign-key attributes. Others are not linked to anything: an unnecessary clone in the create path, an error message that shows all columns where only the unique ones are relevant, row-value getters for wrapper types described as being built in the wrong shape, a hook error that is swallowed because propagating it would require a signature change, and an unanswered correctness question in the `Update` path asking whether the `String` handling, written for single-column indices, also holds for multi-column ones. That last question has an answer now: `string_index_column` pins the single-column shapes and `multiple_dsl_attributes` pins a unique multi-column index over `[database_id, name]` where `name` is a `String`. Its snapshots show the column arriving as `&str` and reaching `filter` as part of the tuple, so the question can be settled by reading them rather than by reasoning about the code.

Several of these markers sit inside `quote!` bodies. They are _not_ emitted into the code users read — the Rust tokenizer drops ordinary comments before `quote!` ever sees them, and `TODO|FIXME` matches none of the 335 snapshots nor `debug-helper/output/lib.expanded.rs`. They are still misplaced: a marker inside a `quote!` body reads as if it described the generated code, when it describes the generator.

Recommendation: open issues for the unlinked markers and reduce each in-source marker to a one-line reference to its issue, so the rationale lives in the tracker where it can be prioritised. Move markers out of `quote!` bodies to the generator statement they actually concern. Note that the swallowed hook error is not merely a cleanup: it silently discards a user-supplied error, which is a behavioural defect and should be triaged as one rather than left as a comment.

### Beyond `method.rs`: runtime paths in the other generators

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse

Surfaced while planning item 5. The fully qualified runtime paths that `method.rs` inlines — `crate::spacetimedsl::error::…`, `crate::spacetimedsl::delete::…`, `crate::spacetimedsl::DSLMethodHooks`, `crate::spacetimedsl::internal::DSLInternals` — are also written out as literal tokens in `derive-input/src/internal/dsl/hook.rs`, `derive-input/src/internal/dsl/wrapper.rs`, `derive-input/src/api/dsl/foreign_key.rs`, and in the separate `derive` crate's `output/function.rs` and `output/hook.rs`. `OnDeleteStrategy`'s `ToTokens` impl emits `crate::spacetimedsl::OnDeleteStrategy::…` while `method.rs` writes `crate::spacetimedsl::delete::OnDeleteStrategy::…` for the same type, so even the spelling is not consistent.

[`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md) gives `method.rs` a single set of constructors for this surface, but deliberately stops at the module boundary: sharing them with the `derive` crate means widening `derive-input`'s public API.

Recommendation: once the `method.rs` constructors exist and have settled, decide whether the generated-code/runtime-crate contract should be a published part of `derive-input`'s API so every generator targets one definition of it.

### Beyond `method.rs`: every diagnostic is spanned at the call site

**Violates:** Code For The Maintainer; Principle of Least Astonishment

Surfaced while planning item 8. Every `syn::Error` the crate raises today is built with `Span::call_site()` — in `internal/column.rs`, `internal/db/column.rs`, `internal/dsl/column.rs` and `internal/dsl/table.rs` — so each of the 14 `compile-tests/tests/ui/*.stderr` files underlines the whole `#[dsl(…)]` attribute, even when the message names one offending column. `syn::Error::new_spanned` on the field would point at it.

[`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md) uses spanned errors for the diagnostics it adds, which makes the two conventions coexist until this is resolved.

Recommendation: migrate the existing diagnostics to spanned errors wherever a span is available, in one pass, so every `.stderr` moves together and the resulting convention is uniform.

---

## Recommended Resolution Order

1. **Add characterization tests for the generator.** Done — see [`.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md`](.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md). Snapshot tests over the generated output and compile tests over the rejection diagnostics now run with `./x.sh unit-test`, so every step below has the safety net `AGENTS.md` requires before refactoring.
2. **Resolve the dead code.** Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 2.
3. **Delete the commented-out `set_none_strategy` blocks.** Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 3.
4. **Investigate and fix the defects the tests expose.** Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 4.
5. **Extract the mechanical duplication.** Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 5. Scoped to `method.rs`; the same literals in the other generators are recorded above as a remaining finding.
6. **Replace string-based type classification** with a classification resolved once onto `InternalColumn`. Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 6. Do this before the structural split, so the split does not carry the string comparisons into several new modules.
7. **Clean up signatures.** Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 7. Also folds in the hand-written `Display` implementations and the deletion of `CreateOrUpdate`, which had no resolution-order item of their own.
8. **Split `reference_integrity_checks_on_create_or_update` per mode** and fix the `try_parse` naming and error strategy. Done — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 8. Also rejected a singleton carrying a multi-column index, which panicked before.
9. **Break up `for_method` into one generator per `DSLMethod` variant.** Done — see [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), step 5. The enum was deleted rather than reshaped, so the "already processed" panics became unwritable: all ten are gone.
10. **Introduce the generation-context struct and remove the `&mut` mutation.** Done — see [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), step 2. Taken _before_ step 9 rather than after: the context changes the signature of every generator the split creates, so doing it first wrote each of them once.
11. **Understand and then consolidate the drifted duplication.** Partly done — the `column_names_and_row_values` builders merged in [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), step 3.2. The `DeleteOne`/`DeleteMany` assembly, the `OneOrMultiple` branch pairs and the `OneOrMultiple` semantic overload were each settled as "no change"; their findings and decisions moved to the same plan, where they acted as constraints on the split.
12. **Consolidate singleton handling** behind a single definition of the singleton contract. Done — see [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), steps 4 and 7. Four of the five sites read `internal/dsl/singleton.rs` now; the fifth injects the field from the `derive` crate and needs the same API decision as item 5's runtime paths, so each side names the other.
13. **Split the module into domain modules**, and regroup the long paired field names behind small structs. Still open — see [module size and placement](#methodrs-module-size-and-placement) and [generated identifiers](#methodrs-generated-identifiers-and-the-field-names-built-from-them) above. Plan 3 unblocks it by separating the responsibilities, but deliberately does not move them into files.
14. **File issues for the unlinked `TODO`/`FIXME` markers** and reduce each in-source marker to an issue reference, triaging the swallowed hook error as a defect rather than a cleanup. Still open — see [unresolved markers](#methodrs-unresolved-fixme-and-todo-markers) above.
15. **`method(delete = false)` suppresses the delete methods.** Done — see [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), step 1.4. `#[referenced_by]` combined with the flag became a rejection, because suppressing the strategy fanout would otherwise remove one half of the paired compile-error checks and break every child table's compilation.
16. **Unify the rule for which unique indices may update a row.** Done — see [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), steps 1.2 and 6. The rule the plan set out with — that a unique index of any width may update a row — turned out not to compile: `update` is defined on `spacetimedb::table::PrimaryKey`, which only the primary key index implements. Only the primary key updates a row, stated once in `update_method_for`.
17. **Route single-column hash indices like every other single-column index.** Done — see [`.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md`](.ai/plans/3_SPLIT_THE_METHOD_GENERATOR.md), steps 1.1 and 1.3. The wording fix was taken first on its own; the routing followed item 16, so a unique hash column never lost and regained its update method.
