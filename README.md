<!-- markdownlint-disable MD033 -->
# ✨ **SpacetimeDSL** — The **SpacetimeDB** Rust Server Module meta-framework

[![dependency status](https://deps.rs/crate/spacetimedsl/latest/status.svg)](https://deps.rs/crate/spacetimedsl/latest)

**SpacetimeDSL** allows you to interact in an ergonomic, more developer-friendly and type-safer way with the data in your [**SpacetimeDB**](https://spacetimedb.com/) server.

## 📑 Table of Contents

See [`docs/DOCUMENTATION.md`](docs/DOCUMENTATION.md)  for a comprehensive reference with all features, examples, and rules.

### Core Unique Features

- [🔗 Foreign Keys / Referential Integrity](docs/DOCUMENTATION.md#foreign-keys--referential-integrity) — Enforce relationships between tables with different strategies on deletion.
- [🏷️ Wrapper Types](docs/DOCUMENTATION.md#wrapper-types) — Type-safe column identifiers that eliminate primitive obsession.
- [🎲 Unique Multi-Column Indices](docs/DOCUMENTATION.md#unique-multi-column-indices) — Enforce uniqueness across multiple columns (because **SpacetimeDB** has no native support).
- [🪝 Hooks System](docs/DOCUMENTATION.md#hooks-system) — Execute custom logic automatically before and after inserts, updates and deletes.
- [🎨 Ergonomic DSL Methods](docs/DOCUMENTATION.md#dsl-methods) — DSL equivalents for all **SpacetimeDB** operations with cleaner syntax and smart defaults.
- [🎯 Singleton Tables](docs/DOCUMENTATION.md#singleton-tables) — Single-row tables for global config or state.
- [👁️ Read-Only View Support](docs/DOCUMENTATION.md#views) — Use DSL methods in **SpacetimeDB** views.

### Enhanced Developer Experience

- [🎛️ Method Configuration](docs/DOCUMENTATION.md#method-configuration) — Explicit control over which operations are allowed on your tables.
- [🚨 Rich Error Types](docs/DOCUMENTATION.md#error-handling) — Detailed error information beyond what **SpacetimeDB** provides.
- [📊 Deletion Results](docs/DOCUMENTATION.md#deletionresult) — Complete audit trails for delete operations with cascade tracking.
- [🔄 Automatic Accessors](docs/DOCUMENTATION.md#accessor-methods-getterssetters) — Generated getters, mut-getters and setters with visibility controls.

### Additional Information

- [⚠️ Current Limitations](#️-current-limitations)
- [❓ FAQ](#-faq)
- [📜 Licensing](#-licensing)

### 📂 Examples

- [**blackholio**](examples/blackholio/) — Real-world multiplayer game using **SpacetimeDSL** with foreign keys, hooks, scheduled tasks, and wrapper types.
- [**test**](examples/test/) — Integration test suite covering all **SpacetimeDSL** features.

---

## 🚀 Example 2D Tile-based Game

### 🔧 Vanilla **SpacetimeDB**

Let's start with a ordinary **SpacetimeDB** schema:

```rust
// FIXME 1: Need to validate that entities are only created and deleted, not updated. Would be cool if the compiler would enforce this, but for now each developer must keep this in mind. I'll create docs on that and hope that everyone reads them...
#[spacetimedb::table(
    accessor = entity,
    public,
)]
pub struct Entity {
    // FIXME 2: Need to manually set it to `0` on creation and let the DB auto-generate the ID.
    // FIXME 3: Need to keep in mind to never set to a non-zero value otherwise that would cause ID conflicts in the DB.
    // FIXME 4: Need to ensure that the column value is never changed.
    #[primary_key]
    #[auto_inc]
    pub id: u128,

    // FIXME 5: Need to manually set it to `ctx.timestamp` on creation.
    // FIXME 4: Again.
    pub created_at: spacetimedb::Timestamp,
}

// FIXME 6: Need to validate where positions are created and changed that x and y is unique together, so that each tile can only contain one Entity. I hope I won't need more unique multi-column indices in the future... Develop a generic solution for this?!
// FIXME 7: Need to validate that positions are in specific bounds where positions are created and changed so that entities like players can't move outside the playable area.
#[spacetimedb::table(
    accessor = position,
    public,
    index(accessor = x_y, btree(columns = [x, y])),
)]
pub struct Position {
    // FIXME 2, 3 and 4: Again.
    #[primary_key]
    #[auto_inc]
    pub id: u128,

    // FIXME 4: Again.
    // FIXME 8: Need to validate where Positions are created that this only accepts Entity IDs.
    // FIXME 9: Need to validate that the referenced Entity actually exists.
    // FIXME 10: Need to ensure that the Position is deleted when the Entity with this ID is deleted.
    #[unique]
    pub entity_id: u128,

    pub x: i128,

    pub y: i128,

    // FIXME 11: Need to set this to `None` on creation
    // FIXME 12: Need to update it to `Some(ctx.timestamp)` on every update.
    pub modified_at: Option<spacetimedb::Timestamp>,
}
```

**The Problem:** **SpacetimeDB** is great technology, but has weaknesses that prevent developers from utilizing its full potential — sometimes you work *against* the database.

### ⚡ **SpacetimeDB** with **SpacetimeDSL**

Let's see what happens when adding **SpacetimeDSL**:

```rust
#[spacetimedsl::dsl( // Added
    plural_name = entities,
    method(update = false, delete = true),
)]
#[spacetimedb::table(
    accessor = entity,
    public,
)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper] // Added
    #[referenced_by(path = self, table = position)] // Added
    id: u128, // no longer pub

    created_at: spacetimedb::Timestamp, // no longer pub
}

#[spacetimedsl::dsl( // Added
    plural_name = positions,
    method(update = true, delete = true),
    unique_index(name = x_y),
    hook(before(insert, update)),
)]
#[spacetimedb::table(
    accessor = position,
    public,
    index(accessor = x_y, btree(columns = [x, y])),
)]
pub struct Position {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper] // Added
    id: u128, // no longer pub

    #[unique]
    #[use_wrapper(EntityId)] // Added
    #[foreign_key(path = self, table = entity, column = id, on_delete = Delete)] // Added
    entity_id: u128, // no longer pub

    pub x: i128, // Still pub because it should be updatable

    pub y: i128, // Still pub because it should be updatable

    modified_at: Option<spacetimedb::Timestamp>, // No longer pub
}

// Added
#[spacetimedsl::hook]
fn before_position_insert(
    _dsl: &spacetimedsl::DSL<'_, T>,
    position: CreatePosition,
) -> Result<CreatePosition, spacetimedsl::SpacetimeDSLError> {
    before_position_hook_helper(&position.x, &position.y)?;

    Ok(position)
}

// Added
#[spacetimedsl::hook]
fn before_position_update(
    _dsl: &spacetimedsl::DSL<'_, T>,
    old: &Position,
    new: Position,
) -> Result<Position, spacetimedsl::SpacetimeDSLError> {
    if *old.get_x() == *new.get_x() && *old.get_y() == *new.get_y() {
        // No change in position, so we can skip validation
        return Ok(new);
    }

    before_position_hook_helper(new.get_x(), new.get_y())?;

    Ok(new)
}

const WORLD_BOUNDARY: i128 = 10_000_000_000_000_000_000;

// Added
fn before_position_hook_helper(
    x: &i128,
    y: &i128,
) -> Result<(), spacetimedsl::SpacetimeDSLError> {
    if *x < -WORLD_BOUNDARY || *x > WORLD_BOUNDARY || *y < -WORLD_BOUNDARY || *y > WORLD_BOUNDARY
    {
        return Err(spacetimedsl::SpacetimeDSLError::Error(
            "Position out of bounds".to_string(),
        ));
    }

    Ok(())
}
```

**✨ What's different?**

**Cleaner Modeling:**

- 🎯 `Entity` is constrained to create/delete only (`update = false`), so lifecycle intent is explicit in the type-level API.
- 🔒 Sensitive columns (`id`, `created_at`, `entity_id`, `modified_at`) are private in the struct, preventing accidental mutation.
- 🧠 Generated getters (without setters / mut-getters for immutable fields) enforce safer usage patterns by default.

**Smart Defaults & Automation:**

- 🤖 Auto-increment IDs are handled automatically (no manual ID assignment, no non-zero ID mistakes).
- ⏰ `created_at` is set automatically on create.
- 🔄 `modified_at` is set to `None` on create and updated to `Some(ctx.timestamp)` on update.
- 🧷 No-op update guard: if position `x`/`y` didn't change, validation is skipped.

**Data Integrity by Construction:**

- 🏷️ Wrapper types (`#[create_wrapper]` + `#[use_wrapper(EntityId)]`) make cross-table ID misuse much harder.
- 🔗 Foreign-key validation ensures referenced `Entity` exists on create (and update, if the `entity_id` column would be mutable).
- 🧹 Referential cleanup on delete keeps dependent `Position` rows in sync automatically when deleting their corresponding `Entity`.
- 🎲 `unique_index(name = x_y)` enforces unique multi-column `(x, y)` positions.

**Hooks for Domain Rules:**

- 🪝 Before-insert and before-update hooks validate world bounds.

---

🚀 **Try SpacetimeDSL for yourself**, by adding it to your server module's `Cargo.toml`:

```toml

# https://crates.io/crates/spacetimedsl The SpacetimeDB Rust Server Module meta-framework
spacetimedsl = { version = "0.22.0" }

```

📖 **Get started** by adding `#[spacetimedsl::dsl]` plus helper attributes

- `#[create_wrapper]`,
- `#[use_wrapper]`,
- `#[foreign_key]` and
- `#[referenced_by]`

to your structs with `#[spacetimedb::table]`!

💬 **Need help?**

- Consult the [FAQ](#-faq)
- Join the [**SpacetimeDSL** channel of the **SpacetimeDB** Discord Server](https://discord.com/channels/1037340874172014652/1395832638966726726)

---

## ⚠️ Current Limitations

- [Using IndexScanRangeBounds / FilterableValue](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21) 🔄

- If you encounter that you can't access a method on the `(Anonymous)ViewContext` type because it's private, please follow these instructions: <https://github.com/tamaro-skaljic/SpacetimeDSL/issues/90#issuecomment-3573925117> until [**SpacetimeDB**#3754](https://github.com/clockworklabs/SpacetimeDB/issues/3754) is resolved and released.

---

## ❓ FAQ

**❔ Why must `#[primary_key]` columns be private?**

> Currently, they are allowed to be public, until [**SpacetimeDB**#3754](https://github.com/clockworklabs/SpacetimeDB/issues/3754) is resolved and released.

- 🔒 They should never change after insertion
- DSL generates setters and mut-getters for non-private columns
- Making them public would:
  - ❌ Allow changes after creation via setters
  - ❌ Allow direct struct member access
  - ❌ Bypass wrapped types

---

## 📜 Licensing

**SpacetimeDSL** is dual-licensed under:

- ⚖️ [MIT License](https://choosealicense.com/licenses/mit/)
- ⚖️ [Apache License (Version 2.0)](https://choosealicense.com/licenses/apache-2.0/)

**Open Source** ❤️

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
