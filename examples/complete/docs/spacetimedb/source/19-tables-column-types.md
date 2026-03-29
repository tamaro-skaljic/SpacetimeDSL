# Column Types

Columns define the structure of your tables. SpacetimeDB supports primitive types, composite types for complex data, and special types for database-specific functionality.

## Representing Collections

When modeling data that contains multiple items, you have two choices: store the collection as a column (using `Vec`) or store each item as a row in a separate table. This decision affects how you query, update, and subscribe to that data.

**Use a collection column when:**

- The items form an atomic unit that you always read and write together
- Order is semantically important and frequently accessed by position
- The collection is small and bounded (e.g., a fixed-size inventory)
- The items are values without independent identity

**Use a separate table when:**

- Items have independent identity and lifecycle
- You need to query, filter, or index individual items
- The collection can grow unbounded
- Clients should receive updates for individual item changes, not the entire collection
- You want to enforce referential integrity between items and other data

Consider a game inventory with ordered pockets. A `Vec<Item>` preserves pocket order naturally, but if you need to query "all items owned by player X" across multiple players, a separate `inventory_item` table with a `pocket_index` column allows that query efficiently. The right choice depends on your dominant access patterns.

## Binary Data and Files

SpacetimeDB includes optimizations for storing binary data as `Vec<u8>`. You can store files, images, serialized data, or other binary blobs directly in table columns.

This approach works well when:

- The binary data is associated with a specific row (e.g., a user's avatar image)
- You want the data to participate in transactions and subscriptions
- The data size is reasonable (up to several megabytes per row)

For very large files or data that changes independently of other row fields, consider external storage with a reference stored in the table.

## Type Performance

SpacetimeDB optimizes reading and writing by taking advantage of memory layout. Several factors affect performance:

**Prefer smaller types.** Use the smallest integer type that fits your data range. A `u8` storing values 0-255 uses less memory and bandwidth than a `u64` storing the same values. This reduces storage, speeds up serialization, and improves cache efficiency.

**Prefer fixed-size types.** Fixed-size types (`u32`, `f64`, fixed-size structs) allow SpacetimeDB to compute memory offsets directly. Variable-size types (`String`, `Vec<T>`) require additional indirection. When performance matters, consider fixed-size alternatives:

- Use `[u8; 32]` instead of `Vec<u8>` for fixed-length hashes or identifiers
- Use an enum with a fixed set of variants instead of a `String` for categorical data

**Consider column ordering.** Types require alignment in memory. A `u64` aligns to 8-byte boundaries, while a `u8` aligns to 1-byte boundaries. When smaller types precede larger ones, the compiler may insert padding bytes to satisfy alignment requirements. Ordering columns from largest to smallest alignment can reduce padding and improve memory density.

For example, a struct with fields `(u8, u64, u8)` may require 24 bytes due to padding, while `(u64, u8, u8)` requires only 16 bytes. This optimization is not something to follow religiously, but it can help performance in memory-intensive scenarios.

## Type Reference

| Category  | Type                                     | Description                                            |
| --------- | ---------------------------------------- | ------------------------------------------------------ |
| Primitive | `bool`                                   | Boolean value                                          |
| Primitive | `String`                                 | UTF-8 string                                           |
| Primitive | `f32`, `f64`                             | Floating point numbers                                 |
| Primitive | `i8`, `i16`, `i32`, `i64`, `i128`        | Signed integers                                        |
| Primitive | `u8`, `u16`, `u32`, `u64`, `u128`        | Unsigned integers                                      |
| Composite | `struct` with `#[derive(SpacetimeType)]` | Product type for nested data                           |
| Composite | `enum` with `#[derive(SpacetimeType)]`   | Sum type (tagged union)                                |
| Composite | `Vec<T>`                                 | Vector of elements                                     |
| Composite | `Option<T>`                              | Optional value                                         |
| Special   | `Identity`                               | Unique identity for authentication                     |
| Special   | `ConnectionId`                           | Client connection identifier                           |
| Special   | `Timestamp`                              | Absolute point in time (microseconds since Unix epoch) |
| Special   | `TimeDuration`                           | Relative duration in microseconds                      |
| Special   | `ScheduleAt`                             | When a scheduled reducer should execute                |

## Complete Example

The following example demonstrates a table using primitive, composite, and special types:

```
use spacetimedb::{SpacetimeType, Identity, ConnectionId, Timestamp, TimeDuration};

// Define a nested struct type for coordinates
#[derive(SpacetimeType)]
pub struct Coordinates {
    x: f64,
    y: f64,
    z: f64,
}

// Define an enum for status
#[derive(SpacetimeType)]
pub enum Status {
    Active,
    Inactive,
    Suspended { reason: String },
}

#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    // Primitive types
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
    level: u8,
    experience: u32,
    health: f32,
    score: i64,
    is_online: bool,

    // Composite types
    position: Coordinates,
    status: Status,
    inventory: Vec<u32>,
    guild_id: Option<u64>,

    // Special types
    owner: Identity,
    connection: Option<ConnectionId>,
    created_at: Timestamp,
    play_time: TimeDuration,
}
```
