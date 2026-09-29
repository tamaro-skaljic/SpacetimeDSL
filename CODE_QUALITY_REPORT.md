# Code Quality Report

This report checks the code of this repository against the programming principles and methodologies in [`AGENTS.md`](AGENTS.md). It covers the runtime crate (`src/`), the procedural macro crate (`derive/`), the generator crate (`derive-input/`), the test harnesses (snapshots in `derive/`, diagnostics in `compile-tests/`, runtime tests in `examples/test/`), the other examples, the developer scripts and the CI workflows. Findings are located by file and item name only, so they stay valid while code moves.

**Reading an entry.** The names after **Violates:** are the headings of AGENTS.md's *Programming Principles*, sometimes followed by the checklist item that is violated; *Self-Documenting Code* and *Test Driven Development* refer to its *Methodologies*. Where an entry describes code that nothing uses, or copies of one piece of logic that have drifted apart, the recommendation leaves the decision to the developers on purpose. Where several fixes are possible, the options are listed.

**Entries moved to the plan.** Every entry with a developer's decision that is not marked as independent was moved into [`docs/plans/2026-09-27-code-quality-developer-decisions.md`](docs/plans/2026-09-27-code-quality-developer-decisions.md), together with the task that carries it out. Where the plan covers only part of an entry, that part was moved and the rest stays here. This report therefore lists only the findings that are still open.

**Carrying out a recommendation.** Every recommendation assumes the workflow AGENTS.md prescribes, which is therefore not repeated in each entry:

- First write the cheapest test kind that can fail for the reason under test — a `.rs` / `.stderr` pair in `compile-tests/tests/ui` for a rejected input, a fixture in `derive/tests/fixtures` for changed output, an assertion in `examples/test/src` for run-time behaviour — and read its failure before changing production code.
- An entry that calls a change a *pure refactoring* expects `git diff derive/tests/snapshots compile-tests/tests/ui` to stay empty. A change that moves generated code or diagnostics regenerates them with `INSTA_FORCE_UPDATE` or `TRYBUILD=overwrite`, and the diff is read before it is committed.
- The gates run through `x.ps1` only, and a change ends with `.\x.ps1 format` until a second run changes nothing.
- Future work that a recommendation removes from the code (TODO and FIXME notes, commented-out code, plans) goes to the issue tracker (*Scope & Goal Discipline*: "Future ideas captured outside codebase").
- Knowledge that AGENTS.md or `docs/DOCUMENTATION.md` states is changed there in the same change (*DRY*: "Sync related artifacts—code, docs, tests—whenever knowledge changes").

---

## `src/lib.rs`

Root of the runtime crate: the public re-exports, the `Context` / `ReadContext` / `WriteContext` trait family that bounds every generated method, the `Wrapper` trait, and the `spacetimedsl!` macro that creates the `crate::spacetimedsl` module in a user's crate.

### `lib.rs`: `pub trait Context`

**Violates:** Interface Segregation Principle (ISP) — "Split fat interfaces so clients receive only needed methods"; Liskov Substitution Principle (LSP) — "Never strengthen preconditions"; Robustness Principle; Code For The Maintainer

`Context` requires every capability trait (`GetAuth`, `GetTimestamp`, `GetRandom`, `NewUUID`, `AsReducerContext`, …) from every SpacetimeDB context type. A context that lacks a capability implements it with a method that always returns `Err` — `ViewContext::timestamp()`, `AnonymousViewContext::sender()` and `ViewContext::as_view_context()` among others. Code that is generic over `Context` or `ReadContext` therefore cannot rely on the methods the bound promises, and what the type system already knows is only reported at run time.

The generator has to compensate for it: `derive-input/src/api/runtime.rs::current_timestamp` emits an `expect` into every generated write path, whose message explains that a `WriteContext` always has a timestamp — the trait makes a capability fallible that every `WriteContext` is guaranteed to have. The bound `T: WriteContext + ReadContext` on `DSL` repeats what `WriteContext: ReadContext + Context` already implies.

Recommendation: Let each context type implement only the capabilities it has, and let `ReadContext` / `WriteContext` require exactly the capabilities they guarantee, so misuse becomes a compile error and the generated `expect` can go. This breaks the public API, so the developers have to decide whether the uniform, always-available `Context` is a deliberate ergonomic choice. Options: (a) split the capabilities as described; (b) keep the uniform interface and document on `Context` which capability each context type provides; (c) keep `Context` for compatibility, offer capability-specific bounds for new code and deprecate the always-failing implementations. Independent of the choice, drop the redundant `ReadContext` bound on `DSL`.

### `lib.rs`: the capability implementations in `src/get_*.rs`, `src/as_*.rs` and `src/new_uuid.rs`

**Violates:** DRY; Encapsulate What Changes; Open/Closed; Maximize Cohesion; Self-Documenting Code — no abbreviations, document "why"

Every capability file defines its own `impl_<capability>_ok` / `impl_<capability>_err` macro pair of the same shape and then lists every context type, so adding a SpacetimeDB context type means editing every one of these files. The macro names are abbreviated (`impl_get_rng_err`, `impl_get_mut_db_ok`, `impl_get_db_err`), and so are public method names such as `rand`, `db` and `mut_db`. The marker implementations of `Context`, `ReadContext` and `WriteContext` are spread over `as_reducer_context.rs`, `as_view_context.rs` and `as_anonymous_view_context.rs`, files named after unrelated traits. The note `FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439` sits above failing implementations and above a marker implementation (`impl crate::ReadContext for spacetimedb::ReducerContext {}`), without stating what is wrong or what changes once the upstream issue is resolved.

Recommendation: Describe per context type, in one place, which capabilities it has (for example with one declarative macro invocation per context type), and keep the genuinely special implementations — `GetSender for TxContext`, `AsAnonymousViewContext for ViewContext` — hand-written next to it. The pattern has been stable across all files, so the abstraction is justified ("Delay abstractions until duplication patterns stay consistent"). Move the marker-trait implementations into one module about the context family. Replace the scattered FIXME links with one doc comment that states the current limitation and which implementations change when it is lifted. Renaming the internal macros is free; renaming public methods such as `rand` and `mut_db` is a breaking change the developers have to schedule.

## `src/as_view_context.rs`

Implements `AsViewContext` for each SpacetimeDB context type.

### `as_view_context.rs`: `impl AsViewContext for spacetimedb::ViewContext`

**Violates:** Code For The Maintainer — Principle of Least Astonishment; Robustness Principle

The implementation for `ViewContext` fails with a message saying that a View Context is only accessible inside views and reducers — the message claims the call works exactly where it just failed.

Recommendation: State the actual limitation (the linked upstream issue prevents producing a `ViewContext` from a `ViewContext`) and pin the message with a runtime assertion from a view. Resolve it together with the `Context` entry in `src/lib.rs`.

## `src/as_anonymous_view_context.rs`

Implements `AsAnonymousViewContext` for each SpacetimeDB context type.

### `as_anonymous_view_context.rs`: `impl AsAnonymousViewContext for spacetimedb::AnonymousViewContext`

**Violates:** Code For The Maintainer — Principle of Least Astonishment

The error says an Anonymous View Context is only accessible from a reducer context, while the `ViewContext` implementation in the same file returns `Ok`.

Recommendation: Let the message name the contexts that do provide it, and pin it with a runtime assertion.

## `src/delete.rs`

The types that describe on-delete strategies and the result of a cascading deletion, plus the textual rendering of that result.

### `delete.rs`: `pub enum OnDeleteStrategy`

**Violates:** DRY — "One authoritative source for each business rule"; Connascence; Scope & Goal Discipline — "Future ideas captured outside codebase"; Refactoring & Change Containment — "Unused scaffolding removed immediately"

The comment asking to copy and paste this enum into `derive_input::api::dsl::foreign_key` documents a manual copy, and the copy has already drifted: the derives differ (`PartialOrd`, `Ord` and `strum::EnumIter` exist only in `derive-input`), and so does the documentation of `Error` and `Delete`. The generator reaches the runtime enum by variant name (`crate::spacetimedsl::delete::OnDeleteStrategy::#variant`), so a renamed variant still compiles in both crates and fails only in a user's crate. The planned `SetNone` variant is kept as commented-out code behind an issue link, and the variants use `/** */` blocks that repeat the same preamble, unlike the `///` used elsewhere.

Recommendation: Give the enum one authoritative definition. Options: (a) a small dependency-free crate that `spacetimedsl` and `spacetimedsl_derive-input` both depend on (one more crate to publish); (b) keep the copies but pin their agreement with a test inside an existing harness — a `#[cfg(test)]` module in the root crate is never run by `x.ps1` (*A test that cannot run is not a test*); (c) keep only the variant names in `derive-input`, which it needs for parsing and ordering, and let the runtime enum own the documentation. Before merging, understand the differences between the copies — the generator relies on `Ord` and `EnumIter` for deterministic output — to make an informed decision which traits the single definition carries. Delete the commented-out `SetNone`.

### `delete.rs`: `DeletionResult::to_csv(&self) -> String` and `DeletionResultEntry::to_csv(&self, entry_id: u128, parent_entry_id: u128, message: String) -> (u128, String)`

**Violates:** Robustness Principle — "Enforce strict output formats before sending responses"; Separation of Concerns — "Keep persistence, formatting, business logic in separate classes"; Hide Implementation Details

The header row ends with a trailing comma and therefore has one field more than every data row. Values are neither quoted nor escaped, although `row_value` is the `Display` output of a wrapper (`ThingId { id: 5 }`) and a wrapper around a string key can contain commas. The recursive helper that threads the running id and the output buffer through its arguments is `pub`, so the traversal mechanics are public API, and the formatting lives inside the data types (`Display for DeletionResult` delegates to it).

Recommendation: The developers first decide what this output is. If other tools parse it, fix the header, quote the fields and document the format; if it is a report for humans, render it as one (for example as an indented tree) and stop calling it CSV. Either way make the recursive helper private or move the rendering into a formatting module, and pin the rendered text of a cascade with a runtime assertion. Decide it together with the wrapper `Display` in `derive-input/src/internal/dsl/wrapper.rs`, whose text every row contains.

## `src/error.rs`

The runtime error types and their messages.

### `error.rs`: `pub enum Action` and `pub enum OneOrMultiple`

**Violates:** DRY; Connascence — of Name

The generator keeps mirrors of these enums (`Action` in `derive-input/src/internal/dsl/method/reference_integrity.rs`, `OneOrMultiple` in `derive-input/src/internal/dsl/one_or_multiple.rs`) and turns their variants into runtime paths by name, so renaming either side breaks only the user's build.

Recommendation: Resolve the mirrors the same way as `OnDeleteStrategy` (see `delete.rs`). The plan's Task 24 makes sure every emitted variant appears in a snapshot fixture, so until then a rename at least shows up as a snapshot diff.

---

## `derive/src/lib.rs`

The procedural macro entry points `#[dsl]` and `#[hook]`, the helper derive `SpacetimeDSL`, and the injection of a singleton's primary key.

### `lib.rs`: `fn is_last_dsl_attribute(derive_input: &syn::DeriveInput) -> bool` and the commented-out `make_struct_fields_private`

**Violates:** YAGNI; Refactoring & Change Containment — "Unused scaffolding removed immediately"; Self-Documenting Code — don't document the past

`expand_dsl_attribute_parts` computes `_is_last_dsl_attribute` and discards it, and the function exists only for that call. `make_struct_fields_private` and its call are commented out under a TODO saying they are temporarily disabled to allow public primary key columns.

Recommendation: Dead code: the developers must decide whether making fields private is no longer needed (then delete the function, the commented-out block and the TODO, and keep the idea in the issue tracker if it is still wanted) or whether its being disabled is a bug (then re-enable it, with a snapshot fixture for a table with a public primary key and a runtime test that uses one).

### `lib.rs`: `fn expand_dsl_attribute_parts(args, item) -> syn::Result<ExpandedDSLAttribute>`

**Violates:** DRY — "One authoritative source for each business rule"; Robustness Principle; Self-Documenting Code — no redundant comments

Whether a table is a singleton is decided in more than one place: here, by looking for an identifier `singleton` among the top-level argument tokens, and in `derive-input` by the real argument parser. They can disagree — the scan also accepts `singleton` in a value position such as `plural_name = singleton` — and a disagreement injects `id: u8` into an ordinary table, followed by errors that have nothing to do with the user's input. Comments such as "Parse the input tokens into a syntax tree" and "Build the output, possibly using quasi-quotation" repeat the code.

Recommendation: Parse the `#[dsl]` arguments once in `derive-input` and hand the result to the injection, or move the injection into `derive-input` (next entry). Delete the comments that repeat the code.

### `lib.rs`: `fn inject_singleton_primary_key(derive_input: &mut syn::DeriveInput) -> syn::Result<()>`

**Violates:** DRY; Connascence — of Meaning, across crates; Separation of Concerns; Hide Implementation Details

The function's own doc comment says that the name, the type and the value of the injected key are spelled out again in `derive-input/src/internal/dsl/singleton.rs`, and that a change here needs the same change there. Its diagnostics are the only user-facing rejections outside `derive-input/src/internal/error.rs`, contradicting that module's statement that it holds every diagnostic. The injected field also leaks into user code: every `DefaultSingleton::get_default` has to write `id: 0` for a field the user never declared (`examples/test/src/singleton_with_default_test.rs`, `examples/test/src/singleton_with_foreign_key_test.rs`).

Recommendation: Move the injection, its constants and its diagnostics into `derive-input` next to `internal::dsl::singleton`, and let the `derive` crate call one API function. The key then has one home and the diagnostics join `internal/error.rs`. Whether users should keep writing the injected field is an API decision for the developers; options include a generated constructor that fills it in, or keeping it and documenting it where `DefaultSingleton` is documented.

## `derive/src/output.rs`

Assembles everything the macro emits from the parsed `Table`.

### `output.rs`: `pub fn build(input: &Table, first_dsl_attribute: bool) -> syn::Result<GeneratedOutput>`

**Violates:** Code For The Maintainer; Law of Demeter — "Avoid chaining through returned collaborators"

The comment saying that wrapper types are only generated for the last DSL attribute contradicts the condition `if first_dsl_attribute`. The function navigates deep into `derive-input`'s model (`column.spacetimedsl_column.getter`, `input.spacetimedsl_table.compile_error_checks`), coupling this crate to that model's layout. `format_ident!("{}", &input.rust_struct.name.to_string())` rebuilds an identifier it already has.

Recommendation: Correct the comment and use the identifier directly. The plan documents `Table` as a deliberate data-transfer structure (its Task 13); decide whether `build` should still navigate that structure this deeply or whether the parts it needs deserve intention-revealing accessors. This is a pure refactoring.

## `derive/src/output/function.rs`

Renders a `SpacetimeDSLMethod` as inherent methods on `DSL` and `ReadOnlyDSL`, or on `DSLInternals`.

### `function.rs`: `pub fn build_public`, `pub fn build_internal`, `struct MethodGenerationConfig` and `enum MethodImplTarget`

**Violates:** KISS; YAGNI; Code For The Maintainer — Principle of Least Astonishment; Scope & Goal Discipline — "Future ideas captured outside codebase"

`MethodGenerationConfig` separates a "doc variant" from the "output variants", but `build_public` passes its first output variant as the doc variant as well, and `build_internal` passes its only variant as both. The `InternalDslInternals` target ignores the context bound it is constructed with (`let _context_bound = …`) and reads `runtime::write_context()` again, so the argument of `MethodImplVariant::internal` has no effect. These builders — like `create_method_arg::build` and `hook::build` — return `syn::Result` without any failure path. `InternalDslInternals` repeats "internal".

Recommendation: Render the doc comment from the first output variant and drop `MethodGenerationConfig`. The ignored bound is dead data: the developers must decide whether internal methods should honour it (then use it; the snapshots show whether anything changes) or not (then remove the parameter). Return `TokenStream` from the infallible builders and move the FIXME to the issue tracker.

---

## `derive-input/src/api/dsl/foreign_key.rs`

The parsed `#[foreign_key]` and the generator's copy of the on-delete strategies.

### `foreign_key.rs`: `pub enum OnDeleteStrategy`

**Violates:** DRY; Connascence

This is the generator's copy of the runtime enum; its comment says so, and it has drifted (see `src/delete.rs`). Its `ToTokens` implementation repeats one structurally identical arm per variant.

Recommendation: Resolve together with `src/delete.rs`. For `ToTokens`, the developers can keep the explicit match, which keeps exhaustiveness checking, or map through the variant name, which removes the repetition.

## `derive-input/src/api/dsl/table.rs`

The public description of the table-level DSL settings and of the cascade entry points.

### `table.rs`: `pub struct SpacetimeDSLTable`

**Violates:** Connascence — of Execution; Code For The Maintainer — Principle of Least Astonishment

`SpacetimeDSLTable::try_parse` builds the table with `create_dsl_method_arg: None`, empty `compile_error_checks` and `compile_error_check_imports`, and an empty `struct_doc_comment`, and method generation fills them in later through `TableContributions::apply_to`. In between, the value looks complete but is not; a generator reading those fields during generation would silently see the empty state.

Recommendation: Keep generation results out of the parsed table — for example move `create_dsl_method_arg`, `compile_error_checks`, `compile_error_check_imports` and `struct_doc_comment` into `SpacetimeDSLTableMethods` or into a separate result type — so that `SpacetimeDSLTable` does not change after parsing. This changes the public API, whose fields the plan's Task 13 documents and pins as a data-transfer contract.

---

## `derive-input/src/internal/column.rs`

Analyses the columns of a table and classifies their types.

### `column.rs`: `pub struct InternalColumn`

**Violates:** DRY; Connascence; Maximize Cohesion

`InternalColumn` copies fields of `RustField`, `SpacetimeDBColumn` and `SpacetimeDSLColumn`, and even the table's singular name, into one flat struct with prefixed field names. Every new column property has to be added to each representation. It exists because column methods are generated before the `Column` values are assembled.

Recommendation: Options for the developers: (a) assemble `Column` first and generate the methods from `&[Column]` into a separate collection; (b) keep a flat view but build it from references instead of copies; (c) keep it and document why the second representation is needed. Pure refactoring.

## `derive-input/src/internal/dsl/table.rs`

Parses the table-level DSL settings and validates the roles of columns.

### `table.rs`: `SpacetimeDSLTable::try_parse(dsl_data: DSLData, column_args: &ColumnArgs<'_>, spacetimedb_table: SpacetimeDBTable) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable)>`

**Violates:** Single Responsibility Principle (SRP); Separation of Concerns; KISS; YAGNI

One function marks DSL-unique indices, builds the hooks, validates the update method, parses the soft-delete marker and the referencing tables, checks visibilities and validates timestamp roles. The DSL's own unique multi-column index feature is stored as `Index::is_unique` in the SpacetimeDB model, so that flag no longer tells whether SpacetimeDB or the generated code enforces uniqueness. `has_update_method` is shadowed from `Option<bool>` to `bool`, and `referencing_tables` is filled by assigning only while it is still empty.

The final check that raises `update_method_disabled_with_set_on_update_column` can never be reached: inside the loop, a `set_on_update` column without the update method and a non-private column without the update method both return earlier, so `all_columns_are_private` only feeds that dead check.

Recommendation: Split into focused validators and keep DSL uniqueness out of the SpacetimeDB model (for example as a set of DSL-unique index names on the DSL side). The unreachable check is dead code: the developers must decide whether its rule is fully covered by `set_on_update_column_without_update_method` (then delete the check, the flag and the error function) or whether its unreachability hides a missing case (then first write the compile-test that should raise it).

### `table.rs`: the types in `DSLData::unique_indices`

**Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"

`#[dsl(unique_index(name = x))]` only has an effect when `x` names a BTree multi-column index. A single-column index or a hash multi-column index is accepted without effect and without a diagnostic.

Recommendation: When single-column and hash indices are meant to be supported, they should be added, with a snapshot and a runtime test. Otherwise reject them with a compile-test.

## `derive-input/src/internal/dsl/singleton.rs`

The generator's knowledge of the primary key injected into singleton tables.

### `singleton.rs`: `PRIMARY_KEY_NAME`, `PRIMARY_KEY_TYPE`, `PRIMARY_KEY_VALUE` and `pub fn is_primary_key_column(name: &Ident, type_name_or_path: &Path) -> bool`

**Violates:** DRY; Connascence — of Meaning, across crates

The module is the `derive-input` half of the knowledge that `derive/src/lib.rs::inject_singleton_primary_key` spells out again.

Recommendation: Resolve together with the `derive/src/lib.rs` entry (one home for the injected key).

## `derive-input/src/internal/dsl/one_or_multiple.rs`

The generator's copy of the runtime's `OneOrMultiple`.

### `one_or_multiple.rs`: `pub enum OneOrMultiple`

**Violates:** DRY; Connascence — of Name

A mirror of `src/error.rs::OneOrMultiple` whose variants reach generated code by name.

Recommendation: Resolve together with the `src/error.rs` entry.

## `derive-input/src/internal/dsl/wrapper.rs`

Parses `#[create_wrapper]` and `#[use_wrapper]` and generates wrapper structs.

### `wrapper.rs`: `fn get_wrapper_impl(struct_name, wrapper_struct_name, wrapped_type_name_or_path, field_name, wraps_uuid) -> TokenStream`

**Violates:** Code For The Maintainer — Principle of Least Astonishment

Every created wrapper displays as `Name { id: value }`, whatever column it wraps; `#[create_wrapper]` is also used on non-key columns such as `name3: String` in `examples/test/src/entity.rs`.

Recommendation: Render the wrapped column's name or a neutral form. The text appears in every `DeletionResult` row, so decide it together with the `src/delete.rs` format entry and pin it with a runtime assertion.

## `derive-input/src/internal/dsl/foreign_key.rs`

Parses and validates `#[foreign_key]`.

### `foreign_key.rs`: `OnDeleteStrategy::try_parse_for_on_soft_delete(meta: &ParseNestedMeta<'_>, tokens: &Meta) -> syn::Result<OnDeleteStrategy>`

**Violates:** Code For The Maintainer; DRY

The doc says the strategies `on_soft_delete` does not accept are rejected by name rather than by the catch-all, but `SetNone` is not named and falls into `unknown_on_soft_delete_strategy`. The variant spellings are repeated in the parsers for `on_delete` and `on_soft_delete`, in `ToTokens` and in the runtime `Display`.

Recommendation: Make doc and code agree — name `SetNone` explicitly (with a compile-test) or change the doc. For the shared spellings, the developers can derive parsing from the variant names while keeping each context's allow-list explicit.

## `derive-input/src/internal/dsl/reference.rs`

Parses `#[referenced_by]`.

### `reference.rs`: `ReferencingTable::try_parse(field: &SatsField<'_>) -> syn::Result<Vec<ReferencingTable>>`

**Violates:** DRY

Whether the field is the primary key is re-derived from its raw attributes, although `SpacetimeDBColumn::is_primary_key` already states it.

Recommendation: Take the primary-key fact from `SpacetimeDBColumn`.

## `derive-input/src/internal/dsl/method/get.rs`

Generates the lookup methods.

### `get.rs`: `pub fn for_get_one(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** DRY; Scope & Goal Discipline — "Future ideas captured outside codebase"

Both branches build the same `not_found_error`.

Recommendation: Build the error once before the branch.

## `derive-input/src/internal/dsl/method/reference_integrity.rs`

The referential-integrity and unique multi-column index checks generated methods run before they write.

### `reference_integrity.rs`: `pub enum Action`

**Violates:** DRY; Connascence — of Name

A mirror of `src/error.rs::Action` whose variants reach generated code through `strum::Display` and `format_ident!`.

Recommendation: Resolve together with the `src/error.rs` entry.

### `reference_integrity.rs`: Robustness in `pub fn multi_column_index_checks(...)` and `pub fn unique_multi_column_index_check(...)`

**Violates:** Robustness Principle

Only BTree multi-column indices are considered, which matches the silent acceptance of other `unique_index` targets (see `internal/dsl/table.rs`).

Recommendation: Resolve the supported index kinds together with the `internal/dsl/table.rs` entry.

## `derive-input/src/internal/dsl/method/on_delete_strategy.rs`

Generates what one on-delete strategy does to the rows that reference a deleted row.

### `on_delete_strategy.rs`: `pub fn on_delete_strategy_implementation(context, referencing_tables: ReferencingTables, on_delete_strategy: &OnDeleteStrategy, columns_by_on_delete_strategy: Vec<&Column>, one_or_multiple: &OneOrMultiple) -> TokenStream`

**Violates:** Single Responsibility Principle (SRP); Curly's Law; KISS; Optimize for Deletion; Testing & Verification — "Added tests justify each new branch"

One very long function covers every strategy, referenced and unreferenced tables, and one and many rows. It collects its output in mutable accumulators (`strategy_for_before_hook`, `strategy_for_after_hook`, `strategy_for_referenced_by`, `strategy_after_all`) that the per-column loop overwrites.

That appears to be a defect: when several foreign-key columns share a `Delete` or `SoftDelete` strategy toward one referenced table, and the table is itself referenced — `Potion` in `examples/test/src/component/hook_test.rs` is such a table — only the last column's `strategy_after_all` survives. It reports every cascaded row under the last column's name in `DeletionResultEntry::column_name`, including rows that were reached through an earlier column. No snapshot fixture covers this combination.

Recommendation: First add a snapshot fixture for that combination and a runtime assertion on `DeletionResultEntry::column_name` for a row reached only through the first column, read the output, and let the developers confirm whether it is a defect. Then give each strategy its own function that returns its fragments as a value instead of overwriting shared accumulators.

Developer's decision: The recommendation should be applied, but independently of any other fix decided by the developer.

### `on_delete_strategy.rs`: the `Delete` and the `SoftDelete` arm for `ReferencingTables::Present`

**Violates:** DRY

These arms build the same scaffolding — the entry maps, `create_entries_and_add_them_to_entries`, the failure handler, the per-row collection and the dispatcher calls — and differ in the write, the hooks, the dispatcher's kind of removal, the strategies called afterwards and the filter for already retired rows. `removal.rs` states such a sequence only once "because two copies of that order would drift", yet here the sequence exists as a copy.

Recommendation: Understand the differences (hook placement, the retired-row filter, the strategy lists) to make an informed decision which parts become one skeleton parameterised by `Removal`, as in `removal.rs`, and which stay separate. The fixtures `on_delete_delete`, `on_soft_delete_cascade`, `on_soft_delete_cascade_with_soft_delete_hooks` and `self_referencing_cascade` guard the change.

Developer's decision: The recommendation should be applied, but independently of any other fix decided by the developer.

### `on_delete_strategy.rs`: the commented-out `set_none_strategy`, `strategy_before_all` and the notes inside `quote!`

**Violates:** YAGNI; Refactoring & Change Containment — "Unused scaffolding removed immediately"; Scope & Goal Discipline — "Future ideas captured outside codebase"

A commented-out block calls `referenced_table_function_call_for_strategy_implementation` with an argument the function no longer takes, and a TODO about `SetNone` sits inside a `quote!`. `strategy_before_all` is always empty (a "deliberate empty slot") and is still emitted.

Recommendation: Delete the commented-out code; the issue tracker holds `SetNone`. `strategy_before_all` is dead code: the developers decide whether it is a planned extension point (then document what belongs there) or not (then delete it).

### `on_delete_strategy.rs`: the rows whose foreign key is `0` or `Uuid::NIL`

**Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"; Code For The Maintainer — Principle of Least Astonishment

A foreign key of `0` (unsigned integer columns) or, after the plan's Task 50, `Uuid::NIL` (`Uuid` columns) means "references no row": the reference-integrity checks of create, update and upsert skip it, and `on_delete = SetZero` writes it. The delete and soft-delete cascades know nothing of that meaning. They find the referencing rows by the deleted row's primary key value, so deleting a row whose primary key is `0` or `Uuid::NIL` — possible for a key without `#[auto_inc]` or `#[auto_gen]` — applies the strategy to every row that references no row at all: `Delete` removes them, `SoftDelete` retires them, `Error` refuses the deletion.

Recommendation: Let the cascades skip rows whose foreign key holds the "no reference" value, or reject a primary key of `0` / `Uuid::NIL` on create, and decide which. Pin the chosen behaviour with a snapshot fixture and a runtime test for both column kinds.

Developer's decision: This must be fixed, but independently of any other fix decided by the developer.

## `derive-input/src/internal/dsl/method/referenced_by.rs`

The referenced side of a foreign key: the cascade entry points and the calls into them.

### `referenced_by.rs`: `pub fn referenced_table_function_call_for_dsl_method(...)` and `on_delete_strategy.rs::referenced_table_function_call_for_strategy_implementation(...)`

**Violates:** DRY

Both emit "call the dispatcher, merge the child entries on success and on failure, run the error handler", but they differ in where the keys come from, which container collects the entries, and how a hook error is kept: the DSL-method variant shadows `error_from_hook` with the failure's value, while the strategy variant keeps the first error it saw.

Recommendation: Understand the differences to make an informed decision whether they are intended — a DSL method starts without a hook error, while a cascade has to keep the first one — before extracting a shared helper, or keep them separate and document why.

Developer's decision: The recommendation should be applied, but independently of any other fix decided by the developer.

## `derive-input/src/internal/dsl/method/update.rs`

Generates `update_<table>_by_<key>`.

### `update.rs`: `pub fn for_update(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** Robustness Principle; Code For The Maintainer — Principle of Least Astonishment; Self-Documenting Code — no abbreviations

The generated method calls SpacetimeDB's `update`, which panics on a constraint violation, instead of `try_update`; the FIXME about `try_update` appears here, in `upsert.rs` and in `on_delete_strategy.rs`. When a before-update hook exists, the generated code fetches the stored row with an `expect`, while the reference-integrity check of the same method returns a `NotFoundError` for the same situation — whether a missing row panics or returns an error depends on whether a hook is declared. `is_singleton_pk` is abbreviated. The same panic reaches the cascades: `on_delete_strategy.rs::store_the_row` emits SpacetimeDB's `update` for every row that `SoftDelete` or `SetZero` writes.

Recommendation: Use `try_update`, map its errors to `SpacetimeDSLError`, and return a `NotFoundError` whenever the row is missing. Pin both with runtime tests (a missing row with and without a hook; a constraint violation on update). Turning panics into errors changes observable behaviour, so the developers decide how to announce it.

Developer's decision: The recommendation should be applied, including turnings panics into errors, but independently of any other fix decided by the developer. The current problem is that no `try_update` exists in SpacetimeDB, so this isn't possible to fix currently!

## `derive-input/src/internal/dsl/method/upsert.rs`

Generates `upsert_<table>` and hosts write-path helpers used by other generators.

### `upsert.rs`: the region "Pieces shared with `update.rs`"

**Violates:** Maximize Cohesion; Single Responsibility Principle (SRP); Self-Documenting Code

The module holds the upsert generator and the helpers that `update.rs`, `removal.rs` and `on_delete_strategy.rs` import, so unrelated generators depend on the upsert module. Its private `row_value_getter` shares its name with a different private function in `reference_integrity.rs`. `updated_at_column` and `set_created_at_on_insert` repeat the "find the column by name or panic" lookup that `index.rs`, `removal.rs` and `reference_integrity.rs` also contain. The doc of `rebind_row_as_mutable_after_hook` refers to `keep_created_at`, whose name is `keep_created_at_on_update`.

Recommendation: Move the shared write-path helpers into their own module, give the `row_value_getter` functions of `upsert.rs` and `reference_integrity.rs` names that say how they differ, and add one lookup helper for columns — the doc of `MethodGenerationContext` forbids generation methods, not lookups, but where it lives is the developers' decision. Correct the doc reference.

Developer's decision: The recommendation should be applied, but independently of any other fix decided by the developer.

## `derive-input/src/internal/error.rs`

Every diagnostic the parser raises for a rejected table.

### `error.rs`: `pub fn update_method_disabled_with_set_on_update_column(struct_name: &Ident) -> Error`

**Violates:** YAGNI

It is only raised from a branch that cannot be reached (see `internal/dsl/table.rs`), so no compile-test pins it.

Recommendation: Dead code: the developers must decide whether its rule is covered by another diagnostic (then delete the function and its branch) or whether the unreachable branch is a bug (then first write the compile-test that should produce it).

---

## `examples/test/src/component/test.rs`

A table that exercises many column, accessor and index shapes, plus related tables.

### `test.rs`: `pub struct Test`

**Violates:** Code For The Maintainer

The doc comments were copied from another table: `Test` is documented as "A Position in the World", its `id` as "The unique ID of the World".

Recommendation: Delete the comments.

## `examples/test/src/component/position.rs`

Runtime tests for `Position` and `UniquePosition`.

### `position.rs`: `pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String>`

**Violates:** YAGNI; F.I.R.S.T Principles of Testing

`position_ids`, `positions`, `unique_position_ids` and `unique_positions` are filled — including a synthetic next id and a trailing `None` — and never read; they look like the remains of an assertion that no longer exists. Several failure messages drop the error they received (`Err(_) =>`).

Recommendation: Dead code: the developers must decide whether the assertion these collections fed is no longer needed (then delete them) or whether its loss is a bug (then restore it). Include the received error in every failure message.

## `examples/test/src/component/hook_test.rs`

Runtime tests for the insert, update and delete hooks.

### `hook_test.rs`: `pub fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String>` and `pub fn my_procedure(ctx: &mut ProcedureContext)`

**Violates:** Code For The Maintainer; F.I.R.S.T Principles of Testing — repeatable; Self-Documenting Code — no abbreviations

The failure message claims the potion is deleted in the after-delete hook of the attribute table, but `after_attribute_delete` only logs, and the potion is removed by `on_delete = Delete`. The expected hook sequence is compared with `get_all_hook_calls()` in iteration order, which assumes that iteration order equals insertion order. `my_procedure` would run `run_tests` a second time against the rows of the first run if it were ever invoked; its FIXME says it cannot be invoked yet. `hc` and `msg` are abbreviations.

Recommendation: Correct the message, order the hook calls by their auto-increment id before comparing, and decide whether `my_procedure` should run the tests (then give it rows of its own) or only prove that procedures compile (then give it a body without side effects).
