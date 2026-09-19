# Soft Deletion Design

Status: approved, not yet implemented.
Date: 2026-09-19.
Issue: [#59](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/59).

## Goal

A table opts into soft deletion with `#[dsl(method(soft_delete = true))]`. Its rows are then
retired by writing a marker column instead of removing the row, and foreign keys of other
tables react to that retirement through their own on-delete strategies.

## Success test

`./x unit-test` is green after every phase, which runs the `insta` snapshots in
`derive/tests/snapshots` and the `trybuild` diagnostics in `compile-tests/tests/ui`.
`./x test` is green after the final phase, which publishes `examples/test` to a local
SpacetimeDB instance and runs its `tester` reducer.

## Out of scope

These are deliberate exclusions, not oversights:

- `get_<table>_by_<index>`, `get_all_<tables>` and `count_of_all_<tables>` keep returning
  soft-deleted rows. Read filtering is a separate story.
- No restore or undelete method exists. A row is un-retired by whatever writes the marker
  column, which is nothing the DSL generates.
- `docs/entity_dsl_methods.png` is not regenerated. The tool that produced it is unknown.

## Attribute surface

```rust
#[dsl(
    plural_name = Users,
    method(update = true, delete = true, soft_delete = true),
    hook(before(soft_delete), after(soft_delete)),
)]
#[spacetimedb::table(name = user)]
pub struct User {
    #[primary_key]
    #[create_wrapper]
    id: u64,
    pub name: String,
    deleted: bool,
}
```

`method(soft_delete)` is an `Option<bool>` in `DSLData` and means `false` while absent.
Setting it explicitly, to either value, makes `method(delete)` mandatory as well.
`method(delete)` keeps its present default of `true` for every table that does not mention
`soft_delete`, so no existing table changes behavior.

`#[set_on_soft_delete]` is a new helper attribute. It is registered in the
`attributes(...)` list of the `SpacetimeDSL` derive in `derive/src/lib.rs`, next to
`created_at` and `updated_at`.

`#[foreign_key]` gains an optional `on_soft_delete` inner field carrying the same
`OnDeleteStrategy` enum as `on_delete`. `on_delete` becomes optional. At least one of the
two must be present.

## The marker column

A soft-deletable table has exactly one marker column. A column claims the marker role by
its name or by carrying `#[set_on_soft_delete]`, and the claimed role fixes its type.

| Claim                                        | Required type       | Written on soft delete            | Already-retired test       |
| -------------------------------------------- | ------------------- | --------------------------------- | -------------------------- |
| `deleted`, `removed`, or the attribute       | `bool`              | `= true`                          | `row.deleted`              |
| `deleted_at`, `removed_at`, or the attribute | `Option<Timestamp>` | `= Some(self.ctx().timestamp()?)` | `row.deleted_at.is_some()` |

A column carrying `#[set_on_soft_delete]` must have one of the two types; which one it has
decides what the generator writes.

The marker column must be private. It therefore earns the ordinary read-only getter that
`SpacetimeDSLColumn::try_parse` already gives every private column, and no setter and no
mutable getter. `soft_delete_<table>_by_<index>` is the only generated writer.

The marker is excluded from the `Create<Table>` request struct. The create generator
initializes it to `false` or `None`, the way it already initializes the `created_at`
column. A row is never born retired.

### Rejections

Each of these returns a spanned `syn::Error`:

1. `method(soft_delete)` is set and `method(delete)` is absent.
2. A marker column exists and `method(soft_delete)` is not `true`.
3. `method(soft_delete = true)` is set and no marker column exists.
4. A column's name claims the marker role and its type does not fit the role.
5. Two columns claim the marker role.
6. The marker column is not private.
7. `method(soft_delete = true)` is set on a `#[dsl(singleton)]` table.
8. `hook(before(soft_delete))` or `hook(after(soft_delete))` is declared without
   `method(soft_delete = true)`.

Rules 4 and 6 mirror what `created_at` and `updated_at` already enforce in
`internal/dsl/table.rs`.

## Generated methods

A soft-deletable table earns one method per index, alongside whatever `delete_*` methods
`method(delete)` grants it:

- `soft_delete_<singular_table_name>_by_<index_name>` per unique index, single- or
  multi-column.
- `soft_delete_<plural_table_name>_by_<index_name>` per non-unique index, single- or
  multi-column.

Both return `Result<DeletionResult, SpacetimeDSLError>`, the same type `delete_*` returns.
The `DeletionResultEntry` a soft deletion contributes carries
`OnDeleteStrategy::SoftDelete` as its strategy.

`method(soft_delete)` is independent of `method(update)`. The generated body writes through
`self.db().<table>().<primary_key>().update(row)` directly rather than calling
`update_<table>_by_<primary_key>`, so a table may be immutable to its callers and still
soft-deletable.

### Body of the one-row method

```text
find the row by the index          -> not found     => Err(NotFoundError)
row already retired?                               => Ok(DeletionResult with no entries)
build the DeletionResultEntry
run the Error strategy pass over the referencing tables
call before_<table>_soft_delete(&old_row, new_row) -> Result<Row>
write the marker onto the row the hook returned
self.db().<table>().<primary_key>().update(row)
call after_<table>_soft_delete(&old_row, &new_row)
run the SoftDelete, SetZero and Ignore strategy passes
Ok(DeletionResult)
```

The Error pass runs before anything is written, so a refusal leaves the database untouched.
This is the order `delete_<table>_by_<index>` already uses.

The many-row method filters retired rows out before it builds entries, before the hooks and
before the strategy passes. An empty remainder returns `Ok` with no entries rather than an
error.

A soft deletion does not stamp the `updated_at` column. `deleted_at` records the
retirement; `updated_at` keeps meaning the last ordinary edit.

### Hooks

The soft-delete hooks take the shape of the update hooks, because a soft deletion writes a
row rather than removing one:

```rust
pub trait BeforeUserSoftDeleteHook<T: crate::spacetimedsl::WriteContext> {
    fn before_user_soft_delete(
        dsl: &crate::spacetimedsl::DSL<'_, T>,
        old_user: &User,
        new_user: User,
    ) -> Result<User, crate::spacetimedsl::error::SpacetimeDSLError>;
}

pub trait AfterUserSoftDeleteHook<T: crate::spacetimedsl::WriteContext> {
    fn after_user_soft_delete(
        dsl: &crate::spacetimedsl::DSL<'_, T>,
        old_user: &User,
        new_user: &User,
    ) -> Result<(), crate::spacetimedsl::error::SpacetimeDSLError>;
}
```

`Operation` in `internal/dsl/hook.rs` gains a `SoftDelete` variant, which produces the
`SoftDelete` infix in the trait name and the `soft_delete` infix in the function name.

## Hook ordering across all write paths

The marker is written after `before_<table>_soft_delete` returns, so the framework has the
last word on the column it owns.

`update_<table>_by_<primary_key>` and both paths of `upsert_<table>` currently do the
reverse: they stamp `created_at` and `updated_at` before calling their hook, which the
module documentation of `internal/dsl/method/upsert.rs` describes as deliberate. That
decision is reversed as part of this work. After it, all five write paths — create, update,
both upsert paths and soft delete — call the hook first and write framework-owned columns
afterwards. The module documentation of `upsert.rs` is rewritten to state the single rule.

This changes generated output for every table that combines a timestamp role with an update
hook. It lands as its own commit with its own snapshot re-recording, so the reordering is
reviewable apart from the soft-deletion work.

## Foreign keys and cascade

### The strategy enum

`OnDeleteStrategy` gains a `SoftDelete` variant declared after `Delete`, giving the order
`Error, Delete, SoftDelete, SetZero, Ignore`. `strum::EnumIter` walks the variants in
declaration order to build the generated match arms, so this position also fixes the arm
order in the generated code. The enum is duplicated between `src/delete.rs` and
`derive-input/src/api/dsl/foreign_key.rs`; both copies change, and `src/error.rs` gains the
matching `Display` arm.

`SoftDelete` retires the referencing rows instead of removing them, and recurses the same
way `Delete` does.

Which strategies each field accepts:

| Field            | Accepted strategies                                  |
| ---------------- | ---------------------------------------------------- |
| `on_delete`      | `Error`, `Delete`, `SoftDelete`, `SetZero`, `Ignore` |
| `on_soft_delete` | `Error`, `SoftDelete`, `Ignore`                      |

`on_soft_delete` rejects `Delete` and `SetZero`: retiring a parent row must not physically
remove a child row, and must not clear a column the retirement was meant to preserve.

`SoftDelete` in either field requires the table that declares the `#[foreign_key]` to be
soft-deletable, because that table's rows are the ones the strategy retires. This is
checkable inside one macro expansion: `SpacetimeDSLColumn::try_parse` already receives the
built `SpacetimeDSLTable`, so the marker travels down the same path `has_delete_method`
already travels.

`ForeignKey` holds `on_delete_strategy: Option<OnDeleteStrategy>` and
`on_soft_delete_strategy: Option<OnDeleteStrategy>`. Neither set is a spanned `syn::Error`
that names both fields and explains that which of them is required depends on whether the
referenced table is deletable, soft-deletable or both.

### Four compile-error-check identifiers per table pair

Two tables in a foreign key relationship verify each other through identifiers they both
build from the same two table names. Each direction is now split by capability, and each
identifier is emitted and imported under one flag, so a table pair that uses only hard
deletion keeps exactly one pairing.

Emitted by the referencing table, imported by the referenced table. Built by
`referencing_table_compile_error_check_for_deletions` and
`..._for_soft_deletions`:

```text
this_compilation_error_occurs_because_the_order_table_has_no_foreign_key_attribute_with_on_delete_defined_referencing_the_user_table
this_compilation_error_occurs_because_the_order_table_has_no_foreign_key_attribute_with_on_soft_delete_defined_referencing_the_user_table
```

Emitted by the referenced table, imported by the referencing table. Built by
`referenced_table_compile_error_check_for_deletions` and `..._for_soft_deletions`:

```text
this_compilation_error_occurs_because_the_user_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_order_table
this_compilation_error_occurs_because_the_user_table_is_not_soft_deletable_or_has_no_referenced_by_attribute_referencing_the_order_table
```

Which identifier each side emits and imports:

| Side                              | Emits                                 | Imports                               |
| --------------------------------- | ------------------------------------- | ------------------------------------- |
| Referencing, `on_delete` set      | `..._with_on_delete_defined_...`      | `..._is_not_deletable_or_...`         |
| Referencing, `on_soft_delete` set | `..._with_on_soft_delete_defined_...` | `..._is_not_soft_deletable_or_...`    |
| Referenced, `method(delete)`      | `..._is_not_deletable_or_...`         | `..._with_on_delete_defined_...`      |
| Referenced, `method(soft_delete)` | `..._is_not_soft_deletable_or_...`    | `..._with_on_soft_delete_defined_...` |

This makes two mistakes ordinary unresolved-import errors. A soft-deletable table referenced
by a foreign key that sets no `on_soft_delete` fails, and an `on_soft_delete` aimed at a
table that is not soft-deletable fails.

`#[referenced_by]` is allowed when the table has `method(delete = true)` or
`method(soft_delete = true)`. Its present error message, which names only the delete method,
is rewritten to name both.

### Dispatcher functions

`referenced_table_function_name` and `referencing_table_function_name` in
`internal/dsl/method/naming.rs` gain soft variants whose names end in `_was_soft_deleted`
and `_were_soft_deleted` rather than `_was_deleted` and `_were_deleted`.

A soft-deletable referenced table earns a second pair of entry points beside the pair it
earns today, and a table that references a soft-deletable table earns a second pair of
strategy functions. `OnDeleteStrategiesOfReferencingTables` and
`OnDeleteStrategiesOfTheReferencedTable` in `api/dsl/table.rs` each gain an optional soft
pair. As with the existing pairs, both members of a soft pair exist or neither does.

The `SoftDelete` arm in `internal/dsl/method/on_delete_strategy.rs` mirrors the `Delete`
arm: it collects the rows an index matches, recurses through the soft dispatcher when
`ReferencingTables::Present`, calls the soft-delete hooks rather than the delete hooks, and
writes the marker rather than calling `.delete()`.

## Module layout

New files:

- `derive-input/src/api/dsl/soft_delete.rs` — `SoftDeleteMarker`, holding the marker's
  column name and its kind.
- `derive-input/src/internal/dsl/soft_delete.rs` — marker role detection and the marker
  rejections. A sibling of the existing `singleton.rs`, which keeps the already busy
  `internal/dsl/table.rs` from growing.
- `derive-input/src/internal/dsl/method/removal.rs` — the body skeleton `delete.rs` and
  `soft_delete.rs` share: find the rows, build the entries, run the Error pass, call the
  hooks, write, run the remaining strategy passes. Parameterized by a `Removal` enum whose
  two variants decide the write statement, the hooks, the dispatcher names and the reported
  strategy.
- `derive-input/src/internal/dsl/method/soft_delete.rs` — `for_soft_delete_one` and
  `for_soft_delete_many`, each a call into `removal.rs` with `Removal::Soft`.

`delete.rs` is refactored in the same pass to call `removal.rs` with `Removal::Hard`, so
the cascade order stays stated once.

Changed files: `api/dsl/{table,column,hook,foreign_key}.rs`,
`internal/dsl/{table,column,foreign_key,reference,hook}.rs`, `internal.rs`,
`internal/dsl/method/{naming,referenced_by,foreign_key,on_delete_strategy,create,update,upsert,method}.rs`,
`derive/src/{lib,output}.rs`, `src/{delete,error}.rs`.

## Runtime crate

`OnDeleteStrategy` gains `SoftDelete` and `src/error.rs` gains its `Display` arm.
`error::Action` gains a `SoftDelete` variant so error prose reads "soft delete" where an
`Action` is rendered. `DeletionResult`, `DeletionResultEntry`,
`OnDeleteStrategyFailure` and `ReferenceIntegrityViolationError::OnDelete` are reused
unchanged.

## Phases

Each phase is one commit that leaves the workspace compiling and `./x unit-test` green.
Tests are written before the production code they cover.

| #  | Phase                                                                                                                                      | Tests written first                                                                                                                                                                                                                                                                        |
| -- | ------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1  | Hook ordering reversed in `update.rs` and both paths of `upsert.rs`; `upsert.rs` module documentation rewritten                            | Existing `update_*` and `upsert_*` snapshots re-recorded                                                                                                                                                                                                                                   |
| 2  | `OnDeleteStrategy::SoftDelete`, `Action::SoftDelete` and their `Display` arms in the runtime crate                                         | None; no generated output changes yet                                                                                                                                                                                                                                                      |
| 3  | Marker column: the two new `soft_delete.rs` modules, `method(soft_delete)` parsing, `#[set_on_soft_delete]` registration, create exclusion | Rejections 1 to 7 as `trybuild` cases; snapshot fixtures for a `bool` marker, an `Option<Timestamp>` marker and an attribute-claimed marker                                                                                                                                                |
| 4  | `removal.rs` extracted from `delete.rs`; `soft_delete.rs` generators; the soft-delete hooks                                                | Rejection 8 as a `trybuild` case; snapshot fixtures for `delete = false, soft_delete = true` and for both marker kinds with hooks                                                                                                                                                          |
| 5a | `on_soft_delete` parsing, `on_delete` made optional, the per-field strategy sets, the four compile-error identifiers                       | `trybuild` cases: neither field set; `Delete` in `on_soft_delete`; `SetZero` in `on_soft_delete`; `SoftDelete` on a table that is not soft-deletable; a foreign key at a soft-deletable table without `on_soft_delete`; `#[referenced_by]` on a `delete = false, soft_delete = true` table |
| 5b | The `SoftDelete` strategy arm, the soft dispatcher pair, the soft entry points in `referenced_by.rs`                                       | Snapshot fixture for a soft-delete cascade, one row and many, including recursion                                                                                                                                                                                                          |
| 6  | `docs/DOCUMENTATION.md`, `README.md`, a soft-deletable table in `examples/test` with `tester` assertions                                   | `./x test` against a local SpacetimeDB instance                                                                                                                                                                                                                                            |

## Blast radius

Renaming the compile-error-check identifiers changes every generated module that has a
foreign key, which makes the snapshot diff in that phase large. It breaks nothing: a
SpacetimeDB server module is compiled as a whole, so the two sides of a pairing are always
regenerated together and never meet across versions.
