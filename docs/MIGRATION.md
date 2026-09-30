# Migration — **SpacetimeDSL**

What changes for a module, or for a crate building on `spacetimedsl_derive-input`, when it moves to a new release of **SpacetimeDSL**.

## 0.23 → 0.24

### Breaking API changes

#### `Wrapper` has one type parameter

The `Wrapper` trait lost its second type parameter, which carried no information. A hand-written wrapper type implements `Wrapper<WrappedType>`:

```rust
// 0.23
impl spacetimedsl::Wrapper<i32, ConfigId> for ConfigId { /* … */ }

// 0.24
impl spacetimedsl::Wrapper<i32> for ConfigId { /* … */ }
```

#### `OnCreateOrUpdate` holds an `error::CreateOrUpdate`

`ReferenceIntegrityViolationError::OnCreateOrUpdate::create_or_update` has the new type `error::CreateOrUpdate` instead of `error::Action`, so it can no longer hold an action that is not a write, and formatting the error can no longer panic:

```rust
// 0.23
if let ReferenceIntegrityViolationError::OnCreateOrUpdate { create_or_update: Action::Create, .. } = error { /* … */ }

// 0.24
if let ReferenceIntegrityViolationError::OnCreateOrUpdate { create_or_update: CreateOrUpdate::Create, .. } = error { /* … */ }
```

### Newly rejected inputs

#### Several `#[table]` attributes, but no `table = <accessor>`

A struct with several `#[table]` attributes needs `table = <accessor>` on each `#[dsl]`, naming the table it belongs to. Before, the `plural_name` was used to guess one, and an arbitrary table was used when the guess failed.

```rust
// 0.23
#[spacetimedsl::dsl(plural_name = offline_players, method(update = true))]
#[spacetimedb::table(accessor = offline_player, public)]
#[spacetimedsl::dsl(plural_name = online_players, method(update = true))]
#[spacetimedb::table(accessor = online_player, public)]
pub struct Player { /* … */ }

// 0.24
#[spacetimedsl::dsl(plural_name = offline_players, table = offline_player, method(update = true))]
#[spacetimedb::table(accessor = offline_player, public)]
#[spacetimedsl::dsl(plural_name = online_players, table = online_player, method(update = true))]
#[spacetimedb::table(accessor = online_player, public)]
pub struct Player { /* … */ }
```

Without it: *There are 2 `#[table]` attributes below this `#[dsl]` (`offline_player`, `online_player`), so it has to name the one it belongs to!* A `#[dsl]` sees only the `#[table]` attributes below it, so the second `#[dsl]` above sees one table and needs no `table`, though it may name it. A `table` that names no `#[table]` below its `#[dsl]` is rejected with *No `#[table]` attribute below this `#[dsl]` has the accessor `…`!*, also when there is a single `#[table]`.

#### An unknown or repeated `unique_index(name = …)`

`unique_index(name = …)` must name an index of the table, and each index only once. A misspelled name used to be accepted without effect; now it is rejected with *No index of this table has the accessor `…`! Its indices: …*. A repeated name is rejected with *`unique_index(name = …)` is given twice!* Fix the name, or remove the repetition. A name of a hash or single-column index is still accepted without effect.

#### A `Timestamp` column on a `singleton(with_default)` table, in every spelling

On a `singleton(with_default)` table, a `Timestamp` column is rejected in every spelling. Before, a qualified spelling such as `::spacetimedb::Timestamp` slipped past the check. Use `Option<Timestamp>`, as the diagnostic says.

#### A column type that is not a path

A column whose type is not a path — an array such as `[u8; 4]`, a tuple, a reference, and the like — used to make the macro panic (*should be parseable as Path*). It now gets a diagnostic on the type: *SpacetimeDSL supports only path types as column types, such as `u64`, `String` or `spacetimedb::Timestamp`! This column's type is an array.* Wrap the value in a type of your own.

#### Two single-column indices on one column

A column with two single-column indices — for example `#[index(btree)]` on the field and `index(accessor = by_name, hash(columns = [name]))` in `#[table]` — is rejected with *The column `name` has two single-column indices, `by_name` and `name`!* Before, the second index generated no methods and nothing said so. Remove one of them.

#### `on_delete = SetZero` on a column that is neither an unsigned integer nor a `Uuid`

`on_delete = SetZero` is rejected on a foreign key column that is neither an unsigned integer (`u8`–`u128`, in any spelling) nor a `Uuid`, at the column's type: *`OnDeleteStrategy::SetZero` is only allowed on unsigned integer and `Uuid` columns, …* This includes columns that used to work: signed integers such as `i32` / `i64`, and type aliases such as `type PlayerId = u64`, which the check cannot see through. Other types, such as `String`, used to fail inside the expanded code. Spell an aliased key as its unsigned type, move a signed key to an unsigned one, or choose another strategy (`Delete`, `Ignore`, …).

#### A foreign key which leaves out the strategy of a removal its table performs

Every `#[foreign_key]` has to set `on_delete` when the referenced table has a delete method, and `on_soft_delete` when it is soft-deletable. Two foreign keys of one table to the same table used to be checked together, so one of them could leave out `on_delete` while the other set it; deleting a referenced row then left the rows of the first one pointing at nothing. Each is checked on its own now: *unresolved import `…::this_compilation_error_occurs_because_your_foreign_key_referencing_the_warehouse_table_needs_to_define_a_strategy_for_on_delete_or_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`*. Add the missing strategy.

#### `on_delete = SetZero` on a primary key, a `#[unique]` column or a column of a unique multi-column index

`SetZero` writes `0` or `Uuid::NIL` into the foreign key column and writes the row back through its primary key. On a primary key column that write finds no row or another one; on a `#[unique]` column, or a column of a `unique_index(name = …)` index, a second cleared row repeats the value. SpacetimeDB's `update` panicked inside the cascade in the first two cases, and the third silently broke the index's uniqueness. All three are rejected at the `#[foreign_key]` attribute. Choose another strategy, such as `Delete`, or remove the uniqueness.

### Newly accepted inputs

#### `#[referenced_by]` on a table without delete and soft-delete methods

A table with `method(delete = false)` and without `method(soft_delete = true)` may carry `#[referenced_by]`. Its rows are never removed through the DSL, so it generates no cascade. *`#[referenced_by]` is only allowed when the table has a delete method …* is gone.

#### `#[foreign_key]` without `on_delete` and `on_soft_delete`

A foreign key to such a table sets no strategy. *A `#[foreign_key]` must set `on_delete`, `on_soft_delete`, or both* is gone; a foreign key to a table which performs a removal still has to set its strategy. Create and update still check that it references a row.

### Changed messages and generated code

#### The error of a failed soft-delete cascade says *Soft Delete*

The error a `soft_delete_*` method returns when a cascade fails after the database already changed now starts with *Soft Delete One Error* / *Soft Delete Many Error* instead of *Delete One Error* / *Delete Many Error*.

#### `on_delete = SetZero` on `Uuid` foreign keys

`on_delete = SetZero` is available on `Uuid` foreign keys and sets them to `Uuid::NIL`. Create, update and upsert treat a `Uuid` foreign key equal to `Uuid::NIL` as referencing no row, as they treat `0` for an unsigned integer: they no longer report a reference integrity violation for it. The generated create and update methods gain that guard.

#### Foreign keys to one table may spell their type and path differently

Foreign keys of one table to the same table may spell their type and path differently (`u64` / `core::primitive::u64`, `::my_crate::tables` / `my_crate::tables`); they used to be rejected as mismatched. A real mismatch is still rejected, and the path message now adds *Spell both paths the same way; a leading `::` makes no difference.*

#### A foreign key column is indexed through any single-column index on it

A foreign key column counts as indexed through every single-column index on it, including one declared as `index(…)` in `#[table]` under an accessor of its own; before, only `#[primary_key]`, `#[unique]` and `#[index]` on the field counted. The cascade reaches the column's rows through that index's accessor.

#### Qualified spellings of the checked types are accepted

Qualified spellings of the types SpacetimeDSL checks are accepted where they used to be rejected: `::spacetimedb::Timestamp` and `std::option::Option<spacetimedb::Timestamp>` for `#[set_on_create]` / `#[set_on_update]`, and `core::primitive::u64` / `std::primitive::u64` count as unsigned integers (so a foreign key spelled that way skips its reference-integrity check for `0`, like `u64`). See *Column Type Spellings* in the documentation. This is not breaking.

Likewise `core::primitive::i64` / `std::primitive::f64` count as the signed integers and floats they name, so two foreign keys of one table to the same table which spell `i64` both ways are no longer rejected as mismatched.

#### The missing-update-method diagnostic names `#[set_on_update]`

The diagnostic for a missing `method(update = …)` on a table with only private columns names `#[set_on_update]` next to the conventional `modified_at` / `updated_at` as a way to make the table mutable.

#### Diagnostics print visibilities as written

Diagnostics about a column's visibility print it as written — `` `pub` ``, `` `pub(crate)` ``, `` `pub(in path)` `` — and ask for "no visibility modifier" instead of naming `syn` types such as `Visibility::Inherited` or `Visibility::Public`. The soft-delete marker diagnostic now also says which visibility it found.

#### A `#[dsl]` needs a `#[table]` anywhere below it

A `#[dsl]` without a `#[table]` attribute below it is rejected with *Haven't found a `#[table]`/`#[spacetimedb::table]` attribute below this `#[dsl]`! `#[dsl]`/`#[spacetimedsl::dsl]` builds on the table it declares, so write it above that `#[table]`.* It used to ask for `#[dsl]` to be *directly* above a `#[table]`; anywhere above it is enough.

#### The crate root re-exports `spacetimedsl::prelude`

The root of the `spacetimedsl` crate re-exports everything in the new `spacetimedsl::prelude`, which adds the context accessor traits, `OnDeleteStrategyFailure`, `NewUUID` and `Itertools` to what `spacetimedsl::X` reaches. This is additive.

#### The preludes export `err!`

`spacetimedsl::prelude`, and with it the prelude `spacetimedsl!()` generates, exports the new macro `err!`. A macro of your own called `err` which another glob import brings into the same scope becomes ambiguous where it is called; import that one by name, which takes precedence over a glob.

#### Attributes spelled with a leading `::` are recognised

`#[::spacetimedsl::dsl(…)]` and `#[::spacetimedb::table(…)]`, spelled with a leading `::`, are recognised like `#[spacetimedsl::dsl(…)]` and `#[spacetimedb::table(…)]`. Before, a `::spacetimedb::table` attribute was not found, so the struct was rejected for missing a table attribute.

#### Generated code reaches the runtime through `crate::spacetimedsl`

Generated code reaches the runtime only through `crate::spacetimedsl`, the module `spacetimedsl!()` generates, and SpacetimeDB only through `::spacetimedb`. A `#[create_wrapper]` table declared at the crate root with `#[::spacetimedsl::dsl]`, next to `spacetimedsl!()`, therefore compiles: before, the generated `use spacetimedsl::Wrapper;` was ambiguous there.

#### An optional used wrapper is converted with `Option::map`

Where a create method or setter takes an optional used wrapper, the generated code converts it with one `Option::map` (`let due_at = due_at.map(|value| Into::<ReminderDueAt>::into(value).value());`) instead of a mutable `None`, an `is_some()` test and an `expect`. Only the code rustdoc shows under *Implementation* changes.

#### A panic inside a generated cascade names its invariant and its key

The lookups inside the generated delete and soft-delete cascades can only fail if SpacetimeDSL generated inconsistent code. Their panic used to name a variable of the generated code, such as *7 should exist in entries.* It now states the invariant that broke and the key it broke for, such as *the referencing tables return child entries only for the primary key values of the rows this table deletes, which does not hold for 7*. The message is formatted only when a lookup fails.

#### The reference-integrity error of `create_<table>` names the column

When `create_<table>`, or the insert path of `upsert_<table>`, rejects a row because a foreign key references no row, the error names the column and its value, `{ warehouse_id : 3 }`, the way `update_<table>_by_<key>` does. It used to print the value in place of the name, `{ 3 : WarehouseId { id: 3 } }`.

#### A missing row in `update_<table>_by_<key>` is reported by its primary key

When the row to update no longer exists, the `NotFoundError` of `update_<table>_by_<key>`, `update_<singleton>` and `upsert_<singleton>` shows the primary key value it looked up, `{ id : 7 }`. It used to show the value of a foreign key column under the primary key's name.

#### The unique-constraint error of `create_<table>` lists the unique columns

When SpacetimeDB rejects the row of `create_<table>`, or of the insert path of `upsert_<table>`, for a unique-constraint violation, the error lists the primary key and the `#[unique]` columns with the values handed to SpacetimeDB, `{ id : 1, code : second }`, instead of the whole row. An `#[auto_inc]` column shows `0`, the value SpacetimeDB replaces. The `Display` text says *here are the unique columns and the values handed to SpacetimeDB* instead of *here are all columns and their values*.

#### `create_<table>` moves the row into `try_insert`

`create_<table>` and the insert path of `upsert_<table>` no longer clone the whole row before they insert it. They copy only the values of the unique columns, which the unique-constraint error needs.

#### The foreign key pairing is imported once per table

The `use` statements which pair a `#[foreign_key]` with its `#[referenced_by]` moved out of the generated cascade methods into one `const _: () = { … };` block per table.

#### The pairing errors name what to change

A broken pairing between `#[foreign_key]` and `#[referenced_by]` is still an unresolved import, with these names:

- A foreign key missing a strategy for a removal its table performs: `…your_foreign_key_referencing_the_<table>_table_needs_to_define_a_strategy_for_on_delete_or_the_<table>_table_has_no_referenced_by_attribute_referencing_the_<other>_table` (or `…on_soft_delete…`).
- A `#[referenced_by]` naming a table without a foreign key back: `…the_<other>_table_has_no_foreign_key_attribute_referencing_the_<table>_table`, which replaces `…has_no_foreign_key_attribute_with_on_delete_defined…` and `…with_on_soft_delete_defined…`.
- A foreign key setting a strategy its table cannot use keeps `…the_<table>_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_<other>_table` (or `…is_not_soft_deletable…`).

#### A `#[referenced_by]` naming a table twice is reported as such

Two `#[referenced_by]` attributes naming the same table used to fail with rustc's *the name `this_compilation_error_occurs_because_…` is defined multiple times* (`E0252`). The second one is now rejected with *`#[referenced_by(table = …)]` is given twice!* Name each referencing table once, however many foreign keys it has to the table.

#### The generated documentation shows foreign keys and cascades

- `create_<table>`, `update_<table>_by_<key>` and `upsert_<table>` list under *Foreign keys* the foreign key columns they check.
- The delete and soft-delete methods of a table with `#[referenced_by]` list under *Cascade* the tables whose strategies they run.
- The getter and setter of a foreign key column say which column and table it references and which strategies it declares.
- The struct gets its foreign keys and the tables referencing it appended to its documentation.

Only rustdoc output changes.

### For crates building on `spacetimedsl_derive-input`

#### The data-transfer contract is documented

The crate documents what you may rely on: `api::Table::try_parse` is the one entry point, and every public field of every `api` type is part of the semver contract. A test in `spacetimedsl_derive` destructures the whole structure without `..`, so a change to it cannot land unnoticed. The generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`, …) are opaque output to splice into your expansion, not to inspect.

#### The constructors of the model are no longer public

The functions `derive-input` used to build the model — among them `RustField::map`, `RustVisibility::map`, `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `SpacetimeDSLColumn::try_parse`, `SpacetimeDSLTable::try_parse`, `WrapperType::try_parse`, `ForeignKey::try_parse`, `ReferencingTable::try_parse`, `UUIDVersion::try_parse`, `Getter::map`, `MutGetter::map`, `Setter::map` and `SpacetimeDSLColumnMethods::map` — are no longer public. `api::Table::try_parse` is the one entry point.

#### `RustVisibility` implements `ToTokens` instead of `Display`

`RustVisibility` no longer implements `Display`; it implements `quote::ToTokens`, which renders `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` or nothing. Compare it with `matches!` rather than as text.

#### `api::attribute::FIELD_ATTRIBUTE_NAMES` lists the field attributes

`api::attribute::FIELD_ATTRIBUTE_NAMES` lists every field attribute `#[spacetimedsl::dsl]` reads (`create_wrapper`, `use_wrapper`, `foreign_key`, `referenced_by`, `set_on_create`, `set_on_update`, `set_on_soft_delete`, `auto_gen`, `creation_default`, `disallow`). A derive of your own that has to accept them as helper attributes can check its list against it.

#### `api::attribute::is_dsl_attribute` recognises `#[dsl]` in every spelling

`api::attribute::is_dsl_attribute(&Attribute)` tells whether an attribute is `#[spacetimedsl::dsl]` in any spelling: `dsl`, `spacetimedsl::dsl` or `::spacetimedsl::dsl`. Use it instead of comparing the stringified path.

#### `SpacetimeDSLTable::kind` replaces `singleton` and `plural_name`

`SpacetimeDSLTable::singleton: Option<SingletonKind>` and `SpacetimeDSLTable::plural_name: Ident` became one field, `kind: SpacetimeDSLTableKind`. Only its `Normal` variant has a `plural_name`. A singleton has none; `plural_name` used to hold its accessor instead.

```rust
// 0.23
if let Some(singleton_kind) = table.spacetimedsl_table.singleton { /* … */ }
let plural_name = &table.spacetimedsl_table.plural_name;

// 0.24
match &table.spacetimedsl_table.kind {
    SpacetimeDSLTableKind::Normal { plural_name } => { /* … */ }
    SpacetimeDSLTableKind::Singleton(singleton_kind) => { /* … */ }
}
```

#### `is_singleton`, `is_soft_deletable` and `singleton_has_default` are no longer public

`SpacetimeDSLTable::is_singleton()`, `is_soft_deletable()` and `singleton_has_default()` are no longer public. Read the fields they read:

```rust
// 0.23
table.spacetimedsl_table.is_singleton()
table.spacetimedsl_table.is_soft_deletable()
table.spacetimedsl_table.singleton_has_default()

// 0.24
matches!(table.spacetimedsl_table.kind, SpacetimeDSLTableKind::Singleton(_))
table.spacetimedsl_table.soft_delete_marker.is_some()
matches!(table.spacetimedsl_table.kind, SpacetimeDSLTableKind::Singleton(SingletonKind::WithDefault))
```

#### `SpacetimeDSLMethodHooks` is a map keyed by `HookKind`

`SpacetimeDSLMethodHooks` is a map keyed by the new `api::dsl::hook::HookKind` (a `Timing` and an `Operation`) instead of eight fields:

```rust
// 0.23
if let Some(hook) = &table.spacetimedsl_table.hooks.before_insert { /* … */ }

// 0.24
if let Some(hook) = table.spacetimedsl_table.hooks.get(HookKind::BEFORE_INSERT) { /* … */ }
```

`hooks.iter()` yields the declared hooks in emission order, and `hooks.declared` is the `BTreeMap<HookKind, SpacetimeDSLMethodHook>` itself. `HookKind::ALL` lists all eight kinds.

#### `api::dsl::hook::hook_trait_name` names the trait of a hook function

`api::dsl::hook::hook_trait_name(&Ident)` maps a hook function name to the trait it implements (`before_entity_insert` → `BeforeEntityInsertHook`). Both the generator and `#[spacetimedsl::hook]` use it; a macro implementing hooks should too.

#### `ScheduledReducer` holds the reducer's path

`ScheduledReducer::reducer_name: Ident` became `ScheduledReducer::reducer_path: syn::Path`, the reducer or procedure exactly as `scheduled(...)` names it. A qualified path such as `scheduled(crate::timers::tick)` used to make the macro panic. Use `reducer_path.get_ident()` where you need a bare name.

#### Changed `api::runtime` constructors, and the new `api::spacetimedb`

Several `api::runtime` token constructors changed with the generated code they emit, which now reaches the runtime through `crate::spacetimedsl` only:

```rust
// 0.23
runtime::wrapper_trait(&wrapped_type, &wrapper_type)          // Wrapper<Wrapped, Wrapper>
runtime::wrapper_trait_path()                                 // ::spacetimedsl::Wrapper
runtime::deletion_result_entry(&table, &column, &strategy, &value, &quote! { child_entries })
runtime::reference_integrity_violation_on_create_or_update(&table, &quote! { Create }, &message)
runtime::error_from_hook_declaration()

// 0.24
runtime::wrapper_trait(&wrapped_type)                         // Wrapper<Wrapped>
runtime::wrapper_trait_import()                               // use crate::spacetimedsl::Wrapper;
runtime::deletion_result_entry(&table, &column, &strategy, &value, &quote! { vec![] }) // the value of `child_entries`
runtime::reference_integrity_violation_on_create_or_update(&table, &quote! { Create }, &message) // names error::CreateOrUpdate::Create
runtime::error_from_hook_declaration(&quote! { error_from_hook }) // the binding to declare
```

`api::spacetimedb` is new. Its `table_traits_import()` is the `use ::spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};` which every `method_impl` relies on without containing it: put it at the top of each method you render, as `spacetimedsl_derive` does.

#### `SpacetimeDBTable::multi_column_indices` holds only indices over several columns

`SpacetimeDBTable::multi_column_indices` holds only indices over several columns. Before, a second single-column index on a column was left in it; that input is now rejected.

#### `SpacetimeDSLArgType::actual_type()` returns the type as written

`SpacetimeDSLArgType::actual_type()` returns the type of a parameter as written, for both variants.

#### `SpacetimeDSLTable::compile_error_check_imports` lists the pairing imports

`SpacetimeDSLTable::compile_error_check_imports: Vec<syn::Path>` holds the marker traits the tables on the other side of the table's foreign keys have to declare, as the paths to import them from. The `method_impl` of a cascade method no longer contains these imports. Emit them in a block scope, as `spacetimedsl_derive` does with `const _: () = { use …; };`, or the pairing checks are lost.

#### Cascade entry points only where a cascade runs

`SpacetimeDSLTableMethods::on_delete_strategies_of_referencing_tables` is `None` for a table which neither deletes nor soft-deletes rows, even when other tables reference it, and `on_delete_strategies_of_this_table` leaves out a referenced table whose foreign keys declare no strategy. `ForeignKey::on_delete_strategy` and `on_soft_delete_strategy` are `None` exactly when the referenced table does not perform that removal.

#### `Getter::doc_comment` and `Setter::doc_comment`

`Getter` and `Setter` gained `doc_comment: String`: what a foreign key column references and the strategies it declares, empty for any other column, followed by the values its `#[disallow]` rules forbid. Put it in front of the accessor's documentation, as `spacetimedsl_derive` does.

#### `SpacetimeDSLTable::struct_doc_comment`

`SpacetimeDSLTable::struct_doc_comment: String` holds the sections `#[spacetimedsl::dsl]` appends to the struct's documentation, empty for a table without foreign keys and without `#[referenced_by]`. Append it to the struct you emit as a `#[doc]` attribute after an empty one, as `spacetimedsl_derive` does.

#### `SpacetimeDSLColumn::creation_default`

`SpacetimeDSLColumn` gained `creation_default: Option<syn::Expr>`, the expression of `#[creation_default(...)]`, which `create_<table>` fills the column with. Such a column is not a member of `CreateDSLMethodArg::struct_members`.

#### `SpacetimeDSLColumn::disallowed`

`SpacetimeDSLColumn` gained `disallowed: BTreeSet<Disallowed>`, the rules of `#[disallow(...)]`, from the new `api::dsl::disallow::Disallowed`. The checks they add live inside the `method_impl` of the write methods.
