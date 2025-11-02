# SpacetimeDSL Copilot Instructions

## Project Overview

SpacetimeDSL is a **procedural macro crate** that provides an ergonomic DSL for [SpacetimeDB](https://spacetimedb.com/). It generates type-safe CRUD operations, enforces referential integrity, and adds features missing from vanilla SpacetimeDB (foreign keys, unique multi-column indices, hooks).

**Key Insight**: SpacetimeDB lacks foreign keys and multi-column unique constraints. SpacetimeDSL **enforces them at compile/runtime** by generating validation code.

## Architecture

### Crate Structure (Workspace)

```
spacetimedsl/             # Main crate - runtime DSL types (DSL, errors)
├─ derive/                # Proc macro entry point (#[dsl], #[hook])
├─ derive-input/          # Parsing logic (transforms syn AST → internal IR)
│  ├─ internal/           # Private IR types (Table, Column, methods)
│  └─ api/                # Public API types (exported to derive/)
├─ examples/
│  ├─ blackholio/         # Real-world example (multiplayer game)
│  └─ test/               # Integration test suite
└─ debug-helper/          # Dev tool for macro expansion debugging
```

**Data Flow**: User code → `derive/lib.rs` → `derive-input` (parse) → `derive/output/*.rs` (codegen) → expanded code

### Key Files

- `derive/src/lib.rs` - Macro entry point, makes fields private
- `derive/src/output.rs` - Orchestrates code generation
- `derive-input/src/internal.rs` - Parsing coordinator
- `derive-input/src/internal/dsl/method.rs` - **Core logic** (3000+ lines, generates all CRUD methods)
- `src/lib.rs` - Runtime types: `DSL<'a>`, `SpacetimeDSLError`, `OnDeleteStrategy`

## Critical Patterns

### 1. Wrapper Types (Type Safety)

**Problem**: Prevent passing wrong IDs (e.g., `PlayerId` where `EntityId` expected)

```rust
#[create_wrapper]           // Creates EntityId wrapper
id: u128,

#[use_wrapper(EntityId)]    // Uses existing wrapper
entity_id: u128,
```

**Generated**:
```rust
pub struct EntityId(u128);
impl Wrapper<u128, EntityId> for EntityId { ... }
impl Display, Clone, PartialEq, SpacetimeType
```

**Naming**: `{TableNamePascalCase}{ColumnNamePascalCase}` (e.g., `UserProfileObjId`)

### 2. Foreign Keys (Manual Enforcement)

SpacetimeDB has **no native foreign keys**. SpacetimeDSL generates checks:

```rust
#[referenced_by(path = crate, table = position)]  // On Entity.id
#[foreign_key(path = crate, table = entity, on_delete = Delete)]  // On Position.entity_id
```

**OnDelete Strategies**:
- `Error` - Block deletion if referenced
- `Delete` - Cascade delete (requires `method(delete = true)`)
- `SetZero` - Set FK to 0 (requires `method(update = true)` + public column)
- `Ignore` - Allow dangling refs (use for audit logs only)

**Critical Rule**: Foreign keys work **ONLY if you use DSL methods**. Bypassing with `ctx.db().table().insert()` breaks integrity!

### 3. Method Configuration

```rust
#[dsl(plural_name = entities, method(update = true, delete = false))]
```

**Compile-time validation**:
- `update = true` requires: public field OR `modified_at`/`updated_at` column
- `delete = true` required if: any FK references with `on_delete = Delete`
- Hooks require matching method (`hook(after(update))` needs `method(update = true)`)

### 4. Hooks System

```rust
#[dsl(hook(before(insert, update), after(delete)))]

#[spacetimedsl::hook]
fn before_player_insert(
    dsl: &impl DSLContext,
    mut req: CreatePlayer,
) -> Result<CreatePlayer, SpacetimeDSLError> {
    req.name = req.name.to_uppercase();
    Ok(req)  // Must return modified request
}
```

**Naming**: `{before|after}_{table_name}_{insert|update|delete}`

### 5. Unique Multi-Column Indices

```rust
#[dsl(unique_index(name = x_y))]  // References SpacetimeDB index
#[spacetimedb::table(index(name = x_y, btree(columns = [x, y])))]
```

Generates: `get_position_by_x_y(&x, &y)`, validates uniqueness on insert/update.

## Developer Workflows

### Running Tests

```powershell
# Integration tests (publishes to local SpacetimeDB)
.\test.cmd

# Unit tests
cargo test --workspace
```

### Debugging Macros

```powershell
# Expand macros to see generated code
.\debug.ps1

# Output:
#   debug-helper/output/lib.expanded.rs  (macro expansion)
#   debug-helper/output/lib.rs.ast      (AST dump)
```

### Release Process

1. Update versions in all `Cargo.toml` files
2. `git commit -m "Release v0.X.Y" && git tag v0.X.Y && git push --tags`
3. GitHub Actions auto-publishes to crates.io (if tests pass)

### Building

```powershell
cargo build --workspace   # All crates
cargo build -p spacetimedsl_derive  # Just proc macro
```

## Common Pitfalls

### 1. Trait Bound Errors with Wrappers

**Error**: `The trait bound WrapperType: From<u128> is not satisfied`

**Cause**: Passed raw numeric where wrapper expected

**Fix**: Use `entity.get_id()` or `EntityId::new(123)`, not bare `123`

### 2. Bypassing DSL Methods

**Never do this**:
```rust
ctx.db().position().insert(position);  // ❌ Breaks FK integrity
```

**Always**:
```rust
dsl.create_position(CreatePosition { ... })?;  // ✅ Enforces FK checks
```

### 3. Update Method Not Generated

**Error**: Method `update_entity_by_id` doesn't exist

**Cause**: No public fields AND no `modified_at`/`updated_at` column

**Fix**: Add `pub` to a field OR add `modified_at: Option<Timestamp>`

### 4. Hook Naming Mismatch

**Error**: Hook not called

**Cause**: Wrong name (e.g., `before_entity_create` instead of `before_entity_insert`)

**Fix**: Use exact pattern `{before|after}_{table_name}_{insert|update|delete}`

## Code Generation Internals

### How `create_entity()` is Generated

1. Parse `#[dsl(...)]` attrs in `derive-input/src/internal.rs`
2. Build `Table` IR with columns, indices, FK relationships
3. `derive-input/src/internal/dsl/method.rs::DSLMethod::Create` builds:
   - Trait `CreateEntityRow`
   - Struct `CreateEntity` (with smart defaults)
   - Method `create_entity(&self) -> Result<Entity, SpacetimeDSLError>`
   - FK validation, unique index checks, hook calls
4. `derive/src/output.rs` assembles traits into `TokenStream`

### Field Visibility Trick

`derive/src/lib.rs` makes fields private **after parsing** to:
- Generate correct setters (based on original `pub` visibility)
- Force use of getters/setters
- Prevent bypassing DSL validation

## Important Conventions

### File Organization

- `derive-input/src/internal/{rust,db,dsl}/` - Separate concerns by layer
- `derive/src/output/function/` - Function generation split by type
- Examples follow SpacetimeDB conventions (single `lib.rs`)

### Error Handling

Use `SpacetimeDSLError` variants with metadata:
```rust
Err(SpacetimeDSLError::NotFoundError {
    table_name: "position".into(),
    column_names_and_row_values: format!("{{ id: {} }}", id).into(),
})
```

### Documentation

- User-facing docs: `README.md` (comprehensive examples)
- LLM-optimized: `llms.txt` (concise reference)
- API docs: Inline doc comments in `src/lib.rs`

## LLM-Specific Guidance

**Before suggesting changes**:
1. Check if it affects `derive-input/src/internal/dsl/method.rs` (most logic here)
2. Verify compile-time validation won't break (see TODOs in code)
3. Test with `examples/blackholio` (real-world complexity)

**Common requests**:
- **New method types**: Modify `DSLMethod` enum, add to `build_method_information()`
- **New attributes**: Add to `derive-input/src/internal/dsl/` parsing
- **Error types**: Add to `src/lib.rs::SpacetimeDSLError` enum

**Unstable features** (per TODOs):
- `SetNone` strategy (Option not allowed on indices yet)
- `try_update` (SpacetimeDB doesn't have it)
- IndexScanRangeBounds support

Refer to `llms.txt` for complete feature reference and examples.
