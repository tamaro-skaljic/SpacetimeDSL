# Code Quality Report

Scope: `derive-input/src/internal/dsl/method.rs`, reviewed against the programming principles in [`AGENTS.md`](AGENTS.md). Two findings that reach beyond that file are marked as such.

## `derive-input/src/internal/dsl/method.rs`

This module is the code generator for every DSL method SpacetimeDSL emits: `create`, `get_all`, `get_count`, index-based `get`/`update`/`delete`, referential-integrity checks, and on-delete strategy execution. It is by far the largest module in the crate — larger than every other module in `internal/dsl` combined.

The central problem is that one module, and inside it one function, owns every decision for every generated method: naming, doc comments, argument shapes, return types, and body assembly. Because there is no seam between "which method are we generating" and "how is that method assembled", every variant's logic is expressed as nesting inside a single control-flow tree, and every shared concept (hook emission, wrapper-type unwrapping, format-string construction, one-row-versus-many-rows handling) is re-implemented at each site where it is needed rather than expressed once.

Two consequences are already visible in the code rather than merely predicted:

- Knowledge that exists in more than one place has **drifted apart** — the two `column_names_and_row_values` builders still construct the same user-facing message independently, and the delete paths still spell out the same stage sequence twice.
- Branches existed that cannot ever have been executed, because the tokens they emitted were not valid Rust. The wrapper-type option mapper was one; `81ada87` has since fixed it, and the `wrapper_optional_index` fixture now pins the corrected emission so it cannot regress unnoticed.

Neither of these could have been caught when this report was written, because the crate then had no automated tests at all. `examples/test` is a `cdylib` that exercises the macro by compiling against it; it proves the happy paths compile, it asserts nothing, and it does not cover branches that no example happens to trigger — which is exactly how both defects survived.

That gap is now closed by [`.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md`](.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md): snapshot tests pin the generated output for every table shape the generator supports, and compile tests pin the diagnostics it emits for the definitions it rejects. Both run with `./x.sh unit-test`. What that suite found while it was being written is collected under [Defects found by the characterization tests](#defects-found-by-the-characterization-tests).

The violations this report raised for items 2 to 8 of the resolution order — the dead scaffolding, the commented-out `SetNone` blocks, the two user-visible defects, the mechanical duplication, the string-based type classification, the dishonest signatures and the error strategy — have moved to [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), together with the developer decisions that settled them. What remains below is the work that plan does not do.

The violations below are described individually, but they share a single root: the module never paid the abstraction cost that its duplication had already earned.

---

### `method.rs`: module size and placement

**Violates:** Maximize Cohesion; Single Responsibility Principle; Optimize for Deletion (_Keep modules small enough to rewrite in a week_); Separation of Concerns

The module holds, in one file: the `DSLMethod` / `OneOrMultiple` / `Action` type definitions, the two public entry points (`SpacetimeDSLColumnMethods::map` and `SpacetimeDSLTableMethods::generate`, the latter renamed from `try_parse` by plan 2 — names below are the ones that exist once that plan has landed), the monolithic method generator, foreign-key and referenced-by generators, on-delete strategy generation, multi-column-index uniqueness checks, and a family of identifier-naming helpers. These are separate responsibilities with separate change drivers — a change to naming conventions, a change to the delete protocol, and a change to argument typing all land in the same file.

Recommendation: split along the domain axis that `AGENTS.md` prescribes for the Maximize Cohesion / Separation of Concerns conflict ("split in domain modules wiring together technical modules"). Natural seams: one module per DSL method family (create, read, update, delete), one module for referential integrity / on-delete strategies, one module for identifier naming, and a small module for the shared generation context. Doing this before the structural refactors below would help; doing it after would also work. The order matters less than doing it once the pieces are separable — which is currently blocked by the god function described next.

### `method.rs`: `for_method(dsl_method, rust_struct, spacetimedb_table, spacetimedsl_table, internal_columns, primary_key_column) -> SpacetimeDSLMethod`

**Violates:** Single Responsibility Principle; Curly's Law; Maximize Cohesion; KISS; Code For The Maintainer; Open/Closed

This one function generates every DSL method variant. It is the overwhelming majority of the module. Inside it, control flow nests to a depth where the reader cannot tell which branch they are in without scrolling back, and the innermost `quote!` blocks sit far from the condition that selects them.

The function also uses the declare-now-assign-deep-inside-a-branch pattern for `doc_comment`, `method_name`, `return_type` and `method_impl`. The compiler enforces that every path assigns them, so the code is correct, but a reader tracing "what is `method_impl` for a unique multi-column `DeleteOne`" must locate one assignment among many scattered across the branch tree. This is a puzzle the maintainer solves by hand on every visit.

Additionally, the same `dsl_method` value is re-matched exhaustively several separate times inside the function — once to pick the doc comment, once for the method name, once for the return type, once for the implementation — and every one of those matches carries `panic!("... should already be processed!")` arms for variants that the enclosing `match` has already excluded. The type system is being asked a question it already answered, and the answer is discarded and re-derived.

Recommendation: invert the structure. Instead of one function that switches on the variant repeatedly, give each variant a single place that produces a whole `SpacetimeDSLMethod`. Two shapes are reasonable, and the choice should be made by the developers:

- **Option A — one function per variant. (developer decision)** `for_create(...)`, `for_get_all(...)`, `for_get_many(...)`, and so on, with a thin dispatcher that matches `DSLMethod` exactly once. Simplest, most direct, no new traits; keeps the code obvious. This aligns best with KISS.
- **Option B — a trait with one implementor per variant.** Better if the variants need to share a documented extension contract, but it adds an abstraction layer that `AGENTS.md` says to defer until duplication justifies it.

Option A is the safer default: it removes all of the redundant matching and every unreachable `panic!` arm without introducing indirection. Either way, the enum should be reshaped so the "index-based" variants and the "table-wide" variants are distinguishable by type, so that the "should already be processed" panics become impossible to express rather than merely unreached.

### `method.rs`: the shared parameter list threaded through the generators

**Violates:** Connascence (connascence of position, high degree, non-local); Minimize Coupling; Law of Demeter; Code For The Maintainer

`rust_struct`, `spacetimedb_table`, `spacetimedsl_table`, `internal_columns`, and `primary_key_column` travel together as a positional bundle through the entry points, the method generator, and most helpers. Because they are positional and several are references to different-but-similar table types (`SpacetimeDBTable` vs `SpacetimeDSLTable`), a transposed argument would compile in some call shapes. Adding one more piece of shared context means editing every signature and every call site in the chain — a wide blast radius for what is conceptually a no-op.

Recommendation: introduce a single generation-context struct (for example `MethodGenerationContext`) holding these values, and pass `&context`. This converts connascence of position into connascence of name, which `AGENTS.md` explicitly prefers ("Prefer weaker connascence forms when refactoring dependencies"), and makes future context additions a one-line change. Keep the struct a plain data carrier — it should not grow generation methods, or it becomes a second god object.

### `method.rs`: mutation of `spacetimedsl_table` through `&mut` inside generator functions

**Violates:** Command Query Separation; Design APIs without cross-cutting side effects; Principle of Least Astonishment; Hide Implementation Details

`for_method`, `for_referenced_by`, and `for_foreign_key` read as queries — they are named for what they produce and they return a `SpacetimeDSLMethod`. They also quietly write into the table they were handed: `for_method` installs `create_dsl_method_arg`, and the foreign-key and referenced-by generators insert into `compile_error_checks`. Nothing in the names or return types signals this. A caller that reorders two seemingly independent generator calls can change the resulting table state.

`AGENTS.md` is explicit that CQS wins over Minimize Coupling for internal methods, and these are internal methods.

Recommendation: make the writes part of the return value. Have each generator return its produced method _together with_ whatever it wants recorded (the create-argument struct, the set of compile-error checks), and let the single orchestrating caller apply them to the table. This also removes the reason `generate` currently takes the table by value and hands it back.

### `method.rs`: two `column_names_and_row_values` builders

**Violates:** Don't Repeat Yourself (_One authoritative source for each business rule_); Duplication Control & Reuse; Connascence of Algorithm

The format string that describes "these columns had these values", used in `NotFoundError` and `UniqueConstraintViolation` messages, is built by imperative string pushes in two places: once inside `for_method` (with a branch per `IndexType`) and again inside `multi_column_index_checks`. The placeholder-to-argument correspondence is maintained by hand in parallel with a separately built list of row-value getters, so nothing keeps the placeholder count and the getter list in step.

The two builders had drifted into three different message shapes; [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md) makes them agree on the shape `docs/DOCUMENTATION.md` documents, but leaves them as two builders.

Recommendation: extract a single builder that emits the format string **and** the matching getter list together as one value, so the two can no longer disagree.

### `method.rs`: the `DeleteOne` and `DeleteMany` implementation assembly

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse; Connascence of Algorithm

The two delete paths assemble their generated bodies from the same sequence of stages: fetch the rows, return early if nothing matched, build deletion-result entries, run the `Error` strategy, run the before-delete hook, delete, run the after-delete hook, run the remaining strategies, return the result. Each path duplicates that sequence, each duplicates the surrounding "has referencing tables or not" split, and each duplicates the wrapper-type extraction and the on-error handler construction.

They are not identical, though: the singular path carries a hard-coded `count_of_rows_to_delete ( 1 )` message while the plural path formats real counts, the entry container is a `Vec` in one and a `HashMap` in the other, and the singleton sub-path of `DeleteOne` is a third, separately written variant that skips the `Itertools` conditionality and hard-codes the primary key.

Recommendation: the logic has drifted and the differences may be intentional. The developers should first understand which differences are _required_ by the one-row versus many-rows semantics and which are accidental — in particular whether the differing error message wording and the singleton path's divergent structure are deliberate. Only after that decision should the shared stage sequence be extracted into one assembler parameterised by the genuinely varying parts. Extracting before understanding the differences risks silently standardising on the wrong behaviour.

Developer decision: Every difference is intentional, required and deliberate, nothing has drifted or is accidental.

### `method.rs`: the `OneOrMultiple` branch pairs

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse; Connascence of Algorithm

`get_referenced_table_function_call_for_dsl_method`, `for_referenced_by`, `for_foreign_key`, and `get_on_delete_strategy_implementation` each contain a `match one_or_multiple` whose two arms emit structurally parallel token streams differing only in whether the accumulator is a `Vec` or a `HashMap` and whether the body is wrapped in a loop. The same `Vec`-versus-`HashMap` accumulation-and-append pattern is spelled out separately in each.

Recommendation: as with the delete paths, the developers should first confirm the arms really are parallel and that no arm has accumulated a fix the other lacks. Where they are confirmed parallel, extract the accumulator handling into a single helper that takes `one_or_multiple` and the per-row body, so the two shapes exist in exactly one place. Where an arm has genuinely diverged, that divergence should be documented as intentional rather than left for the next reader to re-derive.

Developer decision: Same as the delete paths, every difference is intentional, required and deliberate, nothing has drifted or is accidental.

### `method.rs`: `OneOrMultiple` used for two unrelated concepts

**Violates:** Connascence of Meaning; Principle of Least Astonishment; Hide Implementation Details

`OneOrMultiple` means "one row or multiple rows" everywhere except in the `Update` path, where it is derived from `is_multi_column_index` — a single-column index maps to `One` and a multi-column index maps to `Multiple` — and passed to `reference_integrity_checks_on_update` to select a format-string shape. There, the type is silently repurposed to mean "one column or multiple columns". The same value also reaches generated code as a user-visible `OneOrMultiple` in error payloads.

This is exactly the coupling `AGENTS.md` warns about: two call sites agree on a meaning that the type name contradicts, and nothing enforces it.

Recommendation: the developers should determine what the generated error payload is _supposed_ to report in the multi-column update case, since the current behaviour may be reporting a column-count fact in a field documented as a row-count fact. Once that is settled, give the column-arity concept its own type (or pass the column list and let the callee decide), so `OneOrMultiple` retains one meaning.

Developer decision: The type name contradicts nothing, as it doesn't state it is for columns, rows, functions or whatever. "OneOrMultiple" is deliberately just a differentiator between "One" or "Multiple" and will never change. That it is used everywhere for "rows" except at one site where it is used for "columns" does not mean that it was created to be only used for "rows", instead it was intentional design that the type gives only information about whether there is "one" or there are "multiple" of a generic something. Keep it one type, do not introduce multiple types for THE SAME concept.

### `method.rs`: singleton handling scattered across the module

**Violates:** Encapsulate What Changes; Open/Closed; Don't Repeat Yourself; Connascence of Value

The singleton table concept is special-cased independently in the column-method mapper, the table-method entry point, the create/update column processor (where the primary key is identified by the literal field name `"id"`, the literal type `"u8"`, and filled with `0u8`), the method generator (a dedicated `is_singleton_pk` flag driving separate doc comments, method names, get, update, and delete bodies), and the on-delete strategy generator (a different row finder and a different row-value format). The magic values `"id"`, `u8`, `0u8`, and the literal message text `"{ id : 0 }"` recur across these sites with no shared definition.

Changing anything about how singletons are represented therefore requires finding and editing every one of these sites, and missing one produces inconsistent generated code rather than a compile error.

Recommendation: define the singleton contract in one place — the sentinel primary key name, its type, its value, and its rendered representation — and have every site read from it. Then consider whether the singleton generation paths should be selected once (developer decision) at the top (a distinct generation strategy) rather than re-tested inside each branch. The developers should decide how far to take this: a single shared constant set is cheap and clearly correct, whereas a separate singleton generation path (developer decision) is a larger structural change that only pays off if singleton behaviour continues to diverge.

### `method.rs`: generated identifiers and the field names built from them

**Violates:** Code For The Maintainer; Prefer clarity over cleverness; Simplicity & Right-Sized Solutions

Field and local names such as `execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted` are long enough that the formatter is forced to break `let mut` onto its own line, separating the binding keyword from the name. Assignments to these names dominate the visual weight of the function that builds them.

`AGENTS.md` forbids abbreviation, so shortening these by truncating words is not an option and should not be attempted.

Recommendation: shorten by _namespacing_, not abbreviating. Group the related pair into a small struct — for example a type holding `after_one_row_was_deleted` and `after_multiple_rows_were_deleted` — so the shared prefix lives in the type name and each field keeps a fully spelled, unabbreviated name. This satisfies both the no-abbreviation rule and readability. Note that the _generated_ function names (which users see and call) are a separate question and should not be changed without considering the effect on the public generated API.

### `method.rs`: unresolved `FIXME` and `TODO` markers

**Violates:** Documentation & Communication Clarity (_Future ideas captured outside codebase_, _Log blockers to future cleanups for retrospectives_); Refactoring & Change Containment

The module carries a mix of markers. Some are linked to tracked issues — the `try_update` replacement, doc comments influenced by foreign-key attributes. Others are not linked to anything: an unnecessary clone in the create path, an error message that shows all columns where only the unique ones are relevant, row-value getters for wrapper types described as being built in the wrong shape, a hook error that is swallowed because propagating it would require a signature change, and an unanswered correctness question in the `Update` path asking whether the `String` handling, written for single-column indices, also holds for multi-column ones. That last question has an answer now: `string_index_column` pins the single-column shapes and `multiple_dsl_attributes` pins a unique multi-column index over `[database_id, name]` where `name` is a `String`. Its snapshots show the column arriving as `&str` and reaching `filter` as part of the tuple, so the question can be settled by reading them rather than by reasoning about the code.

Several of these markers sit inside `quote!` bodies. They are _not_ emitted into the code users read — the Rust tokenizer drops ordinary comments before `quote!` ever sees them, and `TODO|FIXME` matches none of the 318 snapshots nor `debug-helper/output/lib.expanded.rs`. They are still misplaced: a marker inside a `quote!` body reads as if it described the generated code, when it describes the generator.

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

## Defects found by the characterization tests

Writing the suite described in [`.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md`](.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md) surfaced three behaviors that no example module exercises and that the report could not have found by reading `method.rs` alone. Each is pinned by a fixture, so whichever way the developers decide it, the decision shows up as a snapshot diff.

### `method(delete = false)` disables no delete method

**Violates:** Principle of Least Astonishment; Code For The Maintainer (_names state what the thing does_)

`#[dsl(method(delete = false))]` reads as the counterpart of `method(update = false)`, and `update = false` does what it says: no setters, no update methods. `delete = false` removes nothing. The fixture `derive/tests/fixtures/methods_disabled.rs` declares both flags, and its `table.snap` manifest still lists `delete_audit_entry_by_id` and `delete_audit_entries_by_actor_id`.

The flag reaches only two decisions, neither of which is method generation: `foreign_key.rs:115` rejects an `on_delete = Delete` foreign key while it is set, and `internal.rs:177`/`internal.rs:184` reject a before- or after-delete hook. `method.rs` never reads `has_delete_method` at all. Those two rejections are the flag's entire user-visible effect, and both are pinned by `compile-tests/tests/ui/before_delete_hook_without_delete_method.rs` and `after_delete_hook_without_delete_method.rs`.

Recommendation: decide which of the two readings is intended. Making the flag suppress the delete methods matches its name and its sibling, but it is a breaking change for any module that sets `delete = false` and calls a delete method today (developer decision). Renaming it — to something naming the constraint it actually imposes, such as forbidding cascading deletes and delete hooks — keeps behavior and fixes the surprise. What should not survive is the current pairing, where two flags spelled the same way mean different kinds of thing.

### A `#[unique]` column gets no update method, but a unique multi-column index does

**Violates:** Principle of Least Astonishment; Don't Repeat Yourself (_One authoritative source for each business rule_); Connascence of Algorithm

Both index shapes produce the same `SpacetimeDSLColumnMethodsForUniqueIndex` value, and each decides independently whether to fill its `update` field. The single-column path at `method.rs:163` requires `has_update_method && method_is_for_primary_key`; the multi-column path at `method.rs:366` requires `has_update_method` alone. So a row can be updated by a unique multi-column index but not by a unique single-column one.

The fixtures show it directly: `unique_single_column_index` generates `get_account_by_external_id` and `delete_account_by_external_id` but no `update_account_by_external_id`, while `unique_multi_column_index` generates `update_seat_by_row_and_number` alongside its getter and deleter. It holds across the whole suite. Every other fixture with a non-primary-key unique column — `direct_index`, `string_index_column`, `wrapper_created_named`, `wrapper_created_unnamed`, `wrapper_used`, `hooks_all_six` and `delete_hooks_with_foreign_key_on_unique_index` — generates a getter and a deleter for that column and no updater, while `multiple_dsl_attributes`, the only other fixture with a unique multi-column index, generates `update_module1_by_database_and_name` exactly as `Seat` does.

Recommendation: the rule "which indices can update a row" is currently expressed twice, in two places that disagree. Decide it once — either a unique index of any width may update(developer decision), or only the primary key may — and have both paths read that one decision. Note that this is not only a cleanup: if the multi-column behavior is the intended one, single-column unique indices are missing a method users can reasonably expect(developer decision), and if the single-column behavior is intended, the multi-column path is generating a method that should not exist.

A third case falls out of the defect below: a unique _hash_ column takes the multi-column path by accident, so it does get an update method (developer decision: it should!). Fixing that routing will silently change which of the two rules applies to it, which is a further reason to settle the rule first.

### A single-column hash index is routed through the multi-column path

**Violates:** Principle of Least Astonishment; Duplication Control & Reuse; Unused scaffolding removed immediately

`db/column.rs:63` walks the table's indices to find the one belonging to the column being processed, and moves it onto that column as `single_column_index`. The loop matches `BTreeSingleColumn` and `Direct`. It does not match `HashSingleColumn`.

A `#[index(hash)]` column therefore never gets a `single_column_index`, `SpacetimeDSLColumnMethods::map` returns `None` for it, and the index stays in `multi_column_indices` — where the loop at `method.rs:329` branches on `is_unique` alone, without checking that the index has more than one column. The methods that come out are named correctly, which is why this has gone unnoticed, but they are generated by the wrong branch.

Two consequences are visible in the `hash_index` fixture. `device_id`, which is `#[index(hash)] #[unique]`, gets `update_session_by_device_id` — the method the section above shows a unique _btree_ or _direct_ column does not get, because the multi-column path omits the primary-key condition. And the `HashSingleColumn` arm at `method.rs:156` is dead: it tests whether a single-column hash index is the primary key, and no single-column hash index ever reaches it.

Independently of the routing, every doc comment the fixture generates calls the index a btree index — `method.rs:925` and `method.rs:935` hard-code that word in arms which also handle `HashSingleColumn` and `HashMultiColumn`. This one is a straightforward defect with no decision attached: the text is simply wrong for hash indices, and it is emitted into the documentation users read.

Recommendation: add `HashSingleColumn` to the extraction loop, which routes hash columns like every other single-column index and makes the dead arm live. Do it together with the update-rule decision above, because it changes which rule applies to unique hash columns. Then either gate the `multi_column_indices` loop on the index actually being multi-column, or accept that it is the generic path and rename it — a list called `multi_column_indices` that holds single-column indices is exactly the kind of naming that hid this. The hard-coded `"btree index"` should be derived from the index type in both arms and can be fixed on its own.

---

## Recommended Resolution Order

1. **Add characterization tests for the generator.** Done — see [`.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md`](.ai/plans/1_ADD_CHARACTERIZATION_TESTS.md). Snapshot tests over the generated output and compile tests over the rejection diagnostics now run with `./x.sh unit-test`, so every step below has the safety net `AGENTS.md` requires before refactoring.
2. **Resolve the dead code.** Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 2.
3. **Delete the commented-out `set_none_strategy` blocks.** Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 3.
4. **Investigate and fix the defects the tests expose.** Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 4.
5. **Extract the mechanical duplication.** Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 5. Scoped to `method.rs`; the same literals in the other generators are recorded above as a remaining finding.
6. **Replace string-based type classification** with a classification resolved once onto `InternalColumn`. Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 6. Do this before the structural split, so the split does not carry the string comparisons into several new modules.
7. **Clean up signatures.** Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 7. Also folds in the hand-written `Display` implementations and the deletion of `CreateOrUpdate`, which had no resolution-order item of their own.
8. **Split `reference_integrity_checks_on_create_or_update` per mode** and fix the `try_parse` naming and error strategy. Planned — see [`.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md`](.ai/plans/2_CLEAN_UP_METHOD_GENERATOR.md), step 8. Also rejects a singleton carrying a multi-column index, which panics today.
9. **Break up `for_method` into one generator per `DSLMethod` variant**, reshaping the enum so the "already processed" panics become unrepresentable. This is the largest single change and depends on steps 5 through 7 having removed the noise.
10. **Introduce the generation-context struct and remove the `&mut` mutation**, returning what each generator wants recorded instead of writing through the table reference. Doing this after step 9 means threading the context through a set of small functions rather than through one enormous one.
11. **Understand and then consolidate the drifted duplication** — the `DeleteOne`/`DeleteMany` assembly, the `OneOrMultiple` branch pairs, the `OneOrMultiple` semantic overload, and the `column_names_and_row_values` builders. Each requires deciding which behaviour is correct before merging the copies, which is why these come after the mechanical work rather than with it.
12. **Consolidate singleton handling** behind a single definition of the singleton contract, once the per-variant generators from step 9 make the special cases visible side by side.
13. **Split the module into domain modules**, and regroup the long paired field names behind small structs. Last, because the seams are only clean once the preceding steps have separated the responsibilities.
14. **File issues for the unlinked `TODO`/`FIXME` markers** and reduce each in-source marker to an issue reference, triaging the swallowed hook error as a defect rather than a cleanup.
15. **Decide what `method(delete = false)` means** — either suppress the delete methods(developer decision), or rename the flag to the constraint it actually imposes. Appended after the original fourteen so their numbering keeps its existing references; in priority it is independent of steps 2 through 14 and can be taken at any point, because the `methods_disabled` fixture records whichever answer is chosen.
16. **Unify the rule for which unique indices may update a row**, so the single-column and multi-column paths stop deciding it separately. Best taken with step 11, which consolidates the rest of the drifted duplication. (developer decision: all get one, so single-column path must get a unique DSL method too)
17. **Route single-column hash indices like every other single-column index**, by adding `HashSingleColumn` to the extraction loop, and derive the `"btree index"` wording from the index type instead of hard-coding it. The routing change belongs with step 16, because it decides which update rule applies to unique hash columns; the doc-comment wording is independent and can be fixed immediately.
