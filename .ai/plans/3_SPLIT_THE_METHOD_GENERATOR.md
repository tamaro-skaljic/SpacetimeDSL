# Plan 03 — Split the DSL method generator

Addresses items 9, 10, 12, 15, 16 and 17 of the Recommended Resolution Order, and the
`column_names_and_row_values` half of item 11. That order lived in
`CODE_QUALITY_REPORT.md` until its last findings moved into
[`4_FINISH_THE_GENERATOR_CLEANUP.md`](4_FINISH_THE_GENERATOR_CLEANUP.md) and it was deleted.

---

## Context

[`2_CLEAN_UP_METHOD_GENERATOR.md`](2_CLEAN_UP_METHOD_GENERATOR.md) landed in
`0923f2c`. It removed the dead scaffolding, unified the message formats, resolved
column-type classification, repaired the signatures and turned two proc-macro
panics into compiler diagnostics. What it deliberately did not do is change the
shape of the generator: `for_method` is still one function, and it is still
1 514 lines — `derive-input/src/internal/dsl/method.rs:608`–`2120`, out of a
3 448-line module.

Inside it, the same `dsl_method` value is matched exhaustively five separate
times — once for the doc comment, once for the method name, once for the return
type, once to separate `Update` from the rest, and once to pick the body — and
every one of those matches carries arms for variants the enclosing `match` has
already excluded. There are **ten** `panic!("… handled before this match")` arms.
The type system is asked a question it has already answered, and the answer is
discarded and re-derived four more times.

This plan inverts that. Each generated method gets one place that produces it
whole. `DSLMethod` disappears, because every call site already names its variant
as a literal and nothing outside `method.rs` ever mentions the enum — so the
"already processed" panics stop being unreached and become unwritable.

Four behavioural decisions ride along, because they live in exactly the code this
plan restructures: the update rule that is currently stated twice in two places
that disagree (item 16), the hash-index routing that decides which of those two
rules applies (item 17), the `delete = false` flag that suppresses nothing
(item 15), and the singleton contract that is spelled out by hand at five sites
(item 12). They are taken **first**, on today's code, so their diffs are small
and the snapshots they move are readable without refactoring noise. Everything
after them is strictly output-preserving.

The safety net is [`1_ADD_CHARACTERIZATION_TESTS.md`](1_ADD_CHARACTERIZATION_TESTS.md):
335 snapshots over the generated output and 14 `trybuild` cases over the
rejection diagnostics, all run by `./x.sh unit-test`.

### Findings this plan records that the report does not

The first three were found while planning. The fourth was found while implementing
step 1.2, and it revised one of the locked decisions below.

**Suppressing the strategy fanout breaks every child table.** The report's item 15
treats `delete = false` as a question about which methods are generated. It is
not only that. `for_referenced_by` does two jobs at once: it produces the method
*and* it registers the `compile_error_checks` entry that becomes a `pub trait` in
the parent's output (`derive/src/output.rs:135`). The child table's generated code
then writes `use <parent_path>::<that trait>;`. In the
`foreign_key_and_referenced_by` fixture, `Warehouse/table.snap:4`–`5` defines both
traits and `Shipment` and `Inspection` import them. Suppressing `for_referenced_by`
on a parent that sets `delete = false` therefore makes every child fail to compile
with `unresolved import … this_compilation_error_occurs_because_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`
— a message asserting the parent has no `#[referenced_by]`, which would be false.
The cross-table verification would misfire rather than verify. Step 1.4 rejects
the combination upstream instead.

**The update rule is not merely stated twice — it is the reason the two paths
disagree.** The report records the disagreement between `method.rs:161` and the
multi-column gate at `method.rs:371` as a defect to correct. Correcting both gates
leaves two gates. Step 6 removes the second site entirely, so the rule cannot be
stated twice again. Which way the rule was settled changed during implementation;
see the fourth finding.

**The singleton contract cannot be fully consolidated from `derive-input`.** The
report asks for the sentinel primary key's name, type, value and rendered form to
live in one place. Four of the five sites are in `derive-input` and this plan
unifies them. The fifth — the injection that *creates* the field — is
`derive/src/lib.rs:189`, in the other crate. Reaching it means widening
`derive-input`'s public API, which is the same boundary plan 2 hit for the runtime
paths and deliberately did not cross. Step 4 records the remaining site in a
comment on both sides rather than pretending the contract is closed.

### A fourth finding, from implementing step 1.2

**`update` is a primary-key-only operation, and that is why the two paths disagreed.**
Planning assumed SpacetimeDB offers `update` on any unique index, so item 16 was
settled as "a unique index of any width may update a row", with the generated body
keying on the index it is named for. It does not compile:

```text
error[E0277]: the trait bound
`entity::_::__indices::name1: spacetimedb::table::PrimaryKey` is not satisfied
help: the trait `spacetimedb::table::PrimaryKey` is not implemented for
      `entity::_::__indices::name1`
```

A `#[unique]` non-primary-key index carries `find` and `delete`, not `update`. The
unique multi-column path compiled only because it overrode the index name with the
primary key at `method.rs:1354`, which made its generated method a second name for
`update_<table>_by_<primary_key>`.

The 27 snapshot tests passed against the uncompilable emission, because they pin
tokens rather than compilability. `./x.sh test` caught it. This is the same gap
step 1.3 flags for hash indices, arriving one step early, and it is why the gate
runs all three commands rather than only `./x.sh unit-test`.

**The developer settled it the other way:** only the primary key may update, so the
multi-column path loses its update method rather than the single-column path
gaining one. Step 1.2 below records what was actually done.

Two consequences downstream:

- `for_update` now only ever sees the primary key index, plus the singleton. The
  `is_multi_column_index` index-name override at `method.rs:1354` is dead. Step 5.3
  deletes it rather than carrying it into the new generator.
- Step 6's shared builder gates `update` on `has_update_method` **and** on the index
  being the primary key.

---

## Locked decisions

Made explicitly by the developer during planning; do not re-litigate during
implementation.

| # | Decision |
| --- | --- |
| Scope | Items 9, 10, 12, 15, 16, 17 and the `column_names_and_row_values` half of 11. Items 13 and 14 keep their numbers and stay in the report. |
| Report | The violation sections this plan covers move into this file, as plan 2 did. Items 2–8 of the Recommended Resolution Order become `Done`; 9–17 become one-line pointers or stay open. |
| Commits | One commit per sub-change (the `### N.M` sections below), not one per step. |
| Gate | `./x.sh unit-test`, `./x.sh format` and `./x.sh test` all green before a step counts as done. `./x.sh test` needs a locally running `spacetime start`. |
| Step order | Output-changing steps first (step 1), then the structural work (steps 2–7) strictly output-preserving. |
| `DSLMethod` | Deleted entirely. No dispatcher, no replacement enum. Call sites call the generators directly. |
| Generator names | `for_create`, `for_get_all`, `for_get_count`, `for_get_many`, `for_delete_many`, `for_get_one`, `for_update`, `for_delete_one`, matching the existing `for_referenced_by` / `for_foreign_key` prefix. |
| Update rule | Only the primary key may update a row. The multi-column path drops its update method; the single-column path keeps its primary-key condition. **Revised during step 1.2** — see [A fourth finding, from implementing step 1.2](#a-fourth-finding-from-implementing-step-12). |
| Update body | Unchanged: `self.db().<table>().<primary_key>().update(row)`. No other shape is possible — SpacetimeDB defines `update` on the primary key index only. |
| Hash routing | `HashSingleColumn` joins the extraction loop in `internal/db/column.rs`. The `multi_column_indices` name is kept and the loop gains a guard, so a single-column index leaking into the list can no longer take the wrong path silently. |
| `delete = false` | Suppresses the delete methods. `#[referenced_by]` combined with `delete = false` becomes a rejection, spanned on the attribute, beside its sibling in `ReferencingTable::try_parse`. |
| Hash verification | A hash table in `examples/test`, exercised by the `tester` reducer. **Revised during step 1.3** — the planned `trybuild` pass case cannot work, because a pass case links a native executable and the generated code needs WASM-only host symbols (`LNK2019: unresolved external symbol datastore_table_scan_bsatn`). `compile_fail` cases never link, which is why the existing cases are unaffected. |
| Context | `MethodGenerationContext` holds the five references **plus** the names every generator re-derives, including `field_name_for_found_value`. Plain data carrier — no generation methods. |
| Mutation | Carried all the way out: generators return what they want contributed, `generate` takes `&SpacetimeDSLTable`, `internal/table.rs:39` applies the contributions, and `column::try_parse` and `SpacetimeDSLColumnMethods::map` drop their `&mut` too. |
| `IndexShape` | One struct, with the four prose fragments pre-assembled into the single phrase all five doc comments build identically today. Built by the caller, once per index, and passed as `&IndexShape`. |
| One-vs-many | The per-column argument loop is parameterised by `OneOrMultiple`, not by a new type — one row versus many rows is the concept `OneOrMultiple` exists for. |
| Singleton contract | A new `internal/dsl/singleton.rs`. Not the deferred item 13: it adds a module rather than carving up `method.rs`. |
| Singleton generators | `for_singleton_get` and `for_singleton_delete`, selected once where `map` already tests `is_singleton`. `Create` keeps its singleton branch inside `create_method_column_parts` — it differs by one column's treatment, not by body shape. **Revised during step 7.1** — `for_singleton_update` was planned too, but a singleton's update differs from the ordinary one by a doc comment, a method name and one statement, while sharing about 130 lines; the developer chose to leave `for_update` whole. See [7.1](#71-for_singleton_get-and-for_singleton_delete). |
| Column methods | One `column_methods_for` builder shared by `map` and the multi-column loop. |
| Delete paths | **Do not merge** `for_delete_one` and `for_delete_many`, and do not merge the singleton delete path into either. Every difference is intentional and required. See [Settled findings](#settled-findings-that-constrain-this-plan). |
| `OneOrMultiple` arms | **Do not merge** the `match one_or_multiple` arms in `get_referenced_table_function_call_for_dsl_method`, `for_referenced_by`, `for_foreign_key` or `get_on_delete_strategy_implementation`. |
| `OneOrMultiple` type | **Do not split** into separate row-arity and column-arity types. It is one type for one concept. |
| Module split | Out of scope. `method.rs` stays one file and stays roughly its current length. Item 13. |

---

## Settled findings that constrain this plan

These three findings were raised by the report, answered by the developer, and
moved here because this plan restructures the exact code they describe. They are
constraints on the work, not work items. An implementer who "helpfully" unifies
any of them during the split has broken the plan.

### `method.rs`: the `DeleteOne` and `DeleteMany` implementation assembly

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse; Connascence of
Algorithm

The two delete paths assemble their generated bodies from the same sequence of
stages: fetch the rows, return early if nothing matched, build deletion-result
entries, run the `Error` strategy, run the before-delete hook, delete, run the
after-delete hook, run the remaining strategies, return the result. Each path
duplicates that sequence, each duplicates the surrounding "has referencing tables
or not" split, and each duplicates the wrapper-type extraction and the on-error
handler construction.

They are not identical: the singular path carries a hard-coded
`count_of_rows_to_delete ( 1 )` message while the plural path formats real counts,
the entry container is a `Vec` in one and a `HashMap` in the other, and the
singleton sub-path of `DeleteOne` is a third, separately written variant that
skips the `Itertools` conditionality and hard-codes the primary key.

**Developer decision:** Every difference is intentional, required and deliberate,
nothing has drifted or is accidental. Steps 5.3 and 7.1 move these three bodies
into three functions; they do not merge them.

### `method.rs`: the `OneOrMultiple` branch pairs

**Violates:** Don't Repeat Yourself; Duplication Control & Reuse; Connascence of
Algorithm

`get_referenced_table_function_call_for_dsl_method`, `for_referenced_by`,
`for_foreign_key` and `get_on_delete_strategy_implementation` each contain a
`match one_or_multiple` whose two arms emit structurally parallel token streams
differing only in whether the accumulator is a `Vec` or a `HashMap` and whether
the body is wrapped in a loop.

**Developer decision:** Same as the delete paths — every difference is
intentional, required and deliberate.

### `method.rs`: `OneOrMultiple` used for two unrelated concepts

**Violates:** Connascence of Meaning; Principle of Least Astonishment; Hide
Implementation Details

`OneOrMultiple` means "one row or multiple rows" everywhere except in the `Update`
path, where it is derived from `is_multi_column_index` and passed to
`reference_integrity_checks_on_update` to select a format-string shape, meaning
"one column or multiple columns".

**Developer decision:** The type name contradicts nothing, as it does not state it
is for columns, rows, functions or whatever. `OneOrMultiple` is deliberately just
a differentiator between "One" and "Multiple" and will never change. Keep it one
type; do not introduce multiple types for the same concept. This is also why
step 5.1 parameterises the per-column argument loop with it.

---

## Expected snapshot movement

A step that is output-preserving must finish with `./x.sh unit-test` green and
**zero** pending snapshots. If a snapshot moves in one of those steps, the change
is wrong — do not accept the snapshot.

| Step | Output | May move |
| --- | --- | --- |
| 1.1 | **changing** | every doc comment in `hash_index/Session` |
| 1.2 | **changing** | 4 `.snap` files deleted and their 4 `table.snap` manifests (listed in 1.2) |
| 1.3 | preserving | **nothing** — see 1.3; a new hash table in `examples/test` is the verification |
| 1.4 | **changing** | `methods_disabled/AuditEntry` loses 2 snapshots; one new `.stderr` |
| 2–7 | preserving | **nothing** |

---

## Acceptance criteria

Each is one command, so "done" is not a judgement call. Run from the repository
root after step 7.

```sh
# 1. The god function and its enum are gone.
rg -n 'fn for_method|enum DSLMethod|DSLMethod::' derive-input/src        # 0 matches

# 2. No unreachable arm survives.
rg -n 'handled before this match' derive-input/src                       # 0 matches

# 3. The variant is never re-derived.
#    (Word-bounded: `create_dsl_method_arg`, `dsl_method_hooks_call` and
#     `get_referenced_table_function_call_for_dsl_method` are unrelated names.)
rg -nw 'dsl_method' derive-input/src                                     # 0 matches

# 4. The table is written in one place.
#    (`apply_to` is that place, and `internal/table.rs` is its only caller.)
rg -n '&mut SpacetimeDSLTable' derive-input/src                          # 1 match

# 5. The singleton magic values have one home in derive-input.
#    (10 matches today, all in method.rs. Do not widen this to a bare `"id"`:
#     internal/db/column.rs uses that string in an unrelated diagnostic.)
rg -n '0u8|\{ id : 0 \}|== "id"|eq\("id"\)' derive-input/src             # only internal/dsl/singleton.rs

# 6. The rule for which methods an index earns is stated once.
rg -n 'has_update_method' derive-input/src/internal/dsl/method.rs        # 1 match

# 7. The suite is green with nothing pending.
./x.sh unit-test && ./x.sh test && cargo insta pending-snapshots         # no pending
```

---

## Step 1 — The behaviour changes

Taken first, on today's code, so each diff is a few lines and the snapshots that
move are readable in isolation. Steps 2–7 are then verified against a stable
baseline where any movement at all is a defect.

### 1.1 Derive the index-type wording instead of hard-coding it

**Violates:** Robustness & Reliability (*Enforce strict output formats*); Code For
The Maintainer

`method.rs:887` and `method.rs:897` write the literal `"btree index"` in arms that
also handle `IndexType::HashSingleColumn` and `IndexType::HashMultiColumn`:

```rust
IndexType::BTreeSingleColumn { column } | IndexType::HashSingleColumn { column } => {
    index_documentation = "btree index".to_string();
```

Every doc comment the `hash_index` fixture generates therefore calls a hash index a
btree index, and that text is emitted into the documentation users read. There is
no decision attached — the text is simply wrong.

Split the two arms so each names its own kind, giving `"btree index"`,
`"hash index"` and the existing `"direct index"`. The multi-column arm becomes
``format!("{kind} index `{index_name}`")`` on the same basis.

**Moves:** every doc comment in `hash_index/Session`. No other fixture uses a hash
index, so nothing else can move. Rewrite the second bullet of the
`hash_index.rs` fixture header, which currently documents this as a known defect.

### 1.2 One rule for which unique indices may update a row

**Violates:** Principle of Least Astonishment; Don't Repeat Yourself (*One
authoritative source for each business rule*); Connascence of Algorithm

Both index shapes produce the same `SpacetimeDSLColumnMethodsForUniqueIndex`
value, and each decides independently whether to fill its `update` field. The
single-column path at `method.rs:161` requires
`has_update_method && method_is_for_primary_key`; the multi-column path at
`method.rs:371` requires `has_update_method` alone. So a row can be updated by a
unique multi-column index but not by a unique single-column one.

The fixtures show it directly: `unique_single_column_index` generates
`get_account_by_external_id` and `delete_account_by_external_id` but no
`update_account_by_external_id`, while `unique_multi_column_index` generates
`update_seat_by_row_and_number` alongside its getter and deleter. It holds across
the whole suite.

**Developer decision, revised during implementation:** only the primary key may
update a row. The first decision — a unique index of any width may update —
produced code that does not compile, because SpacetimeDB defines `update` on the
primary key index alone. See
[A fourth finding, from implementing step 1.2](#a-fourth-finding-from-implementing-step-12).

So the multi-column path loses its update method rather than the single-column path
gaining one. `SpacetimeDSLColumnMethods::map` keeps its `method_is_for_primary_key`
condition, and the multi-column loop in `SpacetimeDSLTableMethods::generate`
replaces its `has_update_method` gate with a plain `None` and a comment naming the
reason.

The `panic!("A column's own index is always a single-column index")` arm goes with
the change anyway: the multi-column loop no longer calls `for_method` with
`DSLMethod::Update`, so `method_is_for_primary_key` is the only `IndexType` match
left and it stays in the single-column path where every arm is reachable.

**Moves:** four snapshot files are deleted and four `table.snap` manifests lose one
line each. Nothing is added.

| Fixture / struct | Removed method |
| --- | --- |
| `unique_multi_column_index/Seat` | `update_seat_by_row_and_number` |
| `multiple_dsl_attributes/Module/pass_1` | `update_module1_by_database_and_name` |
| `multiple_dsl_attributes/Module/pass_2` | `update_module2_by_name_and_database` |
| `hash_index/Session` | `update_session_by_device_id` |

`hash_index/Session` is in the list because `device_id` is a single-column hash
index that today reaches the multi-column loop by the routing defect step 1.3
fixes. Its update method disappears now and does not come back after 1.3, which is
consistent either way.

Breaking for anyone calling the four removed methods. Nothing in `examples/`,
`docs/` or `README.md` did — verified by grep before the change.

**Also update:** `docs/DOCUMENTATION.md` states "Note that update methods are not
generated for unique single column indices as of SpacetimeDB 2.0", which names the
right rule for the wrong reason and omits multi-column indices. Replace it with the
primary-key-only rule and the reason, delete the "By Unique Multi-Column Index"
subsection, show the read-then-write-by-primary-key pattern instead, correct the
`update_entity_relationship_by_parent_child_entity_id` call in the unique-index
walkthrough, and change the method table's "Update by PK/unique" row to "Update by
PK". Rewrite the `unique_single_column_index.rs` fixture header, which documents
the old rule with a `(FIXME)`.

### 1.3 Route single-column hash indices like every other single-column index

**Violates:** Principle of Least Astonishment; Duplication Control & Reuse; Unused
scaffolding removed immediately

`internal/db/column.rs:61`–`79` walks the table's indices to find the one belonging
to the column being processed, and moves it onto that column as
`single_column_index`. The loop matches `BTreeSingleColumn` and `Direct`. It does
not match `HashSingleColumn`.

A `#[index(hash)]` column therefore never gets a `single_column_index`,
`SpacetimeDSLColumnMethods::map` returns `None` for it, and the index stays in
`multi_column_indices` — where the loop branches on `is_unique` alone, without
checking that the index has more than one column. The methods that come out are
named correctly, which is why this has gone unnoticed, but they are generated by
the wrong branch. The `HashSingleColumn` arm in `map` is consequently dead: it
tests whether a single-column hash index is the primary key, and no single-column
hash index ever reaches it.

Add `HashSingleColumn` to the extraction loop. Then gate the `multi_column_indices`
loop in `SpacetimeDSLTableMethods::generate` so an index that turns out to carry a
single column cannot silently take the multi-column path — the extraction loop
`break`s after the first match per column, so a column carrying two single-column
indices would still leak one into the list, and the guard makes the wrong path
unreachable rather than merely unvisited. Keep the list's name: after this change
it is accurate for every shape the fixtures cover.

**Taken after 1.2 deliberately.** This step decides which update rule applies to a
unique hash column. With the rule settled on the primary key, `device_id` has
already lost its update method in 1.2 and does not regain it here, so this step
moves only the getter and deleter bodies. Taken before 1.2, the same snapshot would
have moved twice.

**Verification.** No example module and no compile test *compiled* a hash index
before this step; the fixtures only snapshot tokens. A `trybuild` pass case cannot
close that gap: `trybuild::pass` links a native executable, and the generated code
reaches for WASM host imports that do not exist there, so it fails with
`LNK2019: unresolved external symbol datastore_table_scan_bsatn` and sixteen more.
The `compile_fail` cases are unaffected because they never reach the linker.

Add the table to `examples/test` instead. `cargo clippy --workspace --all-targets`
in `./x.sh format` type-checks it without linking, and `./x.sh test` publishes it
to WASM, where it links, and runs the `tester` reducer against it. Exercise all
three shapes: `filter` through the non-unique single-column index, `find` and
`delete` through the unique single-column one, and `filter` through the
multi-column one.

**Moves: nothing, and that is the expected result.** `for_method` already grouped
`HashSingleColumn` with `BTreeSingleColumn` when it decided the body shape, so the
multi-column loop and `map` emitted identical tokens for a single-column hash
index. The routing defect's only user-visible consequence was the update method,
and step 1.2 settled that.

Verify the routing took effect by proof rather than by diff: the guard now skips
both hash indices in `generate`'s loop, so if `hash_index/Session` still lists
`get_sessions_by_token`, `get_session_by_device_id` and their deleters, `map` must
be producing them. The `HashSingleColumn` arm in `map` is live for the first time.

### 1.4 `method(delete = false)` suppresses the delete methods

**Violates:** Principle of Least Astonishment; Code For The Maintainer (*names
state what the thing does*)

`#[dsl(method(delete = false))]` reads as the counterpart of `method(update = false)`,
and `update = false` does what it says: no setters, no update methods.
`delete = false` removes nothing. The fixture `derive/tests/fixtures/methods_disabled.rs`
declares both flags, and its `table.snap` manifest still lists
`delete_audit_entry_by_id` and `delete_audit_entries_by_actor_id`.

The flag reaches only two decisions, neither of which is method generation:
`internal/dsl/foreign_key.rs:115` rejects an `on_delete = Delete` foreign key while
it is set, and `internal.rs:173`–`186` reject a before- or after-delete hook.
`method.rs` never reads `has_delete_method` at all.

**Developer decision:** make the flag suppress the delete methods, matching its
name and its sibling. This is a breaking change for any module that sets
`delete = false` and calls a delete method today. `examples/test` sets the flag on
`HookCall` and never calls a delete method on it, so the example modules are
unaffected.

Three changes:

1. `SpacetimeDSLColumnMethods::map` and `SpacetimeDSLTableMethods::generate` skip
   `DeleteOne` and `DeleteMany` when `has_delete_method` is false. Note that
   `SpacetimeDSLColumnMethodsForIndex` and `…ForUniqueIndex` hold their delete
   method non-optionally, so the field becomes `Option<SpacetimeDSLMethod>` in both,
   and `derive/src/output.rs` handles the `None`.
2. The two `for_referenced_by` calls are skipped as well — nothing can reach the
   strategy fanout when the table has no delete method.
3. Because that would remove one half of the paired compile-error checks and break
   every child table's compilation (see *Three findings this plan records*), reject
   the combination instead: a table carrying `#[referenced_by]` must have a delete
   method. The check goes in `ReferencingTable::try_parse`, which already takes the
   field's attributes and already rejects `#[referenced_by]` without
   `#[primary_key]` with `syn::Error::new_spanned(attr, …)`. Pass `has_delete_method`
   in, exactly as `ForeignKey::try_parse` already takes it.

The two existing rejections stay: both remain correct and necessary, because an
`on_delete = Delete` foreign key and a delete hook both need a delete method that
no longer exists.

**Moves:** `methods_disabled/AuditEntry` loses `delete_audit_entry_by_id.snap` and
`delete_audit_entries_by_actor_id.snap`, and its `table.snap` manifest. One new
`compile-tests/tests/ui/referenced_by_without_delete_method.rs` and its `.stderr`.
Rewrite the `methods_disabled.rs` fixture header, which currently documents the
old behaviour with a `(FIXME)`. Update `docs/DOCUMENTATION.md` wherever it
describes `method(delete = …)`.

---

## Step 2 — Stop mutating the table, and name the parameter bundle

Both halves of item 10. Taken before the split so the eight generators are written
once against their final signature, rather than written with six positional
parameters and re-threaded afterwards.

### 2.1 Return what the generators want recorded

**Violates:** Command Query Separation; *Design APIs without cross-cutting side
effects*; Principle of Least Astonishment; Hide Implementation Details

`for_method`, `for_referenced_by` and `for_foreign_key` read as queries — they are
named for what they produce and they return a `SpacetimeDSLMethod`. They also
quietly write into the table they were handed: `for_method` installs
`create_dsl_method_arg` at `method.rs:710`, and the foreign-key and referenced-by
generators insert into `compile_error_checks` at `method.rs:2646` and
`method.rs:2926`. Nothing in the names or return types signals this. A caller that
reorders two seemingly independent generator calls can change the resulting table
state.

`AGENTS.md` is explicit that CQS wins over Minimize Coupling for internal methods,
and these are internal methods.

Make the writes part of the return value. Each generator returns its produced
method together with whatever it wants contributed, and the single orchestrating
caller applies them:

```rust
struct TableContributions {
    create_dsl_method_arg: Option<CreateDSLMethodArg>,
    compile_error_checks: BTreeSet<Ident>,
}
```

`SpacetimeDSLTableMethods::generate` then takes `&SpacetimeDSLTable` and returns
`syn::Result<(SpacetimeDSLTableMethods, TableContributions)>`;
`internal/table.rs:39` applies the contributions to the table it already owns. This
also removes the reason `generate` currently takes the table by value and hands it
back, which plan 2 deferred to here.

Carry the same through the column side: after the split, `SpacetimeDSLColumnMethods::map`
never generates `Create` and therefore never writes, so it takes
`&SpacetimeDSLTable`, and `internal/column.rs:try_parse` drops its `&mut` with it.
After this sub-change, `SpacetimeDSLTable` is immutable everywhere after
construction except in `internal/table.rs`.

**Ordering note:** this must land before 2.2. A `MethodGenerationContext` holding
`&SpacetimeDSLTable` cannot coexist with a later `&mut` borrow of the same table,
so the mutation has to go first.

### 2.2 One named context instead of five positional parameters

**Violates:** Connascence (connascence of position, high degree, non-local);
Minimize Coupling; Law of Demeter; Code For The Maintainer

`rust_struct`, `spacetimedb_table`, `spacetimedsl_table`, `internal_columns` and
`primary_key_column` travel together as a positional bundle through the entry
points, the method generator and most helpers. Because they are positional and
several are references to different-but-similar table types (`SpacetimeDBTable`
versus `SpacetimeDSLTable`), a transposed argument would compile in some call
shapes. Adding one more piece of shared context means editing every signature and
every call site in the chain — a wide blast radius for what is conceptually a
no-op. Splitting `for_method` into eight functions would multiply that bundle by
eight.

Introduce one context and pass `&context`. It converts connascence of position
into connascence of name, which `AGENTS.md` explicitly prefers.

```rust
struct MethodGenerationContext<'a> {
    rust_struct: &'a RustStruct,
    spacetimedb_table: &'a SpacetimeDBTable,
    spacetimedsl_table: &'a SpacetimeDSLTable,
    internal_columns: &'a [InternalColumn],
    primary_key_column: &'a InternalColumn,

    struct_name: Ident,
    singular_table_name: Ident,
    singular_table_name_as_string: String,
    singular_table_name_pascal_case: String,
    plural_table_name: Ident,
    primary_key_column_name: Ident,
    primary_key_column_name_as_string: String,
    field_name_for_found_value: Ident,
}
```

The derived names are precomputed in the constructor, not exposed as methods. It
stays a plain data carrier — it must not grow generation methods, or it becomes a
second god object.

The precomputed `field_name_for_found_value` is the point of including them:
`format_ident!("the_same_or_another_{…}")` is spelled out identically in
`for_method`, `reference_integrity_checks_on_update`, `multi_column_index_checks`
and `get_unique_multi_column_index_check` today. That is connascence of algorithm
across four functions, and one field removes it.

---

## Step 3 — Extract the shared index analysis

### 3.1 `IndexShape`

**Violates:** Single Responsibility Principle; Don't Repeat Yourself; Maximize
Cohesion

`method.rs:867`–`948` derives everything the five index-based variants need from an
`&Index`: the column list, whether the index is multi-column, the singleton-primary-key
flag, the format string, and four prose fragments. It sits inside `for_method`, so
the split cannot happen until it is a value of its own.

The four prose fragments are then re-assembled into the identical trailing phrase
by all five doc comments at `method.rs:955`–`998`. Assemble it once:

```rust
struct IndexShape {
    index_columns: Vec<Ident>,
    is_multi_column: bool,
    is_unique: bool,
    is_singleton_primary_key: bool,
    index_name: Ident,
    /// `{{ a : {}, b : {} }}`
    column_names_and_row_values: String,
    /// "whose value matches the value from the single-column btree index on the `x` column"
    described_as: String,
    /// The experimental-feature warning, or empty.
    unique_multi_column_hint: String,
}
```

Built by the caller, once per index, and passed as `&IndexShape`: one index drives
up to three generators, so building it at the call site makes the shared analysis
genuinely shared instead of recomputed three times.

### 3.2 One `column_names_and_row_values` builder

**Violates:** Don't Repeat Yourself (*One authoritative source for each business
rule*); Duplication Control & Reuse; Connascence of Algorithm

The format string that describes "these columns had these values", used in
`NotFoundError` and `UniqueConstraintViolation` messages, is built by imperative
string pushes in two places: once inside `for_method` (with a branch per
`IndexType`) and again inside `multi_column_index_checks` at `method.rs:2427`–`2451`.
The placeholder-to-argument correspondence is maintained by hand in parallel with a
separately built list of row-value getters, so nothing keeps the placeholder count
and the getter list in step.

Plan 2 made the two agree on the shape `docs/DOCUMENTATION.md` documents, but left
them as two builders. Extract the one 3.1 already needs:

```rust
fn column_names_and_row_values(column_names: &[Ident]) -> String
```

and have `multi_column_index_checks` call it with the same ordered column list it
iterates to build its getters. The two can then no longer disagree, because the
placeholder count and the getter count are produced from one list.

The report asks for a builder that emits the format string *and* the matching
getter list as one value. That is possible for `multi_column_index_checks`, whose
getters are derived from the columns, but not for the `for_method` path, whose
getters come from the method's arguments rather than from a row. Sharing the
ordered column list is the strongest coupling the two sites can honestly share.

**Note:** this is the whole of item 11 that this plan takes. The rest of item 11 —
the delete paths, the `OneOrMultiple` arms, the `OneOrMultiple` overload — is
settled as "no change"; see [Settled findings](#settled-findings-that-constrain-this-plan).

---

## Step 4 — Define the singleton contract

### 4.1 `internal/dsl/singleton.rs`

**Violates:** Encapsulate What Changes; Open/Closed; Don't Repeat Yourself;
Connascence of Value

The singleton table concept is special-cased independently in the column-method
mapper (`method.rs:116`), the table-method entry point (`method.rs:216`, `229`),
the create column processor (`method.rs:441`–`458`, where the primary key is
identified by the literal field name `"id"`, the literal type `"u8"`, and filled
with `0u8`), the method generator (`method.rs:943`, a dedicated `is_singleton_pk`
flag driving separate doc comments, method names, get, update and delete bodies,
plus `.id().find(&0u8)`, `.id().delete(&0u8)`, `id = 0u8` and the literal message
`"{ id : 0 }"`), and the on-delete strategy generator (`method.rs:2995`–`3040`, a
different row finder and a different row-value format).

Changing anything about how singletons are represented therefore requires finding
and editing every one of these sites, and missing one produces inconsistent
generated code rather than a compile error.

Define the contract once:

```rust
//! internal/dsl/singleton.rs
//!
//! The injected primary key every singleton table carries. The field itself is
//! created in the `derive` crate (`derive/src/lib.rs:189`); this module is the
//! definition everything in `derive-input` reads, so the two must agree.

pub(in crate::internal) const PRIMARY_KEY_NAME: &str = "id";
pub(in crate::internal) fn primary_key_ident() -> Ident;
pub(in crate::internal) fn primary_key_value() -> Literal;               // 0u8
pub(in crate::internal) fn rendered_primary_key_value() -> String;       // "0"
pub(in crate::internal) fn rendered_primary_key() -> String;             // "{ id : 0 }"
pub(in crate::internal) fn is_primary_key_column(name: &Ident, type_name_or_path: &Path) -> bool;
```

and have `internal/dsl/method.rs` read from it.

**Amended while implementing.** The sketch above named a `primary_key_type()`
emitter, but nothing in `derive-input` ever writes the injected key's type — the
create path only asks whether a column *is* that key, which
`is_primary_key_column` now answers, keeping both the name and the type literal
inside the module. The value is a `Literal` rather than a `TokenStream` so the
module can build `0u8` from the `u8` it means, and the rendered forms are `String`
for the same reason: `"{ id : 0 }"` is assembled from the name and the value rather
than written out a second time.

`internal/db/column.rs` and `internal/dsl/column.rs` turned out to have nothing to
read. Both special-case singletons, but they do it through `is_singleton` and
`is_primary_key` flags alone — neither names the injected column, its type or its
value. The `"id"` in `internal/db/column.rs:32` is the *suggestion* an error
message makes when a non-singleton's primary key is prefixed with its table's name;
it is the same word for an unrelated rule, which is why acceptance criterion 5 does
not match it.

**Taken before the split deliberately** — for the same reason plan 2 resolved
column-type classification before its own restructuring: the split must not carry
the magic values into eight new functions.

**The contract is not fully closed.** The fifth site, `derive/src/lib.rs:189`,
injects the field and lives in the other crate; reaching it means widening
`derive-input`'s public API, which is the boundary plan 2 deliberately did not
cross for the runtime paths. Put a comment on each side naming the other, so the
pairing is discoverable, and leave the crossing to the same later decision that
settles the runtime-path surface.

---

## Step 5 — One generator per `DSLMethod` variant

**Violates:** Single Responsibility Principle; Curly's Law; Maximize Cohesion;
KISS; Code For The Maintainer; Open/Closed

`for_method` generates every DSL method variant. It is the overwhelming majority of
the module. Inside it, control flow nests to a depth where the reader cannot tell
which branch they are in without scrolling back, and the innermost `quote!` blocks
sit far from the condition that selects them.

The function also uses the declare-now-assign-deep-inside-a-branch pattern for
`doc_comment`, `method_name`, `return_type` and `method_impl`. The compiler
enforces that every path assigns them, so the code is correct, but a reader tracing
"what is `method_impl` for a unique multi-column `DeleteOne`" must locate one
assignment among many scattered across the branch tree. This is a puzzle the
maintainer solves by hand on every visit.

Additionally, the same `dsl_method` value is re-matched exhaustively five separate
times, and every one of those matches carries `panic!` arms for variants that the
enclosing `match` has already excluded — ten of them. The type system is being
asked a question it already answered, and the answer is discarded and re-derived.

Invert the structure: give each variant a single place that produces a whole
`SpacetimeDSLMethod`. `DSLMethod` is deleted rather than reshaped, because every
call site already names its variant as a literal and no code outside `method.rs`
mentions the enum. With no enum there is no dispatcher and no re-match, so the
"already processed" panics become impossible to express rather than merely
unreached.

### 5.1 One index-column argument builder

`method.rs:1234`–`1437` loops over the index columns and produces, per column, a
`SpacetimeDSLArg`, a row-value getter and an optional wrapper-option mapper. It
branches on `dsl_method` in four places, and in every one of them the
`GetMany`/`DeleteMany` arms are identical to each other and the
`GetOne`/`DeleteOne` arms are identical to each other. The real parameter is one
row versus many rows:

| | many rows | one row |
| --- | --- | --- |
| plain column | `&'a T` | `&T` |
| wrapped column | `impl Into<W>` | `impl Into<W> + Clone` |
| wrapped getter | `c.into().value()` | `c.clone().into().value()` |
| single-column `String` getter | `c` | `c.to_string()` |

Extract it, parameterised by `OneOrMultiple` — the type the developer keeps for
exactly this purpose. The two `panic!` arms for `Update` and for the table-wide
variants disappear with the `dsl_method` match.

```rust
struct IndexColumnArguments {
    method_args: Vec<SpacetimeDSLArg>,
    row_value_getters: Vec<TokenStream>,
    wrapper_option_mappers: Vec<TokenStream>,
}

fn index_column_arguments(
    shape: &IndexShape,
    one_or_multiple: &OneOrMultiple,
    context: &MethodGenerationContext,
) -> IndexColumnArguments
```

Keep the `is_singleton_primary_key` suppression that currently guards the pushes at
`method.rs:1433`: a singleton's get/update/delete take no index arguments. Step 7
removes the need for it.

### 5.2 The three table-wide generators

Lift the `Create`, `GetAll` and `GetCount` arms out as `for_create`, `for_get_all`
and `for_get_count`. Each owns its own doc comment, method name, return type,
`read_context_compatible` value and body, as literals rather than as arms of five
separate matches.

`for_create` returns its `CreateDSLMethodArg` alongside the method, per 2.1. It
keeps its singleton handling inside `create_method_column_parts`, which now reads
the contract from `singleton.rs`: the create path differs for a singleton by one
column's treatment, not by body shape, so it does not earn a generator of its own.

### 5.3 The five index generators

Lift the rest out as `for_get_many`, `for_delete_many`, `for_get_one`, `for_update`
and `for_delete_one`, each taking `(&IndexShape, &MethodGenerationContext)` and, for
the four that need it, the `IndexColumnArguments` from 5.1.

`for_update` is the exception to "move only": after step 1.2 it only ever sees the
primary key index, so the `is_multi_column_index` override that swapped in the
primary key at `method.rs:1354` is dead. Delete it rather than carrying it into the
new generator, and check whether `IndexShape::is_multi_column` still has a reader
in `for_update` afterwards.

Do **not** merge `for_delete_one` and `for_delete_many`, and do not restructure
their bodies while moving them; see
[Settled findings](#settled-findings-that-constrain-this-plan). Apart from the
override above, this sub-change moves code and rewrites its selection, nothing
more.

### 5.4 Delete `DSLMethod` and `for_method`

Rewrite the call sites in `SpacetimeDSLColumnMethods::map` and
`SpacetimeDSLTableMethods::generate` to call the generators directly, then delete
the enum, the dispatcher-shaped function, and all ten
`panic!("… handled before this match")` arms.

Acceptance criteria 1, 2 and 3 must pass after this sub-change.

---

## Step 6 — One statement of which methods an index earns

### 6.1 `column_methods_for`

**Violates:** Don't Repeat Yourself (*One authoritative source for each business
rule*); Connascence of Algorithm

After step 1, `SpacetimeDSLColumnMethods::map` and the multi-column loop in
`SpacetimeDSLTableMethods::generate` apply the same rule: a non-unique index earns
`get_many` and `delete_many`; a unique index earns `get_one_option` and
`delete_one`, plus `update` only when that index is the primary key. `delete` is
gated on `has_delete_method` and `update` on `has_update_method`. Today that rule
is written out twice, in two `match index.is_unique` blocks that construct the same
two struct variants. Writing it twice is what let the two copies disagree and
produced the item-16 defect in the first place.

Step 1.2 corrected both sites but left two sites. Remove the second one:

```rust
fn column_methods_for(
    index: &Index,
    context: &MethodGenerationContext,
) -> SpacetimeDSLColumnMethods
```

Both call sites call it. `map` keeps only its `single_column_index` lookup and,
until step 7, its singleton branch. This is the durable half of item 16 — the
correction in 1.2 fixes today's output, and this makes tomorrow's disagreement
unrepresentable.

Acceptance criterion 6 must pass after this sub-change.

---

## Step 7 — Separate the singleton generators

### 7.1 `for_singleton_get` and `for_singleton_delete`

**Violates:** Encapsulate What Changes; Open/Closed; Maximize Cohesion

After step 5, `is_singleton_primary_key` is still re-tested inside `for_get_one`,
`for_update` and `for_delete_one` — in the doc comment, in the method name and in
the body, three times each. The two branches it selects between share almost
nothing: the singleton bodies take no method arguments, build no wrapper mappers,
collect no row-value getters, run no multi-column check, and use a hard-coded
`.id().find(&0u8)` finder with the literal `rendered_primary_key()` message.

They are already two generators nested in one function. Separate them, and select
once where `map` already tests `is_singleton`:

```rust
match context.spacetimedsl_table.is_singleton {
    true => SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
        get_one_option: for_singleton_get(context),
        update: /* gated on has_update_method */,
        delete_one: /* gated on has_delete_method */,
    }),
    false => column_methods_for(index, context),
}
```

**Amended while implementing: `for_update` is not split.** The paragraph above
describes `for_get_one` and `for_delete_one` correctly and `for_update` not at all.
A singleton's update does take a method argument — the row — and it needs every
part of the ordinary update: the foreign-key row-value getters, the on-update
timestamp column, the reference-integrity checks, the found-value prelude and both
hooks. Its multi-column index check is the only part that is always empty, because
plan 2's step 8 rejects a singleton carrying a multi-column index. What actually
differs is the doc comment, the method name and one `entity.id = 0u8;` statement.
Splitting it would copy about 130 lines to change three, so the singleton update is
the ordinary `for_update`, called from the same selection, and
`is_singleton_primary_key` stays on `IndexShape` with `for_update` as its one
reader.

The suppression in 5.1's argument builder goes regardless: the two generators that
kept it are the two being split out, and no singleton reaches the builder any more.
`for_get_one` and `for_delete_one` lose their singleton branches entirely.

A singleton reaches only these three: its injected primary key is unique, so the
non-unique branch was already unreachable for it, and `internal/db/column.rs`
rejects `#[index]`/`#[unique]` on a singleton's own columns while plan 2's step 8
rejects a singleton carrying a multi-column index.

The on-delete strategy generator (`method.rs:2995`–`3040`) keeps its `is_singleton`
branches. It generates a strategy body, not a DSL method, and it is not part of the
per-variant split; it reads the contract from `singleton.rs` after step 4 and is
otherwise untouched.

Acceptance criterion 5 must pass after this sub-change.

---

## Verification

Per sub-change commit:

```sh
./x.sh unit-test   # insta snapshots + trybuild diagnostics
./x.sh format      # cargo fmt --all + workspace clippy
./x.sh test        # publishes both example modules against a local spacetime start
```

For an output-preserving sub-change — everything from step 2 onward —
`./x.sh unit-test` must leave **zero** pending snapshots. Do not run
`cargo insta accept`. If a snapshot moves, the change is wrong.

For the four output-changing sub-changes in step 1, run `cargo insta review` and
read every diff against the "may move" column before accepting. Regenerate
`.stderr` files with `TRYBUILD=overwrite ./x.sh unit-test` and read those diffs
too.

Run the [acceptance criteria](#acceptance-criteria) after step 7.

## What this plan does not cover

- **Splitting the module into domain modules (item 13).** `method.rs` ends this
  plan holding eleven readable generators instead of one unreadable one, but it
  does not get shorter — it stays around 3 400 lines. The seams the split needs are
  clean after this plan; carving the file is the next piece of work, together with
  regrouping the long paired field names on `SpacetimeDSLTableMethods` behind small
  structs.
- **The `TODO`/`FIXME` markers (item 14).** The split moves several of them; it
  does not triage them or file their issues. The swallowed hook error among them is
  a behavioural defect, not a cleanup.
- **Merging the delete paths, the `OneOrMultiple` arms, or splitting the
  `OneOrMultiple` type.** Settled as "no change"; see
  [Settled findings](#settled-findings-that-constrain-this-plan).
- **Closing the singleton contract across the crate boundary.** Four of the five
  sites are unified in step 4; `derive/src/lib.rs:189` is not, because reaching it
  means widening `derive-input`'s public API. It belongs with the same decision
  that settles the runtime-path surface.
- **The runtime-path literals outside `method.rs`**, and **migrating the crate's
  existing `Span::call_site()` diagnostics to spanned ones** — both still open in
  the report.
