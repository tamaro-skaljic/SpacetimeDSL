# Migration — **SpacetimeDSL**

What changes for a module, or for a crate building on `spacetimedsl_derive-input`, when it moves to a new release of **SpacetimeDSL**.

## 0.23 → 0.24

### Breaking API changes

### Newly rejected inputs

### Changed messages and generated code

### For crates building on `spacetimedsl_derive-input`

- The crate documents what you may rely on: `api::Table::try_parse` is the one entry point, and every public field of every `api` type is part of the semver contract. A test in `spacetimedsl_derive` destructures the whole structure without `..`, so a change to it cannot land unnoticed. The generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`, …) are opaque output to splice into your expansion, not to inspect.
