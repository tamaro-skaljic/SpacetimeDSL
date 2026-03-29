# Reducer Context

Every reducer receives a special context parameter as its first argument. This context provides read-write access to the database, information about the caller, and additional utilities like random number generation.

The reducer context is required for accessing tables, executing database operations, and retrieving metadata about the current reducer invocation.

## Accessing the Database

The primary purpose of the reducer context is to provide access to the module's database tables.

```
use spacetimedb::{table, reducer, ReducerContext};

#[table(accessor = user)]
pub struct User {
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
}

#[reducer]
fn create_user(ctx: &ReducerContext, name: String) {
    ctx.db.user().insert(User { id: 0, name });
}
```

## Caller Information

The context provides information about who invoked the reducer and when.

### Sender Identity

Every reducer invocation has an associated caller identity.

```
use spacetimedb::{table, reducer, ReducerContext, Identity};

#[table(accessor = player)]
pub struct Player {
    #[primary_key]
    identity: Identity,
    name: String,
    score: u32,
}

#[reducer]
fn update_score(ctx: &ReducerContext, new_score: u32) {
    // Get the caller's identity
    let caller = ctx.sender();

    // Find and update their player record
    if let Some(mut player) = ctx.db.player().identity().find(caller) {
        player.score = new_score;
        ctx.db.player().identity().update(player);
    }
}
```

### Connection ID

The connection ID identifies the specific client connection that invoked the reducer. This is useful for tracking sessions or implementing per-connection state.

> **note**
>
> The connection ID may be `None` for reducers invoked by the system (such as scheduled reducers or lifecycle reducers).

### Timestamp

The timestamp indicates when the reducer was invoked. This value is consistent throughout the reducer execution and is useful for timestamping events or implementing time-based logic.

## Random Number Generation

The context provides access to a random number generator that is deterministic and reproducible. This ensures that reducer execution is consistent across all nodes in a distributed system.

> **warning**
>
> Never use external random number generators. These are non-deterministic and will cause different nodes to produce different results, breaking consensus. Always use the context's `rng()` or `random()` methods.

## Module Identity

The context provides access to the module's own identity, which is useful for distinguishing between user-initiated and system-initiated reducer calls.

This is particularly important for [scheduled reducers](/docs/functions/reducers) that should only be invoked by the system, not by external clients.

```
use spacetimedb::{table, reducer, ReducerContext, ScheduleAt};

#[table(accessor = scheduled_task, scheduled(send_reminder))]
pub struct ScheduledTask {
    #[primary_key]
    #[auto_inc]
    task_id: u64,
    scheduled_at: ScheduleAt,
    message: String,
}

#[reducer]
fn send_reminder(ctx: &ReducerContext, task: ScheduledTask) {
    // Only allow the scheduler (module identity) to call this
    if ctx.sender() != ctx.identity() {
        panic!("This reducer can only be called by the scheduler");
    }

    spacetimedb::log::info!("Reminder: {}", task.message);
}
```

## Context Properties Reference

| Property        | Type                   | Description                               |
| --------------- | ---------------------- | ----------------------------------------- |
| `db`            | `Local`                | Access to the module's database tables    |
| `sender`        | `Identity`             | Identity of the caller                    |
| `connection_id` | `Option<ConnectionId>` | Connection ID of the caller, if available |
| `timestamp`     | `Timestamp`            | Time when the reducer was invoked         |

**Methods:**

- `identity() -> Identity` - Get the module's identity
- `rng() -> &StdbRng` - Get the random number generator
- `random<T>() -> T` - Generate a single random value
- `sender_auth() -> &AuthCtx` - Get authorization context for the caller (includes JWT claims and internal call detection)
