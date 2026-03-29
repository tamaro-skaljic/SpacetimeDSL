# Performance Best Practices

Follow these guidelines to optimize table performance in your SpacetimeDB modules.

## Use Indexes for Lookups

Generally prefer indexed lookups over full table scans:

✅ **Good - Using an index:**

```
// Fast: Uses unique index on name
ctx.db.player().name().filter("Alice")
```

❌ **Avoid - Full table scan:**

```
// Slow: Iterates through all rows
ctx.db.player()
    .iter()
    .find(|p| p.name == "Alice")
```

Add indexes to columns you frequently filter or join on. See [Indexes](/docs/tables/indexes) for details.

## Keep Tables Focused

Break large tables into smaller, more focused tables when appropriate:

**Instead of one large table:**

```
#[spacetimedb::table(accessor = player)]
pub struct Player {
    id: u32,
    name: String,
    // Game state
    position_x: f32,
    position_y: f32,
    health: u32,
    // Statistics (rarely accessed)
    total_kills: u32,
    total_deaths: u32,
    play_time_seconds: u64,
    // Settings (rarely changed)
    audio_volume: f32,
    graphics_quality: u8,
}
```

**Consider splitting into multiple tables:**

```
#[spacetimedb::table(accessor = player)]
pub struct Player {
    #[primary_key]
    id: u32,
    #[unique]
    name: String,
}

#[spacetimedb::table(accessor = player_state)]
pub struct PlayerState {
    #[unique]
    player_id: u32,
    position_x: f32,
    position_y: f32,
    health: u32,
}

#[spacetimedb::table(accessor = player_stats)]
pub struct PlayerStats {
    #[unique]
    player_id: u32,
    total_kills: u32,
    total_deaths: u32,
    play_time_seconds: u64,
}

#[spacetimedb::table(accessor = player_settings)]
pub struct PlayerSettings {
    #[unique]
    player_id: u32,
    audio_volume: f32,
    graphics_quality: u8,
}
```

Benefits:

- Reduces data transferred to clients who don't need all fields
- Allows more targeted subscriptions
- Improves update performance by touching fewer rows
- Makes the schema easier to understand and maintain

## Choose Appropriate Types

Use the smallest integer type that fits your data range:

```
// If you only need 0-255, use u8 instead of u64
level: u8,           // Not u64
player_count: u16,   // Not u64
entity_id: u32,      // Not u64
```

This reduces:

- Memory usage
- Network bandwidth
- Storage requirements

## Consider Table Visibility

Private tables avoid unnecessary client synchronization overhead:

```
// Public table - clients can subscribe and receive updates
#[spacetimedb::table(accessor = player, public)]
pub struct Player { /* ... */ }

// Private table - only visible to module and owner
// Better for internal state, caches, or sensitive data
#[spacetimedb::table(accessor = internal_state)]
pub struct InternalState { /* ... */ }
```

Make tables public only when clients need to access them. Private tables:

- Don't consume client bandwidth
- Don't require client-side storage
- Are hidden from non-owner queries

## Batch Operations

When inserting or updating multiple rows, batch them in a single reducer call rather than making multiple reducer calls:

✅ **Good - Batch operation:**

```
#[spacetimedb::reducer]
fn spawn_enemies(ctx: &ReducerContext, count: u32) {
    for i in 0..count {
        ctx.db.enemy().insert(Enemy {
            id: 0, // auto_inc
            health: 100,
        });
    }
}
```

Batch operations are more efficient because:

- Single transaction reduces overhead
- Reduced network round trips
- Better database performance

## Monitor Table Growth

Be mindful of unbounded table growth:

- Implement cleanup reducers for temporary data
- Archive or delete old records
- Use schedule tables to automatically expire data
- Consider pagination for large result sets

## Next Steps

- Learn about [Indexes](/docs/tables/indexes) to optimize queries
- Review [Reducers](/docs/functions/reducers) for efficient data modification patterns
