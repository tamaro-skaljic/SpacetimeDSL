<!-- markdownlint-disable MD024 -->

# SpacetimeDSL Feature Reference

SpacetimeDSL is the first and only SpacetimeDB Rust Server Module meta-framework. It is a procedural macro crate that provides an ergonomic DSL for SpacetimeDB, generates type-safe CRUD operations, enforces referential integrity, and adds features missing from vanilla SpacetimeDB: foreign keys with cascade strategies, unique multi-column indices, before/after hooks, wrapper types, automatic accessors, smart defaults, and rich error types. This document is a comprehensive, self-contained feature reference for LLM coding agents working with SpacetimeDSL.

## Installation & Setup

### Cargo.toml

```toml
[dependencies]
spacetimedsl = "0.18.1"
```

### Required Import (Prelude)

Add this import at the top of every file that uses DSL features, alongside your `use spacetimedb::...` imports:

```rust
use spacetimedsl::prelude::*;
```

### Prelude Exports

The prelude provides these types and functions:

- `DSL`, `ReadOnlyDSL` — DSL context structs
- `DSLContext`, `ReadOnlyDSLContext` — DSL context traits
- `Wrapper` — trait for wrapper types
- `DeletionResult`, `DeletionResultEntry` — deletion audit types
- `dsl`, `read_only_dsl` — constructor functions
- `SpacetimeDSLError`, `ReferenceIntegrityViolationError` — error types
- `hook` — hook attribute macro
- `WriteContext`, `ReadContext` — context traits
- `GetAuth`, `GetSender`, `GetTimestamp`, `GetConnectionId`, `GetModuleIdentity`, `GetRandom`, `GetRandomNumberGenerator`, `GetImmutableDatabase`, `GetMutableDatabase`, `AsReducerContext`, `AsViewContext`, `AsAnonymousViewContext` — context accessor traits
- `Itertools` — re-exported from `itertools` crate

### Cross-Module Imports

The prelude imports DSL core types from `spacetimedsl`. However, generated traits and types (like `CreateEntityRow`, `EntityId`, `GetEntityRowOptionByObjId`) live in the user's own crate modules. In single-module projects, everything is in scope. In multi-module projects, you must import generated traits explicitly:

```rust
use crate::entity::{
    CreateEntityRow, EntityId, GetEntityRowOptionByObjId,
    DeleteEntityRowByObjId, UpdateEntityRowByObjId,
};
```

The `spacetimedsl::itertools` re-export is also available if you need itertools without adding a direct dependency.

---

## SpacetimeDB Basics Through the DSL Lens

All examples below use SpacetimeDSL syntax. This section covers only server-side patterns — no client SDK content.

### Table Definitions

Tables use `#[table(...)]` macro on `pub struct`. Do NOT derive `SpacetimeType` on tables.

**Vanilla SpacetimeDB:**

```rust
use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp};

#[table(accessor = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,

    #[unique]
    username: String,

    online: bool,
}

#[table(accessor = message, public)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    id: u64,

    sender: Identity,
    text: String,
    sent: Timestamp,
}

#[table(accessor = task, public)]
pub struct Task {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    #[index(btree)]
    pub owner_id: Identity,

    pub title: String,
}
```

> **Note on index syntax:** Use `#[index(btree)]` on a single column for single-column indices. Use table-level `index(accessor = ..., btree(columns = [...]))` only for multi-column indices.

**With SpacetimeDSL:**

```rust
use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp};
use spacetimedsl::prelude::*;

#[dsl(plural_name = users, method(update = true, delete = true))]
#[table(accessor = user, public)]
pub struct User {
    #[primary_key]
    #[create_wrapper]
    identity: Identity,

    #[unique]
    #[create_wrapper]
    username: String,

    pub online: bool,
}

#[dsl(plural_name = messages, method(update = false))]
#[table(accessor = message, public)]
pub struct Message {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    sender: Identity,
    text: String,
    sent: Timestamp,
}

#[dsl(plural_name = tasks, method(update = true, delete = true))]
#[table(accessor = task, public)]
pub struct Task {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(UserId)]
    pub owner_id: Identity,

    pub title: String,
}
```

#### Table Options

```rust
#[table(accessor = my_table)]           // Private table (default)
#[table(accessor = my_table, public)]   // Public — clients can subscribe
```

#### Column Attributes

**Vanilla SpacetimeDB:**

```rust
#[primary_key]           // Primary key (auto-indexed, enables .find())
#[auto_inc]              // Auto-increment (use with #[primary_key])
#[unique]                // Unique constraint (auto-indexed)
#[index(btree)]          // B-Tree index for range/filter queries
```

**With SpacetimeDSL (additional attributes):**

```rust
#[create_wrapper]        // Generates a wrapper type (e.g., EntityId) — required on #[primary_key], #[unique], #[index] columns
#[create_wrapper(Name)]  // Generates a wrapper with a custom name
#[use_wrapper(Name)]     // Reuses an existing wrapper type
#[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]  // Foreign key constraint
#[referenced_by(path = self, table = position)]                               // Marks PK as referenced by another table's FK
```

### ReducerContext API

**Vanilla SpacetimeDB:**

```rust
ctx.sender()     // Method call — returns Identity of the caller
ctx.timestamp    // Field access — current Timestamp
ctx.db           // Field access — database handle
ctx.rng()        // Method call — deterministic RNG
```

**With SpacetimeDSL:**

```rust
let dsl = spacetimedsl::dsl(ctx);
dsl.ctx().sender()     // Identity of the caller
dsl.ctx().timestamp    // Current Timestamp
dsl.ctx().db           // Database handle (avoid using directly — prefer DSL methods)
dsl.ctx().rng()        // Deterministic RNG
```

**Common mistakes**:

- `ctx.sender` (field access) — WRONG, use `ctx.sender()` or `dsl.ctx().sender()` (method call)
- `ctx.timestamp()` (method call) — WRONG, use `ctx.timestamp` or `dsl.ctx().timestamp` (field access)
- `ctx.db()` (method call) — WRONG, use `ctx.db` or `dsl.ctx().db` (field access)
- `ctx.rng` (field access) — WRONG, use `ctx.rng()` or `dsl.ctx().rng()` (method call)

### Reducers

```rust
#[reducer]
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);

    if text.is_empty() {
        return Err(SpacetimeDSLError::Error("Message cannot be empty".to_string()));
    }

    // Use DSL methods for all database operations
    dsl.create_message(CreateMessage {
        sender: dsl.ctx().sender(),
        text,
    })?;

    Ok(())
}
```

Key rules:

- Context is always `&ReducerContext` (immutable reference)
- Return `Result<(), SpacetimeDSLError>` (auto-converts to `String` for SpacetimeDB)
- Never return data from reducers — they are transactional
- Reducers must be deterministic — no filesystem, network, or `std::time`

### Lifecycle Reducers

```rust
#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    // Called when module is first published
    Ok(())
}

#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    // ctx.sender() is the connecting identity
    Ok(())
}

#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    Ok(())
}
```

### Scheduled Tables

DSL does not do anything special with `ScheduleAt` columns — use standard SpacetimeDB patterns:

```rust
use spacetimedb::{table, reducer, ReducerContext, ScheduleAt, Timestamp};

#[dsl(plural_name = cleanup_jobs, method(update = false))]
#[table(accessor = cleanup_job, scheduled(cleanup_expired))]
pub struct CleanupJob {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,

    scheduled_at: ScheduleAt,
    target_id: u64,
}

#[reducer]
pub fn cleanup_expired(ctx: &ReducerContext, job: CleanupJob) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    // Do something...
    // Job row is auto-deleted after reducer completes
    log::info!("Cleaning up: {}", job.target_id);
    Ok(())
}

// Schedule a job
#[reducer]
pub fn schedule_cleanup(ctx: &ReducerContext, target_id: u64, delay_ms: u64) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    let future_time = dsl.ctx().timestamp()? + std::time::Duration::from_millis(delay_ms);
    dsl.create_cleanup_job(CreateCleanupJob {
        scheduled_at: ScheduleAt::Time(future_time),
        target_id,
    })?;
    Ok(())
}

// Cancel by deleting the row
#[reducer]
pub fn cancel_cleanup(ctx: &ReducerContext, job_id: u64) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    dsl.delete_cleanup_job_by_scheduled_id(CleanupJobScheduledId::new(job_id))?;
    Ok(())
}
```

### Procedures

Procedures allow side effects (HTTP, filesystem) that reducers cannot do. Use the `try_with_tx` + DSL pattern for database access inside procedures:

```rust
#[spacetimedb::procedure]
pub fn my_procedure(ctx: &mut ProcedureContext) -> Result<(), SpacetimeDSLError> {
    ctx.try_with_tx(|ctx| {
        let dsl = spacetimedsl::dsl(ctx);
        // Use DSL methods here
        Ok(())
    })
}
```

### Views

Views use `read_only_dsl(ctx)` — this is for views only, never for reducers:

```rust
#[spacetimedb::view(accessor = my_view, public)]
pub fn my_view(ctx: &ViewContext) -> Option<Entity> {
    let _dsl = spacetimedsl::read_only_dsl(ctx);
    ctx.db.entity().id().find(0)
}
```

### Custom Types

Use `#[derive(SpacetimeType)]` for non-table structs/enums used as fields or reducer parameters. Never derive `SpacetimeType` on `#[table]` structs.

```rust
use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum LoginStatus {
    LoggedIn,
    LoggedOut,
}
```

### Logging

```rust
use spacetimedb::log;

log::trace!("Detailed trace");
log::debug!("Debug info");
log::info!("Information");
log::warn!("Warning");
log::error!("Error occurred");
```

---

## DSL Core Concepts

### Creating the DSL Context

For reducers (write access):

```rust
let dsl = spacetimedsl::dsl(ctx);
```

For views only (read-only access):

```rust
let dsl = spacetimedsl::read_only_dsl(ctx);
```

Do NOT use `read_only_dsl` in reducers — it is for views only.

### Accessing the Underlying Context

```rust
let ctx = dsl.ctx();        // Returns &ReducerContext
let sender = dsl.ctx().sender();
let ts = dsl.ctx().timestamp;
```

### Best Practice: Create DSL Once

Create the DSL once at reducer start and pass `&DSL` to helper functions:

```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    helper(&dsl)?;
    Ok(())
}
```

### Helper Function Signatures

Concrete signature (recommended for most cases):

```rust
fn helper(dsl: &DSL<'_, ReducerContext>) -> Result<(), SpacetimeDSLError> {
    // ...
    Ok(())
}
```

Trait-bound signature (when needed for more than one context — e.g. a helper used by both a reducer and a view):

```rust
fn helper<T: ReadContext>(dsl: &DSL<'_, T>) -> Result<(), SpacetimeDSLError> {
    // ...
    Ok(())
}
```

`WriteContext` is for write operations (reducers). `ReadContext` is for read-only operations (views). Keep the trait explanation simple — just know they exist for generic helper signatures.

### Method Configuration

Every `#[dsl]` attribute requires a `method(...)` configuration:

```rust
#[dsl(plural_name = entities, method(update = true, delete = true))]
```

Rules:

- `update` is **required** — omitting it causes a compilation error
- `delete` defaults to `true` — recommend always specifying it explicitly for clarity
- `method(update = true, delete = true)` — explicit (recommended)
- `method(update = false)` — delete defaults to true

Compile-time validation:

- `update = true` requires at least one `pub` field ("this column and therefore rows of this table should be mutable") OR a `modified_at`/`updated_at` column
- `update = false` requires all fields to be private AND no `modified_at`/`updated_at` column ("all columns and therefore rows of this table should be immutable")
- `delete = true` is required if any foreign key references this table with `on_delete = Delete` or you want to delete rows of this table in general (`delete = false` is good for audit tables, though you should export and delete rows in them from time to time using plain old SpacetimeDB API since no delete DSL methods are generated)
- Hooks require matching method config (`hook(after(update))` needs `method(update = true)`)

---

## Table Definition with DSL

### Full Syntax

```rust
#[dsl(
    plural_name = entities,
    method(update = true, delete = true),
    unique_index(name = some_index),
    hook(before(insert, update, delete), after(insert, update, delete)),
)]
#[table(accessor = entity, public)]
pub struct Entity {
    // columns...
}
```

Use the short form `#[dsl]` — not `#[spacetimedsl::dsl]` — in all code.

### Pairing #[dsl] with #[table]

The `#[dsl]` attribute must appear directly above the `#[table]` attribute:

```rust
#[dsl(plural_name = entities, method(update = true, delete = true))]
#[table(accessor = entity, public)]
pub struct Entity { ... }
```

### plural_name Requirement

The `plural_name` parameter is required and controls method names for multi-row operations:

- `get_all_{plural_name}()` — e.g., `get_all_entities()`
- `count_of_all_{plural_name}()` — e.g., `count_of_all_entities()`
- `delete_{plural_name}_by_{column}()` — e.g., `delete_entities_by_status()`

### Multiple #[dsl] + #[table] on Same Struct

A single struct can have multiple `#[dsl]` + `#[table]` pairs, each generating a separate table with its own accessor but sharing the same struct definition:

```rust
#[dsl(
    plural_name = modules1,
    method(update = true),
    unique_index(name = database_and_parent_id_and_name),
)]
#[table(
    accessor = module1,
    index(accessor = database_and_parent_id_and_name, btree(columns = [database_id, parent_id, name])),
    public,
)]
#[dsl(
    plural_name = modules2,
    method(update = true),
    unique_index(name = database_and_name_and_parent_id),
)]
#[table(
    accessor = module2,
    index(accessor = database_and_name_and_parent_id, btree(columns = [database_id, name, parent_id])),
    public,
)]
pub struct Module {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,
    database_id: u128,
    pub name: String,
    pub parent_id: u128,
}
```

Wrapper types are shared across all table instances of the same struct.

---

## Wrapper Types

### Problem: Primitive Obsession

Primitive obsession — passing a raw `u128` where an `EntityId` is expected leads to runtime bugs that the compiler cannot catch.

### Solution: Generated Wrapper Types

SpacetimeDSL can generate wrapper types:

```rust
#[primary_key]
#[create_wrapper]            // Generates EntityId wrapper (default: {TableName}{ColumnName})
id: u128,

#[primary_key]
#[create_wrapper(EntityId)]  // Custom name
id: u128,

#[use_wrapper(EntityId)]     // Reuses existing wrapper from same module
entity_id: u128,

#[use_wrapper(crate::entity::EntityId)]  // Reuses wrapper from another module
entity_id: u128,
```

### Naming Convention

Default name: `{SingularTableNamePascalCase}{ColumnNamePascalCase}`

| Table          | Column   | Wrapper Name       |
| -------------- | -------- | ------------------ |
| `entity`       | `id`     | `EntityId`         |
| `entity`       | `obj_id` | `EntityObjId`      |
| `user_profile` | `obj_id` | `UserProfileObjId` |
| `position`     | `id`     | `PositionId`       |

### Wrapper Trait

All wrappers implement:

```rust
pub trait Wrapper<WrappedType: Clone + Default, WrapperType>:
    Default + Clone + PartialEq + PartialOrd + spacetimedb::SpacetimeType + Display
{
    fn new(value: WrappedType) -> WrapperType;
    fn value(&self) -> WrappedType;
}
```

### Requirement

Every `#[primary_key]`, `#[unique]`, and `#[index]` column needs a wrapper — either `#[create_wrapper]` or `#[use_wrapper]`.

### Option\<T\> with Wrapper Types

`Option<WrapperType>` columns are supported:

```rust
#[use_wrapper(crate::entity::EntityId)]
pub wrapped_option: Option<u128>,

#[create_wrapper]
pub wrapped_string_option: Option<String>,
```

Getter returns `Option<WrapperType>`:

```rust
let id: Option<EntityId> = entity.get_wrapped_option();
```

Setter accepts `impl Into<Option<WrapperType>>`:

```rust
entity.set_wrapped_option(None);
entity.set_wrapped_option(some_entity.get_id());  // EntityId → Some(EntityId)
entity.set_wrapped_option(&some_entity);           // &Entity → Some(EntityId)
```

Mutable getters are NOT generated for wrapped columns (Option or otherwise). Use the setter instead.

Note: `#[unique]` and `#[index]` on `Option` columns is not yet supported by SpacetimeDB.

### Common Errors

```txt
The trait bound `WrapperType: From<NumericType>` is not satisfied.
```

This means you passed a raw numeric where a wrapper type is expected. Fix by using wrapper types consistently:

```rust
// Wrong:
dsl.get_entity_by_id(123);

// Right:
dsl.get_entity_by_id(EntityId::new(123))?;
dsl.get_entity_by_id(&some_entity)?;           // Auto-extracts primary key
dsl.get_entity_by_id(some_entity.get_id())?;   // Explicit getter
```

---

## Create Structs & Create Methods

### What Fields are Included

`Create{SingularTableNamePascalCase}` structs include ALL non-auto-defaulted fields **regardless of visibility** (pub or private).

### What Fields are Excluded (Auto-Defaulted)

| Column Pattern                   | Default Value                                |
| -------------------------------- | -------------------------------------------- |
| `#[auto_inc]` columns            | `0` (SpacetimeDB generates the actual value) |
| `created_at: Timestamp`          | `ctx.timestamp`                              |
| `inserted_at: Timestamp`         | `ctx.timestamp`                              |
| `modified_at: Option<Timestamp>` | `None` on create                             |
| `updated_at: Option<Timestamp>`  | `None` on create                             |
| `modified_at: Timestamp`         | `ctx.timestamp` on create                    |
| `updated_at: Timestamp`          | `ctx.timestamp` on create                    |

Both `created_at`/`inserted_at` and `modified_at`/`updated_at` are recognized aliases.

Timestamp protection rules:

- `created_at`/`inserted_at` columns are always private — no setter is generated, ensuring they are never changed after initial creation
- `modified_at`/`updated_at` columns are always private — no setter is generated, preventing manual modification. The DSL exclusively controls these values, updating them automatically on every update operation

### Usage Examples

Simple example:

```rust
#[dsl(plural_name = entities, method(update = true, delete = true))]
#[table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}

// CreateEntity has NO fields (all are auto-defaulted)
let entity = dsl.create_entity()?;
```

Example with explicit fields:

```rust
#[dsl(plural_name = config, method(update = false))]
#[table(accessor = config, public)]
pub struct Config {
    #[primary_key]
    #[create_wrapper]
    id: i32,
    world_size: i64,
}

// CreateConfig has: id (no auto_inc), world_size
let config = dsl.create_config(CreateConfig {
    id: ConfigId::new(0),
    world_size: 1000,
})?;
```

Example with wrapper types:

```rust
let player = dsl.create_player(CreatePlayer {
    identity: ConfigId::new(dsl.ctx().sender()),
    name: String::new(),
    login_status: LoginStatus::LoggedIn,
})?;
```

---

## Get Methods (Read)

### By Primary Key

Returns `Result<{Table}, SpacetimeDSLError>` (e.g., `Result<Entity, SpacetimeDSLError>`) — errors with `NotFoundError` if not found:

```rust
let entity = dsl.get_entity_by_id(&player)?;             // &Entity → auto-extracts PK
let entity = dsl.get_entity_by_id(player.get_id())?; // Explicit EntityId
let entity = dsl.get_entity_by_id(EntityId::new(42))?;   // Construct wrapper
```

### By Unique Column

Returns `Result<{Table}, SpacetimeDSLError>` (e.g., `Result<Identifier, SpacetimeDSLError>`):

```rust
let identifier = dsl.get_identifier_by_entity_id(&entity)?;
let identifier = dsl.get_identifier_by_value("PLAYER")?;
```

### By BTree Index (returns iterator)

Returns an iterator over matching rows:

```rust
let circles = dsl.get_circles_by_player_id(&player);
for circle in circles {
    // process each circle
}

// Collect into Vec
let circles: Vec<Circle> = dsl.get_circles_by_player_id(&player).collect_vec();
```

### Get All (returns iterator)

Returns an iterator over all rows:

```rust
for entity in dsl.get_all_entities() {
    // process each entity
}
```

### Count

Returns `u64`:

```rust
let count = dsl.count_of_all_entities();
```

### By Unique Multi-Column Index

Returns `Result<{Table}, SpacetimeDSLError>` (e.g., `Result<EntityRelationship, SpacetimeDSLError>`):

```rust
let relationship = dsl.get_entity_relationship_by_parent_child_entity_id(
    &parent_id, &child_id
)?;

let module = dsl.get_module1_by_database_and_parent_id_and_name(&0, &0, "")?;
```

---

## Update Methods

Requires `method(update = true)` in `#[dsl]`.

Note that update methods are not generated for unique single column indices as of SpacetimeDB 2.0.

### By Primary Key

Returns `Result<{Table}, SpacetimeDSLError>` (e.g., `Result<Entity, SpacetimeDSLError>`):

```rust
let mut entity = dsl.get_entity_by_id(&some_entity)?;
entity.set_name("new_name".to_string());
let updated = dsl.update_entity_by_id(entity)?;
```

### By Unique Multi-Column Index

```rust
let updated = dsl.update_entity_relationship_by_parent_child_entity_id(relationship)?;
```

### Automatic Timestamp Refresh

On every update:

- `modified_at: Option<Timestamp>` → set to `Some(ctx.timestamp)`
- `updated_at: Option<Timestamp>` → set to `Some(ctx.timestamp)`
- `modified_at: Timestamp` → set to `ctx.timestamp`
- `updated_at: Timestamp` → set to `ctx.timestamp`

### Requirements

`update = true` requires at least one `pub` field (which generates a setter) OR a `modified_at`/`updated_at` column. Without either, there would be nothing to update.

---

## Delete Methods

Requires `method(delete = true)` in `#[dsl]` (defaults to `true` if not specified).

### By Primary Key

Returns `Result<DeletionResult, SpacetimeDSLError>`:

```rust
let result = dsl.delete_entity_by_id(&entity)?;
```

### By Unique Column

Returns `Result<DeletionResult, SpacetimeDSLError>`:

```rust
let result = dsl.delete_identifier_by_entity_id(&entity)?;
```

### By BTree Index (Many)

Returns `Result<DeletionResult, SpacetimeDSLError>`:

```rust
let result = dsl.delete_tests_by_btree_index(some_value)?;
```

### DeletionResult

```rust
pub struct DeletionResult {
    pub table_name: Box<str>,
    pub one_or_multiple: OneOrMultiple,
    pub entries: Vec<DeletionResultEntry>,
}

pub struct DeletionResultEntry {
    pub table_name: Box<str>,
    pub column_name: Box<str>,
    pub strategy: OnDeleteStrategy,
    pub row_value: Box<str>,
    pub child_entries: Vec<DeletionResultEntry>,  // Nested cascade tracking
}
```

Use `result.to_csv()` to get a CSV-formatted audit trail:

```csv
entry_id, parent_entry_id, table_name, column_name, strategy, row_value,
1, 0, entity, id, Delete, 42
2, 1, position, entity_id, Delete, 42
```

---

## Accessor Methods (Getters/Setters/Mut-Getters)

All fields become private automatically when `#[dsl]` is applied.

### Getters

Generated for ALL columns. Return types depend on the column type:

| Column Type              | Getter Return Type          | Example                                            |
| ------------------------ | --------------------------- | -------------------------------------------------- |
| Wrapper type             | The wrapper type (by clone) | `fn get_id(&self) -> EntityId`                     |
| Primitive type           | `&T` (by reference)         | `fn get_name(&self) -> &String`                    |
| `Option<WrapperType>`    | `Option<WrapperType>`       | `fn get_wrapped_option(&self) -> Option<EntityId>` |
| `Option<T>` (no wrapper) | `&Option<T>`                | `fn get_modified_at(&self) -> &Option<Timestamp>`  |

### Setters

Generated for `pub` (non-private) columns only:

**Vanilla SpacetimeDB:**

```rust
#[table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    id: u128,
    pub name: String,
    created_at: Timestamp,
    modified_at: Timestamp,
}
```

**With SpacetimeDSL:**

```rust
#[dsl(plural_name = entities, method(update = true, delete = true))]
#[table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    #[create_wrapper]
    id: u128,                 // Private — getter only

    pub name: String,         // Public — getter + setter + mut-getter

    created_at: Timestamp,    // Private — getter only
    modified_at: Timestamp,   // Private — getter only, but is internally updated by SpacetimeDSL
}

// Usage
entity.set_name("new_name".to_string());
```

### Mut-Getters

Generated for any column that also has a setter (i.e., `pub` columns), unless the column has a wrapper type:

```rust
let tags = entity.get_tags_mut();  // Returns &mut Vec<String>
tags.push("new_tag".to_string());
```

Mut-getters are NOT generated for wrapped columns — use the setter instead.

### impl Into Flexible Input Patterns

DSL methods and setters accept flexible input types via `impl Into`:

```rust
// A reference to a table struct auto-extracts its primary key:
dsl.get_entity_by_id(&player)?;           // &Entity → gets id automatically

// Explicit wrapper value also works, but not recommended:
dsl.get_entity_by_id(player.get_id())?;  // EntityId → explicit

// For non-primary-key columns, use the explicit getter:
dsl.get_tests_by_wrapped_index(player.get_id())?;

// Setters also accept impl Into:
entity.set_some_field(value);  // Accepts both the wrapper type and the raw inner type
```

---

## Foreign Keys & Referential Integrity

### Declaration

```rust
// On the referenced table's primary key column:
#[referenced_by(path = self, table = position)]
id: u128,

// On the referencing table's foreign key column:
#[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]
entity_id: u128,
```

All parameters are required:

- `path` — module path: `path = self` for same module, `path = crate::module::path` for cross-module
- `table` — the SpacetimeDB table accessor name
- `column` — always the **primary key column** of the referenced table
- `on_delete` — strategy: `Error`, `Delete`, `SetZero`, or `Ignore`

### Pairing Requirement

Every `#[foreign_key]` needs a corresponding `#[referenced_by]` on the referenced table's primary key. Missing either produces a descriptive compilation error:

```txt
unresolved import crate::entity::this_compilation_error_occurs_because_the_entity_table_has_no_referenced_by_attribute_referencing_the_entity_relationship_table
unresolved import crate::entity_relationship::this_compilation_error_occurs_because_the_entity_relationship_table_has_no_foreign_key_attribute_referencing_the_entity_table
```

### path Parameter

- `path = self` — reference within the same module
- `path = crate` — reference at the crate root
- `path = crate::entity` — reference in a specific module

### OnDeleteStrategy

#### Error

Prevent deletion if referenced:

- Available for all column types
- Deletion fails with `ReferenceIntegrityViolation` error
- No rows are deleted

**Important:** You MUST return the error from the reducer or procedure, otherwise deletions which occurred before the integrity violation will still be committed by SpacetimeDB. Always use `?` to propagate errors, never silently ignore them.

#### Delete (Cascade)

Cascade delete referencing rows:

- Available for all column types
- First checks if any cascade would hit an `Error` strategy; if so, fails entirely
- All-or-nothing: either all deletes succeed or none happen
- Requires `method(delete = true)` on the referencing table's `#[dsl]`

#### SetZero

Set foreign key column to `0`:

- Numeric types only
- Requires `method(update = true)` on the referencing table's `#[dsl]`
- Requires the foreign key column to be `pub` (so a setter exists)

#### Ignore

Allow dangling references:

- Available for all column types
- Creates dangling references (referenced value no longer exists)
- Integrity only enforced on create/update, not on delete
- Use only for audit logs or append-only tables

### Full Foreign Key Example

```rust
#[dsl(plural_name = entities, method(update = true, delete = true))]
#[table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(EntityId)]
    #[referenced_by(path = self, table = circle)]
    #[referenced_by(path = self, table = food)]
    id: i32,

    pub x: f32,
    pub y: f32,
    pub mass: i32,
}

#[dsl(plural_name = circles, method(update = true, delete = true))]
#[table(accessor = circle, public)]
pub struct Circle {
    #[primary_key]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]
    entity_id: i32,

    #[index(btree)]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id, on_delete = Delete)]
    pub player_id: i32,

    pub direction_x: f32,
    pub direction_y: f32,
}
```

### Self-Referencing Tables

```rust
#[dsl(plural_name = entity_relationships3, method(update = true, delete = true))]
#[table(accessor = entity_relationship3, public)]
pub struct EntityRelationship3 {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::entity, table = entity_relationship3)]
    id: u128,

    #[index(btree)]
    #[use_wrapper(EntityRelationship3Id)]
    #[foreign_key(path = crate::entity, table = entity_relationship3, column = id, on_delete = SetZero)]
    pub parent_entity_relationship3_id: u128,
}
```

### Multiple Foreign Keys to Same Table

```rust
#[dsl(plural_name = entity_relationships, method(update = true, delete = true),
    unique_index(name = parent_child_entity_id))]
#[table(accessor = entity_relationship,
    index(accessor = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])),
    public)]
pub struct EntityRelationship {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    #[index(btree)]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = crate::entity, table = entity, column = id, on_delete = Error)]
    parent_entity_id: u128,

    #[index(btree)]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = crate::entity, table = entity, column = id, on_delete = Delete)]
    child_entity_id: u128,
}
```

### Critical Rule: Always Use DSL Methods

Foreign keys and referential integrity are enforced ONLY via DSL methods. Never bypass the DSL:

```rust
// WRONG — breaks referential integrity:
ctx.db.position().insert(Position { ... });
dsl.ctx().db.position().entity_id().delete(&id);

// RIGHT — enforces FK checks:
dsl.create_position(CreatePosition { ... })?;
dsl.delete_position_by_id(&position)?;
```

You MUST return the error from the reducer or procedure, otherwise deletions which occurred before the integrity violation will still be committed by SpacetimeDB. Always use `?` to propagate errors, never silently ignore them.

---

## Unique Multi-Column Indices

### Declaration

The `unique_index(name = ...)` in `#[dsl]` must match a SpacetimeDB `index(accessor = ...)` on the same table. Use `name` (not `accessor`) in the DSL attribute:

```rust
#[dsl(
    plural_name = entity_relationships,
    method(update = true, delete = true),
    unique_index(name = parent_child_entity_id)
)]
#[table(
    accessor = entity_relationship,
    index(accessor = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])),
    public,
)]
pub struct EntityRelationship {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    #[index(btree)]
    #[use_wrapper(EntityId)]
    parent_entity_id: u128,

    #[index(btree)]
    #[use_wrapper(EntityId)]
    child_entity_id: u128,
}
```

### Generated Methods

```rust
// Get one (unique — returns Result, not iterator)
dsl.get_entity_relationship_by_parent_child_entity_id(&parent_id, &child_id)?;

// Update
dsl.update_entity_relationship_by_parent_child_entity_id(relationship)?;

// Delete one
dsl.delete_entity_relationship_by_parent_child_entity_id(&parent_id, &child_id)?;
```

### Uniqueness Enforcement

On create and update, the DSL checks if a row with the same multi-column values already exists. If so, it returns a `UniqueConstraintViolation` error.

### Critical Rule: Always Use DSL Methods

Like foreign keys, uniqueness is only enforced via DSL methods. Bypassing the DSL with raw SpacetimeDB calls skips the uniqueness check.

---

## Hooks System

### Declaration

```rust
#[dsl(
    plural_name = attributes,
    method(update = true, delete = true),
    hook(before(insert, update, delete), after(insert, update, delete))
)]
#[table(accessor = attribute, public)]
pub struct Attribute {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    pub value: String,
}
```

### Hook Function Naming

Pattern: `{before|after}_{table_name}_{insert|update|delete}`

Examples: `before_attribute_insert`, `after_player_update`, `before_entity_delete`

### The #[hook] Attribute

Apply `#[spacetimedsl::hook]` (or `#[hook]` with prelude) to each hook function. This attribute automatically adds the generic `T: Context<T>` to the function signature.

### Hook Signatures

**before_insert** — Can modify or validate the create request before insertion, may return an error to abort:

`fn before_{table}_insert(dsl: &DSL<'_, T>, create: Create{Table}) -> Result<Create{Table}, SpacetimeDSLError>`

(e.g., `fn before_attribute_insert(dsl: &DSL<'_, T>, create: CreateAttribute) -> Result<CreateAttribute, SpacetimeDSLError>`)

**after_insert** — React to a newly inserted row:

`fn after_{table}_insert(dsl: &DSL<'_, T>, row: &{Table}) -> Result<(), SpacetimeDSLError>`

(e.g., `fn after_attribute_insert(dsl: &DSL<'_, T>, row: &Attribute) -> Result<(), SpacetimeDSLError>`)

**before_update** — Can modify or validate the new row before update, may return an error to abort:

`fn before_{table}_update(dsl: &DSL<'_, T>, old: &{Table}, new: {Table}) -> Result<{Table}, SpacetimeDSLError>`

(e.g., `fn before_attribute_update(dsl: &DSL<'_, T>, old: &Attribute, new: Attribute) -> Result<Attribute, SpacetimeDSLError>`)

**after_update** — React to an update:

`fn after_{table}_update(dsl: &DSL<'_, T>, old: &{Table}, new: &{Table}) -> Result<(), SpacetimeDSLError>`

(e.g., `fn after_attribute_update(dsl: &DSL<'_, T>, old: &Attribute, new: &Attribute) -> Result<(), SpacetimeDSLError>`)

**before_delete** — Validate before deletion, may return an error to abort:

`fn before_{table}_delete(dsl: &DSL<'_, T>, row: &{Table}) -> Result<(), SpacetimeDSLError>`

(e.g., `fn before_attribute_delete(dsl: &DSL<'_, T>, row: &Attribute) -> Result<(), SpacetimeDSLError>`)

**after_delete** — React to deletion:

`fn after_{table}_delete(dsl: &DSL<'_, T>, row: &{Table}) -> Result<(), SpacetimeDSLError>`

(e.g., `fn after_attribute_delete(dsl: &DSL<'_, T>, row: &Attribute) -> Result<(), SpacetimeDSLError>`)

### Error Handling in Hooks

- **before hooks**: Returning an error aborts the operation — no database changes happen
- **after hooks**: Returning an error propagates but the database change already happened. Always use `?` to propagate errors from hooks so that SpacetimeDB doesn't commit the transaction.

### Hook-Method Compatibility

- `before_update`/`after_update` hooks require `method(update = true)`
- `before_delete`/`after_delete` hooks require `method(delete = true)`
- `before_insert`/`after_insert` hooks always allowed (create is always available)

### Location Requirement

Hook functions must be defined in the same module as the table definition.

---

## Error Handling

### SpacetimeDSLError Variants

```rust
pub enum SpacetimeDSLError {
    /// Generic error with message
    Error(String),

    /// Row not found during get/update/delete
    NotFoundError {
        table_name: Box<str>,
        column_names_and_row_values: Box<str>,
    },

    /// Unique constraint violated on create/update
    UniqueConstraintViolation {
        table_name: Box<str>,
        action: Action,          // Create, Get, Update, Delete
        error_from: ErrorFrom,   // SpacetimeDB or SpacetimeDSL
        one_or_multiple: OneOrMultiple,
        column_names_and_row_values: Box<str>,
    },

    /// Auto-increment counter overflow
    AutoIncOverflow {
        table_name: Box<str>,
    },

    /// Foreign key constraint violated
    ReferenceIntegrityViolation(ReferenceIntegrityViolationError),
}

pub enum ReferenceIntegrityViolationError {
    OnCreateOrUpdate {
        table_name: Box<str>,
        create_or_update: Action,
        column_names_and_row_values: Box<str>,
    },
    OnDelete(DeletionResult),
}

pub enum Action {
    Create,
    Get,
    Update,
    Delete,
}

pub enum ErrorFrom {
    SpacetimeDB,
    SpacetimeDSL,
}

pub enum OneOrMultiple {
    One,
    Multiple,
}
```

### Conversion Chain

`SpacetimeDSLError` implements `Display`, `Error`, and `From<SpacetimeDSLError> for String`. This means:

- Reducers can return `Result<(), SpacetimeDSLError>` because SpacetimeDB expects `Result<(), impl Into<String>>`
- The `From<SpacetimeDSLError> for String` impl calls `.to_string()` (the `Display` impl)
- This is why `Result<(), SpacetimeDSLError>` works instead of `Result<(), String>`

### Recommended Pattern: ? Propagation

```rust
#[reducer]
pub fn my_reducer(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = spacetimedsl::dsl(ctx);
    let entity = dsl.create_entity()?;
    let position = dsl.get_position_by_entity_id(&entity)?;
    dsl.delete_entity_by_id(&entity)?;
    Ok(())
}
```

### Explicit Matching for ReferenceIntegrityViolation

```rust
match dsl.delete_entity_by_id(&entity) {
    Ok(deletion_result) => {
        log::info!("Deleted with cascade:\n{}", deletion_result.to_csv());
    }
    Err(SpacetimeDSLError::ReferenceIntegrityViolation(err)) => {
        log::warn!("Cannot delete: referenced by other tables");
        // err contains the DeletionResult showing what would be affected
    }
    Err(e) => return Err(e),
}
```

### Example Error Messages

**NotFoundError:**

```txt
Not Found Error while trying to find a row in the `position` table with {{ entity_id : 1 }}!
```

**UniqueConstraintViolation (from SpacetimeDB):**

```txt
Unique Constraint Violation Error while trying to create a row in the `entity` table!
Unfortunately SpacetimeDB doesn't provide more information, so here are all columns and their values:
{{ entity : Entity { id: EntityId { id: 1 }, created_at: ... } }}
```

**UniqueConstraintViolation (from SpacetimeDSL — multi-column):**

```txt
Unique Constraint Violation Error while trying to create a row in the `entity_relationship` table
because of {{ parent_entity_id : 1, child_entity_id : 2 }}!
```

**AutoIncOverflow:**

```txt
Auto Inc Overflow Error on the `entity` table! Unfortunately SpacetimeDB doesn't provide more information.
```

**ReferenceIntegrityViolation (on create/update):**

```txt
Reference Integrity Violation Error while trying to create a row in the `position` table
because of {{ entity_id : 1 }}!
```

**ReferenceIntegrityViolation (on delete):**

```txt
Reference Integrity Violation Error while trying to delete a row in the `entity` table because of:

entry_id, parent_entry_id, table_name, column_name, strategy, row_value,
1, 0, circle, entity_id, Error, 42
```

---

## Generated Trait Names Reference

All generated traits follow consistent naming patterns. The table name used in trait names is always PascalCase singular.

### Trait Naming Patterns

| Operation                 | Trait Name Pattern              | Method Name Pattern            |
| ------------------------- | ------------------------------- | ------------------------------ |
| Create                    | `Create{Table}Row`              | `create_{table}()`             |
| Get by PK/unique          | `Get{Table}RowOptionBy{Column}` | `get_{table}_by_{column}()`    |
| Get by index (many)       | `Get{Table}RowsBy{Index}`       | `get_{plural}_by_{index}()`    |
| Get all                   | `GetAll{Table}Rows`             | `get_all_{plural}()`           |
| Count all                 | `CountOfAll{Table}Rows`         | `count_of_all_{plural}()`      |
| Update by PK/unique       | `Update{Table}RowBy{Column}`    | `update_{table}_by_{column}()` |
| Delete by PK/unique (one) | `Delete{Table}RowBy{Column}`    | `delete_{table}_by_{column}()` |
| Delete by index (many)    | `Delete{Table}RowsBy{Index}`    | `delete_{plural}_by_{index}()` |

### Struct Naming Patterns

| Type                   | Pattern           | Example                          |
| ---------------------- | ----------------- | -------------------------------- |
| Create argument struct | `Create{Table}`   | `CreateEntity`, `CreatePosition` |
| Wrapper type (default) | `{Table}{Column}` | `EntityObjId`, `PositionId`      |
| Wrapper type (custom)  | User-specified    | `EntityId`, `PlayerId`           |

### Hook Trait Naming Patterns

| Hook          | Trait Name Pattern        |
| ------------- | ------------------------- |
| Before insert | `Before{Table}InsertHook` |
| After insert  | `After{Table}InsertHook`  |
| Before update | `Before{Table}UpdateHook` |
| After update  | `After{Table}UpdateHook`  |
| Before delete | `Before{Table}DeleteHook` |
| After delete  | `After{Table}DeleteHook`  |

### Internal FK Cascade Trait Patterns

These are generated internally and rarely need to be imported:

| Pattern                                                                                     |
| ------------------------------------------------------------------------------------------- |
| `ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThe{Table}TableWasDeleted`        |
| `ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThe{Table}TableWereDeleted` |

### When to Import Generated Traits

In single-module projects, all traits are in scope. In multi-module projects, import the specific traits you need for cross-module helper functions:

```rust
use crate::entity::{
    CreateEntityRow, EntityId, GetEntityRowOptionByObjId,
    DeleteEntityRowByObjId, UpdateEntityRowByObjId,
    CountOfAllEntityRows,
};

use crate::component::position::{
    CreatePosition, CreatePositionRow, PositionId,
    GetPositionRowOptionById, GetAllPositionRows,
    UpdatePositionRowById, CountOfAllPositionRows,
};
```

---

## Common Mistakes & Pitfalls

### SpacetimeDB API Mistakes

| Mistake                                  | Fix                                                                    |
| ---------------------------------------- | ---------------------------------------------------------------------- |
| `ctx.sender` (field access)              | `dsl.ctx().sender()` (method call)                                     |
| `ctx.timestamp()` (method call)          | `dsl.ctx().timestamp` (field access)                                   |
| `ctx.db()` (method call)                 | `dsl.ctx().db` (field access) — but prefer DSL methods over raw access |
| `ctx.rng` (field access)                 | `dsl.ctx().rng()` (method call)                                        |
| `ctx.db.player` (field access)           | Use DSL methods like `dsl.get_player_by_id(...)` instead               |
| `ctx.db.player().find(id)`               | Use DSL methods like `dsl.get_player_by_id(PlayerId::new(id))?`        |
| `#[derive(SpacetimeType)]` on `#[table]` | Remove it — the table macro handles this                               |
| `&mut ReducerContext`                    | `&ReducerContext` — always an immutable reference                      |
| Missing `use spacetimedb::Table;`        | Add import — required for `.insert()`, `.iter()`, etc.                 |
| `ScheduleAt::At(time)`                   | `ScheduleAt::Time(time)` — wrong variant name                          |
| `#[table(name = t, schedule(...))]`      | `#[table(name = t, scheduled(...))]` — `scheduled` not `schedule`      |
| Network/filesystem in reducer            | Use procedures instead — sandbox violation                             |
| Panic for expected errors                | Return `Result<(), SpacetimeDSLError>` — WASM instance destroyed       |

### DSL-Specific Mistakes

| Mistake                                           | Fix                                                                                                 |
| ------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Bypassing DSL: `ctx.db.table().insert(...)`       | Use `dsl.create_table(...)` — raw access breaks FK/unique checks                                    |
| `dsl.ctx().db.table().insert(...)`                | Still bypasses DSL — use DSL methods exclusively                                                    |
| Wrong hook name: `before_entity_create`           | Correct: `before_entity_insert`                                                                     |
| Missing `#[referenced_by]` for a `#[foreign_key]` | Every FK needs a matching `referenced_by` on the referenced PK                                      |
| `update = true` without pub fields or timestamp   | Add a `pub` field or `modified_at`/`updated_at` column                                              |
| Omitting `update` parameter entirely              | Compilation error — `update` is required (unlike `delete` which defaults to `true`)                 |
| `unique_index(accessor = ...)`                    | Correct: `unique_index(name = ...)` — `accessor` is for SpacetimeDB `index`, not DSL `unique_index` |
| Passing raw numeric where wrapper expected        | Use `EntityId::new(42)` or `entity.get_id()`                                                        |
| Using `read_only_dsl` in a reducer                | `read_only_dsl` is for views only — use `dsl` for reducers                                          |
| Missing `column` param in `#[foreign_key]`        | `column` is always required — name the PK column of the referenced table                            |
| `#[spacetimedsl::dsl]` in code                    | Use short form `#[dsl]` with prelude import                                                         |

---

## Debugging Checklist

1. Is the reducer actually being called?
2. Are DSL methods used (not raw SpacetimeDB)?
3. Are all `#[foreign_key]` paired with `#[referenced_by]`?
4. Are wrapper types used consistently (not raw primitives)?
5. Is `update` explicitly set in `method(...)` config?
6. Are hook function names following the `{before|after}_{table}_{insert|update|delete}` pattern?
7. Are errors propagated with `?` (never silently ignored)?
