# Emit Even If `#[dsl]` Is Rejected — Record

**Spec:** [`docs/specs/2026-10-28-emit-even-if-dsl-is-rejected-design.md`](../specs/2026-10-28-emit-even-if-dsl-is-rejected-design.md), issue [#192](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/192).

**Run:** autonomous, in the worktree branch `spacetime-dsl-superpowers-a557f9`. Each green step is committed without waiting; what only the user can decide is skipped and reported at the end.

## Global Constraints

- Build and test only through `x.ps1`.
- Every step: `.\x.ps1 format`, then `.\x.ps1 unit-test`, and the `git diff` of every regenerated snapshot and `.stderr` read before its commit.
- Every task ends with the full gate: `.\x.ps1 format` (a second run changes nothing), `.\x.ps1 lint`, `.\x.ps1 unit-test` and `.\x.ps1 test`.
- User-visible changes go into a new section `## 0.25 → 0.26` at the top of `docs/MIGRATION.md`. No version bump.
- A change of a public `derive-input` type updates `derive/src/data_transfer_contract_tests.rs` and the section *For crates building on `spacetimedsl_derive-input`*, in the same commit.
- Every user-facing `syn::Error` is written in `derive-input/src/internal/error.rs`.
- Knowledge in `AGENTS.md`, `docs/DOCUMENTATION.md` and `docs/MIGRATION.md` changes in the commit of the code it describes.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Tasks

### Task 1: A rejected `#[dsl]` brings its struct back

`derive/src/lib.rs` emits, beside the `compile_error!`, the struct as the expansion holds it when the rejection happens, when it is a struct that carries `#[spacetimedb::table]`: with the derive helper, and a singleton's injected `id` once injected. Any other item gets only the `compile_error!`.

Produces for Tasks 2 and 3: after a rejection, `<table>__TableHandle` and `ctx.db.<table>()` exist.

### Task 2: Cross-table calls go through each table's table type

- Every accepted table emits its table type, `#[doc(hidden)] pub struct <Table>Table;`, named by `naming::table_type_name(singular_table_name)`.
- A non-singleton table gives its table type `get_by_primary_key`, which the existence checks of `create_*`, `update_*_by_*` and `upsert_*` and the referenced-row methods call instead of `get_<table>_by_<primary key>` on `DSL`, with the arguments they pass today.
- `spacetimedsl!()` declares `internal::References<Referenced, PrimaryKey>`, with the four cascade entry points as methods and an `unreachable!` default body each. A referencing table implements it for every table its foreign keys reference, and overrides the removals it declares a strategy for. The referenced table's dispatchers call it instead of `DSLInternals::execute_on_delete_strategies_of_the_<table>_table_…`.
- A struct whose name is its table type's name is rejected with a `syn::Error` that names the fix.

Produces for Task 3: `naming::table_type_name`, the trait `crate::spacetimedsl::internal::References<Referenced, PrimaryKey>`, and the dispatcher calls into it, which Task 3 routes through the check "the named table has a foreign key back".

### Task 3: The pairing checks are traits, and a rejection gives one gate error per attribute

- `spacetimedsl!()` declares the shared facts: `AcceptedTable`, `HasDeleteMethod`, `HasNoDeleteMethod`, `IsSoftDeletable`, `IsNotSoftDeletable` and `ReferencedBy<Referencing>`.
- Every accepted table implements `AcceptedTable` for its `<table>__TableHandle`, and the other facts for its table type.
- `pairing.rs` builds the checks of the spec's table, each a local trait with `#[diagnostic::on_unimplemented]` and a `#[diagnostic::do_not_recommend]` blanket impl over a fact, spanned on the `table = …` value. The check "the named table has a foreign key back" is the one place through which the dispatchers call into a referencing table.
- `SpacetimeDSLTable::pairing` replaces `compile_error_checks` and `compile_error_check_imports`.
- `### Pairing Requirement` in `docs/DOCUMENTATION.md` is rewritten, with the rule that tables paired across modules are visible to each other.

## Decisions

Made by the user, in brainstorming:

- The output after a rejection: the `syn::Error`, one gate error on each `#[foreign_key]` and `#[referenced_by]` that depends on the rejected table, nothing else from generated code.
- Hand-written code that calls a rejected table's DSL methods keeps reporting `E0599`.
- The approach relies on rustc's suppression of missing items in a module with a `compile_error!`.
- Only a struct that carries `#[spacetimedb::table]` comes back.
- Tables paired across modules have to be visible to each other.
- The shared cascade trait has a default body for every removal.

Made by Claude, as the spec leaves them to the builder:

- The table type is named `<Table>Table`, the PascalCase of the `singular_table_name` followed by `Table`, such as `WarehouseTable`.
- The shared traits live in `crate::spacetimedsl::internal`, declared by `spacetimedsl!()`, beside `DSLInternals`, because the cascade trait's methods take `DSL`. `DSLInternals` stays for the referenced table's own dispatchers.
- The lookup through the table type is an associated function, because its argument differs per table: a `&str` for a `String` primary key, the wrapper otherwise.
- A struct named like its table type is rejected, rather than left to rustc's "defined multiple times".

Made by the user, after the pull request was opened:

- `docs/MIGRATION.md` holds only what existing code has to change, which narrows the global constraint on user-visible changes. The changed messages and generated code, and `api::attribute::is_table_attribute`, are named in the pull request's description instead, and `docs/DOCUMENTATION.md` describes them.

## Review Focus

1. A struct named like its table's table type, such as `struct ServerTable` with `accessor = server`: a `syn::Error` that names the clash and the fix.
2. A struct with two `#[dsl]` attributes, one of them rejected: the accepted table still works for the tables referencing it, and each `#[foreign_key]` to the rejected one gets one gate error.
3. A foreign key to the `String` primary key of a rejected table: exactly one gate error.
4. A self-referencing table, and two tables referencing each other: cascades still run, and no check fails on the table itself.
5. A referenced table without delete methods whose referencing table is rejected: one gate error on its `#[referenced_by]`, nothing else.
