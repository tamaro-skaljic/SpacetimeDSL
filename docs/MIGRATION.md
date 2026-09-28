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

### Changed messages and generated code

- The root of the `spacetimedsl` crate re-exports everything in the new `spacetimedsl::prelude`, which adds the context accessor traits, `OnDeleteStrategyFailure`, `NewUUID` and `Itertools` to what `spacetimedsl::X` reaches. This is additive.
- `#[::spacetimedsl::dsl(…)]` and `#[::spacetimedb::table(…)]`, spelled with a leading `::`, are recognised like `#[spacetimedsl::dsl(…)]` and `#[spacetimedb::table(…)]`. Before, a `::spacetimedb::table` attribute was not found, so the struct was rejected for missing a table attribute.
- Generated code reaches the runtime only through `crate::spacetimedsl`, the module `spacetimedsl!()` generates, and SpacetimeDB only through `::spacetimedb`. A `#[create_wrapper]` table declared at the crate root with `#[::spacetimedsl::dsl]`, next to `spacetimedsl!()`, therefore compiles: before, the generated `use spacetimedsl::Wrapper;` was ambiguous there.

### For crates building on `spacetimedsl_derive-input`

- The crate documents what you may rely on: `api::Table::try_parse` is the one entry point, and every public field of every `api` type is part of the semver contract. A test in `spacetimedsl_derive` destructures the whole structure without `..`, so a change to it cannot land unnoticed. The generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`, …) are opaque output to splice into your expansion, not to inspect.
- The functions `derive-input` used to build the model — among them `RustField::map`, `RustVisibility::map`, `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `SpacetimeDSLColumn::try_parse`, `SpacetimeDSLTable::try_parse`, `WrapperType::try_parse`, `ForeignKey::try_parse`, `ReferencingTable::try_parse`, `UUIDVersion::try_parse`, `Getter::map`, `MutGetter::map`, `Setter::map` and `SpacetimeDSLColumnMethods::map` — are no longer public. `api::Table::try_parse` is the one entry point.
- `RustVisibility` no longer implements `Display`; it implements `quote::ToTokens`, which renders `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` or nothing. Compare it with `matches!` rather than as text.
- `api::attribute::is_dsl_attribute(&Attribute)` tells whether an attribute is `#[spacetimedsl::dsl]` in any spelling: `dsl`, `spacetimedsl::dsl` or `::spacetimedsl::dsl`. Use it instead of comparing the stringified path.
- `SpacetimeDSLMethodHooks` is a map keyed by the new `api::dsl::hook::HookKind` (a `Timing` and an `Operation`) instead of eight fields:

  ```rust
  // 0.23
  if let Some(hook) = &table.spacetimedsl_table.hooks.before_insert { /* … */ }

  // 0.24
  if let Some(hook) = table.spacetimedsl_table.hooks.get(HookKind::BEFORE_INSERT) { /* … */ }
  ```

  `hooks.iter()` yields the declared hooks in emission order, and `hooks.declared` is the `BTreeMap<HookKind, SpacetimeDSLMethodHook>` itself. `HookKind::ALL` lists all eight kinds.
- `SpacetimeDSLArgType::actual_type()` returns the type of a parameter as written, for both variants.
