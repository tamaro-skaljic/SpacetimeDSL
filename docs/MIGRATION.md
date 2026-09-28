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

### Newly rejected inputs

### Changed messages and generated code

- The root of the `spacetimedsl` crate re-exports everything in the new `spacetimedsl::prelude`, which adds the context accessor traits, `OnDeleteStrategyFailure`, `NewUUID` and `Itertools` to what `spacetimedsl::X` reaches. This is additive.
- Generated code reaches the runtime only through `crate::spacetimedsl`, the module `spacetimedsl!()` generates, and SpacetimeDB only through `::spacetimedb`. A `#[create_wrapper]` table declared at the crate root with `#[::spacetimedsl::dsl]`, next to `spacetimedsl!()`, therefore compiles: before, the generated `use spacetimedsl::Wrapper;` was ambiguous there.

### For crates building on `spacetimedsl_derive-input`

- The crate documents what you may rely on: `api::Table::try_parse` is the one entry point, and every public field of every `api` type is part of the semver contract. A test in `spacetimedsl_derive` destructures the whole structure without `..`, so a change to it cannot land unnoticed. The generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`, …) are opaque output to splice into your expansion, not to inspect.
- The functions `derive-input` used to build the model — among them `RustField::map`, `RustVisibility::map`, `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `SpacetimeDSLColumn::try_parse`, `SpacetimeDSLTable::try_parse`, `WrapperType::try_parse`, `ForeignKey::try_parse`, `ReferencingTable::try_parse`, `UUIDVersion::try_parse`, `Getter::map`, `MutGetter::map`, `Setter::map` and `SpacetimeDSLColumnMethods::map` — are no longer public. `api::Table::try_parse` is the one entry point.
- `RustVisibility` no longer implements `Display`; it implements `quote::ToTokens`, which renders `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` or nothing. Compare it with `matches!` rather than as text.
- `SpacetimeDSLArgType::actual_type()` returns the type of a parameter as written, for both variants.
