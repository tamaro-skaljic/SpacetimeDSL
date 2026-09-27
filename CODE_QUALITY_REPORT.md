# Code Quality Report

This report checks the code of this repository against the programming principles and methodologies in [`AGENTS.md`](AGENTS.md). It covers the runtime crate (`src/`), the procedural macro crate (`derive/`), the generator crate (`derive-input/`), the test harnesses (snapshots in `derive/`, diagnostics in `compile-tests/`, runtime tests in `examples/test/`), the other examples, the developer scripts and the CI workflows. Findings are located by file and item name only, so they stay valid while code moves.

**Reading an entry.** The names after **Violates:** are the headings of AGENTS.md's *Programming Principles*, sometimes followed by the checklist item that is violated; *Self-Documenting Code* and *Test Driven Development* refer to its *Methodologies*. Where an entry describes code that nothing uses, or copies of one piece of logic that have drifted apart, the recommendation leaves the decision to the developers on purpose. Where several fixes are possible, the options are listed.

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

### `lib.rs`: `pub trait Wrapper<WrappedType: Clone, WrapperType>`

**Violates:** KISS; YAGNI; Code For The Maintainer — Principle of Least Astonishment

No method uses the second type parameter, and every generated implementation passes the implementing type itself (`impl crate::spacetimedsl::Wrapper<u64, ThingId> for ThingId`). A reader has to work out that the parameter carries no information.

Recommendation: Reduce the trait to `Wrapper<WrappedType>` and adapt `derive-input/src/api/runtime.rs::wrapper_trait`. This breaks code that spells out the second parameter, so it belongs to the next breaking release; the snapshots change accordingly.

Developer's decision: The recommendation should be applied.

### `lib.rs`: `macro_rules! spacetimedsl` — re-export lists, `DSL` and `ReadOnlyDSL`

**Violates:** DRY; YAGNI; KISS

The public surface of the runtime is listed repeatedly — in the `pub use` at the crate root, in the module-level re-exports of the generated `crate::spacetimedsl`, in its "flat re-exports" and in its `prelude` — and the lists have already drifted: `OnDeleteStrategyFailure` is re-exported by the macro but not by the crate root.

Recommendation: Define the prelude once in the runtime crate and let the macro re-export it, so a new runtime item is one edit.

Developer's decision: The recommendation should be applied.

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

### `error.rs`: `impl Display for SpacetimeDSLError`

**Violates:** Robustness Principle; Code For The Maintainer — Principle of Least Astonishment; KISS; Self-Documenting Code — descriptive identifiers

Formatting an error can panic: `ReferenceIntegrityViolationError::OnCreateOrUpdate` stores a general `Action`, and `Display` panics for `Get`, `Delete` and `SoftDelete`. The panic exists only because the field can hold states the variant forbids. The message is built into an intermediate `String` before it is written, and the local `dig_spacetimedb` does not say what it holds.

Recommendation: Make the illegal states unrepresentable with a dedicated type for `create_or_update` that has only the create and update cases; that removes the panic. Write to the formatter directly and name the local after its content. Changing the field type breaks the public API, so it belongs to the next breaking release.

Developer's decision: The recommendation should be applied.

### `error.rs`: `pub enum Action`, `pub enum OneOrMultiple` and `impl Display for OnDeleteStrategy`

**Violates:** DRY; Connascence — of Name; Maximize Cohesion

The generator keeps mirrors of these enums (`Action` in `derive-input/src/internal/dsl/method/reference_integrity.rs`, `OneOrMultiple` in `derive-input/src/internal/dsl/one_or_multiple.rs`) and turns their variants into runtime paths by name, so renaming either side breaks only the user's build. `Display for OnDeleteStrategy` lives here, away from its enum in `delete.rs`.

Recommendation: Move `Display for OnDeleteStrategy` next to its enum. Resolve the mirrors the same way as `OnDeleteStrategy` (see `delete.rs`); at the very least make sure every emitted variant appears in a snapshot fixture, so a rename shows up as a snapshot diff.

---

## `derive/src/lib.rs`

The procedural macro entry points `#[dsl]` and `#[hook]`, the helper derive `SpacetimeDSL`, and the injection of a singleton's primary key.

### `lib.rs`: `fn is_last_dsl_attribute(derive_input: &syn::DeriveInput) -> bool` and the commented-out `make_struct_fields_private`

**Violates:** YAGNI; Refactoring & Change Containment — "Unused scaffolding removed immediately"; Self-Documenting Code — don't document the past

`expand_dsl_attribute_parts` computes `_is_last_dsl_attribute` and discards it, and the function exists only for that call. `make_struct_fields_private` and its call are commented out under a TODO saying they are temporarily disabled to allow public primary key columns. The dead function also recognises the attribute by comparing its stringified path with `"dsl"` and `"spacetimedsl :: dsl"`, which misses spellings such as `::spacetimedsl::dsl`.

Recommendation: Dead code: the developers must decide whether making fields private is no longer needed (then delete the function, the commented-out block and the TODO, and keep the idea in the issue tracker if it is still wanted) or whether its being disabled is a bug (then re-enable it, with a snapshot fixture for a table with a public primary key and a runtime test that uses one).

### `lib.rs`: `fn expand_dsl_attribute_parts(args, item) -> syn::Result<ExpandedDSLAttribute>`

**Violates:** DRY — "One authoritative source for each business rule"; Robustness Principle; Self-Documenting Code — no redundant comments

Whether a table is a singleton is decided in more than one place: here, by looking for an identifier `singleton` among the top-level argument tokens, and in `derive-input` by the real argument parser. They can disagree — the scan also accepts `singleton` in a value position such as `plural_name = singleton` — and a disagreement injects `id: u8` into an ordinary table, followed by errors that have nothing to do with the user's input. Comments such as "Parse the input tokens into a syntax tree" and "Build the output, possibly using quasi-quotation" repeat the code.

Recommendation: Parse the `#[dsl]` arguments once in `derive-input` and hand the result to the injection, or move the injection into `derive-input` (next entry). Delete the comments that repeat the code.

### `lib.rs`: `fn inject_singleton_primary_key(derive_input: &mut syn::DeriveInput) -> syn::Result<()>`

**Violates:** DRY; Connascence — of Meaning, across crates; Separation of Concerns; Hide Implementation Details

The function's own doc comment says that the name, the type and the value of the injected key are spelled out again in `derive-input/src/internal/dsl/singleton.rs`, and that a change here needs the same change there. Its diagnostics are the only user-facing rejections outside `derive-input/src/internal/error.rs`, contradicting that module's statement that it holds every diagnostic. The injected field also leaks into user code: every `DefaultSingleton::get_default` has to write `id: 0` for a field the user never declared (`examples/test/src/singleton_with_default_test.rs`, `examples/test/src/singleton_with_foreign_key_test.rs`).

Recommendation: Move the injection, its constants and its diagnostics into `derive-input` next to `internal::dsl::singleton`, and let the `derive` crate call one API function. The key then has one home and the diagnostics join `internal/error.rs`. Whether users should keep writing the injected field is an API decision for the developers; options include a generated constructor that fills it in, or keeping it and documenting it where `DefaultSingleton` is documented.

### `lib.rs`: `pub fn hook(_args: TokenStream, item: TokenStream) -> TokenStream`

**Violates:** Connascence — of Algorithm, across crates; DRY

`#[hook]` derives the trait to implement as the PascalCase form of the function name followed by `Hook`, while `derive-input/src/internal/dsl/hook.rs` builds the same trait name from timing, table and operation. The algorithms agree only implicitly, and a change to either one breaks every hook in user code.

Recommendation: Expose one function from `derive-input` that maps a hook function name to its trait name, use it on both sides, and cover a table name with digits and underscores in a runtime hook.

Developer's decision: The recommendation should be applied.

### `lib.rs`: `pub fn table_helper(_input) -> TokenStream` and `fn derive_table_helper_attr() -> syn::Attribute`

**Violates:** DRY; Scope & Goal Discipline — "Future ideas captured outside codebase"; KISS

The helper-attribute list of the `SpacetimeDSL` derive is one more spelling of the field-attribute vocabulary, which `derive-input` recognises through `symbol!` declarations in `internal/dsl.rs` for some attributes and through string literals (`is_ident("set_on_create")`, `"set_on_update"`, `"set_on_soft_delete"`) for others. A new attribute has to be added in several places, and a forgotten entry surfaces as an unknown-attribute error in user code. `derive_table_helper_attr` parses a constant with `unwrap()` calls where `syn::parse_quote!` says the same.

Recommendation: Give the attribute names one list in `derive-input` that all of its parsers use. `proc_macro_derive(attributes(...))` needs literal identifiers, so the helper list cannot be generated from that list; pin their agreement instead, for example with a snapshot fixture that uses every field attribute on an accepted table. Use `syn::parse_quote!`. Keep the reason for the helper derive as a comment.

Developer's decision: The recommendation should be applied.

## `derive/src/output.rs`

Assembles everything the macro emits from the parsed `Table`.

### `output.rs`: `pub fn build(input: &Table, first_dsl_attribute: bool) -> syn::Result<GeneratedOutput>`

**Violates:** Code For The Maintainer; DRY; Law of Demeter — "Avoid chaining through returned collaborators"; Open/Closed

The comment saying that wrapper types are only generated for the last DSL attribute contradicts the condition `if first_dsl_attribute`. The iteration over `[&strategies.on_deletion, &strategies.on_soft_deletion]` is written out for the referencing and for the referenced side. Every hook is fetched by field name (`hooks.before_insert` … `hooks.after_soft_delete`), so a new hook kind means editing this function (see the hook matrix in `derive-input/src/internal/dsl/hook.rs`). The function navigates deep into `derive-input`'s model (`input.spacetimedsl_table.hooks.before_insert`, `column.spacetimedsl_column.getter`), coupling this crate to that model's layout. `format_ident!("{}", &input.rust_struct.name.to_string())` rebuilds an identifier it already has.

Recommendation: Correct the comment. Give the cascade entry-point types a method that yields the entry points they hold, and the hooks an iterator, so `build` does not need to know their fields. Then decide whether `Table` should offer intention-revealing accessors or be documented as a deliberate data-transfer structure ("Document justified exceptions when structure must stay public"); see `derive-input/src/lib.rs`. This is a pure refactoring.

### `output.rs`: `fn map_args(args: &Vec<SpacetimeDSLArg>) -> Vec<TokenStream>` and `pub fn malformed_code_generation_result(result: String) -> String`

**Violates:** Law of Demeter — "Push delegation into owning object interfaces"; KISS; Scope & Goal Discipline — "Future ideas captured outside codebase"

`map_args` destructures `SpacetimeDSLArgType` through fully qualified paths only to take `actual_type` from either variant; `derive-input/src/internal/dsl/method/create.rs` does the same. A FIXME above `malformed_code_generation_result` records an idea.

Recommendation: Add an `actual_type()` method to `SpacetimeDSLArgType` in `derive-input` and use it in both places, and move the FIXME to the issue tracker.

Developer's decision: The recommendation should be applied.

## `derive/src/output/function.rs`

Renders a `SpacetimeDSLMethod` as inherent methods on `DSL` and `ReadOnlyDSL`, or on `DSLInternals`.

### `function.rs`: `pub fn build_public`, `pub fn build_internal`, `struct MethodGenerationConfig` and `enum MethodImplTarget`

**Violates:** KISS; YAGNI; Code For The Maintainer — Principle of Least Astonishment; Scope & Goal Discipline — "Future ideas captured outside codebase"

`MethodGenerationConfig` separates a "doc variant" from the "output variants", but `build_public` passes its first output variant as the doc variant as well, and `build_internal` passes its only variant as both. The `InternalDslInternals` target ignores the context bound it is constructed with (`let _context_bound = …`) and reads `runtime::write_context()` again, so the argument of `MethodImplVariant::internal` has no effect. These builders — like `create_method_arg::build` and `hook::build` — return `syn::Result` without any failure path. `InternalDslInternals` repeats "internal".

Recommendation: Render the doc comment from the first output variant and drop `MethodGenerationConfig`. The ignored bound is dead data: the developers must decide whether internal methods should honour it (then use it; the snapshots show whether anything changes) or not (then remove the parameter). Return `TokenStream` from the infallible builders and move the FIXME to the issue tracker.

## `derive/src/output/accessor.rs`

Emits the getters, mutable getters and setters on the table struct.

### `accessor.rs`: `fn definition(&self) -> syn::Result<AccessorDefinition<'a>>` and `fn method_tokens(&self) -> TokenStream`

**Violates:** Connascence — of Meaning; DRY; KISS

The visibility of mutable getters and setters travels between the crates as text: `derive-input` renders `RustVisibility` with `Display` (`pub (crate)`), and this function parses it back with `parse_str`. Every accessor body imports `spacetimedsl::Wrapper` through a path written here by hand, although `derive-input/src/api/runtime.rs` declares itself the single place for runtime paths.

Recommendation: Let `RustVisibility` implement `ToTokens` and drop the parse. Emit the import through `runtime`, after the spelling decision described in `derive-input/src/api/runtime.rs`.

Developer's decision: The recommendation should be applied.

## `derive/src/output/hook.rs`

Emits the hook traits a table declares.

### `hook.rs`: `pub fn build(hook: &Option<SpacetimeDSLMethodHook>) -> syn::Result<TokenStream>`

**Violates:** KISS

An `is_none()` check followed by `as_ref().unwrap()` spells out what `let Some(hook) = hook else` expresses, the parameter is `&Option<T>` instead of `Option<&T>`, and the function cannot fail although it returns `syn::Result`.

Recommendation: `fn build(hook: Option<&SpacetimeDSLMethodHook>) -> TokenStream` with a `let … else`.

Developer's decision: The recommendation should be applied.

## `derive/src/characterization_tests.rs`

The snapshot harness for input the macro accepts.

### `characterization_tests.rs`: the hand-registered `#[test]` functions and the snapshot directories

**Violates:** Connascence — of Name; Testing & Verification — *A test that cannot run is not a test*; Lifecycle & Deletion Strategy — "Delete code, tests, and configuration in the same pass"

Each fixture needs a hand-written test whose name repeats the fixture's file name as a string; a fixture without such a test never runs, and nothing reports it. Snapshots whose fixture is gone stay behind unnoticed: `derive/tests/snapshots/update_hook_with_updated_at/` has neither a fixture nor a test (it predates the rename of `updated_at` to `set_on_update`). The fixture `hooks_all_six` claims in its name and doc to cover all hooks, while the soft-delete hooks added since are pinned by `soft_delete_hooks`. `is_dsl_attribute` repeats the string comparison of attribute paths from `derive/src/lib.rs`.

Recommendation: Delete the orphaned snapshot directory after confirming that `update_hook_with_set_on_update` covers its content. To keep fixtures and registrations in step, the options are: (a) enumerate `tests/fixtures` in one test, the way `trybuild` globs its directory; (b) keep the explicit registration AGENTS.md prescribes and add a test that fails for an unregistered fixture or for a snapshot directory without a fixture; (c) make `x.ps1 unit-test` reject unreferenced snapshots. Options (a) and (b) change the rule written in AGENTS.md, which then has to change with them. Rename `hooks_all_six` after the shape it pins.

Developer's decision: The recommendation (b) should be applied.

---

## `derive-input/src/lib.rs`

The public API of the generator crate: `Table`, `Column` and the modules below `api`.

### `lib.rs`: `pub struct Table` and `pub struct Column`

**Violates:** Hide Implementation Details — "Keep member data private and encapsulated"; Minimize Coupling; YAGNI; DRY; Documentation & Communication Clarity — "Document concern boundaries and integration contracts"

The crate advertises itself as a base for other procedural macro crates and exposes its whole analysis through public fields, including generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`) and bookkeeping (`compile_error_checks`). Every internal restructuring is therefore a breaking change for such crates, and the `derive` crate reads deep into the graph. The doc of `Table::try_parse` uses `/** */` and speaks of a derive macro, although it is called from an attribute macro.

Recommendation: The developers decide what external crates may rely on. Options: (a) keep public fields, document every one of them as a data-transfer contract and pin that contract with a consumer-side test; (b) make the fields private and expose accessors for the parts meant to be stable; (c) withdraw the promise to external crates and reduce the model to crate visibility.

Developer's decision: The recommendation (a) should be applied.

### `lib.rs`: `mod internal` and the `pub` inherent functions its modules add to `api` types

**Violates:** Hide Implementation Details — "Minimize class and member accessibility", "Exclude private implementation details from public interfaces"

The modules below the private `internal` module add `pub` inherent functions to public `api` types — for example `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `RustField::map`, `RustVisibility::map`, `SpacetimeDSLColumn::try_parse`, `WrapperType::try_parse`, `Getter::map`, `Setter::map` — and `impl Display for RustVisibility`. An inherent method's visibility follows its own `pub`, not the module of its `impl` block, so these internal constructors are callable from every dependent crate. `#[doc(hidden)]` on a private module has no effect, which suggests the intent was to hide them.

Recommendation: Reduce these functions to `pub(crate)`, keeping `Table::try_parse` as the only entry point; replace the `Display` implementation as described in `internal/rust.rs`; remove the ineffective `#[doc(hidden)]`.

Developer's decision: The recommendation should be applied.

## `derive-input/src/api/runtime.rs`

The token constructors for every path into the `spacetimedsl` runtime that generated code uses.

### `runtime.rs`: the module's promise that every runtime path is written here exactly once

**Violates:** DRY; Encapsulate What Changes; Code For The Maintainer

The module states that every `crate::spacetimedsl::…` path any generator emits is written here exactly once. The `Wrapper` trait nevertheless appears as `crate::spacetimedsl::Wrapper` (`wrapper_trait`), as `::spacetimedsl::Wrapper` (`wrapper_trait_path`) and as `spacetimedsl::Wrapper` (typed by hand in `derive/src/output/accessor.rs` and `internal/dsl/wrapper.rs`); `itertools_import` and `derive/src/lib.rs` use `::spacetimedsl::…`. Paths into `spacetimedb` (`spacetimedb::TryInsertError`, `spacetimedb::SpacetimeType`, `spacetimedb::{CtxDbRead, CtxDbWrite, Table}`) have no such home at all, so a SpacetimeDB rename is exactly the text hunt through `quote!` bodies the module was written to prevent. `deletion_result_entry` takes its last field with the trailing comma included, to accommodate one call site's shorthand, which bends the API to a stylistic difference.

Recommendation: Route every emitted path through this module and add a counterpart for `spacetimedb` paths. The `Wrapper` spellings resolve differently in a user's crate (crate-local re-export, extern crate, relative path): understand why each one was chosen before unifying them, and pin the result with snapshots. Let `deletion_result_entry` take the child-entries expression and write the field itself.

Developer's decision: The recommendation should be applied.

## `derive-input/src/api/dsl/foreign_key.rs`

The parsed `#[foreign_key]` and the generator's copy of the on-delete strategies.

### `foreign_key.rs`: `pub enum OnDeleteStrategy`

**Violates:** DRY; Connascence

This is the generator's copy of the runtime enum; its comment says so, and it has drifted (see `src/delete.rs`). Its `ToTokens` implementation repeats one structurally identical arm per variant.

Recommendation: Resolve together with `src/delete.rs`. For `ToTokens`, the developers can keep the explicit match, which keeps exhaustiveness checking, or map through the variant name, which removes the repetition.

## `derive-input/src/api/dsl/method.rs`

The public description of one generated DSL method and its arguments.

### `method.rs`: `pub struct SpacetimeDSLArg` and `pub enum SpacetimeDSLArgType`

**Violates:** YAGNI; Law of Demeter — "Push delegation into owning object interfaces"

`is_option` and `Wrapped::wrapped_type` only consumers (`derive/src/output.rs::map_args`, `internal/dsl/method/create.rs`) match on the enum just to take `actual_type` from either variant.

Recommendation: Add `SpacetimeDSLArgType::actual_type()`.

Developer's decision: The recommendation should be applied.

## `derive-input/src/api/dsl/column.rs`

The public description of a column's DSL features and of the methods an index earns.

### `column.rs`: `pub struct SpacetimeDSLColumn` and `pub struct SpacetimeDSLColumnMethodsForUniqueIndex` / `ForIndex`

**Violates:** Documentation & Communication Clarity — "Document concern boundaries and integration contracts"; Self-Documenting Code

The conditions under which the public fields are `Some` are written as `//` comments, so the rustdoc of this published crate shows none of them, and "Only `Some(T)` if mutable" does not say what makes a column mutable (a non-private field). The types in `method.rs`, `hook.rs`, `getter.rs` and `setter.rs` document no fields at all, and `table.rs` documents only some; `SpacetimeDSLMethod::read_context_compatible`, for instance, does not say why `get_all_<tables>` is unavailable in a read-only context while `count_of_all_<tables>` is available.

Recommendation: Turn the comments into `///` documentation stated in DSL terms, and document the SpacetimeDB rule behind each `read_context_compatible` value, which the developers need to confirm.

Developer's decision: The recommendation should be applied.

## `derive-input/src/api/dsl/table.rs`

The public description of the table-level DSL settings and of the cascade entry points.

### `table.rs`: `pub struct SpacetimeDSLTable`

**Violates:** Connascence — of Execution; Code For The Maintainer — Principle of Least Astonishment

`SpacetimeDSLTable::try_parse` builds the table with `create_dsl_method_arg: None` and empty `compile_error_checks`, and method generation fills both in later through `TableContributions::apply_to`. In between, the value looks complete but is not; a generator reading those fields during generation would silently see the empty state.

Recommendation: Keep generation results out of the parsed table — for example move `create_dsl_method_arg` and `compile_error_checks` into `SpacetimeDSLTableMethods` or into a separate result type — so that `SpacetimeDSLTable` does not change after parsing. This changes the public API (see `derive-input/src/lib.rs`).

### `table.rs`: `pub struct OnDeleteStrategiesOfReferencingTables` and `pub struct OnDeleteStrategiesOfTheReferencedTable`

**Violates:** DRY

These types have the same shape — an optional pair of entry points per kind of removal — and consumers iterate their fields the same way in several places.

Recommendation: Options: (a) one shared type with an iterator over the entry points it holds; (b) keep both names for their different roles and give each such an iterator. Either one removes the repeated iteration in `derive/src/output.rs`.

Developer's decision: The recommendation (b) should be applied.

---

## `derive-input/src/internal.rs`

Entry point of the parser: reads the `#[dsl(...)]` arguments and hands them to the table, column and method analysis.

### `internal.rs`: `fn try_parse_dsl(args: &proc_macro2::TokenStream) -> syn::Result<DSLData>`

**Violates:** Single Responsibility Principle (SRP); Curly's Law; DRY; Self-Documenting Code — no redundant or misleading comments

One function parses every `#[dsl(...)]` argument and validates how they combine. The `before(...)` and `after(...)` branches are copies that differ only in the variables they write, and each hook travels as its own `Option<Span>` local and then as its own `bool` in `DSLData` (see the hook matrix in `internal/dsl/hook.rs`). The comment above the function says it parses `plural_name`, which is a small part of what it does, and the comment in `try_parse` about passing the parsed `plural_name` refers to an argument that is not passed.

Recommendation: Separate parsing from validation, and parse hooks with one helper that serves `before` and `after` and fills a structure keyed by timing and operation. Correct the comments. The diagnostics are pinned by compile-tests, so the `.stderr` files must stay unchanged.

Developer's decision: The recommendation should be applied.

### `internal.rs`: the placeholder plural name `__singleton_placeholder`

**Violates:** Connascence — of Execution; Coupling Awareness — "Anticipate maintenance; avoid hidden coupling or magic"

For a singleton, `try_parse_dsl` returns a fabricated plural name, and `try_parse` overwrites it with the table accessor afterwards. In between, `DSLData::plural_name` holds a magic value, and that value is handed to the table selection in `integration.rs`.

Recommendation: Model the difference in the type — an enum that distinguishes a singleton from a table with a plural name — so no code can read a placeholder.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/integration.rs`

Selects which `#[table]` attribute a `#[dsl]` attribute describes.

### `integration.rs`: `fn select_table_with_heuristics(input: &DeriveInput, plural_name: &syn::Ident) -> syn::Result<(TableArgs, ColumnArgs)>`

**Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"; Code For The Maintainer — Principle of Least Astonishment; KISS; Coupling Awareness — "avoid hidden coupling or magic"

When a struct carries several `#[table]` attributes, `plural_name` doubles as a table selector: exact match first, then "intelligent" matching, then a fallback that sums the characters of the plural name modulo the number of tables — so an arbitrary table is used instead of an error. The substring rule accepts the first table whose name the plural contains, so `offline_players` selects a table named `player` if that `#[table]` comes first. `plural_to_singular` strips `es` from every word that ends in it (`tables` becomes `tabl`); a special case for exactly `tables` patches that, although the substring rule would accept it anyway. `docs/DOCUMENTATION.md` describes none of this. The comments call the fallback "index-based" and a "hash of plural name"; it is neither. The `all_tables.is_empty()` branch can never be taken (see `internal/error.rs`), and the entry function's name, `spacetime_bindings_macro_input`, is a noun for an operation that selects a table.

Recommendation: Replace guessing with a documented rule. This changes which inputs are accepted, so the developers decide. Options: (a) the `#[table]` directly below a `#[dsl]` belongs to it; (b) an explicit selector such as `#[dsl(table = accessor)]` whenever a struct has several `#[table]` attributes; (c) keep the plural-name matching but reject an ambiguous or unmatched name with a diagnostic instead of the fallback. Pin the ambiguous case with a compile-test and the chosen resolution with a snapshot, and document the rule in `docs/DOCUMENTATION.md`. Give the entry function a name that says it selects the table.

Developer's decision: The recommendation (b) should be applied.

### `integration.rs`: `fn get_all_table_attributes(input: &DeriveInput) -> syn::Result<Vec<(TableArgs, ColumnArgs)>>`

**Violates:** Robustness Principle; DRY; Self-Documenting Code — no redundant comments

Attributes are recognised by comparing their stringified path with `"table"` and `"spacetimedb :: table"`, which misses `::spacetimedb::table`; `derive/src/lib.rs` and `derive/src/characterization_tests.rs` recognise `#[dsl]` the same way. Comments such as "Find all table attributes" repeat the code.

Recommendation: One helper that recognises an attribute by the last segment of its path (with an optional crate prefix), shared by all these places; delete the comments that repeat the code.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/table.rs`

Orchestrates the analysis of one table.

### `table.rs`: `pub fn try_parse(input, dsl_data, table_args, column_args) -> syn::Result<Table>`

**Violates:** Command Query Separation (CQS); Connascence — of Execution; KISS

`SpacetimeDBTable` is moved into `SpacetimeDSLTable::try_parse` and into `column::try_parse`, and each returns it changed: the first sets `is_unique` on indices, the second removes the single-column indices. Each of these internal functions is a command and a query at once, and the order of the calls carries meaning nothing states.

Recommendation: Compute the index assignment — which index belongs to which column, which multi-column indices the DSL treats as unique — with queries that return new values, then assemble `SpacetimeDBTable` once. Pure refactoring.

Developer's decision: The recommendation should be applied.

### `table.rs`: `pub fn rm_rsharp(ident: syn::Ident) -> syn::Ident`

**Violates:** Self-Documenting Code — no abbreviations; KISS; Maximize Cohesion

The name abbreviates "remove the `r#` prefix", the function re-implements `syn::ext::IdentExt::unraw`, and it lives in the table module although `internal.rs`, `rust/table.rs` and `db/table.rs` use it for arbitrary identifiers.

Recommendation: Use `IdentExt::unraw()`.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/column.rs`

Analyses the columns of a table and classifies their types.

### `column.rs`: `pub fn try_parse(column_args, rust_struct, spacetimedb_table, spacetimedsl_table) -> syn::Result<(SpacetimeDBTable, Vec<Column>, Column, Vec<InternalColumn>, InternalColumn)>`

**Violates:** Connascence — of Position; KISS; Self-Documenting Code — no abbreviations

The result is a positional tuple kept behind `#[allow(clippy::type_complexity)]`. The primary key column is searched for in the column list and again in the list of internal columns, by comparing the `to_string()` of identifiers, which can be compared directly; `res.0`, `res.1` and the message "PK column should be present" use abbreviations.

Recommendation: A named result struct; compare identifiers directly; find the primary key once.

Developer's decision: The recommendation should be applied.

### `column.rs`: `pub struct InternalColumn`

**Violates:** DRY; Connascence; Maximize Cohesion

`InternalColumn` copies fields of `RustField`, `SpacetimeDBColumn` and `SpacetimeDSLColumn`, and even the table's singular name, into one flat struct with prefixed field names. Every new column property has to be added to each representation. It exists because column methods are generated before the `Column` values are assembled.

Recommendation: Options for the developers: (a) assemble `Column` first and generate the methods from `&[Column]` into a separate collection; (b) keep a flat view but build it from references instead of copies; (c) keep it and document why the second representation is needed. Pure refactoring.

### `column.rs`: `ColumnTypeKind::of(type_name_or_path: &Path) -> ColumnTypeKind` and the string-based type checks elsewhere

**Violates:** DRY — "One authoritative source for each business rule"; Robustness Principle

`ColumnTypeKind::of` classifies a column's type by its path and accepts qualified spellings. Timestamps and flags, however, are checked by comparing stringified tokens (such as `"Option < spacetimedb :: Timestamp >"`) in `internal/dsl/table.rs` and `internal/dsl/soft_delete.rs`, the singleton key type in `internal/dsl/singleton.rs`, and foreign-key type equality in `internal/dsl/method/foreign_key.rs`. The accepted spellings therefore differ from check to check: `::spacetimedb::Timestamp` and `std::option::Option<Timestamp>` fail the string checks, although `ColumnTypeKind` treats a qualified `Option` as optional.

Recommendation: Extend the classification (for example with timestamp, optional timestamp and flag kinds) and use it for every type check. Extend the `qualified_type_spellings` fixture and add compile-tests for qualified spellings before switching.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/rust.rs`

Maps Rust visibilities into the API model.

### `rust.rs`: `impl fmt::Display for RustVisibility`

**Violates:** Connascence — of Meaning; KISS; Hide Implementation Details

A visibility is rendered to text so that the `derive` crate can parse it back (`derive/src/output/accessor.rs`), and it is compared as text in `internal/dsl/method/reference_integrity.rs` and `internal/dsl/method/upsert.rs` (`rust_field_visibility.to_string()` against `RustVisibility::Private.to_string()`) where `matches!` would do. As an implementation on a public type, the rendering is public API.

Recommendation: Implement `ToTokens`, compare with `matches!`, and remove `Display` once nothing needs it.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/rust/column.rs`

Maps a struct field into the API model.

### `column.rs`: `RustField::map(field: &SatsField<'_>) -> RustField`

**Violates:** Robustness Principle

The field type is re-parsed as a `Path` with an `expect`, so a field whose type is not a path (an array, a tuple or a reference) makes the macro panic. `#[dsl]` expands before `#[table]`, so the user sees a panic from SpacetimeDSL instead of a spanned diagnostic from either crate. `internal/dsl/wrapper.rs` re-parses types the same way.

Recommendation: The developers decide which column types SpacetimeDSL supports. For unsupported ones, return a spanned error built in `internal/error.rs` (compile-test first: red is the panic, green the diagnostic); for supported ones, keep a `syn::Type` instead of a `Path`.

Developer's decision: Arrays, tuples and references are unsupported. The recommendation should be applied.

## `derive-input/src/internal/db/table.rs`

Maps the `#[table]` arguments into the SpacetimeDB part of the model.

### `table.rs`: `SpacetimeDBTable::map(table: &TableArgs, is_singleton: bool) -> syn::Result<SpacetimeDBTable>` and the field `multi_column_indices`

**Violates:** Code For The Maintainer — Principle of Least Astonishment; Connascence — of Execution; Separation of Concerns

The comment on the field admits that `multi_column_indices` contains all indices during processing; the name only becomes true once the columns have taken their indices. The mapping also enforces a DSL rule (no multi-column index on a singleton).

Recommendation: Keep all indices in an accurately named collection while processing (or split them up front, see `internal/table.rs`), and move the singleton rule to the DSL layer.

Developer's decision: The recommendation should be applied.

### `table.rs`: `ScheduledReducer::map(scheduled: &ScheduledArg) -> ScheduledReducer`

**Violates:** Robustness Principle; YAGNI

The reducer name becomes an `Ident` by formatting its tokens into a string. If the bindings parser accepts a qualified path there, such as `scheduled(crate::timers::tick)`, `format_ident!` panics. The value this produces, `SpacetimeDBTable::scheduled_reducer`, is read nowhere in the repository.

Recommendation: Keep the `Path` and pin a qualified path in a snapshot fixture.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/db/column.rs`

Maps a field into the SpacetimeDB part of the model and assigns its index.

### `column.rs`: `SpacetimeDBColumn::map(rust_field, spacetimedb_table, auto_inc_column_names, primary_key_column_name, is_singleton) -> syn::Result<(SpacetimeDBTable, SpacetimeDBColumn)>`

**Violates:** Separation of Concerns; Command Query Separation (CQS); DRY; Robustness Principle

The mapping of the SpacetimeDB column also enforces DSL rules (no primary key prefixed with the table name, no index on a singleton's column). It walks the indices with the same `match` for the validation and again for the lookup. The lookup stops at the first single-column index, so a column with more than one single-column index leaves the others in `multi_column_indices`, where `SpacetimeDSLTableMethods::generate` has to skip them — silently, without a diagnostic and without methods.

Recommendation: Move the DSL validation to the DSL layer and return the index assignment as data (see `internal/table.rs`). An additional single-column index on one column must be rejected with a diagnostic. Pin that with a compile-test.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/table.rs`

Parses the table-level DSL settings and validates the roles of columns.

### `table.rs`: `SpacetimeDSLTable::try_parse(dsl_data: DSLData, column_args: &ColumnArgs<'_>, spacetimedb_table: SpacetimeDBTable) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable)>`

**Violates:** Single Responsibility Principle (SRP); Separation of Concerns; KISS; YAGNI

One function marks DSL-unique indices, builds the hooks, validates the update method, parses the soft-delete marker and the referencing tables, checks visibilities and validates timestamp roles. The DSL's own unique multi-column index feature is stored as `Index::is_unique` in the SpacetimeDB model, so that flag no longer tells whether SpacetimeDB or the generated code enforces uniqueness. `has_update_method` is shadowed from `Option<bool>` to `bool`, and `referencing_tables` is filled by assigning only while it is still empty.

The final check that raises `update_method_disabled_with_set_on_update_column` can never be reached: inside the loop, a `set_on_update` column without the update method and a non-private column without the update method both return earlier, so `all_columns_are_private` only feeds that dead check.

Recommendation: Split into focused validators and keep DSL uniqueness out of the SpacetimeDB model (for example as a set of DSL-unique index names on the DSL side). The unreachable check is dead code: the developers must decide whether its rule is fully covered by `set_on_update_column_without_update_method` (then delete the check, the flag and the error function) or whether its unreachability hides a missing case (then first write the compile-test that should raise it).

### `table.rs`: `fn get_timestamp_role(field: &SatsField<'_>) -> syn::Result<Option<TimestampRole>>`

**Violates:** DRY; Coupling Awareness — "avoid hidden coupling or magic"; KISS

Column names claim framework roles implicitly — `created_at`, `inserted_at`, `modified_at` and `updated_at` here, `deleted`, `removed`, `deleted_at` and `removed_at` in `internal/dsl/soft_delete.rs`. The behaviour is documented, but the timestamp roles and the soft-delete roles use different mechanisms (inline string comparisons here, constant arrays there), so renaming a role or adding one follows no single pattern. `(true, true) => unreachable!()` restates a case the early return already excluded.

Recommendation: Keep the conventional names of all roles as constants in one module and look them up the same way.

Developer's decision: The recommendation should be applied.

### `table.rs`: the names in `DSLData::unique_indices`

**Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"

`#[dsl(unique_index(name = x))]` A misspelled name is accepted without effect and without a diagnostic, and a repeated name is not detected.

Recommendation: Reject names that do not match an eligible index with a compile-test.

Developer's decision: The recommendation should be applied.

### `table.rs`: the types in `DSLData::unique_indices`

**Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"

`#[dsl(unique_index(name = x))]` only has an effect when `x` names a BTree multi-column index. A single-column index or a hash multi-column index is accepted without effect and without a diagnostic.

Recommendation: When single-column and hash indices are meant to be supported, they should be added, with a snapshot and a runtime test. Otherwise reject them with a compile-test.

## `derive-input/src/internal/dsl/soft_delete.rs`

Decides which column a soft deletion writes and validates it.

### `soft_delete.rs`: `fn claimed_kind(field: &SatsField<'_>) -> syn::Result<Option<SoftDeleteMarkerKind>>` and `fn type_fits(kind: SoftDeleteMarkerKind, field_type: &str) -> bool`

**Violates:** DRY

The marker's type is checked by comparing stringified tokens, restating `is_optional_timestamp_type` from `internal/dsl/table.rs`, and the conventional names live in constants while the timestamp roles use inline strings.

Recommendation: Resolve together with the type classification in `internal/column.rs` and the role names in `internal/dsl/table.rs`.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/hook.rs`

Builds the hook traits and signatures a table declares.

### `hook.rs`: the hook matrix — `pub fn build(singular_table_name, singleton, declared: DeclaredHooks) -> SpacetimeDSLMethodHooks` and `pub struct DeclaredHooks`

**Violates:** DRY; Open/Closed; Encapsulate What Changes

The combinations of timing and operation are written out by hand: one `build_any` call per hook here, one field per hook in `DeclaredHooks`, in `SpacetimeDSLMethodHooks` and in `DSLData`, one `Option<Span>` local and copied parser branches in `internal.rs::try_parse_dsl`, and one `hook::build` call per hook in `derive/src/output.rs`. Adding the soft-delete operation had to touch every one of these places, and the struct literal in `build` lists its fields in a different order than their declaration.

Recommendation: Represent the declared hooks as a collection keyed by `(Timing, Operation)` with an iterator over all kinds, so that parsing, building and emitting loop over the kinds; keep named accessors where a generator asks for one specific hook. Pure refactoring.

Developer's decision: The recommendation should be applied.

### `hook.rs`: `fn get_function_args(...)` and `fn get_return_type(...)` — the `Create<Table>` name

**Violates:** DRY — the code's own FIXME calls it a "Single Source of Truth Violation"

The name of the create-request struct is formatted here, in the arguments and again in the return type, and in `internal/dsl/method/create.rs`, which defines the struct. The hook signature and the struct agree only by convention.

Recommendation: One naming function in `internal/dsl/method/naming.rs`; the FIXME goes with the fix.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/singleton.rs`

The generator's knowledge of the primary key injected into singleton tables.

### `singleton.rs`: `PRIMARY_KEY_NAME`, `PRIMARY_KEY_TYPE`, `PRIMARY_KEY_VALUE` and `pub fn is_primary_key_column(name: &Ident, type_name_or_path: &Path) -> bool`

**Violates:** DRY; Connascence — of Meaning, across crates

The module is the `derive-input` half of the knowledge that `derive/src/lib.rs::inject_singleton_primary_key` spells out again. `is_primary_key_column` compares the type as text with `"u8"`, and `rendered_primary_key` restates the message format of `internal/dsl/method/index.rs::column_names_and_row_values`, as its doc comment says.

Recommendation: Resolve together with the `derive/src/lib.rs` entry (one home for the injected key), the type classification in `internal/column.rs` and the message formats in `internal/dsl/method/index.rs`.

## `derive-input/src/internal/dsl/one_or_multiple.rs`

The generator's copy of the runtime's `OneOrMultiple`.

### `one_or_multiple.rs`: `pub enum OneOrMultiple`

**Violates:** DRY; Connascence — of Name

A mirror of `src/error.rs::OneOrMultiple` whose variants reach generated code by name.

Recommendation: Resolve together with the `src/error.rs` entry.

## `derive-input/src/internal/dsl/wrapper.rs`

Parses `#[create_wrapper]` and `#[use_wrapper]` and generates wrapper structs.

### `wrapper.rs`: `WrapperType::try_parse(rust_struct: &RustStruct, rust_field: &RustField, field: &SatsField<'_>) -> syn::Result<Option<WrapperType>>`

**Violates:** KISS; Connascence — of Meaning; Robustness Principle; Single Responsibility Principle (SRP); Scope & Goal Discipline — "Future ideas captured outside codebase"

Names and types are turned into `String` and parsed back with `expect`; a local declared without a value is assigned `Some` in some branches and unwrapped with `expect` right after; the comparison with `create_wrapper` is repeated. Parsing the attribute and generating the wrapper struct (`get_wrapper_impl`) happen in one flow.

Recommendation: Keep `syn` values (`Ident`, `Path`, `Type`) from parsing to generation, and separate parsing from generating the wrapper.

Developer's decision: The recommendation should be applied.

### `wrapper.rs`: `fn get_wrapper_impl(struct_name, wrapper_struct_name, wrapped_type_name_or_path, field_name, wraps_uuid) -> TokenStream`

**Violates:** Code For The Maintainer — Principle of Least Astonishment; DRY

Every created wrapper displays as `Name { id: value }`, whatever column it wraps; `#[create_wrapper]` is also used on non-key columns such as `name3: String` in `examples/test/src/entity.rs`. The generated `From` implementations import `spacetimedsl::Wrapper` through a hand-written path (see `api/runtime.rs`).

Recommendation: Render the wrapped column's name or a neutral form. The text appears in every `DeletionResult` row, so decide it together with the `src/delete.rs` format entry and pin it with a runtime assertion.

### `wrapper.rs`: `WrapperType::map(value: &WrapperType) -> Type`, `WrapperType::map_to_wrapped_type(value: &WrapperType) -> Type` and `WrapperType::struct_name_or_path_tokens(&self) -> TokenStream`

**Violates:** DRY; Self-Documenting Code — don't document the past; KISS

`map` and `struct_name_or_path_tokens` both produce the wrapper's name or path, one of them by formatting and re-parsing it. The panic messages name `WrapperType::map_to_wrapper_type`, `WrapperType::Wrap` and `WrapperType::Wrapped`, none of which exist (the current names are `map`, `Created` and `Used`), and `map_to_wrapped_type` claims to parse an `Ident` while it parses a `Type`. `map` and `map_to_wrapped_type` take the wrapper as an argument while their siblings take `&self`.

Recommendation: One `&self` method that returns the wrapper type without a string round trip and one for the wrapped type; correct the messages.

Developer's decision: The recommendation should be applied.

### `wrapper.rs`: `pub fn map_wrapper_type_option_to_wrapped_type_option(column_name: &Ident, wrapper_type_name_or_path: &Type) -> TokenStream`

**Violates:** KISS — the generated code is read by users in the rustdoc "Implementation" section

The generated code declares a mutable `None`, tests `is_some()` and reassigns through `expect`, which is `Option::map` written out by hand.

Recommendation: Emit `Option::map`. The snapshots change.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/foreign_key.rs`

Parses and validates `#[foreign_key]`.

### `foreign_key.rs`: `ForeignKey::try_parse(has_delete_method: &bool, is_soft_deletable: bool, is_singleton: bool, field: &SatsField<'_>) -> syn::Result<Option<ForeignKey>>`

**Violates:** Robustness Principle; DRY; KISS

`OnDeleteStrategy::SetZero` is documented as available only for numeric columns, but nothing checks the column type — the TODO on `try_parse_for_on_delete` lists the missing checks. A `Uuid` foreign key with `on_delete = SetZero` generates an assignment of `0`, which fails to compile inside the expanded code instead of at the attribute. Whether the column has an index is re-derived by scanning its raw attributes, although `SpacetimeDBColumn` already knows it (`reference.rs` does the same for `#[primary_key]`). A private visibility is detected by comparing token strings, while `internal/dsl/table.rs` uses `matches!`. `has_delete_method` is passed as `&bool`.

Recommendation: Add the type check with a diagnostic (compile-test first), pass the facts `SpacetimeDBColumn` already holds instead of re-scanning attributes, use `matches!(field.vis, syn::Visibility::Inherited)`, and pass `bool` by value.

Developer's decision: The recommendation should be applied. Allow `SetZero` also on the `Uuid` type, which should set `Uuid::NIL` rather than `0`. Treat `Uuid::NIL` like `0` in create/update/delete/soft_delete DSL methods ("no row of another table referenced").

### `foreign_key.rs`: `OnDeleteStrategy::try_parse_for_on_soft_delete(meta: &ParseNestedMeta<'_>, tokens: &Meta) -> syn::Result<OnDeleteStrategy>`

**Violates:** Code For The Maintainer; DRY

The doc says the strategies `on_soft_delete` does not accept are rejected by name rather than by the catch-all, but `SetNone` is not named and falls into `unknown_on_soft_delete_strategy`. The variant spellings are repeated in the parsers for `on_delete` and `on_soft_delete`, in `ToTokens` and in the runtime `Display`.

Recommendation: Make doc and code agree — name `SetNone` explicitly (with a compile-test) or change the doc. For the shared spellings, the developers can derive parsing from the variant names while keeping each context's allow-list explicit.

## `derive-input/src/internal/dsl/reference.rs`

Parses `#[referenced_by]`.

### `reference.rs`: `ReferencingTable::try_parse(has_delete_method: &bool, is_soft_deletable: bool, field: &SatsField<'_>) -> syn::Result<Vec<ReferencingTable>>`

**Violates:** DRY

Whether the field is the primary key is re-derived from its raw attributes, although `SpacetimeDBColumn::is_primary_key` already states it, and `has_delete_method` is passed as `&bool`.

Recommendation: Resolve together with `internal/dsl/foreign_key.rs`.

## `derive-input/src/internal/dsl/getter.rs`

Generates the getter of a column.

### `getter.rs`: `pub fn get_getter_method_name(column_name: &Ident) -> Ident` and `mut_getter.rs::get_mut_getter_method_name`

**Violates:** DRY; Hide Implementation Details — "Minimize class and member accessibility"

The getter naming rule has a helper, but `internal/dsl/method/reference_integrity.rs` and `internal/dsl/method/upsert.rs` format `get_{column}` again. The helpers are `pub` without callers outside their own files.

Recommendation: Move all naming of generated methods into `internal/dsl/method/naming.rs`, and reduce the visibility.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/setter.rs`

Generates the setter of a column.

### `setter.rs`: `Setter::map(rust_field: &RustField, is_option: bool, wrapper_type: &Option<WrapperType>) -> Option<Setter>`

**Violates:** KISS; Self-Documenting Code — no abbreviations

The locals `method_arg`, `return_type` and `return_expr` are declared first and assigned in nested branches, and `method_impl` is initialised, then prepended to in one branch and replaced in another, so the resulting method body is hard to predict. The generated `match` over `old_value` is `Option::map` written out. `rt` and `return_expr` are abbreviations. `getter.rs` and `mut_getter.rs` use the same deferred style, and `mut_getter.rs` matches on `wrapper_type` only to return early in one arm.

Recommendation: Let each branch produce its argument, return type and body as one expression, emit `Option::map`, and spell names out. The snapshots show any change in the generated accessors.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method.rs`

Decides which DSL methods a table and each of its indices earn.

### `method.rs`: `SpacetimeDSLTableMethods::generate(context: &MethodGenerationContext, columns: &[Column]) -> syn::Result<(SpacetimeDSLTableMethods, TableContributions)>`

**Violates:** Single Responsibility Principle (SRP); DRY; KISS

One function produces the create, get-all and count methods, the referenced side's cascade entry points, the grouping of foreign-key columns by referenced table, the referencing side's strategy entry points, the wrapper methods and the multi-column index methods. The block "for each kind of removal: build the one-row and the many-row method, merge the contributions, assign to `on_deletion` or `on_soft_deletion`" is written for the referenced side and again for the referencing side. The grouping uses `contains_key`, `insert` and `get_mut().expect()` where `entry(key).or_default()` suffices; `internal/dsl/method/foreign_key.rs` does the same.

Recommendation: One function per concern, a shared helper for the entry points per kind of removal, and `entry().or_default()`. Pure refactoring.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/create.rs`

Generates `create_<table>` and the `Create<Table>` argument struct.

### `create.rs`: `fn create_method_column_parts(spacetimedsl_table: &SpacetimeDSLTable, internal_column: &InternalColumn) -> CreateMethodColumnParts`

**Violates:** KISS; Open/Closed; YAGNI

A chain of `if … else if` over column roles — injected singleton key, generated UUID, auto-increment, created-at, updated-at, soft-delete marker, then the wrapper kinds — assigns locals declared without a value and returns early. The conditions are written as block expressions (`&& { … }`), and inside them the local `column_name` is shadowed by the table's timestamp column name. A special case for `String` columns yields `String` as the argument type where the general branch would yield the column's own type, which for such a column is `String` as well — it looks vestigial. Each new column role adds another link to the chain.

Recommendation: Classify each column into a role once — an enum that `upsert.rs`, which re-implements the created-at and updated-at assignments, can use too — and map the role to its parts with a `match`. For the `String` special case, some spelling may depend on it; the snapshot diff after removing it will show.

Developer's decision: The recommendation should be applied.

### `create.rs`: `pub fn for_create(context: &MethodGenerationContext) -> (SpacetimeDSLMethod, TableContributions)`

**Violates:** DRY; Scope & Goal Discipline — "Future ideas captured outside codebase"

The `try_insert` statement with its mapping of `UniqueConstraintViolation` and `AutoIncOverflow`, the message format that renders the whole row, and the FIXMEs "Only show unique columns here" and "No clone?" are copied into the insert path of `upsert.rs`.

Recommendation: One helper that emits the insert-and-map-errors statement for `for_create` and `for_singleton_upsert`.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/get.rs`

Generates the lookup methods.

### `get.rs`: `pub fn for_get_one(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** DRY; Scope & Goal Discipline — "Future ideas captured outside codebase"

Both branches build the same `not_found_error`.

Recommendation: Build the error once before the branch.

## `derive-input/src/internal/dsl/method/index.rs`

What an index tells the generators that look rows up through it.

### `index.rs`: `pub fn column_names_and_row_values(column_names: &[Ident]) -> String` and the message formats elsewhere

**Violates:** DRY; Connascence — of Meaning

The shape of "column : value" messages is stated here, restated in `internal/dsl/singleton.rs::rendered_primary_key`, formatted by hand for a whole row in `create.rs` and `upsert.rs`, and formatted again for a single column in `reference_integrity.rs`. Format strings that are built at generation time and escaped for a second `format!` at run time are hard to read and easy to break.

Recommendation: One module that owns every message shape and returns the finished `format!` tokens.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/reference_integrity.rs`

The referential-integrity and unique multi-column index checks generated methods run before they write.

### `reference_integrity.rs`: the identifiers `the_same_or_another_<table>`, `get_<column>` and `get_<table>_by_<key>`

**Violates:** DRY; Connascence — of Name, across tables; Code For The Maintainer

`MethodGenerationContext` documents itself as the one place that states the name `field_name_for_found_value`, yet several functions here format `the_same_or_another_{table}` again. Getter names are formatted again as well (see `internal/dsl/getter.rs`). The method a referencing table calls on the referenced table, `get_<table>_by_<key>`, must match what `get.rs` generates in the other table's expansion — exactly the kind of name `internal/dsl/method/naming.rs` says it owns, but it is not there.

Recommendation: Pass `context.field_name_for_found_value` in, move the `get_<table>_by_<index>` rule into `naming.rs` and use it in `get.rs` and here, and use the getter helper.

Developer's decision: The recommendation should be applied.

### `reference_integrity.rs`: `pub enum Action`

**Violates:** DRY; Connascence — of Name

A mirror of `src/error.rs::Action` whose variants reach generated code through `strum::Display` and `format_ident!`.

Recommendation: Resolve together with the `src/error.rs` entry.

### `reference_integrity.rs`: DRY in `pub fn multi_column_index_checks(...)` and `pub fn unique_multi_column_index_check(...)`

**Violates:** DRY

Both functions build the same unique-constraint-violation error.

Recommendation: Build the error once and pass it on.

Developer's decision: The recommendation should be applied.

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

### `on_delete_strategy.rs`: the bindings shared with `method/foreign_key.rs`

**Violates:** Connascence — of Name, only checked after expansion; Hide Implementation Details

The fragments generated here depend on bindings that `for_foreign_key` declares in another module — `dsl`, `entries`, `error`, `error_from_hook`, the label `'outer` and `primary_key_value_of_a_row_of_another_table_to_delete` — and on bindings their sibling fragments introduce. A rename on one side still compiles in `derive-input` and fails only when a user's crate expands the macro, because the snapshot harness never compiles tokens.

Recommendation: Name the shared bindings once in `naming.rs`, so a rename is one change. The runtime tests remain the only gate that compiles these fragments; keep every strategy covered there.

Developer's decision: The recommendation should be applied.

### `on_delete_strategy.rs`: `fn hooks_around_the_write(before_hook, after_hook, old_row) -> ((TokenStream, TokenStream), (TokenStream, TokenStream))` and `fn store_the_row(spacetimedb_call_prefix, primary_key_column_name, after_hook) -> TokenStream`

**Violates:** Connascence — of Position; Robustness Principle

The nested tuple can only be read by position. `store_the_row` emits SpacetimeDB's `update`, which panics on a constraint violation (see `update.rs`), and the generated cascade also panics through `expect` calls with messages such as "Should exist".

Recommendation: A named struct for the hook fragments; resolve the panics together with the `update.rs` entry and give the remaining `expect` calls messages that state the broken invariant.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/removal.rs`

The shared body of the `delete_*` and `soft_delete_*` methods.

### `removal.rs`: `pub fn for_removal_many(removal: Removal, shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod` and `pub fn for_removal_one(removal: Removal, shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** DRY; Single Responsibility Principle (SRP); Code For The Maintainer — Principle of Least Astonishment; Connascence — of Name

`for_removal_many` and `for_removal_one` repeat one structure — the hook blocks of a hard deletion, the reported strategy, the deletion-result entry, the handler for an error after the state changed, the call of the `Error` strategy, the strategies after the write, the final assembly and the naming — and differ mainly in how rows are found and how results are collected. The errors raised after a state change call themselves "Delete Many Error" or "Delete One Error" for soft deletions as well. `retire_row_named_old_row` expects the surrounding code to have bound a variable called `old_row`, a contract carried by the function's name instead of a parameter.

Recommendation: Extract the shared skeleton, parameterised by one or many rows, as the module already does for hard and soft removal; let the messages name the actual removal; pass the row binding as an `Ident` parameter. The snapshots guard the refactoring.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/referenced_by.rs`

The referenced side of a foreign key: the cascade entry points and the calls into them.

### `referenced_by.rs`: `pub fn referenced_table_function_call_for_dsl_method(...)` and `on_delete_strategy.rs::referenced_table_function_call_for_strategy_implementation(...)`

**Violates:** DRY

Both emit "call the dispatcher, merge the child entries on success and on failure, run the error handler", but they differ in where the keys come from, which container collects the entries, and how a hook error is kept: the DSL-method variant shadows `error_from_hook` with the failure's value, while the strategy variant keeps the first error it saw.

Recommendation: Understand the differences to make an informed decision whether they are intended — a DSL method starts without a hook error, while a cascade has to keep the first one — before extracting a shared helper, or keep them separate and document why.

Developer's decision: The recommendation should be applied, but independently of any other fix decided by the developer.

### `referenced_by.rs`: `pub fn for_referenced_by(removal, one_or_multiple, spacetimedb_table, spacetimedsl_table, primary_key_column) -> (SpacetimeDSLMethod, TableContributions)`

**Violates:** DRY

The dispatcher signature (DSL, strategy, one key or a slice of keys, entries in a `Vec` or a `HashMap`, the failure type) is built here and again in `method/foreign_key.rs::for_foreign_key`. Both contain the same `past_tense` wording, which `naming.rs::removal_suffix` repeats in snake case.

Recommendation: One helper for the dispatcher signature, and one table for the removal wording from which both the prose and the snake-case suffix are derived.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/foreign_key.rs`

The referencing side of a foreign key: the function a table generates for each table it references.

### `foreign_key.rs`: `pub fn for_foreign_key(removal, one_or_multiple, referencing_tables, context, referenced_table_name, columns_with_foreign_key) -> syn::Result<(SpacetimeDSLMethod, TableContributions)>`

**Violates:** KISS; DRY; Robustness Principle; Self-Documenting Code

Within one loop iteration, the same `foreign_key` option is unwrapped in different ways (`expect` and `unwrap_or_else` with `panic!`). Type and path equality between the grouped columns are decided by comparing token strings, so `u64` and `core::primitive::u64`, or a relative and an absolute path to the same module, are reported as mismatches.

Recommendation: Bind the foreign key once per column, and compare types through the classification in `internal/column.rs`.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/update.rs`

Generates `update_<table>_by_<key>`.

### `update.rs`: `pub fn for_update(shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** Robustness Principle; Code For The Maintainer — Principle of Least Astonishment; Self-Documenting Code — no abbreviations

The generated method calls SpacetimeDB's `update`, which panics on a constraint violation, instead of `try_update`; the FIXME about `try_update` appears here, in `upsert.rs` and in `on_delete_strategy.rs`. When a before-update hook exists, the generated code fetches the stored row with an `expect`, while the reference-integrity check of the same method returns a `NotFoundError` for the same situation — whether a missing row panics or returns an error depends on whether a hook is declared. `is_singleton_pk` is abbreviated.

Recommendation: Use `try_update`, map its errors to `SpacetimeDSLError`, and return a `NotFoundError` whenever the row is missing. Pin both with runtime tests (a missing row with and without a hook; a constraint violation on update). Turning panics into errors changes observable behaviour, so the developers decide how to announce it.

Developer's decision: The recommendation should be applied, including turnings panics into errors, but independently of any other fix decided by the developer. The current problem is that no `try_update` exists in SpacetimeDB, so this isn't possible to fix currently!

## `derive-input/src/internal/dsl/method/upsert.rs`

Generates `upsert_<table>` and hosts write-path helpers used by other generators.

### `upsert.rs`: the region "Pieces shared with `update.rs`"

**Violates:** Maximize Cohesion; Single Responsibility Principle (SRP); Self-Documenting Code

The module holds the upsert generator and the helpers that `update.rs`, `removal.rs` and `on_delete_strategy.rs` import, so unrelated generators depend on the upsert module. Its private `row_value_getter` shares its name with a different private function in `reference_integrity.rs`. `updated_at_column` and `set_created_at_on_insert` repeat the "find the column by name or panic" lookup that `index.rs`, `removal.rs` and `reference_integrity.rs` also contain. The doc of `rebind_row_as_mutable_after_hook` refers to `keep_created_at`, whose name is `keep_created_at_on_update`.

Recommendation: Move the shared write-path helpers into their own module, give the `row_value_getter` functions of `upsert.rs` and `reference_integrity.rs` names that say how they differ, and add one lookup helper for columns — the doc of `MethodGenerationContext` forbids generation methods, not lookups, but where it lives is the developers' decision. Correct the doc reference.

Developer's decision: The recommendation should be applied, but independently of any other fix decided by the developer.

### `upsert.rs`: `pub fn for_singleton_upsert(context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** DRY

The insert path copies the `try_insert` statement of `for_create`, its error mapping, its message format and its FIXMEs (see `create.rs`).

Recommendation: Use the shared insert helper proposed in the `create.rs` entry.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/dsl/method/singleton_table.rs`

Generates `get_<table>` and `delete_<table>` for singleton tables.

### `singleton_table.rs`: `pub fn for_singleton_delete(context: &MethodGenerationContext) -> SpacetimeDSLMethod`

**Violates:** DRY; YAGNI

The hook blocks and the text of the count-mismatch error are copied from `removal.rs::for_removal_one`, and the generated body imports `Itertools` without calling any of its methods.

Recommendation: Share the hook and message helpers with `removal.rs` and drop the import; the snapshot shows the change.

Developer's decision: The recommendation should be applied.

## `derive-input/src/internal/error.rs`

Every diagnostic the parser raises for a rejected table.

### `error.rs`: `pub fn no_table_attribute_found(struct_name: &Ident) -> Error` and `pub fn update_method_disabled_with_set_on_update_column(struct_name: &Ident) -> Error`

**Violates:** YAGNI

Both are only raised from branches that cannot be reached (see `internal/integration.rs` and `internal/dsl/table.rs`), so no compile-test pins them.

Recommendation: Dead code: the developers must decide for each whether its rule is covered by another diagnostic (then delete the function and its branch) or whether the unreachable branch is a bug (then first write the compile-test that should produce it).

### `error.rs`: `fn visibility_variant_name(visibility: &Visibility) -> &'static str`

**Violates:** Hide Implementation Details; Code For The Maintainer — Principle of Least Astonishment

User-facing messages print names of `syn` types — a column should have `Visibility::Inherited`, found `Visibility::Public` — instead of Rust syntax, while neighbouring messages print the visibility as the user wrote it.

Recommendation: Render the visibility as written (`pub`, `pub(crate)`, no modifier) and regenerate the affected `.stderr` files.

Developer's decision: The recommendation should be applied.

### `error.rs`: `pub fn missing_update_method_with_only_private_columns(struct_name: &Ident) -> Error`

**Violates:** Code For The Maintainer; DRY — "Sync related artifacts—code, docs, tests—whenever knowledge changes"

The message tells users that a mutable table needs a non-private column or one named `modified_at` / `updated_at`, without mentioning `#[set_on_update]`, which has the same effect.

Recommendation: Name the attribute next to the conventional names and regenerate the `.stderr` files.

Developer's decision: The recommendation should be applied.

---

## `debug-helper/src/main.rs`

A development tool that writes the syntax trees of the runtime test module to files.

### `main.rs`: `impl Display for Error` and `fn render_location(formatter, err: &syn::Error, filepath: &Path, code: &str) -> fmt::Result`

**Violates:** Code For The Maintainer — Principle of Least Astonishment; Self-Documenting Code — no abbreviations

The usage message calls the program `dump-syntax`, the name of the `syn` example it was taken from, while it is the `spacetimedsl-debug` crate started through `x debug`. `err` and `n` are abbreviations.

Recommendation: Name the actual program in the usage text and spell the identifiers out.

Developer's decision: The recommendation should be applied.

## `examples/test/src/lib.rs`

The `tester` reducer, which runs every group of runtime tests.

### `lib.rs`: `fn tester(ctx: &ReducerContext) -> Result<(), String>`

**Violates:** F.I.R.S.T Principles of Testing — independent, repeatable; Self-Documenting Code — don't document the past

Every group runs in one reducer on one database. Later groups see the rows earlier groups left behind, and absolute assertions such as `count_of_all_entity_relationships().ne(&3)` depend on that; the first failing group stops all later ones and hides their results. The module `spacetimedsl_cascade_delete_hook_repro` is named after the bug report it reproduced rather than the behaviour it pins.

Recommendation: Options: (a) one reducer per group, each called and checked by `x.ps1 test`; (b) run every group and report all failures together before failing. In both cases make count assertions relative to a count taken before the act, as `update_and_soft_delete_hook_test.rs` already does with `logged_before`. Rename the module after the behaviour it covers. AGENTS.md describes this harness and has to follow any change to it.

Developer's decision: The recommendation (b) should be applied.

## `examples/test/src/spacetimedsl_cascade_delete_hook_repro.rs`

Tables whose cascade reaches a hooked table through a primary-key foreign key.

### `spacetimedsl_cascade_delete_hook_repro.rs`: the tables `ParentRecord` and `ChildMarker` and their hooks

**Violates:** Testing & Verification — *A test that cannot run is not a test*, "Make assertions binary without manual inspection"

The module declares tables and hooks but has no `run_tests`, and `tester` does not call it. It only proves that the expansion compiles, which the runtime gate checks as a side effect rather than by an assertion.

Recommendation: Add a `run_tests` that deletes a `ParentRecord` and asserts the cascade and the hook calls.

Developer's decision: The recommendation should be applied.

## `examples/test/src/entity.rs`

Runtime tests around the `Entity` table and its relationships.

### `entity.rs`: `pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String>`

**Violates:** Arrange, Act, Assert (3A); F.I.R.S.T Principles of Testing; Code For The Maintainer; Self-Documenting Code — no abbreviations

One function arranges, acts and asserts many unrelated behaviours in sequence. The failure message after deleting `player2` says that `player` should be deletable. Absolute counts depend on no other group having created relationships. `er4_1` and `er4_2` are abbreviations, and `match` blocks that return an error alternate with `?` without a reason.

Recommendation: One function per behaviour with its own rows, a corrected message, relative counts, spelled-out names and one error-handling style.

Developer's decision: The recommendation should be applied.

## `examples/test/src/component/test.rs`

A table that exercises many column, accessor and index shapes, plus related tables.

### `test.rs`: `pub struct Test`

**Violates:** Code For The Maintainer

The doc comments were copied from another table: `Test` is documented as "A Position in the World", its `id` as "The unique ID of the World".

Recommendation: Delete the comments.

### `test.rs`: `pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String>`

**Violates:** F.I.R.S.T Principles of Testing — self-validating; Arrange, Act, Assert (3A); Testing & Verification — "Make assertions binary without manual inspection"

Many calls discard their results (`let _ = dsl.get_tests_by_wrapped_index(…)`, `let _ = dsl.delete_tests_by_wrapped_index(…)`): they only prove that the argument types are accepted, which is a compile-time property, and they ignore run-time failures. `let _ = dsl.create_ship_object(…)` is the arrange step of the following assertion, so if it fails, the assertion checks something else.

Recommendation: Assert what each call should return. Treat a failing arrange step as a failure (`?`).

Developer's decision: The recommendation should be applied.

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

## `examples/test/src/update_and_soft_delete_hook_test.rs`

Runtime tests for the hooks that run when cascades write a row.

### `update_and_soft_delete_hook_test.rs`: `fn soft_delete_skips_update_hooks_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String>`

**Violates:** F.I.R.S.T Principles of Testing — self-validating; Test Driven Development — *Read the failure, not just the fact of it*

The last check reads `modified_at` from `member`, the value `create_guild_member` returned before any soft deletion happened, instead of the stored row afterwards, so it passes whatever the soft deletion does. Its message speaks of `deleted_at`, but the table's marker column is `deleted: bool`.

Recommendation: Re-read the row with `get_guild_member_by_id` after the soft deletion and correct the message. Observe the check fail before trusting it, for example by temporarily asserting the opposite.

Developer's decision: The recommendation should be applied.

## `examples/blackholio/src/lib.rs`

The Blackholio tutorial module, ported to SpacetimeDSL.

### `lib.rs`: the FIXME and TODO comments

**Violates:** Self-Documenting Code — don't document the past

The FIXMEs on `Entity::position` and `Entity::login_status` report that `DbVector2` and `LoginStatus` lack `PartialEq`, but both derive it now. The FIXME above `Player` says `update = true` should not have been valid because all fields were private, but the struct has public fields now.

Recommendation: Delete the outdated FIXMEs.

Developer's decision: The recommendation should be applied.

## `gen-x.sh`

Generates the developer entry points `x.sh` and `x.ps1`, which must not be edited directly.

### `gen-x.sh`: `generate_test` and `generate_debug`

**Violates:** Robustness Principle — "Log and surface malformed partner payloads immediately"; F.I.R.S.T Principles of Testing — self-validating; Modular Boundaries & Separation — "Design APIs without cross-cutting side effects"

The generated `test` command runs each `spacetime` command without checking its result and ends with a change of directory, so it reports success even when publishing fails or the `tester` reducer returns an error. AGENTS.md documents searching the output for the success marker as the workaround, and every tool built on top of the command inherits the problem (CI, `compare-performance.ps1`). The PowerShell `debug` command sets `$env:RUSTFLAGS` and never restores it, so every later build in the same session runs with `-Zmacro-backtrace`. `compare-performance.ps1` already shows the robust pattern in this repository: `$ErrorActionPreference = "Stop"`, explicit `$LASTEXITCODE` checks and waiting for the server.

Recommendation: Generate fail-fast scripts (`set -euo pipefail`; `$ErrorActionPreference = 'Stop'` with `$LASTEXITCODE` checks), let `test` fail when the success marker is missing, and restore location and environment (`Push-Location` / `Pop-Location`, `try` / `finally`). Regenerate `x.sh` and `x.ps1`, and update AGENTS.md's description of the gate in the same change.

Developer's decision: The recommendation should be applied.

### `gen-x.sh`: `generate_usage`, the final message and the embedded `loc` programs

**Violates:** Code For The Maintainer; Self-Documenting Code — no redundant comments

The usage text describes `format` as running a formatter check, although it rewrites files and checks nothing, and its description of `test` does not mention that `test` also publishes `blackholio`. The final message claims to have generated `x` instead of `x.sh`. The embedded `loc` programs carry comments that repeat the statement below them.

Recommendation: Correct the texts and delete the comments that repeat the code.

Developer's decision: The recommendation should be applied.

## `.github/workflows/test.yml`

The CI workflow that builds and tests every push and pull request.

### `test.yml`: job `test`

**Violates:** Robustness Principle; F.I.R.S.T Principles of Testing — self-validating; DRY; Code For The Maintainer

The step "Build & test SpacetimeDSL" relies on the exit status of `./x.sh test`, which is always success (see `gen-x.sh`), so this workflow — and the release workflow, which waits for it — can pass while the runtime tests fail. The clippy step appends `|| echo …` to every invocation, so it can never fail, and it lints directory by directory, which the comment in `gen-x.sh` explains leaves crates unlinted: the examples, `compile-tests` and `debug-helper` are never linted in CI. The comment above the installation retry loop states a different number of attempts than `MAX_ATTEMPTS`.

Recommendation: Gate on the fixed script. Add a lint command to `gen-x.sh` that runs clippy over the workspace in check mode with warnings denied, and let CI and developers use the same command.

Developer's decision: The recommendation should be applied.

## `.github/workflows/release.yml`

The workflow that publishes the crates for a version tag.

### `release.yml`: step "Install Rust toolchain"

**Violates:** DRY; Code For The Maintainer

The test workflow states that the toolchain comes from `rust-toolchain.toml`, while the release workflow asks for `toolchain: stable`, which the pinned toolchain file overrides inside the repository — the configuration says something the build does not do.

Recommendation: Drop the explicit `toolchain` input, so both workflows take the toolchain from `rust-toolchain.toml`.

Developer's decision: The recommendation should be applied.
