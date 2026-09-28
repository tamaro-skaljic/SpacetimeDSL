# Migration — **SpacetimeDSL**

What changes for a module, or for a crate building on `spacetimedsl_derive-input`, when it moves to a new release of **SpacetimeDSL**.

## 0.23 → 0.24

### Breaking API changes

- The `Wrapper` trait lost its second type parameter, which carried no information. A hand-written wrapper type implements `Wrapper<WrappedType>`:

  ```rust
  // 0.23
  impl spacetimedsl::Wrapper<i32, ConfigId> for ConfigId { /* … */ }

  // 0.24
  impl spacetimedsl::Wrapper<i32> for ConfigId { /* … */ }
  ```

- `ReferenceIntegrityViolationError::OnCreateOrUpdate::create_or_update` has the new type `error::CreateOrUpdate` instead of `error::Action`, so it can no longer hold an action that is not a write, and formatting the error can no longer panic:

  ```rust
  // 0.23
  if let ReferenceIntegrityViolationError::OnCreateOrUpdate { create_or_update: Action::Create, .. } = error { /* … */ }

  // 0.24
  if let ReferenceIntegrityViolationError::OnCreateOrUpdate { create_or_update: CreateOrUpdate::Create, .. } = error { /* … */ }
  ```

### Newly rejected inputs

- A struct with several `#[table]` attributes needs `table = <accessor>` on each `#[dsl]`, naming the table it belongs to. Before, the `plural_name` was used to guess one, and an arbitrary table was used when the guess failed.

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

  Without it: *This struct has 2 `#[table]` attributes (`offline_player`, `online_player`), so `#[dsl]` has to name the one it belongs to!* A `table` that names no `#[table]` of the struct is rejected with *No `#[table]` attribute of this struct has the accessor `…`!*, also on a struct with a single `#[table]`.

- `unique_index(name = …)` must name an index of the table, and each index only once. A misspelled name used to be accepted without effect; now it is rejected with *No index of this table has the accessor `…`! Its indices: …*. A repeated name is rejected with *`unique_index(name = …)` is given twice!* Fix the name, or remove the repetition. A name of a hash or single-column index is still accepted without effect.

- On a `singleton(with_default)` table, a `Timestamp` column is rejected in every spelling. Before, a qualified spelling such as `::spacetimedb::Timestamp` slipped past the check. Use `Option<Timestamp>`, as the diagnostic says.

- A column whose type is not a path — an array such as `[u8; 4]`, a tuple, a reference, and the like — used to make the macro panic (*should be parseable as Path*). It now gets a diagnostic on the type: *SpacetimeDSL supports only path types as column types, such as `u64`, `String` or `spacetimedb::Timestamp`! This column's type is an array.* Wrap the value in a type of your own.

- A column with two single-column indices — for example `#[index(btree)]` on the field and `index(accessor = by_name, hash(columns = [name]))` in `#[table]` — is rejected with *The column `name` has two single-column indices, `by_name` and `name`!* Before, the second index generated no methods and nothing said so. Remove one of them.

- `on_delete = SetZero` is rejected on a foreign key column that is neither an unsigned integer nor a `Uuid`, at the column's type: *`OnDeleteStrategy::SetZero` is only allowed on unsigned integer and `Uuid` columns, …* Before, it failed inside the expanded code. Choose another strategy for such a column.

### Changed messages and generated code

- `on_delete = SetZero` is available on `Uuid` foreign keys and sets them to `Uuid::NIL`. Create, update and upsert treat a `Uuid` foreign key equal to `Uuid::NIL` as referencing no row, as they treat `0` for an unsigned integer: they no longer report a reference integrity violation for it. The generated create and update methods gain that guard.
- Foreign keys of one table to the same table may spell their type and path differently (`u64` / `core::primitive::u64`, `::my_crate::tables` / `my_crate::tables`); they used to be rejected as mismatched. A real mismatch is still rejected, and the path message now adds *Spell both paths the same way; a leading `::` makes no difference.*
- A foreign key column counts as indexed through every single-column index on it, including one declared as `index(…)` in `#[table]`; before, only `#[primary_key]`, `#[unique]` and `#[index]` on the field counted.

- Qualified spellings of the types SpacetimeDSL checks are accepted where they used to be rejected: `::spacetimedb::Timestamp` and `std::option::Option<spacetimedb::Timestamp>` for `#[set_on_create]` / `#[set_on_update]`, and `core::primitive::u64` / `std::primitive::u64` count as unsigned integers (so a foreign key spelled that way skips its reference-integrity check for `0`, like `u64`). See *Column Type Spellings* in the documentation. This is not breaking.

- The diagnostic for a missing `method(update = …)` on a table with only private columns names `#[set_on_update]` next to the conventional `modified_at` / `updated_at` as a way to make the table mutable.

- Diagnostics about a column's visibility print it as written — `` `pub` ``, `` `pub(crate)` ``, `` `pub(in path)` `` — and ask for "no visibility modifier" instead of naming `syn` types such as `Visibility::Inherited` or `Visibility::Public`. The soft-delete marker diagnostic now also says which visibility it found.

- A struct without a `#[table]` attribute is rejected with *Haven't found a `#[table]`/`#[spacetimedb::table]` attribute on this struct! `#[dsl]`/`#[spacetimedsl::dsl]` builds on the table it declares.* It used to ask for `#[dsl]` to be directly above a `#[table]`, which was never the rule.

- The root of the `spacetimedsl` crate re-exports everything in the new `spacetimedsl::prelude`, which adds the context accessor traits, `OnDeleteStrategyFailure`, `NewUUID` and `Itertools` to what `spacetimedsl::X` reaches. This is additive.
- `#[::spacetimedsl::dsl(…)]` and `#[::spacetimedb::table(…)]`, spelled with a leading `::`, are recognised like `#[spacetimedsl::dsl(…)]` and `#[spacetimedb::table(…)]`. Before, a `::spacetimedb::table` attribute was not found, so the struct was rejected for missing a table attribute.
- Generated code reaches the runtime only through `crate::spacetimedsl`, the module `spacetimedsl!()` generates, and SpacetimeDB only through `::spacetimedb`. A `#[create_wrapper]` table declared at the crate root with `#[::spacetimedsl::dsl]`, next to `spacetimedsl!()`, therefore compiles: before, the generated `use spacetimedsl::Wrapper;` was ambiguous there.

- Where a create method or setter takes an optional used wrapper, the generated code converts it with one `Option::map` (`let due_at = due_at.map(|value| Into::<ReminderDueAt>::into(value).value());`) instead of a mutable `None`, an `is_some()` test and an `expect`. Only the code rustdoc shows under *Implementation* changes.

### For crates building on `spacetimedsl_derive-input`

- The crate documents what you may rely on: `api::Table::try_parse` is the one entry point, and every public field of every `api` type is part of the semver contract. A test in `spacetimedsl_derive` destructures the whole structure without `..`, so a change to it cannot land unnoticed. The generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`, …) are opaque output to splice into your expansion, not to inspect.
- The functions `derive-input` used to build the model — among them `RustField::map`, `RustVisibility::map`, `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `SpacetimeDSLColumn::try_parse`, `SpacetimeDSLTable::try_parse`, `WrapperType::try_parse`, `ForeignKey::try_parse`, `ReferencingTable::try_parse`, `UUIDVersion::try_parse`, `Getter::map`, `MutGetter::map`, `Setter::map` and `SpacetimeDSLColumnMethods::map` — are no longer public. `api::Table::try_parse` is the one entry point.
- `RustVisibility` no longer implements `Display`; it implements `quote::ToTokens`, which renders `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` or nothing. Compare it with `matches!` rather than as text.
- `api::attribute::FIELD_ATTRIBUTE_NAMES` lists every field attribute `#[spacetimedsl::dsl]` reads (`create_wrapper`, `use_wrapper`, `foreign_key`, `referenced_by`, `set_on_create`, `set_on_update`, `set_on_soft_delete`, `auto_gen`). A derive of your own that has to accept them as helper attributes can check its list against it.
- `api::attribute::is_dsl_attribute(&Attribute)` tells whether an attribute is `#[spacetimedsl::dsl]` in any spelling: `dsl`, `spacetimedsl::dsl` or `::spacetimedsl::dsl`. Use it instead of comparing the stringified path.
- `SpacetimeDSLMethodHooks` is a map keyed by the new `api::dsl::hook::HookKind` (a `Timing` and an `Operation`) instead of eight fields:

  ```rust
  // 0.23
  if let Some(hook) = &table.spacetimedsl_table.hooks.before_insert { /* … */ }

  // 0.24
  if let Some(hook) = table.spacetimedsl_table.hooks.get(HookKind::BEFORE_INSERT) { /* … */ }
  ```

  `hooks.iter()` yields the declared hooks in emission order, and `hooks.declared` is the `BTreeMap<HookKind, SpacetimeDSLMethodHook>` itself. `HookKind::ALL` lists all eight kinds.
- `api::dsl::hook::hook_trait_name(&Ident)` maps a hook function name to the trait it implements (`before_entity_insert` → `BeforeEntityInsertHook`). Both the generator and `#[spacetimedsl::hook]` use it; a macro implementing hooks should too.
- `ScheduledReducer::reducer_name: Ident` became `ScheduledReducer::reducer_path: syn::Path`, the reducer or procedure exactly as `scheduled(...)` names it. A qualified path such as `scheduled(crate::timers::tick)` used to make the macro panic. Use `reducer_path.get_ident()` where you need a bare name.
- `OnDeleteStrategiesOfReferencingTables::entry_points()` and `OnDeleteStrategiesOfTheReferencedTable::entry_points()` yield the `CascadeEntryPoints` each holds, those for deletion before those for soft deletion, so you need not name `on_deletion` and `on_soft_deletion` yourself.
- `SpacetimeDSLArgType::actual_type()` returns the type of a parameter as written, for both variants.
