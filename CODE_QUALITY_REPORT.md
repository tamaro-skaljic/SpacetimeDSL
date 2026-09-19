# Code Quality Report

An audit of the SpacetimeDSL workspace against the programming principles in
[`AGENTS.md`](AGENTS.md).

The report is organised by file. A `##` heading summarises a file (or, where a violation
genuinely spans the workspace, a cross-cutting concern), and each `###` heading below it
describes one type, function or concept. A prioritised resolution order closes the report.

Findings are about design and principle adherence, not about lints: `cargo clippy
--workspace --all-targets --all-features` passes cleanly.

---

## Workspace-Wide

Violations that no single file owns, listed here so they are not repeated under every file
they touch.

### `src/delete.rs` + `derive-input/src/api/dsl/foreign_key.rs`: `OnDeleteStrategy`

**Violates:** Don't Repeat Yourself, Connascence, Code For The Maintainer, Minimize Coupling

The `OnDeleteStrategy` enum is defined independently in the runtime crate and in
`derive-input`, and both definitions carry a comment acknowledging the copy-paste
(`// Don't forget to copy + paste this enum into derive_input::api::dsl::foreign_key if you
change it` and `// This enum is copy+paste of the enum in the SpacetimeDSL crate`). A comment
is the only thing keeping them in sync, which is the weakest possible enforcement of a
contract that spans a crate boundary.

The two copies have already drifted:

- The doc comments differ. The runtime copy says a deletion "fails with a Reference
  Integrity Violation Error"; the `derive-input` copy says only that it "fails".
- The derives differ. `derive-input` additionally derives `PartialOrd`, `Ord` and
  `strum::EnumIter`, the last of which `for_foreign_key` depends on to emit match arms in
  declaration order.

On top of that, the variant *names* are spelled again as string literals in
`OnDeleteStrategy::try_parse`, so adding a variant means editing the enum twice and the
parser once, with nothing failing to compile if a step is missed.

Recommendation: this is duplication where the two copies have drifted and carry invariants
(the derive set, the doc wording, the `EnumIter` ordering dependency). The developers should
first work out which differences are deliberate and which are accidental, so the merge
preserves the behaviour that is actually relied on. Several resolutions are available and
the choice has consequences the developers are better placed to weigh:

- Have `derive-input` depend on the runtime crate and re-export the single enum. Simplest,
  but it makes the input crate depend on the runtime crate, which may be undesirable for a
  crate whose job is to describe macro input.
- Have the runtime crate depend on `derive-input` and re-export from there. Inverts the same
  trade-off.
- Extract the enum into a third, dependency-free crate that both depend on. Most decoupled,
  at the cost of another published crate.

Whichever is chosen, replace the string-literal parser with a derived
`std::str::FromStr`/`strum::EnumString` so a new variant cannot be added without the parser
learning about it.

### `derive-input/src/internal/dsl/method/reference_integrity.rs` + `src/error.rs`: `Action` and `OneOrMultiple`

**Violates:** Don't Repeat Yourself, Connascence, Single Responsibility Principle

`Action` (`Create`/`Get`/`Update`/`Delete`) exists in the runtime crate's `error` module and
again, independently, in `reference_integrity.rs`. `OneOrMultiple` exists in the runtime
crate's `error` module and again in `internal/dsl/one_or_multiple.rs`. In each pair the
generator's copy exists only to be rendered into the runtime crate's copy by name, via
`format_ident!("{action}")` and a `ToTokens` impl. The relationship is Connascence of Name
across a crate boundary, enforced by nothing.

Recommendation: same decision as `OnDeleteStrategy` above, and ideally resolved in the same
pass so the three shared vocabularies end up in the same place. Note that the two `Action`
copies are not equivalent in intent — the generator's copy is a code-generation selector
while the runtime copy is an error payload — so merging them may be the wrong call. If the
developers decide they are genuinely separate concepts, the alternative is to keep them
apart but make the rendering explicit and total (a single `fn runtime_action_variant(...)`
in `api::runtime`, matched exhaustively) so a new variant on either side forces a
compile error.

### `derive-input`: generated identifier names built outside `naming.rs`

**Violates:** Don't Repeat Yourself, Connascence, Hide Implementation Details

`internal/dsl/method/naming.rs` exists precisely to be the one place that builds identifiers
two generated modules have to agree on, and its module documentation says so. The rule is
not applied consistently:

- The column getter name `get_<column>` is built by `get_getter_method_name` in `getter.rs`,
  which is used only inside its own module. Other modules — `reference_integrity.rs` and
  `update.rs` — rebuild the same name with an inline `format_ident!`.
- The DSL lookup method name `get_<table>_by_<column>` is built in `get.rs`, which defines
  it, and rebuilt inline in `reference_integrity.rs`, which calls it.
- The `Create<Table>` argument struct name is built in `create.rs`, which defines it, and
  rebuilt inline in `hook.rs`, which references it. The code already flags this itself:
  `// FIXME: Single Source of Truth Violation for arg type name`.
- `field_name_for_found_value` is resolved in `MethodGenerationContext::new`, whose doc
  comment states that the rule "has one place that states it" — and is then rebuilt with
  the same `format_ident!` expression in `reference_integrity.rs`, in
  `multi_column_index_checks` and in `unique_multi_column_index_check`.

Recommendation: move every generated-identifier rule into `naming.rs` (or a sibling module
for the intra-table names, if mixing them with the cross-table names would blur the module's
stated purpose), and have every site call it. Where a name is already available on
`MethodGenerationContext`, read it from there instead of rebuilding it, and delete the
rebuilt expressions so the context's doc comment becomes true. This is the "Sync related
artifacts whenever knowledge changes" and "One authoritative source for each business rule"
items from the Duplication Control checklist.

### `derive-input`: types and visibilities compared by rendering them to strings

**Violates:** Connascence, Prefer clarity over cleverness, Robustness, Code For The Maintainer

A recurring idiom across the generator is to answer a question about a syntax node by
rendering it with `to_token_stream().to_string()` and comparing the result to a literal:

- `internal/dsl/table.rs` recognises timestamp columns with `field_type.eq("spacetimedb ::
  Timestamp")` and `field_type.eq("Option < spacetimedb :: Timestamp >")`.
- `internal/integration.rs` recognises the table attribute with `.eq("spacetimedb ::
  table")`, and `derive/src/lib.rs` and `derive/src/characterization_tests.rs` each
  recognise the DSL attribute with `== "spacetimedsl :: dsl"`.
- `internal/dsl/method/foreign_key.rs` compares two foreign key types, and two foreign key
  paths, by rendering both sides.
- `internal/dsl/method/update.rs` and `reference_integrity.rs` ask "is this column private?"
  with `rust_field_visibility.to_string().ne(&RustVisibility::Private.to_string())`.
- `internal/dsl/foreign_key.rs` asks the same question a third way, by rendering both
  `field.vis` and `syn::Visibility::Inherited` to strings and comparing those.
- `internal/db/column.rs` compares two `Ident`s with `c.to_string().eq(&column_name
  .to_string())` in one place and with `column_name.eq(primary_key_column_name)` in another.

Every one of these couples correct behaviour to `proc_macro2`'s spacing conventions — the
`" :: "` and `" < "` spellings are not part of any contract. They are also inconsistent with
each other and with the codebase's own better answers: `ColumnTypeKind::of` already
classifies a column type structurally from its `Path`, handling bare and `std`/`core`/
`alloc`-rooted spellings, and `mut_getter.rs` and `setter.rs` already ask about visibility
with `if let RustVisibility::Private = …`.

Recommendation: add `PartialEq` (and `Eq`) to `RustVisibility` — its absence is the direct
cause of the string comparisons — and replace every visibility check with a pattern match or
`==`. Extend `ColumnTypeKind` (or add a sibling classifier in the same module) to cover
`Timestamp` and `Option<Timestamp>`, and route the timestamp validation in
`internal/dsl/table.rs` through it. Compare `Ident`s and `Path`s with their own `PartialEq`.
For the attribute paths, extract a single `fn is_dsl_attribute(attr) -> bool` and a single
`fn is_table_attribute(attr) -> bool` used by the derive crate, the tests and
`integration.rs` alike, matching on `Path` segments rather than on a rendered string.

### `derive-input` and `src`: no unit tests

**Violates:** F.I.R.S.T Principles of Testing, Testing & Verification, Arrange/Act/Assert

The only automated coverage in the workspace is the `insta` snapshots in `spacetimedsl_derive`,
the `trybuild` compile-fail cases in `spacetimedsl-compile-tests`, and the `tester` reducer in
`examples/test`, which needs a running SpacetimeDB instance. Neither `derive-input` — which
holds the great majority of the parsing and generation logic — nor the runtime crate has a
single `#[test]`.

The snapshots do exercise `derive-input`, but only end to end: a snapshot failure says "the
generated output changed", not "this rule is wrong". Meanwhile a set of small, pure,
trivially testable functions has no direct coverage at all, among them `ColumnTypeKind::of`,
`plural_to_singular`, `is_plural_match`, `remove_trailing_digits`,
`deterministic_selection_by_name`, `documentation_on_columns`, `column_names_and_row_values`,
`rm_rsharp`, `RustVisibility`'s `Display` impl, `DeletionResultEntry::to_csv` and the
`Display` impl of `SpacetimeDSLError`.

Recommendation: add `#[cfg(test)] mod tests` blocks next to these functions, following the
Arrange/Act/Assert shape the checklist asks for. This matters before, not after, the
refactorings recommended elsewhere in this report — the Testing checklist says "Add tests
before refactoring or when missing", and the duplication findings below cannot be merged
safely without them. The runtime crate's `Display` impls and `to_csv` are the highest-value
targets because they produce user-facing output that nothing currently pins.

### `.github/workflows/test.yml` + `x.sh`: the quality gates do not fail

**Violates:** Testing & Verification, Make assertions binary without manual inspection, Robustness

Two gates report success regardless of outcome:

- CI runs clippy as `cargo clippy --all-targets --all-features || echo "Clippy warnings in
  derive-input"` (and the same for the other crates). The `|| echo` swallows every failure,
  so clippy cannot fail the build. The workspace happens to be clippy-clean today, which
  means this is currently free to fix.
- `x.sh` does not set `set -e`. Its `test` case publishes the example module, calls the
  `tester` reducer, prints logs and deletes the module; the script's exit status is that of
  the final `spacetime delete`. A failing publish or a failing `tester` call therefore cannot
  fail `./x.sh test`, and so cannot fail the CI step that invokes it. The integration suite
  is effectively advisory.

CI also invokes clippy per directory (`cd derive-input && cargo clippy …`), while `gen-x.sh`
carries a comment explaining that this specific approach was abandoned in the `format` case
because it left `derive-input` unlinted. The knowledge was updated in one artifact and not
the other.

Recommendation: drop the `|| echo` guards and let clippy fail the job, adding `-D warnings`
if the intent is to gate on warnings too. Add `set -euo pipefail` to `x.sh` — and the
equivalent `$ErrorActionPreference` / explicit `$LASTEXITCODE` checks to `x.ps1` — by
changing `gen-x.sh`, never the generated files. Replace CI's per-directory clippy invocation
with the single workspace-wide invocation `gen-x.sh` already documents as correct, ideally by
having CI call `./x.sh` for it so there is one authoritative spelling. This is the Duplication
Control checklist's "Sync related artifacts — code, docs, tests — whenever knowledge changes".

### Workspace: untracked technical debt markers

**Violates:** Future ideas captured outside codebase, Log blockers to future cleanups, Documentation & Communication Clarity

Most `TODO`/`FIXME` markers in the codebase link to a tracking issue, which is exactly what
the Scope & Goal Discipline checklist asks for. A minority do not, and those are the ones
that will be lost: the malformed-output improvement in `derive/src/output.rs`, the
import-narrowing note in `derive/src/output/function.rs`, the unique-columns and clone notes
in `create.rs`, the row-value-getter note in `get.rs`, the single-source-of-truth note in
`hook.rs`, the wrapper doc-comment note in `wrapper.rs`, and the feature checklist embedded
in `examples/blackholio/src/lib.rs`.

Recommendation: file an issue for each and replace the marker with a link, or delete the
marker if the idea is no longer wanted. The Scope & Goal Discipline checklist is explicit
that future ideas belong outside the codebase.

---

## `src/lib.rs`

The runtime crate's root: the context traits, the error helper, and the `spacetimedsl!`
macro that emits the per-crate DSL module. The macro is the crate's largest single item and
the contract every generated method compiles against.

### `src/lib.rs`: `ContextType::Transaction`

**Violates:** Unused scaffolding removed immediately, Optimize for Deletion, YAGNI

`ContextType` declares a `Transaction` variant and `get_err` renders it as `"Transaction"`,
but nothing in the workspace ever constructs it. `TxContext`, the context it presumably
describes, is mapped to `ContextType::Reducer` everywhere it appears — including in
`get_immutable_database.rs`, where the error message a user of a `TxContext` receives will
claim they tried to access from a "Reducer Context".

Recommendation: this is dead code, and the developers must decide for themselves whether the
variant is no longer needed or whether the fact that it isn't being used is actually a bug —
that is, whether `TxContext` was meant to report itself as a Transaction context and the
wiring was simply never done. If it is genuinely unused, delete it in the same pass as its
match arm, per the Lifecycle & Deletion checklist. If it was meant to be used, the fix is to
pass `ContextType::Transaction` from the `TxContext` error impls, which would also make the
diagnostics honest.

### `src/lib.rs`: `spacetimedsl!`

**Violates:** Don't Repeat Yourself, Curly's Law, Hide Implementation Details

The macro body re-exports the same runtime items twice: once flat at the module level ("so
that `spacetimedsl::X` paths work without needing the sub-module prefix") and again inside
`prelude`. `Context`, `ReadContext`, `WriteContext`, `Wrapper`, the `delete` items and the
`error` items each appear in both lists, and a new runtime export has to be added to both or
it will work through one path and not the other.

The macro also does several jobs at once: it declares `DSL` and `ReadOnlyDSL` with their
constructors and accessors, declares the `DSLMethodHooks` and `DSLInternals` carrier types,
and assembles the two re-export surfaces.

Recommendation: define the prelude once and have the flat surface re-export it (`pub use
prelude::*;`), or vice versa, so a runtime item is named in exactly one list. Consider
splitting the macro body into smaller macros — one per concern — invoked from the top-level
macro, which would let each part be read and changed on its own. Note that macro-emitted
code is awkward to factor and the current form is at least contiguous and readable; if the
developers judge that splitting it would hurt more than help, the de-duplication of the two
export lists is the part worth doing regardless.

---

## `src/get_*.rs` and `src/as_*.rs`

The context accessor family. Each file declares one trait, defines a macro pair that
implements it as either an error or a delegation, and applies that pair to the four
SpacetimeDB context types.

### `src/get_*.rs`, `src/as_*.rs`: the `impl_*_err!` / `impl_*_ok!` macro pairs

**Violates:** Don't Repeat Yourself, Open/Closed, Encapsulate What Changes, Boy Scout Rule

Every file in this family repeats the same structure: a trait with one method, an
`impl_<name>_err!` macro that emits an impl returning `Err(crate::get_err(<message>,
ContextType::$variant))`, an `impl_<name>_ok!` macro that emits an impl returning
`Ok(<delegation>)`, and then one macro invocation per context type. Only four things vary
between files — the trait name, the method signature, the error message, and the delegation
expression — yet the scaffolding around them is written out afresh each time.

The consequences are already visible. Adding a fifth SpacetimeDB context type means editing
every file in the family. And the pattern has drifted: `get_sender.rs` and
`as_anonymous_view_context.rs` each hand-write one impl because the delegation differs
slightly from what their own `_ok!` macro emits, so those files say the same thing two ways,
side by side.

Recommendation: collapse the family behind one macro that takes the trait name, the method
signature, the message and a per-context delegation, so adding a context type is one edit and
adding an accessor is one invocation. If the signatures vary too much for a single macro to
stay readable — `GetRandom` is generic, and the delegations are genuinely heterogeneous —
an acceptable middle ground is to keep one file per accessor but share a single
`impl_context_accessor!` macro from a common module, so at least the error half stops being
copied. The developers should judge which of the two reads better; the current arrangement,
where the scaffolding is copied and some files then break from it, is the one option that is
clearly worst.

### `src/as_reducer_context.rs`, `src/as_view_context.rs`, `src/as_anonymous_view_context.rs`: marker trait impls

**Violates:** Maximize Cohesion, Separation of Concerns, Principle of Least Astonishment

The `Context`, `ReadContext` and `WriteContext` impls for the SpacetimeDB context types are
spread across the `as_*.rs` files — `ReducerContext` and `TxContext` get theirs in
`as_reducer_context.rs`, `ViewContext` in `as_view_context.rs`, `AnonymousViewContext` in
`as_anonymous_view_context.rs`. A maintainer asking "which contexts can write?" has to read
several files that are named after something else entirely. The traits themselves are
declared in `lib.rs`.

Recommendation: move all of these impls next to the trait declarations in `lib.rs`, or into a
dedicated module named for what they establish. The Cohesion checklist's "Group related
operations so components stay reusable" and "Localize each change request to one module"
both point the same way: which contexts implement which capability is one fact and belongs
in one place.

### `src/get_random_number_generator.rs`: `impl_get_rng_ok!`

**Violates:** Prefer clarity over cleverness, Principle of Least Astonishment

The macro emits `Ok(&self.rng())`, taking a reference to the result of a call that already
returns a reference. It compiles only because of auto-deref, and reads as if it were
returning a reference to a temporary. Every sibling in the family writes `Ok(&self.db)` or
`Ok(self.sender_auth())` — this is the odd one out.

Recommendation: write `Ok(self.rng())`. A small clarity fix in a file that will be touched
anyway when the macro family is consolidated, which is exactly what the Boy Scout Rule's
"Fix small clarity issues during adjacent work" asks for.

---

## `src/error.rs`

The runtime error type and its rendering. Everything a user of the DSL sees when an
operation fails passes through here, and none of it is covered by a test.

### `src/error.rs`: `impl Display for SpacetimeDSLError`

**Violates:** Single Responsibility Principle, Curly's Law, Robustness Principle, Keep It Simple

One `fmt` method renders every variant of `SpacetimeDSLError`, including the nested variants
of `ReferenceIntegrityViolationError`, in a single deeply nested `match` whose arms build
strings out of conditionally-assembled fragments. It carries several distinct
responsibilities — phrasing the not-found message, phrasing the unique-constraint message and
its SpacetimeDB-versus-SpacetimeDSL split, phrasing the auto-inc message, and phrasing both
reference-integrity messages — and any change to one risks the others.

Worse, one arm contains a `panic!`. When a `ReferenceIntegrityViolationError::OnCreateOrUpdate`
carries `Action::Get` or `Action::Delete`, formatting the error panics. A `Display` impl is
invoked by `to_string()`, by `format!`, by `{}` in a log line and by the `Error` trait — it
is the last place that should be able to abort. The panic exists because the type permits a
state the code considers illegal.

The method also builds its output into a `let mut message: String` via `push_str` of a
single `match` expression, then writes that string out — an indirection with no purpose,
since the match result could be written directly.

Recommendation: give each variant its own rendering function (or its own `Display` impl on
a smaller type) and reduce `fmt` to a dispatch, per "Split methods/classes until each has
one change driver". Remove the panic by making the illegal state unrepresentable: give
`ReferenceIntegrityViolationError::OnCreateOrUpdate` a dedicated two-variant enum
(`Create`/`Update`) instead of the four-variant `Action`, so the compiler rejects the case
the panic guards against. If the developers prefer to keep `Action` shared, the fallback is
to render the unexpected variants as a plain message rather than panicking — but the type
change is the resolution the Robustness and Composition checklists point to. Replace the
`message` accumulator with a direct `write!`.

### `src/error.rs`: `impl Display for OnDeleteStrategy`

**Violates:** Separation of Concerns, Maximize Cohesion

`OnDeleteStrategy` is declared in `delete.rs`; its `Display` impl lives in `error.rs`. The
error module is not where a reader looks for how a deletion strategy renders itself, and the
impl has nothing to do with errors.

Recommendation: move the impl next to the type in `delete.rs`. If the intent was that all
user-facing text lives in `error.rs`, that intent is not held anywhere else — `delete.rs`
already contains the `Display` impl for `DeletionResult` — so consistency argues for the
move.

---

## `src/delete.rs`

The deletion result model: what a cascade reports back, and how it renders.

### `src/delete.rs`: `DeletionResultEntry::to_csv`

**Violates:** Hide Implementation Details, Command Query Separation, Keep It Simple, Principle of Least Astonishment

A public method named `to_csv` takes three parameters — an entry counter, a parent counter
and a `String` accumulator — and returns a tuple of the updated counter and the updated
string. The recursion's internal bookkeeping is part of the public signature, so a caller
outside the crate can see and must understand it in order to call the method at all. The
name promises a conversion and the signature delivers a fold step.

Its only caller is `DeletionResult::to_csv`, which threads the same accumulators through a
loop with `(entry_id, message) = entry.to_csv(entry_id, parent_entry_id, message)`. The
parameters are declared `mut` and reassigned, which makes the flow harder to follow than a
plain recursive descent would be. The counters are `u128`, far wider than any plausible
number of cascade entries.

Recommendation: make the recursive helper private and give it a name that says what it does
(`append_csv_rows`, say), taking `&mut String` and a counter by `&mut` rather than
threading values through a return tuple. Keep a public `to_csv(&self) -> String` on
`DeletionResult` as the only published entry point, per "Exclude private implementation
details from public interfaces" and "Minimize class and member accessibility". Consider
narrowing the counters to `u32` or `usize` unless there is a reason for `u128` that is not
apparent from the code. Note that `to_csv` is part of the published crate API, so removing
the public recursive form is a breaking change — the developers should decide whether to
take it now or stage it behind a deprecation.

### `src/delete.rs`: `OnDeleteStrategyFailure<Entries>`

**Violates:** Documentation & Communication Clarity, Depend on contracts/interfaces, Principle of Least Astonishment

The `Entries` type parameter is completely unconstrained, and its actual contract is stated
only in prose: "`Entries` is a `Vec<DeletionResultEntry>` when one row of the referenced
table was deleted and a `HashMap<&PrimaryKeyValue, Vec<DeletionResultEntry>>` when several
were." Nothing stops a caller instantiating it with anything at all, and nothing tells a
reader of the struct which of the two shapes they are holding.

Recommendation: either constrain the parameter with a sealed trait implemented by exactly the
two shapes, which turns the prose into a compiler-checked contract, or replace the generic
with a two-variant enum if the set really is closed. A sealed trait is the better fit for the
Dependency & Interface Management checklist's "Define interfaces to represent dependencies",
but it adds a type that only exists to express a constraint; the developers should weigh that
against how likely a third shape is.

---

## `derive/src/lib.rs`

The procedural macro entry points: `#[dsl]`, the `SpacetimeDSL` helper derive, and `#[hook]`.

### `derive/src/lib.rs`: `is_last_dsl_attribute`

**Violates:** Unused scaffolding removed immediately, Optimize for Deletion, YAGNI

`expand_dsl_attribute_parts` calls `is_last_dsl_attribute` and discards the result into
`let _is_last_dsl_attribute`. The function it calls is fully implemented and documented; the
only consumer of its answer is the commented-out `make_struct_fields_private` block directly
below, disabled with `// TODO: Temporarily disabled to allow public primary key columns`.

Recommendation: this is dead code, and the developers must decide for themselves whether it
is no longer needed or whether the fact that it isn't being used is actually a bug — here,
whether making non-primary-key fields private on the final `#[dsl]` pass is still wanted
behaviour that regressed, or a design that was abandoned. If abandoned, delete the call, the
function and the commented-out block together, per "Delete code, tests, and configuration in
the same pass". If still wanted, the TODO needs to become an issue and the discarded binding
needs to become a real use.

### `derive/src/lib.rs`: `make_struct_fields_private`

**Violates:** Unused scaffolding removed immediately, Optimize for Deletion, Documentation & Communication Clarity

A complete function body, along with its doc comment, sits commented out. Commented-out code
is invisible to the compiler, to clippy, to `cargo fmt` and to every refactoring tool, so it
rots silently while looking like it could be switched back on.

Recommendation: as above, this is dead code whose fate the developers must decide. If it is
wanted, git history is the right place to keep it while the TODO is tracked as an issue — the
Refactoring checklist's "Unused scaffolding removed immediately" does not make an exception
for code that has been commented instead of deleted.

### `derive/src/lib.rs`: `expand_dsl_attribute_parts` singleton detection

**Violates:** Don't Repeat Yourself, Connascence, Separation of Concerns

The function scans the raw argument tokens for an identifier equal to `"singleton"` in order
to decide whether to inject a primary key. `Table::try_parse` — called a few lines later on
the same arguments — parses the same keyword properly through `match_meta!`. The macro crate
therefore has its own ad-hoc parser for one argument of a grammar that the input crate owns,
and the two will disagree if the grammar ever changes (for example if `singleton` gains
arguments, or if the word appears in a nested position the token scan does not distinguish).

Recommendation: have the singleton decision come from `derive-input`, which owns the grammar
— either by exposing a small `fn is_singleton(args) -> syn::Result<bool>` from the input
crate, or by restructuring the expansion so parsing happens before injection. The second is
cleaner but more invasive, because the injection currently has to happen before `try_parse`
sees the struct; the developers should decide which is worth the change radius. Either way,
the Modular Boundaries checklist's "Shared utilities expose APIs, not internal details"
argues against the derive crate re-deriving what the input crate already knows.

### `derive/src/lib.rs`: `derive_table_helper_attr`

**Violates:** Law of Demeter, Robustness, Prefer clarity over cleverness

The function builds an attribute by parsing a `quote!` literal and then reaching through the
result: `.unwrap().into_iter().next().unwrap()`. Two `unwrap`s on a value the function itself
constructed, and a chain through intermediate collaborators, to extract the single attribute
it knows is there.

Recommendation: use `syn::parse_quote!` to build the `syn::Attribute` directly, which removes
both the chain and the panics. This also removes the need for the reader to verify that the
quoted source really does contain exactly one attribute.

### `derive/src/lib.rs`: redundant comments

**Violates:** No redundant comments, Remove comments that repeat the code

Several comments restate their statement: `// Parse the input tokens into a syntax tree`
above a `syn::parse2` call, `// Build the output, possibly using quasi-quotation` above a
call to `output::build`, `// Create the field: #[primary_key] id: u8` above the construction
of exactly that field, and `// Insert as the first field` above `fields.named.insert(0, …)`.

Recommendation: delete them. AGENTS is explicit that a comment repeating the code adds no
value; the comments worth keeping in this file are the ones that explain *why*, such as the
note about `#[dsl]` attributes stripping themselves.

---

## `derive/src/output.rs`

Assembles everything the macro emits: wrapper types, accessors, compile-error checks, the
create-argument struct, the hook traits and the DSL methods.

### `derive/src/output.rs`: `build`

**Violates:** Single Responsibility Principle, Curly's Law, Maximize Cohesion

`build` collects wrapper types, dispatches the table-level DSL methods, dispatches the
on-delete cascade methods for both directions, dispatches the multi-column index methods,
walks the columns to collect accessors and per-column methods, assembles the compile-error
check traits, builds the create-argument struct and builds the hook traits. Each of those is
a separate reason for the function to change.

Recommendation: extract one function per group — `build_wrapper_types`,
`build_table_methods`, `build_cascade_methods`, `build_accessors`,
`build_compile_error_checks` — and reduce `build` to composing their results into
`GeneratedOutput`. The Cohesion checklist's "Split methods/classes until each has one change
driver" is the direct instruction; the existing `// Two loops, not one: …` comment shows the
function is already accumulating explanations that a named function would carry in its name.

### `derive/src/output.rs`: `malformed_code_generation_result`

**Violates:** Minimize class and member accessibility, Keep It Simple, Avoid Premature Optimization

The function is declared `pub` while everything around it in the same module is
`pub(crate)`; since `mod output` is private, the wider visibility buys nothing and only
misleads a reader into thinking it is part of an external surface.

Its whitespace-collapsing loop repeatedly calls `result.replace("  ", " ")` until no double
space remains, which rescans and reallocates the whole string on each pass. The input is a
malformed token stream, so it can be large; the straightforward single pass
(`split_whitespace().join(" ")`) is both simpler and linear.

Recommendation: narrow the visibility to `pub(crate)` per "Minimize class and member
accessibility", and replace the loop with the single-pass form. The simplification is not a
performance optimisation looking for a profile — the linear version is the more
straightforward code, which is what the Simplicity checklist asks for first.

### `derive/src/output.rs`: `get_column_dsl_methods` and `map_args`

**Violates:** Command Query Separation, Name methods to signal query versus command, Prefer stable abstractions

`get_column_dsl_methods` is named as a query but constructs new `GeneratedDSLMethod` values;
the sibling functions doing the same thing are named `build_public_dsl_method` and
`build_internal_dsl_method`. `map_args` takes `&Vec<SpacetimeDSLArg>` where `&[SpacetimeDSLArg]`
would do, needlessly constraining every caller to own a `Vec`.

Recommendation: rename `get_column_dsl_methods` to `build_column_dsl_methods` for consistency
with its siblings, per "Name methods to signal query versus command behavior". Change
`map_args` to take a slice.

---

## `derive/src/output/function.rs`

Renders one DSL method into its `impl` block, once per context bound it is available under.

### `derive/src/output/function.rs`: `MethodImplVariant::internal` and `render_impl`

**Violates:** Unused scaffolding removed immediately, Principle of Least Astonishment, Optimize for Deletion

`MethodImplVariant` carries a `context_bound` field, and `build_internal` constructs one with
`MethodImplVariant::internal(runtime::write_context())`. In `render_impl`, the
`InternalDslInternals` branch discards it — `let _context_bound = &variant.context_bound;` —
and then calls `runtime::write_context()` again for itself. The field is populated, threaded
through and thrown away.

Recommendation: this is dead code, and the developers must decide for themselves whether it
is no longer needed or whether the fact that it isn't being used is actually a bug — the
plausible bug reading is that an internal method was meant to be able to carry a bound other
than `WriteContext` and the branch simply never learned to read it. If the field is genuinely
unneeded on this variant, restructure `MethodImplVariant` so only the associated variant
carries a bound, which makes the discard impossible rather than merely absent.

### `derive/src/output/function.rs`, `hook.rs`, `create_method_arg.rs`: `syn::Result` returns that never fail

**Violates:** YAGNI, Principle of Least Astonishment, Remove nonessential requirements

`build_public`, `build_internal`, `build_with_config`, `hook::build` and
`create_method_arg::build` all return `syn::Result<TokenStream>` and none of them can produce
an `Err`. Every caller writes a `?` for an error that cannot occur, and a reader has to
verify that for themselves before they can reason about the control flow.

Recommendation: change the signatures to return `TokenStream` directly and drop the `?`s. The
Scope & Goal Discipline checklist's "Remove nonessential requirements, defer speculative
work" applies: the error channel can be reintroduced the day a function actually needs it,
and the compiler will point at every call site.

### `derive/src/output/function.rs`: `render_impl` generated attributes

**Violates:** Robustness & Reliability, Documentation & Communication Clarity

Every generated associated method is emitted with `#[allow(clippy::needless_lifetimes,
clippy::too_many_arguments)]` unconditionally. Suppressing a lint on all generated output
means a genuinely over-wide generated signature — which is a real usability problem for the
user of the DSL — can never surface. The `// FIXME: We should probably only import one of
CtxDbRead or CtxDbWrite per method implementation.` immediately above is a related admission
that the emitted prelude is broader than it needs to be.

Recommendation: emit each `allow` only on the methods that need it — `needless_lifetimes`
only where a lifetime is actually elidable, `too_many_arguments` only where the argument
count exceeds the threshold — so that the suppression documents a specific, justified
exception rather than a blanket one. The Documentation checklist's "Document justified
exceptions" and "Explain why added complexity was unavoidable" both point at making the
suppression conditional and explained. If narrowing proves impractical, at minimum add a
comment stating which generated shapes need it and why.

---

## `derive/src/output/doc_comment.rs`

Builds the `Implementation:` section appended to every generated method's doc comment.

### `derive/src/output/doc_comment.rs`: `implementation_section`

**Violates:** Orthogonality, Separation of Concerns, Robustness Principle

The function contains `if cfg!(test) { return String::default(); }`. Production behaviour is
conditioned on whether the crate is being tested, so the code path the snapshots exercise is
not the code path users get. The comment explains the motivation honestly — the snapshots
would otherwise duplicate every method body — but the mechanism puts a test concern inside a
production function, which is precisely what the Modular Boundaries checklist's "Keep
unrelated responsibilities decoupled across modules" warns against.

The same function reacts to a formatting failure with `panic!`. Inside a procedural macro
that means an ICE-style abort rather than a diagnostic pointing at the user's code, even
though `syn::Error` is available and every caller already sits under a `syn::Result`.

Recommendation: move the test-versus-production decision out of the function and into its
caller — for example by having `build_with_config` take a flag, or by having the test harness
pass a no-op doc-comment renderer. Either resolution restores the property that the snapshots
test the shipping code path. Replace the `panic!` with a propagated `syn::Error` so a
malformed emission becomes a compile error attached to the offending table; the text
`malformed_code_generation_result` already produces makes a perfectly good error message.
The developers should note that the second change ripples into the `syn::Result` signatures
discussed above — those functions would then genuinely need their error channel, which is
worth knowing before removing it.

---

## `derive/src/output/accessor.rs`

Generates the getter, mut-getter and setter methods on the user's struct.

### `derive/src/output/accessor.rs`: `Accessor::definition`

**Violates:** Connascence, Prefer clarity over cleverness, Robustness

Visibility is recovered with `parse_str(&mut_getter.method_visibility.to_string())?` — the
`RustVisibility` is rendered to a string by its `Display` impl and re-parsed as a
`syn::Visibility`. This couples `accessor.rs` to the exact spelling `RustVisibility`'s
`Display` produces, including the space in `pub (crate)`, and it is a fallible round trip
through text for data that was a `syn::Visibility` to begin with.

`definition` also returns `syn::Result` while the `Getter` arm can never fail, so the
fallibility is an artefact of this round trip.

Recommendation: keep the original `syn::Visibility` (or store a token stream) on
`RustVisibility` so it can be spliced directly, and delete the round trip. With it goes the
`syn::Result`. This is the same class of problem as the string comparisons noted in the
workspace-wide section and is best fixed in the same pass.

---

## `derive/src/output/hook.rs`

### `derive/src/output/hook.rs`: `build`

**Violates:** Prefer clarity over cleverness, Principle of Least Astonishment

The function opens with `if hook.is_none() { return Ok(TokenStream::default()); }` followed
by `let hook = hook.as_ref().unwrap();`. The `unwrap` is safe only because of the check three
lines above, which the compiler does not verify and a future edit could separate.

Recommendation: use `let Some(hook) = hook else { return TokenStream::default(); };`, which
states the same thing without a panic path. The rest of the codebase already uses this form
in `characterization_tests.rs` and `ColumnTypeKind::of`.

---

## `derive/src/characterization_tests.rs`

The snapshot suite. The strongest safety net the project has, and the reason the refactorings
recommended in this report are feasible at all.

### `derive/src/characterization_tests.rs`: the per-fixture `#[test]` functions

**Violates:** Don't Repeat Yourself, Testing & Verification

Every fixture has a hand-written test whose body is `snapshot_fixture("<its own name>")`, so
the function name and the string literal repeat each other. More importantly the list is
maintained by hand: a fixture file added to `tests/fixtures` without a matching `#[test]` is
silently never run, and the suite will still report green.

Recommendation: either generate the test functions with a small declarative macro
(`snapshot_fixtures! { plain_table, singleton, … }`), which removes the name/string
repetition but keeps the manual list, or drive the suite from the fixture directory so a new
file is picked up automatically. The second closes the coverage gap and is what the Testing
checklist's "Reference user-impacting behaviors with guardrails or tests" argues for; the
cost is that a single test failure no longer names the fixture in the test name, which some
CI reporting relies on. A third option keeps both properties: generate the list at build time
from the directory. The developers should pick based on how much they value per-fixture test
names.

### `derive/src/characterization_tests.rs`: `is_dsl_attribute`

**Violates:** Don't Repeat Yourself, Connascence

`is_dsl_attribute` here and `is_last_dsl_attribute` in `derive/src/lib.rs` both decide
whether an attribute is a `#[dsl]` attribute, and both do it with `path == "dsl" || path ==
"spacetimedsl :: dsl"`. The test file and the production file now have to agree on a magic
string.

Recommendation: covered by the workspace-wide recommendation to extract a single attribute
predicate. Doing it here has the extra benefit that the tests would then exercise the same
predicate production uses.

---

## `derive-input/src/internal.rs`

Parses the `#[dsl(...)]` arguments and hands the result to the table parser.

### `derive-input/src/internal.rs`: `try_parse_dsl`

**Violates:** Single Responsibility Principle, Curly's Law, Keep It Simple, Don't Repeat Yourself

One function declares a long run of mutable option locals, runs a `parse2` whose callback
nests `parse_nested_meta` closures several levels deep to reach the individual hook flags,
then validates the parsed result, then assembles `DSLData`. Parsing the grammar, validating
the combination of options, and building the result are three reasons for this function to
change.

Inside the validation, four error branches are near-identical: hook-before-update,
hook-after-update, hook-before-delete and hook-after-delete each check the same shape and
produce the same sentence with two words swapped.

The function's doc comment — `// Parse plural_name from DSL arguments` — describes a small
fraction of what it does, which is what happens when a function outgrows its name.

Recommendation: split it. A `parse_hook_arguments` function for the nested hook grammar, a
`parse_method_arguments` function for the `method(...)` grammar, and a
`validate_hooks_against_methods` function for the cross-checks, leaving `try_parse_dsl` to
orchestrate. Collapse the four validation branches into one loop over
`(hook_span, hook_name, method_name, method_enabled)` tuples, per "Abstract repeated logic
instead of copy/paste fixes". Correct or delete the stale comment.

### `derive-input/src/internal.rs`: `DSLData`

**Violates:** Maximize Cohesion, Don't Repeat Yourself, Connascence

`DSLData` flattens the six hook flags into six sibling `bool` fields. The same six flags are
then passed as six positional `bool` arguments to `dsl::hook::build`, and land in the six
fields of `SpacetimeDSLMethodHooks`. The same bundle is spelled several different ways, and the
positional call in the middle is one transposition away from silently wiring
`before_insert` to `after_delete` — a mistake the compiler cannot catch.

Recommendation: introduce one small struct for the hook flags (`RequestedHooks`, with
`before_insert`, `after_delete` and so on) and pass that value through, replacing the
positional argument list. This removes the Connascence of Position entirely and makes the
those spellings one. The same treatment suits `update_method`/`delete_method`, which travel
together as `Option<bool>` and are unwrapped with `.unwrap_or(true)` at several separate
sites.

### `derive-input/src/internal.rs`: the `__singleton_placeholder` identifier

**Violates:** Principle of Least Astonishment, Connascence, Avoid hidden coupling or magic

For a singleton table, `try_parse_dsl` sets `plural_name` to a synthetic
`__singleton_placeholder` identifier, and `try_parse` overwrites it afterwards with the name
derived from the table accessor. Between those two points `DSLData` holds a value that is
not a plural name, and nothing in the type says so. Anything that reads `plural_name` in
between gets nonsense. This is Connascence of Timing dressed up as a sentinel value.

Recommendation: make `plural_name` an `Option<Ident>` in `DSLData` and resolve it to a
non-optional `Ident` when `SpacetimeDSLTable` is built, so the "not yet known" state is
visible in the type and the placeholder disappears. If the developers prefer not to widen
the field, the alternative is a dedicated `PluralName` enum with a `Singleton` variant — more
explicit, but a larger change. Either way the current form, where a magic string stands in
for "absent", is the one that a maintainer cannot discover without reading both functions.

---

## `derive-input/src/internal/table.rs`

### `derive-input/src/internal/table.rs`: `rm_rsharp`

**Violates:** No abbreviations, Descriptive identifiers

The name abbreviates both the verb and the noun. AGENTS is explicit — "Use `InputOutput` not
`Io`, `FileSystemWatcher` not `Watcher`" — and this name is harder to decode than either
example, since `rsharp` is not a standard abbreviation for the raw-identifier prefix.

Recommendation: rename to `remove_raw_identifier_prefix`. The function is small and its
callers are few, so the change radius is shallow, which is what the Refactoring checklist
wants.

### `derive-input/src/internal/table.rs` + `column.rs`: `MethodGenerationContext` built twice

**Violates:** Don't Repeat Yourself, Avoid Premature Optimization, Connascence

`MethodGenerationContext::new` is called with the same five arguments in `column::try_parse`
and again in `table::try_parse`, a few statements later. The context's own documentation says
its purpose is that derived names "are resolved once here rather than in each generator" —
but the context itself is constructed twice per table, so every derived name is computed
twice.

Recommendation: build the context once and pass it to whichever function needs it. This is
mostly a clarity fix rather than a performance one; the value of doing it is that the
context's stated invariant ("resolved once") becomes true, which is what the Documentation
checklist's "Document intent so future maintainers understand choices" depends on.

---

## `derive-input/src/internal/column.rs`

Maps the parsed column arguments into the API column types and the generator's internal view
of them.

### `derive-input/src/internal/column.rs`: `try_parse`

**Violates:** Keep It Simple, Principle of Least Astonishment, Single Responsibility Principle

The function returns a five-element tuple and carries `#[allow(clippy::type_complexity)]` to
silence the lint that noticed. Suppressing the warning does not make the signature easier to
call; every caller has to destructure five positional values whose meaning comes only from
their order.

Inside, `SpacetimeDBColumn::map`'s tuple result is unpacked by index — `spacetimedb_table =
res.0; let spacetimedb_column = res.1;` — rather than destructured, which loses the names
entirely. The same "find the column whose name matches the primary key, and panic if absent"
search is written twice, once over `internal_columns` and once over `columns`.

Recommendation: replace the tuple with a named struct (`ParsedColumns { spacetimedb_table,
columns, primary_key_column, internal_columns, internal_primary_key_column }`) and delete the
`allow`. A suppressed complexity lint is exactly the "Explain why added complexity was
unavoidable" item going unanswered. Destructure `SpacetimeDBColumn::map`'s result by pattern.
Extract the primary-key search into one helper used by both lookups.

### `derive-input/src/internal/column.rs`: `InternalColumn`

**Violates:** Don't Repeat Yourself, Maximize Cohesion, Hide Implementation Details

`InternalColumn` is a flattened, cloned copy of selected fields from `RustField`,
`SpacetimeDBColumn` and `SpacetimeDSLColumn`, with the source struct encoded into each field
name (`rust_field_visibility`, `spacetimedb_column_is_auto_inc`,
`spacetimedsl_column_wrapper_type`). Every column's data therefore exists twice, and adding a
field that a generator needs means editing the source struct, `InternalColumn`, and the
copying code.

The field-name prefixes are a strong hint that this type exists to work around borrow
constraints rather than to model anything.

Recommendation: the developers should first establish why the flattened copy exists — if it
is a borrow-checker workaround, holding `&RustField`, `&SpacetimeDBColumn` and
`&SpacetimeDSLColumn` behind one struct with accessor methods would keep the single source of
truth while giving the generators the same convenience. If the copying is load-bearing for
lifetime reasons that references cannot satisfy, the fallback is to keep the struct but drop
the prefixes and document why the copy exists, per "Explain why added complexity was
unavoidable". Note that `ColumnTypeKind` genuinely is derived rather than copied and belongs
here either way.

---

## `derive-input/src/internal/integration.rs`

Bridges to `spacetime-bindings-macro-input` and picks which `#[table]` attribute a `#[dsl]`
attribute belongs to.

### `derive-input/src/internal/integration.rs`: `select_table_with_heuristics`

**Violates:** Robustness Principle, Principle of Least Astonishment, Keep It Simple, Accept unknown inputs only when semantics remain clear

When a struct carries more than one `#[table]` attribute, this function tries to guess which
one the `#[dsl]` attribute means. It tries an exact match on the accessor name; then
`is_plural_match`, which strips trailing digits, hard-codes the special case `"tables"` ↔
`"table"`, applies a homegrown `plural_to_singular`, and finally accepts any case where the
plural name merely *contains* the table name as a substring; and then, if all of that fails,
falls back to `deterministic_selection_by_name`, which sums the character codes of the plural
name and takes the remainder modulo the number of tables.

The fallback does not select a plausible table. It selects an arbitrary one, deterministically,
and the generated DSL then silently operates on the wrong table. No diagnostic is produced.
The Robustness checklist is explicit that unknown inputs are acceptable "only when semantics
remain clear", and that malformed input should be surfaced immediately; this does the
opposite.

The substring rule is nearly as risky: a struct with tables `item` and `item_archive` and a
plural name `items_archive` can match the wrong one.

Recommendation: replace the guessing with an explicit association and a compile error. The
cleanest form is to let `#[dsl]` name the table it belongs to (`#[dsl(table = my_table, …)]`)
and to emit a `syn::Error` when a multi-table struct does not disambiguate, listing the
candidate accessors in the message. That is a grammar change affecting existing users, so the
developers may prefer a staged path: keep the exact-name match, delete the fuzzy matching and
the hash fallback, and emit an error when the exact match fails. Either way the hash fallback
should go — there is no reading under which silently choosing a table by character sum is
correct.

`plural_to_singular`, `is_plural_match`, `remove_trailing_digits` and
`deterministic_selection_by_name` all disappear with it, which also removes the untested
pure functions noted in the workspace-wide testing finding.

### `derive-input/src/internal/integration.rs`: the unreachable empty-tables branch

**Violates:** Unused scaffolding removed immediately, Prefer clarity over cleverness

`select_table_with_heuristics` opens with a guard that returns an `Err` whose message reads
"No `#[table]`… found", but `get_all_table_attributes` — the function that just produced
`all_tables` — already returns an `Err` when it finds none. The branch cannot be reached, and
its error message differs from the one that actually fires, so a maintainer debugging a
diagnostic may look for the wrong text.

Recommendation: this is dead code, and the developers must decide for themselves whether it
is no longer needed or whether the fact that it isn't being reached is actually a bug — in
particular whether the two error messages were meant to describe different situations and one
of the checks is in the wrong place. If the branch is simply redundant, delete it.

---

## `derive-input/src/internal/db/table.rs` and `derive-input/src/internal/db/column.rs`

Map the SpacetimeDB table and column arguments into the API's database view.

### `derive-input/src/api/db/table.rs`: `SpacetimeDBTable::multi_column_indices`

**Violates:** Principle of Least Astonishment, Avoid hidden coupling or magic, Hide Implementation Details

The field is named `multi_column_indices` but, as its own comment admits, it "contains all
indices during processing for the moment, but all single column indices are removed from the
Vector after the columns are processed". Between `SpacetimeDBTable::map` and the end of
`column::try_parse` the field's name is a lie, and correctness depends on every reader
knowing which side of that transition they are on. The field is `pub` on a public API type,
so external consumers of `derive-input` see the same trap.

The removal itself is done with `swap_remove`, which reorders the remaining indices, and the
search stops at the first single-column index found per column. `internal/dsl/method.rs`
documents the consequence: "It stops at the first index per column, though, so a column
carrying two single-column indices would leak one into this list. Skip it rather than
generate it from the wrong path." The generator therefore carries a filter whose only purpose
is to work around this leak — a known defect papered over at the consumer rather than fixed
at the source.

Recommendation: separate the two lifecycle stages so the name is always true. The
straightforward form is for `SpacetimeDBTable::map` to return all indices in a local, and for
`column::try_parse` to produce the final `SpacetimeDBTable` holding only genuine multi-column
indices. Handle the multiple-single-column-index case explicitly — the developers should
decide whether a column carrying two single-column indices is legal (in which case
`SpacetimeDBColumn` needs to hold a collection) or should be rejected with a diagnostic. Once
the source is correct, the defensive filter in `method.rs` can be deleted; leaving it in place
after the fix would be exactly the "Delete code, tests, and configuration in the same pass"
omission the Lifecycle checklist warns about.

### `derive-input/src/internal/db/column.rs`: `SpacetimeDBColumn::map`

**Violates:** Prefer clarity over cleverness, Don't Repeat Yourself

Two `match` blocks over `index.index_type` sit a few statements apart, both enumerating the
single-column index variants — one to reject them on singleton tables, one to find the
column's own index. The auto-inc check compares identifiers by rendering both to strings
while the primary-key check a few lines above compares them with `Ident`'s own `eq`. The
error message for a name-prefixed primary key ends with `.unwrap_or("id")`, which quietly
substitutes a different suggestion when the strip fails, for reasons the code does not
explain.

Recommendation: extract a `fn single_column_of(index_type: &IndexType) -> Option<&Ident>` and
use it in both places. Compare identifiers with `==` throughout. Either document why
`unwrap_or("id")` is the right fallback or remove it — if the `strip_prefix` can fail here at
all, the reader deserves to know when.

---

## `derive-input/src/internal/dsl/table.rs`

Validates the table-level DSL configuration and builds `SpacetimeDSLTable`.

### `derive-input/src/internal/dsl/table.rs`: `SpacetimeDSLTable::try_parse`

**Violates:** Single Responsibility Principle, Curly's Law, Keep It Simple, Maximize Cohesion

One function applies the `unique_index` flags to the parsed indices, builds the hook
descriptors, decides and validates `has_update_method`, walks every field to collect
referencing tables, validates column visibility against the update setting, identifies and
validates the insert-timestamp column, identifies and validates the update-timestamp column,
and then performs a final cross-check. Each of those is a separate change driver, and they
are interleaved rather than sequenced, so the reader has to hold all of them at once.

Recommendation: extract one function per concern — `apply_unique_index_flags`,
`resolve_update_method`, `collect_referencing_tables`, `resolve_timestamp_columns` — leaving
`try_parse` to sequence them and assemble the result. The Cohesion checklist's "Split
methods/classes until each has one change driver" is the instruction; the presence of a
final cross-check that depends on results from several earlier stages is a sign the
stages want names.

### `derive-input/src/internal/dsl/table.rs`: the timestamp column validation

**Violates:** Don't Repeat Yourself, Connascence, Duplication Control & Reuse

The `created_at`/`inserted_at` block and the `modified_at`/`updated_at` block are
structurally the same: reject a second column of the kind, check the field type, then match
on visibility with an error for `Public`, an error for `Restricted` and an assignment for
`Inherited`. The visibility match arms are word-for-word identical apart from the column
names in the message.

The two blocks have drifted, and the differences carry real invariants:

- The update block additionally rejects the column when `has_update_method` is false; the
  insert block has no such check.
- The update block accepts `Option<Timestamp>` as well as `Timestamp`; the insert block
  accepts only `Timestamp`.
- The set of accepted type spellings differs accordingly, and each is written out as a list
  of rendered-token literals.

Recommendation: this is duplication where the logic has drifted and the differences encode
invariants, so the developers should first work out which of the differences are deliberate
design (the `Option<Timestamp>` allowance for update timestamps looks deliberate; the missing
`has_update_method` check on the insert side may or may not be) before deciding what to keep.
Once that is settled, extract a single `resolve_timestamp_column` parameterised by the
accepted names, the accepted types and whether the update method is required, so the shared
visibility and duplicate checks are stated once. Route the type check through
`ColumnTypeKind` rather than through rendered strings, as the workspace-wide finding
recommends.

### `derive-input/src/internal/dsl/table.rs`: `referencing_tables` accumulation

**Violates:** Principle of Least Astonishment, Avoid hidden coupling or magic

The loop over fields does `let refs = ReferencingTable::try_parse(…)?; if
referencing_tables.is_empty() { referencing_tables = refs; }`. Any `#[referenced_by]`
attribute found on a later field, after an earlier field already contributed some, is parsed
and then silently discarded. In practice `#[referenced_by]` requires `#[primary_key]` and a
table has one primary key, so the case may be unreachable — but the code does not say that,
and a reader cannot tell whether the guard is a deliberate "first wins" rule or a bug.

Recommendation: if the case is genuinely impossible, replace the guard with an
`extend`/`append` (which is correct under all readings) or with an explicit assertion that
documents why it cannot happen. If it is possible, decide whether later attributes should be
appended or rejected with a diagnostic — silently dropping user-written attributes is the one
behaviour that should not survive. The variable name `refs` is also an abbreviation and
should be spelled out.

---

## `derive-input/src/internal/dsl/hook.rs`

Builds the trait, function name, arguments and return type of each hook.

### `derive-input/src/internal/dsl/hook.rs`: `build`

**Violates:** Connascence, Don't Repeat Yourself

`build` takes a run of positional `bool` parameters in the order before-insert,
before-update, before-delete, after-insert, after-update, after-delete, and its body then
repeats `build_any(<flag>, Timing::X, singular_table_name, Operation::Y)` once per pair.
Transposing two arguments at the call site produces working code with wrong behaviour.

Recommendation: pass the `RequestedHooks` struct proposed under `DSLData` above, and replace
the repeated `build_any` calls with a loop over the `(Timing, Operation)` pairs. Both changes
remove Connascence of Position, which the Coupling checklist asks to weaken wherever
possible.

### `derive-input/src/internal/dsl/hook.rs`: the `Create<Table>` argument type name

**Violates:** Don't Repeat Yourself, Connascence

`get_function_args` and `get_return_type` each build `format_ident!("Create{…}")`, and
`create.rs` builds the same name when it declares the struct. The code flags this itself with
`// FIXME: Single Source of Truth Violation for arg type name`.

Recommendation: covered by the workspace-wide recommendation on generated identifier names —
move this into `naming.rs` (or an intra-table sibling) and call it from all sites. Delete the
FIXME in the same pass.

---

## `derive-input/src/internal/dsl/wrapper.rs`

Parses `#[create_wrapper]` / `#[use_wrapper]` and emits the wrapper struct.

### `derive-input/src/internal/dsl/wrapper.rs`: `WrapperType::try_parse`, `map`, `map_to_wrapped_type`

**Violates:** Connascence, Robustness Principle, Prefer clarity over cleverness, Principle of Least Astonishment

Types are routed through text repeatedly: a `syn::Type` is rendered with
`to_token_stream().to_string()`, carried as a `String`, and re-parsed with `parse_str`.
Each re-parse is followed by `.expect("should be parseable")` or an
`.unwrap_or_else(|_| panic!(…))`, so a spelling the round trip cannot survive aborts the
macro instead of producing a diagnostic. `WrapperType::map` in particular renders a value
that is already a `Path` or `Ident` and re-parses it as a `Type`.

The same `impl` block mixes conventions: `map` and `map_to_wrapped_type` are associated
functions taking `&WrapperType`, while `struct_name_or_path_tokens` in the same block takes
`&self`. A caller has to remember which is which.

Recommendation: keep the parsed `syn::Type`/`syn::Path` values from the moment they are
parsed and splice them directly, deleting the string round trips and the panics with them.
Where a fallible conversion genuinely remains, return `syn::Result` so the error reaches the
user as a diagnostic — the Robustness checklist's "Log and surface malformed partner payloads
immediately" applies to macro input as much as to network input. Convert `map` and
`map_to_wrapped_type` to `&self` methods for consistency with their neighbour.

### `derive-input/src/internal/dsl/wrapper.rs`: `map_wrapper_type_option_to_wrapped_type_option`

**Violates:** Prefer clarity over cleverness, Robustness

The emitted code is `let mut x_option = None; if x.is_some() { x_option = Some(Into::<T>::into(x
.expect("value should exist")).value()); } let x = x_option;` — a check-then-`expect` pair
where `if let Some(x) = x` would express the same thing without a panic in user-facing
generated code.

Recommendation: emit `let x = x.map(|x| Into::<T>::into(x).value());`. Generated code is read
by users in the `Implementation:` doc comment this project attaches to every method, so its
clarity is part of the published interface.

---

## `derive-input/src/internal/dsl/foreign_key.rs`

Parses `#[foreign_key(...)]`.

### `derive-input/src/internal/dsl/foreign_key.rs`: `OnDeleteStrategy::try_parse`

**Violates:** Don't Repeat Yourself, Connascence, Robustness Principle, Documentation & Communication Clarity

The parser matches the strategy against string literals — `"Error"`, `"Delete"`, `"SetNone"`,
`"SetZero"`, `"Ignore"` — duplicating the variant names of an enum that is itself already
duplicated across crates. Adding or renaming a variant requires a matching edit here that
nothing enforces.

The two error messages contradict each other: the `"SetNone"` arm rejects the value and
states that the strategy "must be one of `Error`, `Delete`, `SetZero` or `Ignore`", while the
catch-all arm for an unrecognised value states that it "must be one of `Error`, `Delete`,
`SetNone`, `SetZero` or `Ignore`" — offering the user the very value the other arm refuses.

Recommendation: derive the parse from the enum (`strum::EnumString` or a hand-written
`FromStr` next to the definition) so the variant list has one home, and keep only the
deliberate `SetNone` rejection as an explicit special case. Correct the catch-all message to
list the strategies that are actually accepted. Both changes land naturally alongside the
workspace-wide `OnDeleteStrategy` consolidation.

### `derive-input/src/internal/dsl/foreign_key.rs`: the `SetZero` visibility check

**Violates:** Connascence, Prefer clarity over cleverness

The check renders both `field.vis` and `syn::Visibility::Inherited` to strings and compares
the results, which is a third distinct way of asking "is this column private?" in a codebase
that already has two others.

Recommendation: `matches!(field.vis, syn::Visibility::Inherited)`. Covered by the
workspace-wide recommendation; noted here because this site does not even go through
`RustVisibility` and so would be missed by a search for that type.

---

## `derive-input/src/internal/dsl/method.rs`

Decides which DSL methods each table and column earns, and drives the generators.

### `derive-input/src/internal/dsl/method.rs`: `SpacetimeDSLTableMethods::generate`

**Violates:** Single Responsibility Principle, Curly's Law, Don't Repeat Yourself

The function generates the create method, decides on `get_all`/`get_count`, generates both
cascade entry points for referencing tables, groups the foreign key columns by referenced
table and generates a cascade pair for each, and filters and generates the multi-column index
methods — accumulating `TableContributions` across all of it.

The grouping is done by hand: `if !map.contains_key(k) { map.insert(k, vec![]); }
map.get_mut(k).expect("The entry was inserted above when it was missing").push(v)`. The
standard `map.entry(k).or_default().push(v)` does the same thing without the lookup, the
insert, the second lookup and the `expect` that only exists because of them. The same
hand-rolled pattern appears again in `for_foreign_key`.

The paired `for_referenced_by(&OneOrMultiple::One, …)` / `for_referenced_by(&OneOrMultiple::
Multiple, …)` calls, and the identical pairing for `for_foreign_key`, repeat a long argument
list twice each.

Recommendation: extract `build_cascade_methods_for_referencing_tables` and
`build_cascade_methods_for_referenced_tables`, leaving `generate` to compose. Replace both
hand-rolled grouping blocks with `entry().or_default()`. Iterate over `[OneOrMultiple::One,
OneOrMultiple::Multiple]` rather than writing each call twice — which also removes the risk
of the two calls drifting in their argument lists, a risk that has already materialised
elsewhere in this crate.

---

## `derive-input/src/internal/dsl/method/delete.rs`

Generates `delete_<table>_by_<index>` and `delete_<tables>_by_<index>`.

### `derive-input/src/internal/dsl/method/delete.rs`: `for_delete_one` and `for_delete_many`

**Violates:** Don't Repeat Yourself, Single Responsibility Principle, Curly's Law

The two generators are structurally the same function written twice. Both build a
before-delete hook block and an after-delete hook block from identical closures; both build a
count-mismatch error, a deletion-result value with and without `error_from_hook`, and a
return block; both then branch on `spacetimedsl_table.referencing_tables.is_empty()` and, in
the non-empty branch, build an `error_after_state_change` message, an `on_error_handler`, and
four `referenced_table_function_call_for_dsl_method` calls for the `Error`, `Delete`,
`SetZero` and `Ignore` strategies before assembling the same sequence of fragments. The two
differ only in whether they operate on one row or many, and in how they look the row(s) up.

The duplication has already drifted: the commented-out `set_none_strategy` block in
`for_delete_one` passes a different argument list from the one in `for_delete_many` — the
`for_delete_one` copy is missing `primary_key_column_name`. Both are commented out, so
neither compiles, and the divergence went unnoticed. That is exactly the failure mode
duplication produces.

Within each function, the before-delete and after-delete hook blocks are byte-for-byte
identical apart from which hook field they read.

Recommendation: this is duplication where the copies have drifted, so the developers should
first read the two carefully and establish which of the remaining differences are essential —
the row-lookup body and the one-versus-many entry shape clearly are; whether anything else is
needs confirming before the merge. Once that is settled, extract the shared cascade assembly
into one function parameterised by `OneOrMultiple`, and extract the hook-block construction
into a single helper taking the hook and the loop shape. Do this only after the snapshot
suite is confirmed green, since the snapshots are the sole guard on the generated output.
Resolve the commented-out `SetNone` scaffolding in the same pass — it is dead code, and the
developers must decide whether the strategy is still planned (in which case the tracked issue
is the right home for the sketch) or abandoned.

### `derive-input/src/internal/dsl/method/delete.rs`: generated comparisons

**Violates:** Prefer clarity over cleverness, Code For The Maintainer

The emitted code uses `count_of_rows_to_delete.ne(&count_of_deleted_rows)` where `!=` would
read plainly. The same `.ne(...)`/`.eq(...)` style appears throughout the generator and the
example crate. Because this project publishes the generated body in each method's doc
comment, the generated code's readability is part of the user-facing documentation.

Recommendation: prefer `!=` and `==` in emitted code and in the generator itself, keeping the
method-call form only where it is genuinely needed (for example to disambiguate a
deref). A low-risk, high-readability sweep that fits the Boy Scout Rule.

---

## `derive-input/src/internal/dsl/method/singleton_table.rs`

Generates the get and delete methods for singleton tables.

### `derive-input/src/internal/dsl/method/singleton_table.rs`: `for_singleton_delete` and `for_singleton_get`

**Violates:** Don't Repeat Yourself, Duplication Control & Reuse

`for_singleton_delete` reproduces `for_delete_one`'s structure — the same hook blocks, the
same count-mismatch error, the same deletion-result entry, the same not-found handling — with
one substantive difference: it emits no on-delete cascade at all. `for_singleton_get`
likewise reproduces the single-column branch of `for_get_one`.

The missing cascade may be correct (a singleton's own primary key is injected and is not a
foreign key target), or it may be a gap that shows up the first time someone points a
`#[referenced_by]` at a singleton. The code does not say which, and the module documentation
in `method.rs` explains why the *get* and *delete* bodies were split out without addressing
the cascade at all.

Recommendation: this is duplication where the paths have diverged and the divergence encodes
an invariant, so the developers should establish first whether a singleton can be referenced
by another table. If it cannot, say so in a comment where the cascade would otherwise be, and
consider rejecting `#[referenced_by]` on a singleton with a diagnostic so the invariant is
enforced rather than assumed. If it can, the missing cascade is a bug and the singleton
generators should share the cascade assembly extracted from `delete.rs` above. Either way the
duplicated hook blocks and result construction should come from the shared helpers once those
exist.

---

## `derive-input/src/internal/dsl/method/foreign_key.rs` and `referenced_by.rs`

The two halves of the cascade: the function a referencing table generates, and the entry
points a referenced table generates.

### `derive-input/src/internal/dsl/method/foreign_key.rs` + `referenced_by.rs`: `for_foreign_key` and `for_referenced_by`

**Violates:** Don't Repeat Yourself, Single Responsibility Principle, Connascence

The two functions share a large amount of structure written out twice:

- The same deferred-initialisation shape (`let doc_comment; let return_type; let arg_name;`
  followed by a `match one_or_multiple`), which defeats the compiler's ability to see the
  bindings as immutable and forces the reader to scan for the assignments.
- The same `entries_type` / `failure_type` / `Result<…, …>` construction, written once per
  branch in each function — so the same three lines appear in both files, twice each.
- The same `create_entries` shape, differing only in the identifier the `Multiple` branch
  iterates.
- The same tail: `error_from_hook_declaration`, `let mut error = false;`, the strategy calls,
  and `match error { false => Ok(entries), true => Err(failure) }`.

Inside each, the `Ok` and `Err` arms of the strategy-call match contain the same loop body
over `(primary_key_value, child_entries)`, so the loop is written twice per arm pair.

Recommendation: extract the shared shape — a `CascadeSignature` helper that, given
`one_or_multiple` and the primary-key type, returns the argument name, the entries type, the
failure type and the return type — and use it from both files. Replace the deferred-init
`let` declarations with `let x = match … { … };`. Factor the identical `Ok`/`Err` loop bodies
into a single emitted block. The functions will still be long, because they genuinely emit a
lot, but the parts a reader must compare by eye today will be stated once.

### `derive-input/src/internal/dsl/method/foreign_key.rs`: `for_foreign_key`'s parameter list

**Violates:** Connascence, Minimize Coupling, Law of Demeter

The function takes seven positional parameters, several of which are already reachable from
each other (`spacetimedb_table` and `spacetimedsl_table` are both fields of the
`MethodGenerationContext` that the sibling generators receive as a single value).
`on_delete_strategy_implementation` has the same shape. Every caller has to get seven
positions right, and adding an eighth means editing every call site.

Recommendation: pass `&MethodGenerationContext` as the other generators do, adding to it
whatever these two need that it does not already carry, and keep only the genuinely
per-invocation parameters (`one_or_multiple`, `referenced_table_name`,
`columns_with_foreign_key`) explicit. This is the Coupling checklist's "Prefer weaker
connascence forms when refactoring dependencies" — Connascence of Name beats Connascence of
Position.

### `derive-input/src/internal/dsl/method/foreign_key.rs`: repeated `expect` messages for one invariant

**Violates:** Don't Repeat Yourself, Code For The Maintainer

The same invariant — "a column in a foreign key group carries a foreign key" — is asserted
several different ways in one function: `.expect("A table grouped by referenced table must
have at least one foreign key column")`, `.expect("The first column of a foreign key group
carries the foreign key that grouped it")`, and `.unwrap_or_else(|| panic!("the column {} is
in a foreign key group, so it carries a foreign key", …))`. Several phrasings, more than one
mechanism, one fact.

Recommendation: extract a small helper (`fn foreign_key_of(column: &Column) -> &ForeignKey`)
that states the invariant once, and call it everywhere. Better still, make the invariant
structural: group the columns into a type that can only be constructed with a foreign key
present, so the assertions disappear entirely. The developers should judge whether the
stronger form is worth the extra type.

---

## `derive-input/src/internal/dsl/method/on_delete_strategy.rs`

Emits the body of one on-delete strategy.

### `derive-input/src/internal/dsl/method/on_delete_strategy.rs`: `on_delete_strategy_implementation`

**Violates:** Single Responsibility Principle, Curly's Law, Keep It Simple, Connascence

The function takes seven positional parameters and is by a wide margin the most deeply nested
in the workspace: a `for` over the columns, a `match` on the strategy inside it, a `match` on
`referencing_tables` inside the `Delete` arm, and a further `match` on `one_or_multiple`
inside that — with several `quote!` blocks defined at each level and appended to mutable
`TokenStream` accumulators declared before the loop (`strategy_for_before_hook`,
`strategy_for_after_hook`, `strategy_for_referenced_by`, `strategy_after_all`). Those
accumulators are assigned inside the loop and read after it, so the last column processed
silently wins for several of them — a coupling that is invisible at the assignment sites.

The `Delete`-with-referencing-tables arm alone builds the hook calls, its own `HashMap`
declarations, the per-row strategy, the delete loop and the cascade strategy calls, and is
longer than most complete functions in the crate.

Recommendation: extract one function per strategy — `error_strategy`, `delete_strategy`,
`set_zero_strategy`, `ignore_strategy` — each returning the fragments it contributes, and
have the loop merge those results rather than assigning into shared mutable accumulators.
That makes the "last column wins" behaviour either explicit or impossible, and it makes each
strategy readable on its own. Replace the positional parameters with the context value, as
recommended for `for_foreign_key`. Note that this is the highest-risk refactoring in the
report, because the cascade semantics are subtle and the only guard is the snapshot suite
plus the integration reducer — the developers should make sure both gates actually fail on
regressions (see the CI finding) before starting.

### `derive-input/src/internal/dsl/method/on_delete_strategy.rs`: `referenced_table_function_call_for_strategy_implementation`

**Violates:** Don't Repeat Yourself

The `Err` arm and the `Ok` arm of the emitted match contain the same `for
(primary_key_value_of_a_row_to_delete, mut child_entries) in …` loop with the same body; only
the collection being iterated and the extra error bookkeeping differ.

Recommendation: emit the loop body once into a local and splice it into both arms, or restructure
the emitted code so both arms feed a single loop. The same pattern appears in
`referenced_by.rs` and is worth fixing in the same pass.

### `derive-input/src/internal/dsl/method/on_delete_strategy.rs`: mixed conditional idioms and eager `expect` formatting

**Violates:** Prefer clarity over cleverness, Avoid Premature Optimization (inverted)

The function mixes `if is_singleton { … } else { … }` with `match … { true => …, false => … }`
for equivalent boolean decisions, sometimes within a few lines of each other. The `match`-on-
`bool` idiom is used widely across the generator; it is defensible as a house style, but
mixing both forms in one function is not.

The emitted code repeatedly uses `.expect(&format!("…"))`, which builds the message on every
call regardless of whether the `Option` is `None`. `expect` is for the impossible case, so
the allocation is always wasted.

Recommendation: settle on one conditional idiom and apply it consistently — the choice matters
less than the consistency. In emitted code, prefer `unwrap_or_else(|| panic!("…"))` so the
message is built only on failure, or better, restructure so the lookup cannot fail.

---

## `derive-input/src/internal/dsl/method/reference_integrity.rs`

The checks a generated method runs before it writes.

### `derive-input/src/internal/dsl/method/reference_integrity.rs`: `Action` and `multi_column_index_checks`

**Violates:** Don't Repeat Yourself, Unused scaffolding removed immediately, Interface Segregation

The local `Action` enum duplicates the runtime crate's, as noted workspace-wide. Within this
file it also carries variants the module does not use: `multi_column_index_checks` is only
ever called with `Action::Create` and `Action::Update`, yet its `on_some` match names
`Action::Get` and `Action::Delete` in an arm that cannot be reached from any call site in the
workspace. (`Action::Get` and `Action::Delete` do reach `unique_multi_column_index_check`
from `get.rs` and `delete.rs`, so the enum as a whole is used — it is this function's arms
that are not.)

Recommendation: the unreachable arms are dead code, and the developers must decide for
themselves whether they are no longer needed or whether the fact that they aren't being
reached is actually a bug — specifically whether a multi-column index check was meant to run
on the get and delete paths too. If they are genuinely unreachable, consider giving
`multi_column_index_checks` a narrower parameter type (a two-variant `WriteAction`) so the
compiler enforces the restriction, per "Split fat interfaces so clients receive only needed
methods".

### `derive-input/src/internal/dsl/method/reference_integrity.rs`: `field_name_for_found_value` and getter names

**Violates:** Don't Repeat Yourself, Connascence

`format_ident!("the_same_or_another_{…}")` is rebuilt in `reference_integrity_checks_on_update`,
in `multi_column_index_checks` and in `unique_multi_column_index_check`, despite
`MethodGenerationContext` already carrying the resolved value and documenting itself as the
single place the rule is stated. The `get_<column>` and `get_<table>_by_<column>` names are
likewise rebuilt here rather than taken from the modules that define them.

Recommendation: covered by the workspace-wide recommendation on generated identifiers. This
file is the densest concentration of the problem and is the right place to start.

### `derive-input/src/internal/dsl/method/reference_integrity.rs`: duplicated unique-constraint error construction

**Violates:** Don't Repeat Yourself

`multi_column_index_checks` and `unique_multi_column_index_check` each build the same
`runtime::unique_constraint_violation(...)` call with the same `SpacetimeDSL` origin and the
same `OneOrMultiple::Multiple`, differing only in how the action identifier is obtained.

Recommendation: build it once in a shared helper and call that from both. Small, safe, and
squarely within the Boy Scout Rule.

---

## `derive-input/src/internal/dsl/method/index.rs`

Per-index and per-column analysis for the index-based generators.

### `derive-input/src/internal/dsl/method/index.rs`: `IndexShape::of`

**Violates:** Connascence, Prefer clarity over cleverness

The function destructures a five-element tuple out of a `match` on the index type — `let
(index_columns, is_multi_column, value_matches, single_or_multi, on_the_columns) = match …`.
A reader has to count positions across the arms to know which literal means what, and adding
another piece of information means touching every arm and the destructuring pattern.

Recommendation: give the match a small named struct to return, or split the three independent
questions (columns, multi-column-ness, doc phrasing) into separate small functions over
`&IndexType` — `index_kind` already sets that precedent in the same file.

### `derive-input/src/internal/dsl/method/index.rs`: `index_column_arguments`

**Violates:** Keep It Simple, Curly's Law, Prefer clarity over cleverness

The body declares `let wrapper_option_mapper; let method_arg; let row_value_getter;` and then
assigns them from inside a nest of `match`/`if`/`match`, with an assignment site per binding
in every leaf of that nest. Following any one of them requires reading the whole nest.

Recommendation: extract one function per case — the wrapper/string case, the wrapper/option
case, the wrapper/plain case and the no-wrapper case — each returning a small struct with the
three values. The function's own documentation already identifies the axes (`wrapper type` ×
`string-ness` × `option-ness` × `one-or-many`), which is a good basis for the split.

---

## `derive-input/src/internal/dsl/method/create.rs`

Generates `create_<table>` and the `Create<Table>` argument struct.

### `derive-input/src/internal/dsl/method/create.rs`: `create_method_column_parts`

**Violates:** Keep It Simple, Don't Repeat Yourself, Command Query Separation

The function declares three mutable `Option` locals, assigns them across a chain of
`if`/`else if` branches and a nested `match`, and returns from several different points, most
of them early returns that construct the same struct with different fields populated. Whether
a given column contributes an argument, a mapper and a constructor line is only discoverable
by simulating the whole chain.

The `String`-typed argument branch is written out twice, once in the `WrapperType::Created`
arm and once in the `None` arm, with identical bodies.

Recommendation: make each case a small function returning a fully-populated
`CreateMethodColumnParts`, and have `create_method_column_parts` dispatch to them — replacing
the mutable accumulators and multiple exits with one `match` over the cases. Extract the
duplicated `String` branch into a shared helper.

### `derive-input/src/internal/dsl/method/create.rs`: untracked FIXMEs in emitted code

**Violates:** Future ideas captured outside codebase, Log blockers to future cleanups

`// FIXME: Only show unique columns here` sits above the unique-constraint error, which
currently renders the whole struct with `{:?}` into a user-facing message, and `// FIXME: No
clone?` sits on a `try_insert(#name.clone())` in the hot path of every create. Neither links
to an issue.

Recommendation: file both — the first is a diagnostic-quality issue users will notice, the
second a performance question that the Performance checklist says should be settled by
profiling ("Profile hotspots before considering any micro-optimization") rather than by a
comment. Replace the markers with links.

---

## `derive-input/src/internal/dsl/method/update.rs`

Generates `update_<table>_by_<index>`.

### `derive-input/src/internal/dsl/method/update.rs`: `for_update` emitted panics

**Violates:** Robustness Principle, Code For The Maintainer

The emitted body contains `.expect("Row should exist for update")` when pre-loading the row
for a before-update hook, and `#field_name_for_found_value.as_ref().unwrap()` when passing the
old row into both the before- and after-update hooks. These are panics in generated code
running inside a database reducer, where the surrounding API is otherwise uniformly
`Result`-based.

`reference_integrity.rs` emits a third, `.expect("field_name_for_found_value should be
Some(_)")`, in the same family.

Recommendation: emit a `SpacetimeDSLError` return in place of each panic, using the not-found
error the module already constructs elsewhere. If the developers are confident the states are
genuinely unreachable, the panics are at least honest — but the generated body is published in
each method's doc comment, so a user reading it sees an `unwrap` in code they are told is the
implementation of their DSL method. The Robustness checklist's "Enforce strict output formats
before sending responses" and the crate's own error-first design both argue for the `Result`.

### `derive-input/src/internal/dsl/method/update.rs`: the visibility filter

**Violates:** Connascence, Prefer clarity over cleverness

`internal_column.rust_field_visibility.to_string().ne(&RustVisibility::Private.to_string())`
— one of the string comparisons covered workspace-wide.

Recommendation: derive `PartialEq` on `RustVisibility` and write `!=`, as recommended there.

---

## `derive-input/src/api/rust/visibility.rs`

### `derive-input/src/api/rust/visibility.rs`: `RustVisibility`

**Violates:** Principle of Least Astonishment, Prefer stable abstractions, Connascence

The enum derives only `Clone`. The absence of `PartialEq` is the direct cause of every
visibility-by-string comparison in the generator, and the absence of `Debug` makes it awkward
in error messages. Its `Display` impl is also load-bearing in a way the type does not
advertise: `accessor.rs` parses that output back into a `syn::Visibility`, so the spacing in
`"pub (crate)"` is part of a contract stated nowhere.

Recommendation: derive `Debug`, `PartialEq` and `Eq`. Either document the `Display` round-trip
contract at the impl, or — better — remove the need for it by having the type carry or
produce a `TokenStream` directly, so `accessor.rs` can splice instead of re-parse. The second
resolution removes a whole class of fragility; the first is cheaper. The developers should
pick based on whether the `Display` output is needed for human-readable messages anywhere
else.

---

## `derive-input/src/api/dsl/column.rs`

### `derive-input/src/api/dsl/column.rs`: field documentation

**Violates:** Documentation & Communication Clarity, Code For The Maintainer, Hide Implementation Details

The conditions under which each `Option` field is populated are stated in `//` line comments
(`// Only Some(T) if mutable`, `// Only Some(T) if the table has a delete method.`) rather
than `///` doc comments. On a `pub` struct in a published crate, that means the conditions do
not appear in the rendered documentation — the one place a consumer of `derive-input` would
look for them. The same pattern appears on `SpacetimeDSLColumnMethodsForUniqueIndex` and
`SpacetimeDSLColumnMethodsForIndex`.

Recommendation: convert them to `///`. The crate elsewhere writes excellent doc comments
(`api/runtime.rs`, `method/context.rs`, `method/naming.rs` are all exemplary), so this is an
inconsistency rather than a gap in practice.

---

## `examples/test/src/lib.rs`

The integration test module: table definitions covering every DSL feature, plus a `tester`
reducer that exercises them against a live SpacetimeDB.

### `examples/test/src/lib.rs`: the `tester` reducer

**Violates:** F.I.R.S.T Principles of Testing, Arrange/Act/Assert, Single Responsibility Principle, Curly's Law

`tester` is one function that occupies most of the file and asserts essentially every runtime
behaviour the DSL has: creation, getters, setters, counts, unique multi-column indices,
foreign key cascades in several configurations, timestamps, wrapper types and more. It fails
on the first assertion and returns a `String`, so everything after the failure is never
reached — a regression in an early feature hides the state of every later one. All of its
assertions share one database and one transaction, so nothing is independent. It cannot be
run with `cargo test`; it needs a published module and a CLI call.

Three helpers — `hash_index_test`, `singleton_test`, `hook_call_test` — have been split out,
which shows the intended direction; the bulk has not followed.

Recommendation: continue the split that is already underway. Give each feature its own
function with its own arrange/act/assert shape, and have `tester` call them all, collecting
failures rather than returning on the first, so one run reports every broken feature. Longer
term the Testing checklist's "Keep unit tests running in milliseconds" and "Ensure each test
has no shared state" point toward moving what can be tested without a live database into
`cargo test`, leaving only genuinely end-to-end behaviour here. The developers should decide
how much of this is practical given SpacetimeDB's testing surface — but the per-feature split
and the collect-all-failures change are worth doing regardless, and neither depends on that
question.

### `examples/test/src/lib.rs`: the repeated result-handling idiom

**Violates:** Don't Repeat Yourself, Prefer clarity over cleverness

`match dsl.<op>(…) { Ok(x) => …, Err(error) => return Err(format!("<message>! Got:\n{error}")) }`
appears throughout, with only the operation and the message varying. `if <count>.ne(&<n>) {
return Err("…".to_string()) }` appears similarly often.

Recommendation: extract small assertion helpers — `expect_ok(result, message)` and
`expect_count(actual, expected, message)` — so each test step reads as one line of intent.
This also makes the collect-all-failures change above straightforward to implement.

### `examples/test/src/lib.rs`: identifier naming

**Violates:** No abbreviations, Descriptive identifiers

`er4_1`, `er4_2`, `obj_id` and `sobj_id` abbreviate entity relationships, object identifiers
and ship-object identifiers. AGENTS asks for `FileSystemWatcher` over `Watcher`; these go
further than the examples it rejects.

Recommendation: spell them out. `obj_id` and `sobj_id` are column names that appear in
generated method names (`get_entity_by_obj_id`), so renaming them changes the generated API
in the example — which is fine for an example, and is itself a useful demonstration of the
naming the framework encourages.

---

## `examples/blackholio/src/lib.rs`

A port of the Blackholio sample game, used as a realistic second example.

### `examples/blackholio/src/lib.rs`: `move_all_players`

**Violates:** Single Responsibility Principle, Curly's Law, Robustness Principle, Don't Repeat Yourself

One reducer performs split-circle gravitation, split-circle separation, player input
application and collision detection. Each is a distinct algorithm with its own reason to
change, and they are interleaved with the database reads and writes they depend on.

The body is dense with `.unwrap()`: on `HashMap::remove`, on `HashMap::get`, on
`HashMap::get_mut`, and on `Timestamp::duration_since`. Any of these panicking aborts the
reducer. One of them has an acknowledged hole already — `// FIXME: What does that mean for
the foreign key relationship between circles and entities?` sits on the one lookup that *is*
handled, because a circle can be eaten mid-tick.

The "Force circles apart" loop is nested *inside* the gravitation loop and rebinds `i`,
shadowing the outer loop variable. As written, the separation pass runs once per circle
rather than once per tick, and the shadowing hides it. This reads as a bug rather than a
deliberate choice, but the behaviour is observable (circles separate faster the more of them
there are) and may have been tuned around.

The distance-and-overlap preamble — `diff`, `squared_distance`, the `<= 0.0001` guard, the
`radius_sum` computation — is written twice, identically.

Recommendation: the shadowed loop needs a decision from the developers before anything else:
whether the separation pass was meant to run once per tick, in which case it belongs outside
the gravitation loop, or whether the current repetition is load-bearing for the game feel. If
it was unintentional, fixing it is a behaviour change and should be verified by running the
example. Separately, extract each phase into its own function, extract the duplicated distance
preamble into a helper, and replace the `unwrap`s with explicit handling — the Robustness
checklist's stance on unknown inputs applies with particular force in a reducer, where a panic
aborts a transaction. Note that this is example code adapted from an upstream sample, so the
developers may reasonably decide to keep it close to its source; if so, that decision is worth
a comment at the top of the file, because nothing currently signals it.

### `examples/blackholio/src/lib.rs`: the in-source feature checklist and denormalised `login_status`

**Violates:** Future ideas captured outside codebase, Don't Repeat Yourself, One authoritative source for each business rule

A markdown TODO checklist of unimplemented game features sits at the top of the file. Separately,
`login_status` is stored on `Entity`, on `Circle` and on `Player` — the same fact in three
tables, each indexed, with nothing keeping them consistent. A later `// FIXME: This makes no
sense for food` marks a site where the denormalisation has already produced a meaningless
value.

Recommendation: move the checklist to the issue tracker. For `login_status`, the developers
should decide whether the duplication is a deliberate denormalisation for query performance —
in which case it wants a comment saying so and a single place that updates all three — or an
accident, in which case `Player` is the natural owner and the others should follow the foreign
key. The `// FIXME` on food is evidence for the second reading, but the query patterns are
better known to the developers than to this report.

### `examples/blackholio/src/lib.rs`: constant naming

**Violates:** No abbreviations, Descriptive identifiers

`SPLIT_RECOMBINE_DELAY_SEC`, `SPLIT_GRAVITY_PULL_BEFORE_RECOMBINE_SEC` and
`ALLOWED_SPLIT_CIRCLE_OVERLAP_PCT` abbreviate "seconds" and "percent".

Recommendation: spell them out — `..._SECONDS`, `..._PERCENT`. Note that
`ALLOWED_SPLIT_CIRCLE_OVERLAP_PCT` holds `0.9`, a ratio rather than a percentage, so the
rename is also a chance to make the unit honest.

---

## `examples/blackholio/src/math.rs`

### `examples/blackholio/src/math.rs`: `DbVector2`

**Violates:** No abbreviations, Descriptive identifiers, Don't Repeat Yourself, Robustness Principle, Principle of Least Astonishment

`sqr_magnitude` abbreviates "square"; `Sum::sum` uses `r` and `val` as its locals. The `Add`
impls for `DbVector2` and `&DbVector2` have identical bodies, as do the two `Sub` impls.

`Div<f32>` returns `DbVector2 { x: 0.0, y: 0.0 }` when the divisor is zero. A division that
silently yields a wrong answer instead of `inf`/`NaN` — or a panic, or a `Result` — hides the
bug at the call site. `normalized()` divides by `magnitude()`, so normalising a zero vector
silently produces a zero vector rather than an undefined one, and `move_all_players` calls
`normalized()` on differences it has just guarded against being zero, suggesting the callers
do not rely on the silent behaviour anyway.

Recommendation: rename `sqr_magnitude` to `squared_magnitude` and give the `Sum` locals real
names. Implement the reference-taking operators by delegating to the value-taking ones so each
formula is written once. For `Div`, the developers should decide what the right contract is —
propagating `inf`/`NaN` is the honest float behaviour and the cheapest change; returning
`Option<DbVector2>` is the safest; keeping the zero fallback is defensible only if some caller
depends on it, and none appears to. Whichever is chosen, the current behaviour should at
minimum be documented, since it is the opposite of what `Div` conventionally means. As with
`lib.rs`, note that this file is adapted from an upstream sample and the developers may
prefer to stay close to it.

---

## `debug-helper/src/main.rs`

### `debug-helper/src/main.rs`: `Error::IncorrectUsage`

**Violates:** Documentation & Communication Clarity, Principle of Least Astonishment

The usage message reads `"Usage: dump-syntax path/to/filename.rs"`, naming a binary that does
not exist in this workspace. The crate is `spacetimedsl-debug` and is invoked as `cargo run
-- <file>`, which is what the module documentation at the top of the file correctly says. A
user who hits the error gets the wrong instruction.

Recommendation: correct the message to match how the tool is actually invoked. The file is
adapted from `syn`'s `dump-syntax` example, which explains the origin; adapting the message is
part of adopting the code.

---

## `x.sh`, `x.ps1`, `gen-x.sh`, `compare-performance.ps1`

The developer task scripts. `x.sh` and `x.ps1` are generated from `gen-x.sh`, which is the
right shape for keeping two shells in sync.

### `gen-x.sh`: the `loc` command's hard-coded crate list

**Violates:** Don't Repeat Yourself, One authoritative source for each business rule

The `loc` case sums lines only for the directories `src`, `derive-input` and `derive`, named
as string literals. The authoritative list of workspace crates is in `Cargo.toml`, and the
two will drift the first time a crate is added.

Recommendation: derive the list from `cargo metadata`, or state plainly in a comment that the
total deliberately covers only the crates that make up the framework proper. The second
is perfectly acceptable if that is the intent — the problem is that the code does not say
which of the two it means.

### `compare-performance.ps1`: no cross-shell counterpart

**Violates:** Don't Repeat Yourself (inverted), Documentation & Communication Clarity

Every other developer task is generated for both shells from `gen-x.sh`; this one exists only
as PowerShell and only at the repository root, outside the `x` mechanism. A developer on Linux
or macOS — including CI — cannot run it.

Recommendation: either fold it into `gen-x.sh` as another case so both shells get it, or
document at the top of the file that it is intentionally Windows-only and why. The current
state gives a reader no way to tell which.

### `gen-x.sh`: the `format` case's usage text

**Violates:** Documentation & Communication Clarity, Sync related artifacts

The usage help describes `format` as "Run cargo fmt check and clippy fixes", but the case runs
`cargo fmt --all`, which rewrites files rather than checking them. A developer reading the
help would not expect their working tree to change.

Recommendation: correct the description to say that it formats and applies clippy fixes. Worth
noting that CI separately runs `cargo fmt --all -- --check`, so the checking behaviour the
help describes does exist — just not in this command.

---

## Recommended Resolution Order

The order below front-loads the changes that make the rest safe, then the correctness risks,
then the structural work, then the cleanups. Several entries deliberately group related
findings so they are resolved in one pass rather than revisited.

1. **Make the quality gates fail.** Remove the `|| echo` guards around clippy in
   `.github/workflows/test.yml`, add `set -euo pipefail` to `x.sh` (via `gen-x.sh`) and the
   PowerShell equivalent to `x.ps1`, and switch CI to the workspace-wide clippy invocation
   `gen-x.sh` already documents as correct. Nothing else in this list can be verified until
   the gates actually report failure.

2. **Add unit tests to `derive-input` and the runtime crate.** Start with the pure functions
   (`ColumnTypeKind::of`, `DeletionResultEntry::to_csv`, the `Display` impls, the naming
   helpers) and with whatever the refactorings below will touch first. AGENTS is explicit that
   tests come before refactoring; every structural item from step 7 onward depends on this.

3. **Replace the heuristic table selection in `integration.rs`.** A silent wrong-table
   selection is the most severe defect in the report: it produces a DSL that compiles and
   operates on the wrong data with no diagnostic. Decide the grammar question (explicit
   `table = …` versus exact-match-or-error) and remove the fuzzy matching and hash fallback.

4. **Resolve the emitted panics and the `Display` panic.** The `panic!` in
   `SpacetimeDSLError`'s `Display`, the `panic!` in `doc_comment.rs`, and the `expect`/`unwrap`
   calls emitted into generated update and reference-integrity code. These are user-facing
   failure modes with cheap fixes, and the `Action` type change that removes the `Display`
   panic is a prerequisite for cleanly resolving step 6.

5. **Decide the fate of the dead code, in one sweep.** `ContextType::Transaction`,
   `is_last_dsl_attribute` and the commented-out `make_struct_fields_private`,
   `MethodImplVariant`'s discarded `context_bound`, the unreachable empty-tables branch in
   `integration.rs`, the unreachable `Action` arms in `multi_column_index_checks`, and the
   commented-out `SetNone` scaffolding. Each needs the same judgement — no longer needed, or
   not-yet-wired and therefore a bug — and doing them together means making that judgement
   once, in one frame of mind.

6. **Consolidate the cross-crate duplicates.** `OnDeleteStrategy`, `Action` and
   `OneOrMultiple`. Establish which of the drifted differences are deliberate, pick the
   ownership arrangement, and replace the string-literal parser with a derived one. This
   removes the sharpest correctness trap in the codebase and is a prerequisite for trusting
   any later change to the cascade.

7. **Replace the string-comparison idiom.** Derive `PartialEq` on `RustVisibility`, extend
   `ColumnTypeKind` to cover timestamps, extract single attribute-path predicates, and compare
   `Ident`s and `Path`s with their own equality. One mechanical sweep across the generator that
   removes a whole class of fragility and makes several later refactorings easier to read.

8. **Centralise the generated identifier names.** Move the getter, DSL-method and
   `Create<Table>` name rules into `naming.rs`, and read `field_name_for_found_value` from
   `MethodGenerationContext` instead of rebuilding it. Cheap, and it makes the existing doc
   comments true.

9. **Merge the delete generators.** `for_delete_one` and `for_delete_many`, and then
   `for_singleton_delete` and `for_singleton_get` against their non-singleton counterparts.
   The singleton cascade question has to be settled as part of this. The largest single
   duplication in the codebase, and the one with demonstrated drift.

10. **Merge the cascade signature construction.** `for_foreign_key` and `for_referenced_by`:
    the shared return-type shape, the shared tail, the duplicated `Ok`/`Err` loop bodies, and
    the positional parameter lists.

11. **Split `on_delete_strategy_implementation`.** One function per strategy, results merged
    rather than accumulated into shared mutable state. The highest-risk refactoring here, which
    is why it comes after steps 1, 2, 9 and 10 have established both the safety net and the
    shared helpers it will use.

12. **Split the two large parsers.** `try_parse_dsl` in `internal.rs` and
    `SpacetimeDSLTable::try_parse` in `internal/dsl/table.rs`, including the drifted
    timestamp-column validation and the `__singleton_placeholder` sentinel. Introduce the
    `RequestedHooks` struct and thread it through `DSLData` and `hook::build`.

13. **Fix the index lifecycle.** Stop `SpacetimeDBTable::multi_column_indices` from holding
    single-column indices, decide what a column with two single-column indices should mean, and
    delete the defensive filter in `method.rs` once the source is correct.

14. **Split the remaining oversized builders.** `output::build`, `column::try_parse`'s
    five-tuple, `create_method_column_parts`, `index_column_arguments`, `IndexShape::of`, and
    `SpacetimeDSLTableMethods::generate`. Individually modest; together they account for most of
    what makes the generator hard to read.

15. **Consolidate the runtime context accessors.** Collapse the repeated `impl_*_err!` /
    `impl_*_ok!` macro pairs in `src/get_*.rs` and `src/as_*.rs`, and gather the
    `Context`/`ReadContext`/`WriteContext` impls next to their declarations.

16. **Tidy the runtime error and deletion modules.** Split `SpacetimeDSLError`'s `Display` into
    per-variant renderers, move `OnDeleteStrategy`'s `Display` to `delete.rs`, make
    `to_csv`'s recursion private, and constrain `OnDeleteStrategyFailure`'s type parameter.

17. **Restructure the integration test.** Split `tester` into per-feature functions with
    arrange/act/assert shape, collect failures instead of returning on the first, and extract
    the repeated result-handling helpers. Move what can be tested without a live database into
    `cargo test`.

18. **Address the examples.** The shadowed separation loop in `move_all_players` first, since
    it may be a real bug; then the phase extraction, the `unwrap` sweep, the `Div` contract in
    `math.rs`, and the naming. Decide and record whether these files are meant to track their
    upstream sample.

19. **Finish the small clarity items.** Convert `//` field comments to `///` on the public API
    structs, delete the redundant comments in `derive/src/lib.rs`, rename `rm_rsharp`, narrow
    `malformed_code_generation_result`'s visibility and simplify its loop, replace the
    speculative `syn::Result` returns, adopt `let … else` in `hook::build`, settle on one
    conditional idiom, prefer `!=`/`==` over `.ne`/`.eq`, correct the `debug-helper` usage
    string and the `format` help text, and resolve the `compare-performance.ps1` and `loc`
    inconsistencies.

20. **File issues for the untracked markers.** The remaining `TODO`/`FIXME` comments with no
    issue link, plus anything deferred out of the steps above, so nothing in this report is
    lost to the codebase.
