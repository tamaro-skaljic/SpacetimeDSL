# Code Quality Report

Scope: `derive-input/src/internal/dsl/method.rs`, reviewed against the programming principles in [`AGENTS.md`](AGENTS.md).

## `derive-input/src/internal/dsl/method.rs`

This module is the code generator for every DSL method SpacetimeDSL emits: `create`, `get_all`, `get_count`, index-based `get`/`update`/`delete`, referential-integrity checks, and on-delete strategy execution. It is by far the largest module in the crate — larger than every other module in `internal/dsl` combined.

The central problem is that one module, and inside it one function, owns every decision for every generated method: naming, doc comments, argument shapes, return types, and body assembly. Because there is no seam between "which method are we generating" and "how is that method assembled", every variant's logic is expressed as nesting inside a single control-flow tree, and every shared concept (hook emission, wrapper-type unwrapping, format-string construction, one-row-versus-many-rows handling) is re-implemented at each site where it is needed rather than expressed once.

Two consequences are already visible in the code rather than merely predicted:

- Knowledge that exists in more than one place has **drifted apart**, and at least one of the drifted copies produces a defect (see `column_names_and_row_values`).
- Branches exist that cannot ever have been executed, because the tokens they emit are not valid Rust (see the wrapper-type option mapper).

Neither of these could have been caught, because the crate has no automated tests at all. `examples/test` is a `cdylib` that exercises the macro by compiling against it; it proves the happy paths compile, it asserts nothing, and it does not cover branches that no example happens to trigger.

The violations below are described individually, but they share a single root: the module never paid the abstraction cost that its duplication had already earned.

---

### `method.rs`: module size and placement

**Violates:** Maximize Cohesion; Single Responsibility Principle; Optimize for Deletion (_Keep modules small enough to rewrite in a week_); Separation of Concerns

The module holds, in one file: the `DSLMethod` / `OneOrMultiple` / `CreateOrUpdate` / `Action` type definitions, the two public entry points (`SpacetimeDSLColumnMethods::map` and `SpacetimeDSLTableMethods::try_parse`), the monolithic method generator, foreign-key and referenced-by generators, on-delete strategy generation, multi-column-index uniqueness checks, and a family of identifier-naming helpers. These are separate responsibilities with separate change drivers — a change to naming conventions, a change to the delete protocol, and a change to argument typing all land in the same file.

Recommendation: split along the domain axis that `AGENTS.md` prescribes for the Maximize Cohesion / Separation of Concerns conflict ("split in domain modules wiring together technical modules"). Natural seams: one module per DSL method family (create, read, update, delete), one module for referential integrity / on-delete strategies, one module for identifier naming, and a small module for the shared generation context. Doing this before the structural refactors below would help; doing it after would also work. The order matters less than doing it once the pieces are separable — which is currently blocked by the god function described next.

### `method.rs`: `for_method(dsl_method, rust_struct, spacetimedb_table, spacetimedsl_table, internal_columns, primary_key_column) -> SpacetimeDSLMethod`

**Violates:** Single Responsibility Principle; Curly's Law; Maximize Cohesion; KISS; Code For The Maintainer; Open/Closed

This one function generates every DSL method variant. It is the overwhelming majority of the module. Inside it, control flow nests to a depth where the reader cannot tell which branch they are in without scrolling back, and the innermost `quote!` blocks sit far from the condition that selects them.

The function also uses the declare-now-assign-deep-inside-a-branch pattern for `doc_comment`, `method_name`, `return_type`, `method_impl`, and `additional_paths_to_use`. The compiler enforces that every path assigns them, so the code is correct, but a reader tracing "what is `method_impl` for a unique multi-column `DeleteOne`" must locate one assignment among many scattered across the branch tree. This is a puzzle the maintainer solves by hand on every visit.

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

Recommendation: make the writes part of the return value. Have each generator return its produced method _together with_ whatever it wants recorded (the create-argument struct, the set of compile-error checks), and let the single orchestrating caller apply them to the table. This also removes the reason `try_parse` currently takes the table by value and hands it back.

### `method.rs`: `SpacetimeDSLTableMethods::try_parse(...) -> syn::Result<(SpacetimeDSLTableMethods, SpacetimeDSLTable)>`

**Violates:** Principle of Least Astonishment; Robustness Principle; Self-Documenting Code (descriptive identifiers); Hide Implementation Details

Three separate problems in one signature:

- The name says `try_parse`, but the function parses nothing. It consumes already-parsed models and generates methods. A reader looking for the parsing stage will land here and be misled.
- It returns `syn::Result`, promising fallibility, but no path in it ever produces an `Err`. Meanwhile the module's genuine error cases — a user writing foreign keys with mismatched types or mismatched paths — `panic!` instead of returning `Err`. The error channel that exists is unused, and the cases that need it bypass it.
- It takes `SpacetimeDSLTable` by value and returns it, purely to work around the `&mut` aliasing described above. This ownership shuffle is not expressing anything about the domain.

Recommendation: rename to describe what it does (for example `generate` or `for_table`). Then decide, deliberately, which of two directions to take the `Result`:

- **Keep `syn::Result` and start using it (developer decision)** — convert the user-facing `panic!`s into `syn::Error` with the offending span, so invalid attribute usage produces a pointed compiler diagnostic instead of a macro panic. This is the better outcome for users and is what the Robustness Principle asks for.
- **Drop `syn::Result`** — only defensible if the developers conclude that every remaining failure really is an internal invariant violation.

These are not equivalent, and the choice affects the module's whole error strategy, so the developers should make it explicitly rather than by default. Once the `&mut` mutation is removed, the by-value/return-it-back parameter can become a plain reference.

### `method.rs`: `additional_paths_to_use`

**Violates:** Unused scaffolding removed immediately; YAGNI; Optimize for Deletion

`additional_paths_to_use` is declared as an empty vector, passed into `reference_integrity_checks_on_create_or_update`, returned from it completely unchanged, reassigned from the returned tuple, and finally stored on the generated method — where the downstream output stage faithfully emits `use <path> as _;` for each entry. It is never pushed to, anywhere in the codebase. The other generators simply declare it as an empty vector and store it. The entire round trip is a no-op that complicates two signatures and one return type.

Recommendation: the developers must decide for themselves whether this is genuinely no longer needed (developer decision) or whether the fact that it is unused is actually a bug — that is, whether `reference_integrity_checks_on_create_or_update` was _supposed_ to collect the paths of referenced tables so the generated code can `use` them, and silently stopped doing so. The generated `use ... as _;` emission downstream suggests a real intended purpose. If the feature is wanted, restore the population; if not, delete the field, the parameter, the tuple return, and the downstream emission in the same pass.

### `method.rs`: `get_referencing_table_trait_name(...)` and its discarded call

**Violates:** Unused scaffolding removed immediately; Optimize for Deletion; Boy Scout Rule

`for_referenced_by` computes `referencing_table_trait_name`, then discards it with `let _ = &referencing_table_trait_name;` and an inline comment stating the trait is no longer generated and an inherent `impl` is used instead. The helper function that builds the name is otherwise unreferenced, and the PascalCase conversions of the table names exist only to feed it.

Recommendation: the developers must decide for themselves whether this trait naming is genuinely no longer needed (developer decision) or whether its absence is actually a bug — the comment claims the trait was intentionally replaced, but a comment is not verification. If the replacement is complete, delete the helper, the discarding statement, and the PascalCase values that only feed it. The comment explaining the removal belongs in the commit message, not in the source.

### `method.rs`: `strategy_before_all`

**Violates:** Unused scaffolding removed immediately; YAGNI; KISS

`strategy_before_all` is bound to an empty `quote! {}` and never reassigned, yet it is interpolated into both output arms of `get_on_delete_strategy_implementation` as a placeholder. It emits nothing.

Recommendation: the developers must decide for themselves whether this is dead scaffolding or a missing implementation — the name and its symmetry with `strategy_after_all` (which _is_ populated) suggest a slot someone intended to fill. Either populate it or remove the binding and both interpolations.

Developer decision: Keep, even if it violates YAGNI and KISS that it is there and unused at the moment.

### `method.rs`: commented-out `set_none_strategy` blocks and in-`quote!` TODO markers

**Violates:** Unused scaffolding removed immediately; Optimize for Deletion; Documentation & Communication Clarity (_Future ideas captured outside codebase_, _Log blockers to future cleanups for retrospectives_)

The `SetNone` on-delete strategy is present as commented-out code in several generator functions, plus `//TODO ... #set_none_strategy` markers _inside_ `quote!` bodies, meaning the comment is emitted into the generated source that users read. The work is already tracked in a linked issue.

Recommendation: delete every commented-out `set_none_strategy` block and the in-`quote!` markers. The issue is the correct home for the deferred work, and it already exists — the commented code adds nothing the issue does not already carry, while making every surrounding function longer and noisier. If the commented code contains a design detail not captured in the issue, move that detail into the issue first, then delete.

### `method.rs`: `column_names_and_row_values` format-string construction

**Violates:** Don't Repeat Yourself (_One authoritative source for each business rule_); Duplication Control & Reuse; Connascence of Algorithm

The format string that describes "these columns had these values", used in `NotFoundError` and `UniqueConstraintViolation` messages, is built by imperative string pushes in more than one place: once inside `for_method` (with a branch per `IndexType`) and again inside `multi_column_index_checks`. The two builders produce **different** formats — one wraps the result in braces, the other does not — and the placeholder-to-argument correspondence is maintained by hand in parallel with a separately built list of row-value getters.

The copies have already drifted into a defect. In `for_method`, the opening brace is pushed first and then the single-column branches push a separator-prefixed column segment, so single-column indices produce a message with a stray leading comma before the first column name. The multi-column branch pushes its first column without the separator and is correct (developer decision). This is precisely the failure mode duplication causes: one copy was fixed, the other was not.

Recommendation: this is a case where the logic has drifted and carries an invariant — the format string's placeholder count must always match the row-value getter list. The developers should first understand the differences between the copies and decide which output format is the intended one (braced (developer decision) or unbraced, and what the correct separator placement is), because the messages are user-facing and changing them changes what users see. Once decided, extract a single builder that emits the format string **and** the matching getter list together as one value, so the two can no longer disagree. Add a test pinning the exact message text for a single-column index, a multi-column index, and a direct index before changing anything, so the fix is provably a fix.

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

`OneOrMultiple` means "one row or multiple rows" everywhere except in the `Update` path, where it is derived from `is_multi_column_index` — a single-column index maps to `One` and a multi-column index maps to `Multiple` — and passed to `reference_integrity_checks_on_create_or_update` to select a format-string shape. There, the type is silently repurposed to mean "one column or multiple columns". The same value also reaches generated code as a user-visible `OneOrMultiple` in error payloads.

This is exactly the coupling `AGENTS.md` warns about: two call sites agree on a meaning that the type name contradicts, and nothing enforces it.

Recommendation: the developers should determine what the generated error payload is _supposed_ to report in the multi-column update case, since the current behaviour may be reporting a column-count fact in a field documented as a row-count fact. Once that is settled, give the column-arity concept its own type (or pass the column list and let the callee decide), so `OneOrMultiple` retains one meaning.

Developer decision: The type name contradicts nothing, as it doesn't state it is for columns, rows, functions or whatever. "OneOrMultiple" is deliberately just a differentiator between "One" or "Multiple" and will never change. That it is used everywhere for "rows" except at one site where it is used for "columns" does not mean that it was created to be only used for "rows", instead it was intentional design that the type gives only information about whether there is "one" or there are "multiple" of a generic something. Keep it one type, do not introduce multiple types for THE SAME concept.

### `method.rs`: hook token emission

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse

Every hook — before/after insert, before/after update, before/after delete — is emitted by the identical shape: match the `Option` on the table's hooks, return `TokenStream::default()` for `None`, and for `Some` destructure `trait_name` and `function_name` and build a `quote!` that emits a `use self::<trait>;` followed by a `DSLMethodHooks::<function>` call. The delete hooks in particular are re-written verbatim in each delete-generating branch, including the singleton branch.

Recommendation: extract a single helper that takes the optional hook and the call shape and returns the tokens. This is low-risk, mechanical, and does not require any behavioural decision — the copies are genuinely identical here. Do it early; it shrinks the delete paths enough to make the harder drift analysis above easier to perform.

### `method.rs`: wrapper-type struct path extraction

**Violates:** Don't Repeat Yourself; Law of Demeter; Duplication Control & Reuse

Resolving a `WrapperType` to its struct name or path — matching `Created` to `wrapper_struct_name` and `Used` to `wrapper_struct_name_or_path`, then `to_token_stream()` — is written out inline at each site that needs it, each time preceded by an `expect` on the primary key column's optional wrapper type, and each time with a _differently worded_ expect message for the same condition.

The inline matching also reaches through the primary key column into its wrapper type into that wrapper's fields — a Law of Demeter violation that couples this module to `WrapperType`'s internal variant layout, so adding a variant forces edits here.

Recommendation: add a method on `WrapperType` that returns the struct name or path as tokens, and call it. Standardise the single expect message. This is mechanical and safe.

### `method.rs`: type classification by string comparison

**Violates:** Connascence of Meaning; Robustness & Reliability; Code For The Maintainer; Don't Repeat Yourself

Column types are classified throughout by rendering them to a token stream, converting to `String`, and comparing text: equality against `"String"`, `starts_with("Option")`, and a match over `"u8" | "u16" | "u32" | "u64" | "u128"`. One of these sites calls `.trim()` first; the others do not. The same classification question is asked repeatedly at different sites, each re-deriving the answer.

This silently misclassifies any type the user writes differently from the expected spelling — a fully qualified path, or a spacing the token renderer does not produce identically — and the failure is not a diagnostic but wrong generated code. The `.trim()` inconsistency shows that whitespace sensitivity has already bitten someone at one site and was patched there only.

Recommendation: classify each column once, when `InternalColumn` is built, into an explicit enum capturing what the generator actually needs to know (is it a `String`, is it an `Option`, is it an unsigned integer, what is the inner type). Store that on the column and have the generator read the field. This gives one authoritative source, removes the string comparisons entirely, and turns misclassification into something that can be detected and reported where the user's attribute is, with a span. Note that the "remove `Option` from the type representation" helper already referenced in a TODO comment in this module is part of the same concern and should move with it.

### `method.rs`: `process_columns_for_create_and_update_method(spacetimedsl_table, create_or_update, internal_column) -> (Option<SpacetimeDSLArg>, Option<TokenStream>, Option<TokenStream>, TokenStream)`

**Violates:** Self-Documenting Code (descriptive identifiers, no abbreviations); Principle of Least Astonishment; Single Responsibility Principle; Interface Segregation Principle

The name is plural but the function handles exactly one column. It returns an unlabeled four-element tuple of mostly-`Option` token streams, so every call site must know the positional meaning of each slot — and one call site destructures it as `(_, _, column_getter, _)`, discarding three quarters of the work the function just did.

Internally it serves two different callers through a mode flag, with early returns that apply to only one mode, so reading the Create path means skipping over Update concerns and vice versa.

Recommendation: rename to the singular. Replace the tuple with a named struct whose fields say what they are — this alone removes the positional coupling at every call site. Then decide whether Create and Update should be separate functions (developer decision: yes): they share the wrapper-type handling but differ in whether they read the field directly or through a getter, and in Create's auto-increment/timestamp/singleton pre-handling. Splitting them and sharing only the wrapper-type logic is probably cleaner, but the developers should confirm the shared portion really is identical before splitting, since a split that duplicates drifting logic would trade one violation for another.

Developer decision: Split into separate functions. While they share wrapper type handling in this isolated function, the update path is discarding them silently anyway, so they are only relevant for the create-path and can be removed from the update-related function extracted from this function. Note that `wrapper_type_option_to_wrapped_type_option_mapper` is only irrelevant for update paths in this specific, isolated function. There are other functions which have a local variable with the same name and there it is assigned in several ways, so the change is really only isolated to this one function and the two call sited.

### `method.rs`: `reference_integrity_checks_on_create_or_update(...)`

**Violates:** Interface Segregation Principle; Principle of Least Astonishment; Command Query Separation

Beyond the `additional_paths_to_use` pass-through already described, this function takes `column_names_and_row_values_and_column_names: Option<(&String, &Vec<Ident>)>` — a parameter that must be `Some` for the Update mode and is `None` for Create, enforced only by an `expect` at runtime. The signature therefore advertises a contract it does not encode: callers must know which mode requires which arguments.

The parameter name itself concatenates two descriptions, and the body immediately re-splits the tuple into locals with better names.

Recommendation: split into two functions, one per mode, each taking exactly the arguments its mode needs. This is the ISP-aligned fix ("Split fat interfaces so clients receive only needed methods") and eliminates the runtime `expect`. If the shared body is substantial enough that splitting would duplicate it, extract the shared part and have both modes call it — but the mode-specific parameters should not survive in a merged signature.

### `method.rs`: boolean and positional parameters in helper signatures

**Violates:** Connascence of Position; Principle of Least Astonishment; Code For The Maintainer

`strategy_by_row(mut_row, is_unique_index, row_finder, strategy_by_row)` is called with bare boolean literals at every site, so the call sites read as two unexplained booleans followed by token streams. `for_foreign_key` similarly takes a bare `has_referenced_bys` boolean positioned among several references. A transposition of two booleans compiles and produces subtly wrong generated code.

The final parameter of `strategy_by_row` also shares its name with the function itself, so the body reads ambiguously.

Recommendation: replace the booleans with small named enums (mutable-versus-immutable binding; unique-versus-non-unique index), which makes call sites self-documenting and transposition a compile error. Rename the shadowing parameter. This is mechanical and low-risk.

### `method.rs`: signature types across the module

**Violates:** Hide Implementation Details; Principle of Least Astonishment; Code For The Maintainer

Several signatures over-constrain their callers or leak representation:

- `&Vec<InternalColumn>`, `&Vec<TokenStream>`, and `&Vec<Ident>` appear where a slice would do, forcing callers to own a `Vec`.
- `&String` appears where `&str` would do.
- `columns_with_foreign_key: &Vec<&&Column>` and `columns_by_on_delete_strategy: Vec<&&&Column>` expose multiple levels of reference nesting that exist only as an artifact of how the collections were built upstream, not because the domain has three levels of indirection.
- `get_unique_multi_column_index_check` is declared `pub(in crate::internal::dsl::method)` — a visibility scoped to the module that defines it, which is exactly private, written in a way that implies otherwise.

Recommendation: take slices and `&str`; flatten the reference nesting by collecting owned references once at the point of construction; make the module-scoped function plainly private. All mechanical, all safe, all reduce reader friction — this is the Boy Scout Rule applied to signatures that are touched anyway during the larger refactors.

### `method.rs`: singleton handling scattered across the module

**Violates:** Encapsulate What Changes; Open/Closed; Don't Repeat Yourself; Connascence of Value

The singleton table concept is special-cased independently in the column-method mapper, the table-method entry point, the create/update column processor (where the primary key is identified by the literal field name `"id"`, the literal type `"u8"`, and filled with `0u8`), the method generator (a dedicated `is_singleton_pk` flag driving separate doc comments, method names, get, update, and delete bodies), and the on-delete strategy generator (a different row finder and a different row-value format). The magic values `"id"`, `u8`, `0u8`, and the literal message text `"{ id : 0 }"` recur across these sites with no shared definition.

Changing anything about how singletons are represented therefore requires finding and editing every one of these sites, and missing one produces inconsistent generated code rather than a compile error.

Recommendation: define the singleton contract in one place — the sentinel primary key name, its type, its value, and its rendered representation — and have every site read from it. Then consider whether the singleton generation paths should be selected once (developer decision) at the top (a distinct generation strategy) rather than re-tested inside each branch. The developers should decide how far to take this: a single shared constant set is cheap and clearly correct, whereas a separate singleton generation path (developer decision) is a larger structural change that only pays off if singleton behaviour continues to diverge.

### `method.rs`: fully qualified generated paths repeated inline

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse

Generated code references `crate::spacetimedsl::error::SpacetimeDSLError`, `crate::spacetimedsl::delete::DeletionResult`, `crate::spacetimedsl::delete::DeletionResultEntry`, `crate::spacetimedsl::internal::DSLInternals`, and `crate::spacetimedsl::DSLMethodHooks` as literal paths written out at each use inside `quote!` bodies. Renaming or relocating any of these runtime items means a text hunt through this module, with no compiler assistance, because the paths only exist as tokens.

Recommendation: define each path once as a helper returning the tokens (or as a set of small constructor helpers for the common error and entry constructions) and interpolate them. This gives a single authoritative source for the runtime surface this generator targets, and makes the generated-code/runtime-crate contract explicit rather than implied by repeated literals.

### `method.rs`: `CreateOrUpdate`, `Action`, and their `Display` implementations

**Violates:** Don't Repeat Yourself; Simplicity & Right-Sized Solutions

`CreateOrUpdate` and `Action` overlap — `Action` contains `Create` and `Update` alongside `Get` and `Delete`, so the smaller enum is a subset of the larger one used for a narrower purpose. Both carry hand-written `Display` implementations that do nothing but echo the variant name, even though `strum` is already a dependency of this crate and derives exactly that. `CreateOrUpdate` is compared with `.eq(...)` at one site and destructured with `match` at another, for no reason.

Recommendation: derive `Display` rather than hand-writing it. Then decide whether the two enums should be unified: if `CreateOrUpdate` exists to make invalid states unrepresentable in the create/update path, keeping it is justified and the duplication is acceptable — but that intent should be stated. If it exists only because `Action` was introduced later, collapse them.

### `method.rs`: generated identifiers and the field names built from them

**Violates:** Code For The Maintainer; Prefer clarity over cleverness; Simplicity & Right-Sized Solutions

Field and local names such as `execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted` are long enough that the formatter is forced to break `let mut` onto its own line, separating the binding keyword from the name. Assignments to these names dominate the visual weight of the function that builds them.

`AGENTS.md` forbids abbreviation, so shortening these by truncating words is not an option and should not be attempted.

Recommendation: shorten by _namespacing_, not abbreviating. Group the related pair into a small struct — for example a type holding `after_one_row_was_deleted` and `after_multiple_rows_were_deleted` — so the shared prefix lives in the type name and each field keeps a fully spelled, unabbreviated name. This satisfies both the no-abbreviation rule and readability. Note that the _generated_ function names (which users see and call) are a separate question and should not be changed without considering the effect on the public generated API.

### `method.rs`: panics used for both internal invariants and user input errors

**Violates:** Robustness & Reliability (_Log and surface malformed partner payloads immediately_); Principle of Least Astonishment; Code For The Maintainer

The module panics in two distinct situations that it does not distinguish:

- **Internal invariants** — "should already be processed", "When this code is called, it should be a single column index", and the many `expect` calls on collection lookups. These are the generator asserting its own consistency.
- **User input errors** — mismatched foreign key column types and mismatched foreign key paths both `panic!` with a prose message. These are reachable purely by a user writing a valid-looking but unsupported attribute combination, and the result is a proc-macro panic with no span, rather than a compiler error pointing at the offending attribute.

Recommendation: separate the two. (developer decision) User input errors should become `syn::Error` values carrying the span of the offending field or attribute, surfaced through the `syn::Result` that the entry point already returns but never uses (see the `try_parse` entry above — the two changes belong together). Internal invariants may remain panics, but each should state the invariant it protects. The developers should audit which of the current `expect` calls are genuinely unreachable and which are user-reachable; that audit is the deciding input, and it cannot be made from the message text alone.

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

### `method.rs`: unresolved `FIXME` and `TODO` markers

**Violates:** Documentation & Communication Clarity (_Future ideas captured outside codebase_, _Log blockers to future cleanups for retrospectives_); Refactoring & Change Containment

The module carries a mix of markers. Some are linked to tracked issues — the `try_update` replacement, the `SetNone` strategy, doc comments influenced by foreign-key attributes. Others are not linked to anything: an unnecessary clone in the create path, an error message that shows all columns where only the unique ones are relevant, row-value getters for wrapper types described as being built in the wrong shape, a hook error that is swallowed because propagating it would require a signature change, and the multi-column `String` handling question noted above.

Several of these markers are inside `quote!` bodies and are therefore emitted into the code users read.

Recommendation: open issues for the unlinked markers and reduce each in-source marker to a one-line reference to its issue, so the rationale lives in the tracker where it can be prioritised. Remove markers from inside `quote!` bodies entirely — generated code should not carry the generator's internal notes. Note that the swallowed hook error is not merely a cleanup: it silently discards a user-supplied error, which is a behavioural defect and should be triaged as one rather than left as a comment.

---

## Recommended Resolution Order

1. **Add characterization tests for the generator.** Snapshot the generated output for the table shapes listed above, plus compile-failure tests for invalid attribute usage. Nothing else in this list can be done safely first — `AGENTS.md` requires tests before refactoring, and this module currently has none.
2. **Resolve the dead code.** `additional_paths_to_use`, `get_referencing_table_trait_name` and its discarded call, and `strategy_before_all`. Each requires a developer decision on whether the code is genuinely obsolete or whether its disuse is a bug; make that call explicitly, then delete or restore in a single pass including downstream emission.
3. **Delete the commented-out `set_none_strategy` blocks and the in-`quote!` markers**, moving any detail they carry into the existing issue first. This shrinks several functions immediately and makes the following steps easier to read.
4. **Investigate and fix the defects the tests expose.** The stray leading comma in the single-column `column_names_and_row_values` format, and the apparently non-compiling optional-wrapper-type mapper. Both are user-visible; both need the tests from step 1 to confirm the fix.
5. **Extract the mechanical duplication.** Hook token emission, wrapper-type struct path resolution, and the repeated fully qualified runtime paths. These copies are genuinely identical, so no behavioural decision is needed, and removing them materially reduces the size of the functions analysed in later steps.
6. **Replace string-based type classification** with a classification resolved once onto `InternalColumn`. Do this before the structural split, so the split does not carry the string comparisons into several new modules.
7. **Clean up signatures.** Slices instead of `&Vec`, `&str` instead of `&String`, flattened reference nesting, named enums instead of boolean parameters, named structs instead of unlabeled tuple returns, corrected visibility, and the `process_columns_for_create_and_update_method` rename. Mechanical, and it makes the call graph legible before it is rearranged.
8. **Split `reference_integrity_checks_on_create_or_update` per mode** and fix the `try_parse` naming and error strategy, converting user-input panics into spanned `syn::Error`s. These two are coupled — the error strategy decision determines whether the `syn::Result` stays.
9. **Break up `for_method` into one generator per `DSLMethod` variant**, reshaping the enum so the "already processed" panics become unrepresentable. This is the largest single change and depends on steps 5 through 7 having removed the noise.
10. **Introduce the generation-context struct and remove the `&mut` mutation**, returning what each generator wants recorded instead of writing through the table reference. Doing this after step 9 means threading the context through a set of small functions rather than through one enormous one.
11. **Understand and then consolidate the drifted duplication** — the `DeleteOne`/`DeleteMany` assembly, the `OneOrMultiple` branch pairs, the `OneOrMultiple` semantic overload, and the `column_names_and_row_values` builders. Each requires deciding which behaviour is correct before merging the copies, which is why these come after the mechanical work rather than with it.
12. **Consolidate singleton handling** behind a single definition of the singleton contract, once the per-variant generators from step 9 make the special cases visible side by side.
13. **Split the module into domain modules**, and regroup the long paired field names behind small structs. Last, because the seams are only clean once the preceding steps have separated the responsibilities.
14. **File issues for the unlinked `TODO`/`FIXME` markers** and reduce each in-source marker to an issue reference, triaging the swallowed hook error as a defect rather than a cleanup.
