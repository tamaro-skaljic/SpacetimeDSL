# Migration — **SpacetimeDSL**

What changes for a module, or for a crate building on `spacetimedsl_derive-input`, when it moves to a new release of **SpacetimeDSL**.

## 0.25 → 0.26

### Newly rejected inputs

#### A struct named like the table type of its table

Every table now gets a type `{Table}Table`, such as `EntityTable` for `accessor = entity`, through which the tables on the other side of its foreign keys reach it. A struct with that name is rejected: *`#[dsl]` generates the type `EntityTable` for the `entity` table, so the struct cannot be called `EntityTable` as well!* Rename the struct, or change the table's `accessor`. Any other item of that name in the table's module, such as a second struct or a `use`, now fails with `E0428` as well.

#### A private struct paired with a table in another module

Tables paired across modules have to be visible to each other. A struct which a `#[foreign_key]` or a `#[referenced_by]` in another module names fails with `E0603` for its table type and its `<accessor>__TableHandle` when it is private. A referencing struct used to need no visibility. Make the struct `pub(crate)` or `pub`.

### For crates building on `spacetimedsl_derive-input`

#### `SpacetimeDSLTable::pairing` replaces `compile_error_checks` and `compile_error_check_imports`

`SpacetimeDSLTable::compile_error_checks` and `compile_error_check_imports` are gone. `SpacetimeDSLTable::pairing: TokenStream` holds the items which pair the table with the tables on the other side of its foreign keys: the facts it states about itself for their checks, the checks it makes of them, and the traits through which its cascades reach the tables referencing it. Emit it once per table at module level, as `spacetimedsl_derive` does. The cascades of `on_delete_strategies_of_referencing_tables` call into those traits, so they do not compile without it.

The facts are traits of `crate::spacetimedsl::internal`, built by `runtime::accepted_table_trait`, `has_delete_method_trait`, `has_no_delete_method_trait`, `is_soft_deletable_trait`, `is_not_soft_deletable_trait` and `referenced_by_trait`.

#### `api::dsl::table::table_type_name`

`api::dsl::table::table_type_name(&Ident) -> Ident` builds the table type of a table from its accessor, such as `WarehouseTable` for `warehouse`. Emit a `#[doc(hidden)]` unit struct of that name for every table, with the visibility of the table's struct.

#### `SpacetimeDSLTableMethods::get_by_primary_key`

`SpacetimeDSLTableMethods::get_by_primary_key: Option<SpacetimeDSLMethod>` is the lookup the table type offers the tables referencing it, `None` for a singleton. Its first parameter is the read-only DSL. Emit it as an associated function of the table type, generic over `T: ReadContext`.

#### `OnDeleteStrategiesOfTheReferencedTable` is the impl of `internal::References`

- `SpacetimeDSLTableMethods::on_delete_strategies_of_this_table` holds an entry for every table the foreign keys reference, also when they declare no strategy and both pairs are `None`.
- `OnDeleteStrategiesOfTheReferencedTable` gained `referenced_table_type: syn::Path`, such as `crate::warehouse::WarehouseTable`, and `referenced_primary_key_type: syn::Path`.
- Its `CascadeEntryPoints` are methods of `internal::References`, named `after_one_row_was_deleted` and so on, with `&self` before their parameters. Emit one `impl internal::References<referenced_table_type, referenced_primary_key_type> for {Table}Table` per entry, with the methods of both pairs, or empty. `runtime::references_trait` builds the trait's path.

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

#### The preludes export `err!`

`spacetimedsl::prelude`, and with it the prelude `spacetimedsl!()` generates, exports the new macro `err!`. A macro of your own called `err` which another glob import brings into the same scope becomes ambiguous where it is called; import that one by name, which takes precedence over a glob.

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

#### Two single-column indices on one column

A column with two single-column indices — for example `#[index(btree)]` on the field and `index(accessor = by_name, hash(columns = [name]))` in `#[table]` — is rejected with *The column `name` has two single-column indices, `by_name` and `name`!* Before, the second index generated no methods and nothing said so. Remove one of them.

#### `on_delete = SetZero` on a column that is neither an unsigned integer nor a `Uuid`

`on_delete = SetZero` is rejected on a foreign key column that is neither an unsigned integer (`u8`–`u128`, in any spelling) nor a `Uuid`, at the column's type: *`OnDeleteStrategy::SetZero` is only allowed on unsigned integer and `Uuid` columns, …* This includes columns that used to work: signed integers such as `i32` / `i64`, and type aliases such as `type PlayerId = u64`, which the check cannot see through. Other types, such as `String`, used to fail inside the expanded code. Spell an aliased key as its unsigned type, move a signed key to an unsigned one, or choose another strategy (`Delete`, `Ignore`, …).

#### A foreign key which leaves out the strategy of a removal its table performs

Every `#[foreign_key]` has to set `on_delete` when the referenced table has a delete method, and `on_soft_delete` when it is soft-deletable. Two foreign keys of one table to the same table used to be checked together, so one of them could leave out `on_delete` while the other set it; deleting a referenced row then left the rows of the first one pointing at nothing. Each is checked on its own now: *unresolved import `…::this_compilation_error_occurs_because_your_foreign_key_referencing_the_warehouse_table_needs_to_define_a_strategy_for_on_delete_or_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`*. Add the missing strategy.

#### `on_delete = SetZero` on a primary key, a `#[unique]` column or a column of a unique multi-column index

`SetZero` writes `0` or `Uuid::NIL` into the foreign key column and writes the row back through its primary key. On a primary key column that write finds no row or another one; on a `#[unique]` column, or a column of a `unique_index(name = …)` index, a second cleared row repeats the value. SpacetimeDB's `update` panicked inside the cascade in the first two cases, and the third silently broke the index's uniqueness. All three are rejected at the `#[foreign_key]` attribute. Choose another strategy, such as `Delete`, or remove the uniqueness.

#### Two wrapper-type methods of the same name

Every `#[foreign_key]` column besides the primary key now adds a method to the wrapper type of its table's primary key, which looks up the row it references (*Look Up the Referenced Row From a Wrapper* in the documentation). Where that method takes a name another method of the wrapper type already has, the build fails:

- Two tables which reference each other through unique foreign keys, and two tables which share a primary key wrapper and reference the same table: rustc's *duplicate definitions with name `get_…`* (E0592).
- A method of your own on a wrapper type with the name of such a method: E0592.
- Two foreign keys of one table to the same table whose columns differ only in the key suffix, such as `owner_id` and `owner`: *The foreign key columns `owner_id` and `owner` would both add `get_owner` to `GameId`!*

Add `referenced_row_method = false` to the `#[foreign_key]` whose method you do not need, or rename the column.

### Changed messages and generated code

#### `on_delete = SetZero` on `Uuid` foreign keys

`on_delete = SetZero` is available on `Uuid` foreign keys and sets them to `Uuid::NIL`. Create, update and upsert treat a `Uuid` foreign key equal to `Uuid::NIL` as referencing no row, as they treat `0` for an unsigned integer: they no longer report a reference integrity violation for it. The generated create and update methods gain that guard.

### For crates building on `spacetimedsl_derive-input`

#### The constructors of the model are no longer public

The functions `derive-input` used to build the model — among them `RustField::map`, `RustVisibility::map`, `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `SpacetimeDSLColumn::try_parse`, `SpacetimeDSLTable::try_parse`, `WrapperType::try_parse`, `ForeignKey::try_parse`, `ReferencingTable::try_parse`, `UUIDVersion::try_parse`, `Getter::map`, `MutGetter::map`, `Setter::map` and `SpacetimeDSLColumnMethods::map` — are no longer public. `api::Table::try_parse` is the one entry point.

#### `RustVisibility` implements `ToTokens` instead of `Display`

`RustVisibility` no longer implements `Display`; it implements `quote::ToTokens`, which renders `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` or nothing. Compare it with `matches!` rather than as text.

#### `api::attribute::FIELD_ATTRIBUTE_NAMES` lists the field attributes

`api::attribute::FIELD_ATTRIBUTE_NAMES` lists every field attribute `#[spacetimedsl::dsl]` reads (`create_wrapper`, `use_wrapper`, `foreign_key`, `referenced_by`, `set_on_create`, `set_on_update`, `set_on_soft_delete`, `auto_gen`, `creation_default`, `disallow`). A derive of your own that has to accept them as helper attributes can check its list against it.

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

#### `SpacetimeDSLArgType::actual_type()` returns the type as written

`SpacetimeDSLArgType::actual_type()` returns the type of a parameter as written, for both variants.

#### `SpacetimeDSLTable::compile_error_check_imports` lists the pairing imports

`SpacetimeDSLTable::compile_error_check_imports: Vec<syn::Path>` holds the marker traits the tables on the other side of the table's foreign keys have to declare, as the paths to import them from. The `method_impl` of a cascade method no longer contains these imports. Emit them in a block scope, as `spacetimedsl_derive` does with `const _: () = { use …; };`, or the pairing checks are lost.

#### Cascade entry points only where a cascade runs

`SpacetimeDSLTableMethods::on_delete_strategies_of_referencing_tables` is `None` for a table which neither deletes nor soft-deletes rows, even when other tables reference it, and `on_delete_strategies_of_this_table` leaves out a referenced table whose foreign keys declare no strategy. `ForeignKey::on_delete_strategy` and `on_soft_delete_strategy` are `None` exactly when the referenced table does not perform that removal.

#### `SpacetimeDSLColumn::creation_default`

`SpacetimeDSLColumn` gained `creation_default: Option<syn::Expr>`, the expression of `#[creation_default(...)]`, which `create_<table>` fills the column with. Such a column is not a member of `CreateDSLMethodArg::struct_members`.
