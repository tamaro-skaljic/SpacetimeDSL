# Emit Even If `#[dsl]` Is Rejected — Design

- **Issue:** [tamaro-skaljic/SpacetimeDSL#192](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/192)
- **Date:** 2026-10-28
- **Status:** draft for review, not built. Once it's approved, `implementing-specs` builds it.

## Intent

When the `#[spacetimedsl::dsl]` of one table is rejected, the compiler shows why. At each place that depends on that table, it shows one error that points back at it. Today the useful error drowns: ten tables that reference a rejected `server` table add about twenty `E0599`s about methods that `server` would have generated.

The checks that pair a `#[foreign_key]` with its `#[referenced_by]` become trait checks. Their messages say what to change, and they're spanned on the attribute that has to change. Today they're unresolved imports, spanned on `#[dsl]`, of names such as `this_compilation_error_occurs_because_your_foreign_key_referencing_the_warehouse_table_needs_to_define_a_strategy_for_on_delete_or_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`.

## The Output It Aims For

When the `#[dsl]` of the `server` table is rejected:

1. The `syn::Error` that rejects it, at the span it has today. As today, only the first one.
2. One error on each `#[foreign_key]` that references `server`, saying that `server` has to be fixed first. A table with two foreign keys to `server` gets two.
3. One error on each `#[referenced_by]` that names a rejected table, saying the same about that table.
4. Nothing else from generated code, and no chain reaction. A `message` table that references the accepted `channel` table, which references `server`, gets no error.

Hand-written code that calls a rejected table's DSL methods, such as `dsl.get_server_by_id(…)` in a reducer, keeps reporting `E0599`.

Some rejections are ones that SpacetimeDB shares, such as a column of an array or tuple type. These also show SpacetimeDB's own errors on that column, because the struct comes back with `#[spacetimedb::table]`.

The shape of the second error, adapted from the spike's output to this example. The wording is a draft. Where the `help:` line points depends on how SpacetimeDB spans `server__TableHandle`, which the spike didn't cover.

```txt
error[E0277]: the `server` table can't be referenced, because its `#[spacetimedsl::dsl]` is rejected
  --> src/channel.rs:14:53
   |
14 |         #[foreign_key(path = crate::server, table = server, column = id, on_delete = Delete)]
   |                                                     ^^^^^^ fix the error of the `server` table first
   |
help: the trait `ServerTableIsAccepted` is not implemented for `server__TableHandle`
  --> src/server.rs:8:5
   |
 8 |     #[spacetimedb::table(accessor = server, public)]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: this error originates in the attribute macro `spacetimedb::table` (in Nightly builds, run with -Z macro-backtrace for more info)
```

## Goals, Non-Goals and Anti-Goals

**Goals**

- The four points of *The Output It Aims For*.
- The pairing checks become traits with `#[diagnostic::on_unimplemented(message, label, note)]`, with both table names filled in, spanned on the attribute they check.
- One rule per check, and each broken rule gives exactly one error.
- The long marker traits and their `const _: () = { use …; };` imports go.

**Non-goals**

- Quieting hand-written code that calls a rejected table's DSL methods.
- Reporting more than the first `syn::Error` of one `#[dsl]`.
- Compilers older than rustc 1.95. They're below `rust-version` anyway.
- Changing what the pairing accepts or rejects. Only the form, the wording and the span of its errors change. There is one exception: tables that are paired across modules have to be visible to each other, as *Visibility* says.

**Anti-goals**

- A false pairing error after a rejection, such as "the `server` table has no `#[referenced_by]` naming the `channel` table" when `server` is only rejected.
- An error that names only generated items and doesn't say what to change.
- Stand-ins that are generated per table for the rejection path, such as items with `unreachable!()` bodies. One shared default body in `spacetimedsl!()` that runs in no program that compiles is fine, as *The Checks* says.

## How Rustc Behaves Today

- Since rustc 1.95, a module that contains a `compile_error!` suppresses the errors about items that are missing in it, and about paths into it: a `use`, a type, a function, `Type::function()`, an impl header ([rust-lang/rust#149484](https://github.com/rust-lang/rust/pull/149484), pinned by rustc's [`tests/ui/macros/compile_error_macro-suppress-errors.rs`](https://rust.googlesource.com/rust/+/HEAD/tests/ui/macros/compile_error_macro-suppress-errors.rs)).
- `rust-version = "1.97.1"` in `Cargo.toml`, so every supported compiler does this.
- The suppression covers name resolution only. A method that is missing on a type that exists is still reported, and so is an unsatisfied trait bound.
- `DSL`, `ReadOnlyDSL` and `DSLInternals` are declared by `spacetimedsl!()` in `crate::spacetimedsl`, so they always exist. A table that calls a method on them, which a rejected table would have generated, gets 2 × `E0599`. All 14 compile tests where one table is rejected and another one reacts show exactly this, and nothing else.

## Approach

The design relies on that suppression, and the user chose it. The alternative was dropped: a rejected table emitting stand-ins for everything that other tables use. It generates code on a path that never runs, and needs a stand-in for every new cross-table item. Its one advantage, not depending on rustc 1.95, is covered by `rust-version`.

### What a Rejected `#[dsl]` Emits

- Its `compile_error!`.
- **A struct that carries `#[spacetimedb::table]`** also comes back, as the expansion holds it when the rejection happens:
  - It has the derive that declares the DSL's field attributes, `derive_table_helper_attr()` in `derive/src/lib.rs`, so that `#[foreign_key]` and the other field attributes stay valid.
  - A singleton keeps its injected `#[primary_key] id: u8` once the injection has happened. Without it, the user's `impl DefaultSingleton { … id: 0 … }` would fail with `E0560`, which points at the wrong fix.
  - `#[spacetimedb::table]` and every other attribute stay, so `<table>__TableHandle` and `ctx.db.<table>()` still exist.
- **Any other item** gets only its `compile_error!`, as today. Without `#[spacetimedb::table]`, nothing would declare `#[primary_key]` and SpacetimeDB's other field attributes. SpacetimeDB refuses a table that isn't a struct.
- Nothing that the parse of the DSL's arguments produces: no wrapper types, no methods, no hidden type, no gate impl, no checks.

### What an Accepted `#[dsl]` Emits in Addition

- **The gate impl:** `impl <gate trait> for <table>__TableHandle {}`. The handle exists after a rejection too, so a handle without the gate impl belongs to a rejected table.
- **The hidden type:** one per table, named from the `singular_table_name` in PascalCase, such as `ServerTable`. It is `pub`, so that every module can name it, and `#[doc(hidden)]`. Only an accepted table emits it. The pairing traits are implemented for it, and every cross-table call goes through it. After a rejection, every path to it is suppressed.

### Cross-Table Calls

Wherever the generated code of one table uses an item that another table's expansion generates, the use moves off the shared `DSL`, `ReadOnlyDSL` and `DSLInternals` types, onto a path through that table's hidden type:

- The existence checks of `create_*`, `update_*_by_*` and `upsert_*`. Today they call `get_<referenced table>_by_<primary key>` on `DSL`.
- The referenced-row methods, such as `ShipmentId::get_origin_warehouse`, which call the same method.
- The cascade entry points. The referencing table emits them on `DSLInternals`, such as `execute_on_delete_strategies_of_the_shipment_table_after_one_row_of_the_warehouse_table_was_deleted`, and the referenced table's dispatchers call them.

The moved calls keep the arguments that they take today:

- A lookup takes what `lookup_argument_from_row` and `lookup_argument_from_wrapper` in `derive-input/src/internal/dsl/method/index.rs` give it: a `&str` for a `String` primary key, and the wrapper otherwise. So a shared lookup trait has to accept both.
- A cascade entry point takes keys cloned from the rows, because a `String` key isn't `Copy`.

Some items stay as they are, because a path into a rejected table's module is suppressed already:

- items that the user names, such as the wrapper type in `#[use_wrapper(crate::server::ServerId)]`, and the wrapper methods that a referencing table adds to it;
- items that SpacetimeDB generates, such as `<path>::<table>__TableHandle`.

### The Checks

| On | Check | Fails when |
| --- | --- | --- |
| each `#[foreign_key]` | the referenced table is accepted | its `#[dsl]` is rejected |
| each `#[foreign_key]` | the referenced table names this table | its primary key has no `#[referenced_by]` naming this table |
| each `#[foreign_key]`, per removal | the strategy can apply | `on_delete` is set, but the referenced table has no delete method; or `on_soft_delete` is set, but it isn't soft-deletable |
| each `#[foreign_key]`, per removal | the strategy is set | the referenced table has a delete method, but `on_delete` is missing; or it's soft-deletable, but `on_soft_delete` is missing |
| each `#[referenced_by]` | the named table is accepted | its `#[dsl]` is rejected |
| each `#[referenced_by]` | the named table has a foreign key back | it has no `#[foreign_key]` referencing this table |

- **One rule per check.** The "or" in today's names goes. A foreign key whose referenced table doesn't name its table fails only the second check.
- **The rules keep their meaning.** What is accepted and what is rejected doesn't change, apart from *Visibility*. Each foreign key is still checked on its own. "Has a delete method" is still the resolved `has_delete_method`. A table that neither deletes nor soft-deletes rows is still paired the same way.
- **After a rejection, only the gates fail.** The other checks name the rejected table's hidden type, which is missing.
- **One error per broken rule.** A cross-table call must not add a second error for a rule that a check reports already. Today a foreign key that lacks a strategy gives 3 errors (`foreign_key_without_strategy_to_deletable_table.stderr`). The spike showed a second case: a cascade call into a table without a foreign key back failed with `the trait bound … is not satisfied`, beside the check's own error. The two cases are handled differently:
  - **A foreign key without a strategy that its referenced table needs.** The shared cascade trait, declared by `spacetimedsl!()`, has a default body for every removal: `unreachable!` with a message. A referencing table implements the trait for every table that it has a foreign key to, and overrides only the removals that it declares a strategy for. The referenced table's call then compiles, and only the pairing check fails. The default body runs in no program that compiles, because the pairing check fails exactly when the default would be called.
  - **A `#[referenced_by]` naming a table without a foreign key back.** That table doesn't implement the cascade trait for this table at all. The referenced table's calls into one referencing table go through one checked call, and that call is the check "the named table has a foreign key back" itself. A missing foreign key then fails there once, with the check's message. An example is a generic function that is instantiated once per referencing table, bounded by the check's trait.

### Visibility

SpacetimeDB gives `<table>__TableHandle` the visibility of the table's struct (`#vis struct #tablehandle_ident` in `spacetimedb-bindings-macro-2.13.0/src/table.rs`). The gate of a `#[foreign_key]` names the handle of the referenced table, and the gate of a `#[referenced_by]` names the handle of the referencing table.

So tables that are paired across modules have to be visible to each other. Otherwise they fail with `E0603`. This is a newly rejected input:

- Today a referenced struct has to be visible already, for the referenced-row methods, unless `referenced_row_method = false`.
- A referencing struct doesn't have to be visible today.

`docs/MIGRATION.md` records it under *Newly rejected inputs*, with what to write instead: `pub(crate)` or `pub` on the struct. `### Pairing Requirement` in `docs/DOCUMENTATION.md` states it as a rule.

### The Form of a Check

Each check is a trait with `#[diagnostic::on_unimplemented(message = …, label = …, note = …)]`. It is declared in the checking table's `const _: () = { … };` block, with both table names written into its text. A blanket impl, marked `#[diagnostic::do_not_recommend]`, implements it for every type that implements a shared trait. The shared trait carries the fact. The local trait carries the words.

The check is written as `const _: () = <subject as Check>::HOLDS;`, or as a `where` clause on a function. Every token of that line is spanned on the `table = …` value of the attribute. A generic function `fn holds<T: Check>()` works as well, but adds a note `required by a bound in holds` that points at the macro.

The local trait's name shows in rustc's `help:` line, so it has to read well, such as `ServerTableIsAccepted`.

### Messages

A draft, for the builder to refine. `label` says what to change in the attribute that it underlines. `note` says what to write where the fix lies in the other table. A table doesn't know its own module path, so a `note` writes `path = …`.

| Check | `message` | `label` | `note` |
| --- | --- | --- | --- |
| accepted, on `#[foreign_key]` | the `server` table can't be referenced, because its `#[spacetimedsl::dsl]` is rejected | fix the error of the `server` table first | — |
| names this table | the `warehouse` table has no `#[referenced_by]` naming the `shipment` table | the `warehouse` table doesn't know about this foreign key | add `#[referenced_by(path = …, table = shipment)]` to the primary key column of the `warehouse` table |
| strategy can apply | the `warehouse` table has no delete method, so `on_delete` can't apply | remove `on_delete` | — |
| strategy is set | the `warehouse` table has a delete method, so this foreign key needs `on_delete` | add `on_delete = …` | `on_delete` takes `Error`, `Delete`, `SoftDelete`, `SetZero` or `Ignore` |
| accepted, on `#[referenced_by]` | the `channel` table can't reference the `server` table, because its `#[spacetimedsl::dsl]` is rejected | fix the error of the `channel` table first | — |
| foreign key back | the `shipment` table has no `#[foreign_key]` referencing the `warehouse` table | remove this `#[referenced_by]`, or add the foreign key to the `shipment` table | — |

Soft deletion reads the same, with "isn't soft-deletable", "is soft-deletable" and `on_soft_delete`, which takes `Error`, `SoftDelete` or `Ignore`.

## Verified by a Spike

A throwaway proc macro that emits this shape was compiled with rustc 1.97.1. It showed:

- Paths to a missing hidden type stay quiet: in impl headers, in `<…Table as Trait>::function()` calls, in `where` clauses and in `const` checks, from another module and from the module of the rejected table. A missing wrapper type stays quiet too.
- The gate fails as `E0277` with the local trait's `message` and `label`, through the `do_not_recommend` blanket impl. The pairing checks do the same, and show their `note`.
- The error is spanned on the `table = …` value of the `#[foreign_key]` or the `#[referenced_by]`. Two foreign keys give two errors.
- Every form adds `help: the trait … is not implemented for server__TableHandle`, which points at the rejected table's attribute, and `this error originates in the attribute macro`.

## Constraints

- Build and test only through `x.ps1`. Never run `cargo` directly in the workspace.
- `SpacetimeDSLTable::compile_error_checks` and `compile_error_check_imports` are public in `derive-input`'s API. Their replacement updates `derive/src/data_transfer_contract_tests.rs` and adds an entry under *For crates building on `spacetimedsl_derive-input`* in `docs/MIGRATION.md`. `derive-input/src/api/` gains only what another crate reads.
- `v0.25.0` is released, so every user-visible change goes into a new section `## 0.25 → 0.26` at the top of `docs/MIGRATION.md`:
  - *Newly rejected inputs*: tables that are paired across modules have to be visible to each other.
  - *Changed messages and generated code*: what a rejected `#[dsl]` emits, and the pairing errors.
- `docs/DOCUMENTATION.md`: the section `### Pairing Requirement` quotes the old import errors and gets rewritten. Its *DSL-Specific Mistakes* table changes where it describes these errors.
- `AGENTS.md` says, in its rules for diagnostics tests: "A rejected `#[dsl]` emits the error and nothing else, so every companion or neighbouring table naming its struct or a generated item fails as well." That rule changes with this design, and so do the `//!` comments of the fixtures that explain follow-on errors, such as `referenced_by_repeated.rs`.
- Knowledge in `AGENTS.md`, `docs/DOCUMENTATION.md` and `docs/MIGRATION.md` changes in the same commit as the code that it describes.
- Names follow *Self-Documenting Code* in `AGENTS.md`: no abbreviations, and no comments about the past or about this issue.

## Tests

- **Diagnostics** (`compile-tests/tests/ui`), one rejection or one rule per file:
  - A rejected referenced table, with two referencing tables, one of them with two foreign keys to it, and a third table that references one of those two. Expected: the `syn::Error`, three gate errors, nothing at the third table.
  - A rejected referencing table. Expected: the `syn::Error` and one gate error on the `#[referenced_by]` that names it.
  - A rejected table in the same module as a table that references it.
  - One file for each of the four pairing rules. The existing files keep their rule and get the new output. The rule "the referenced table names this table" probably needs a file of its own, now that it is split off.
  - A private referencing struct in another module: `E0603`, the newly rejected input.
  - The 14 existing files with follow-on `E0599`s get gate errors instead.
  - `missing_table_attribute.rs` and `creation_default_on_singleton_with_default.rs` keep only their `syn::Error`. `column_with_array_type.rs` and `column_with_tuple_type.rs` gain SpacetimeDB's own errors on the column.
- **Snapshots** (`derive/tests/fixtures`): every table with a foreign key or a `#[referenced_by]` changes. Every table gains the gate impl and the hidden type.
- **Runtime** (`examples/test/src`): cascades and existence checks keep working through the new paths, and `.\x.ps1 test` stays green. The existing groups cover them. One of them is `string_reference_test`, which references a `String` primary key through each kind of foreign key, and cascades through a table with a `String` key.

## Edge Cases

- A rejected `#[dsl]` on an item that isn't a struct, or on a struct without `#[spacetimedb::table]`: only the `compile_error!` comes back, so no handle exists. Its referencing tables stay quiet, and only its `syn::Error` shows.
- A typo in `table = …` gives "cannot find type" errors for the other table's handle and hidden type, as it gives unresolved imports today.
- Several `#[dsl]` attributes on one struct: each is judged on its own. A rejected one leaves the struct's other `#[dsl]` attributes in place.

## Left to the Builder

- The names of the hidden type, of the shared traits and of the local check traits.
- Where the shared traits live: in the runtime crate, or in `spacetimedsl!()`. A trait whose methods take `DSL`, `ReadOnlyDSL` or `DSLInternals` has to be declared by `spacetimedsl!()`, which declares those types.
- Whether `DSLInternals` stays.
- The wording of the messages, within the rules of *Messages*.
- The shape of the `derive-input` API that replaces `compile_error_checks` and `compile_error_check_imports`.
- The form of the one checked call through which a referenced table calls into a referencing table.

## Calls Made in Brainstorming, for Review

Decided by the user:

- The output of *The Output It Aims For*.
- Hand-written code keeps reporting `E0599`.
- The approach that relies on rustc's suppression, not stand-ins.
- The types that the traits name are named from the `singular_table_name`.
- Only a struct that carries `#[spacetimedb::table]` comes back. SpacetimeDB's own errors after a rejection that it shares are accepted.
- Tables that are paired across modules have to be visible to each other, as a newly rejected input.
- The shared cascade trait has a default body for every removal, so that a missing strategy gives one error.

Proposed by Claude:

- One rule per check, which splits the "or" of today's names.
- The span of every check: the `table = …` value of its attribute.
- The struct that comes back is the one the expansion holds when the rejection happens, with the derive helper and a singleton's injected `id`.
- The form of a check: a `const` or a `where` clause, not a generic function.
- This document's location, `docs/specs/`, and the migration section `## 0.25 → 0.26` without a version bump.
