# Singleton Table Support Implementation Plan

## Summary

Add `#[dsl(singleton)]` attribute support for tables that hold 0-1 rows (e.g. config/state tables). The macro automatically injects `#[primary_key] id: ()`, removes `_by_id` suffixes from methods, omits iter/count methods, disallows user-defined constraints, and adapts FK logic.

## User-Facing Syntax

```rust
#[dsl(singleton, method(update = true, delete = true))]
#[table(name = active_round, public)]
pub struct ActiveRound {
    pub current_player: u64,
    pub round_number: u32,
}
```

**Generated methods:**
- `create_active_round(CreateActiveRound { ... })` — inserts a row (same as normal)
- `get_active_round()` → `Option<ActiveRound>` (no parameter, no error)
- `update_active_round(active_round: ActiveRound)` → `Result<ActiveRound, SpacetimeDSLError>` (no parameter for pk lookup)
- `delete_active_round()` → `Result<DeletionResult, SpacetimeDSLError>` (no parameter)
- **NOT generated:** `get_all_*`, `count_of_all_*`

## Implementation Steps

### Step 1: Parse `singleton` in DSL attribute args
**File:** `derive-input/src/internal.rs`

- Add `is_singleton: bool` to `DSLData` struct.
- In `try_parse_dsl()`, add a `singleton` match arm (as a flag, no value).
- When `singleton` is set:
  - `plural_name` is NOT required (keep it `Option<Ident>` internally).
  - Error if `unique_index(...)` is also specified.
- When `singleton` is NOT set:
  - `plural_name` remains required (existing behavior).

### Step 2: Validate singleton constraints in `SpacetimeDSLTable::try_parse`
**File:** `derive-input/src/internal/dsl/table.rs`

When `is_singleton`:
- Validate that `unique_indices` is empty (error if any `unique_index` declared).
- Set `plural_name` to a dummy value (same as singular_name, since it's only used for `get_all_*`/`count_of_all_*` which won't be generated).

### Step 3: Validate singleton has exactly one `#[table]` attribute
**File:** `derive-input/src/internal/integration.rs`

- When `is_singleton`, instead of `select_table_with_heuristics`, check that there's exactly 1 `#[table]` attribute, error otherwise.
- Pass `is_singleton` flag to `spacetime_bindings_macro_input()`.

### Step 4: Inject `#[primary_key] id: ()` into struct AST
**File:** `derive/src/lib.rs`

- When `singleton` is detected in args (before full parsing), modify the `DeriveInput` to:
  1. Error if user has a field named `id` with type `()`.
  2. Add `#[primary_key] id: ()` as the first field of the struct.
- This happens BEFORE `Table::try_parse()` so SpacetimeDB's `#[table]` also processes it.

### Step 5: Validate disallowed attributes on singleton fields
**File:** `derive-input/src/internal/column.rs` (and/or `derive-input/src/internal/dsl/table.rs`)

When `is_singleton`:
- Error if any user-defined field has `#[primary_key]`.
- Error if any field has `#[index]` or `#[unique]`.
- Error if any multi-column indices exist on the `#[table]` attribute.
- Error if any field has `#[referenced_by]`.
- Allow `#[foreign_key]` WITHOUT requiring `#[index]`/`#[unique]`/`#[primary_key]` (skip the check in `ForeignKey::try_parse`).

### Step 6: Propagate `is_singleton` flag through the type system
**Files:** `derive-input/src/api/dsl/table.rs`, `derive-input/src/internal/dsl/table.rs`, `derive-input/src/internal.rs`

- Add `pub is_singleton: bool` to `SpacetimeDSLTable`.
- Thread `is_singleton` from `DSLData` → `SpacetimeDSLTable` → used in method generation.

### Step 7: Add new DSL method variants for singleton
**File:** `derive-input/src/internal/dsl/method.rs`

Add new `DSLMethod` variants:
- `DSLMethod::GetSingleton` — generates `get_{singular_name}()` → `Option<T>`, implementation: `self.db().{accessor}().iter().next()`
- `DSLMethod::UpdateSingleton` — generates `update_{singular_name}(entity: T)` → `Result<T, Error>`, implementation: same as Update but pk value is always `()` (no pk parameter needed by the user)
- `DSLMethod::DeleteSingleton` — generates `delete_{singular_name}()` → `Result<DeletionResult, Error>`, implementation: find via `iter().next()`, then delete via pk

### Step 8: Modify `SpacetimeDSLTableMethods::try_parse` for singletons
**File:** `derive-input/src/internal/dsl/method.rs`

When `is_singleton`:
- Generate `create` method as normal.
- Generate `GetSingleton` instead of `GetAll`.
- Skip `GetCount`.
- Skip all per-column method generation for the pk column (no `get_by_id`, `update_by_id`, `delete_by_id`).
- Generate `UpdateSingleton` and `DeleteSingleton` as table-level methods.

### Step 9: Update `SpacetimeDSLTableMethods` struct
**File:** `derive-input/src/api/dsl/table.rs`

- Change `get_all` and `get_count` to `Option<SpacetimeDSLMethod>` (None for singletons).
- Add `update_singleton: Option<SpacetimeDSLMethod>` and `delete_singleton: Option<SpacetimeDSLMethod>`.

### Step 10: Update output generation
**File:** `derive/src/output.rs`

- Only emit `get_all` and `get_count` if they are `Some`.
- Emit `update_singleton` and `delete_singleton` if they are `Some`.

### Step 11: FK on singleton — skip index requirement
**File:** `derive-input/src/internal/dsl/foreign_key.rs`

- Pass `is_singleton` to `ForeignKey::try_parse`.
- When `is_singleton`, skip the `has_index` check (line 35-39).

### Step 12: FK cascade implementation for singletons
**File:** `derive-input/src/internal/dsl/method.rs`

In `get_on_delete_strategy_implementation` and `for_foreign_key`:
- When the current table is a singleton (check via `spacetimedsl_table.is_singleton`):
  - The `row_finder` should use `dsl.db().{accessor}().iter().next()` (for ctx-level code) or `dsl.get_{singular_name}()` (for DSL-level code) instead of `filter_by_{column}`.
  - Since there's at most 1 row and no index, iterate and check the FK column value manually.

### Step 13: Singleton pk column — no wrapper, no getter/setter generation
**Files:** `derive-input/src/internal/column.rs`, `derive-input/src/internal/dsl/method.rs`

- The injected `id: ()` column should:
  - Not generate a wrapper type.
  - Not generate getter/setter/mut_getter (since it's always `()`).
  - Be skipped in the `Create{Table}` struct generation.
  - Be handled in `process_columns_for_create_and_update_method` to always use `()` as the constructor arg.

### Step 14: Add example/test for singleton tables
**File:** `examples/test/src/lib.rs`

Add a singleton table to the test suite:
```rust
#[dsl(singleton, method(update = true, delete = true))]
#[table(name = active_round, public)]
pub struct ActiveRound {
    pub current_player: u64,
    pub round_number: u32,
}
```

Test create, get, update, delete operations.

### Step 15: Update README
**File:** `README.md`

Add singleton table documentation with example usage.

### Step 16: Build and test
- `cargo build --workspace`
- `cargo test --workspace`
