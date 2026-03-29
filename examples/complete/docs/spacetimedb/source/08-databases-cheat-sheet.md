# Cheat Sheet

Quick reference for SpacetimeDB module syntax in Rust.

## Tables

```
use spacetimedb::{table, SpacetimeType};

// Basic table
#[table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    #[unique]
    username: String,
    #[index(btree)]
    score: i32,
}

// Multi-column index
#[table(accessor = score, index(accessor = idx, btree(columns = [player_id, level])))]
pub struct Score {
    player_id: u64,
    level: u32,
}

// Custom types
#[derive(SpacetimeType)]
pub enum Status {
    Active,
    Inactive,
}
```

## Reducers

```
use spacetimedb::{reducer, ReducerContext};

// Basic reducer
#[reducer]
pub fn create_player(ctx: &ReducerContext, username: String) {
    ctx.db.player().insert(Player { id: 0, username, score: 0 });
}

// With error handling
#[reducer]
pub fn update_score(ctx: &ReducerContext, id: u64, points: i32) -> Result<(), String> {
    let mut player = ctx.db.player().id().find(id)
        .ok_or("Player not found")?;
    player.score += points;
    ctx.db.player().id().update(player);
    Ok(())
}

// Query examples
let player = ctx.db.player().id().find(123);           // Find by primary key
let players = ctx.db.player().username().filter("Alice"); // Filter by index
let all = ctx.db.player().iter();                      // Iterate all
ctx.db.player().id().delete(123);                      // Delete by primary key
```

## Lifecycle Reducers

```
#[reducer(init)]
pub fn init(ctx: &ReducerContext) { /* ... */ }

#[reducer(client_connected)]
pub fn on_connect(ctx: &ReducerContext) { /* ... */ }

#[reducer(client_disconnected)]
pub fn on_disconnect(ctx: &ReducerContext) { /* ... */ }
```

## Schedule Tables

```
#[table(accessor = reminder, scheduled(send_reminder))]
pub struct Reminder {
    #[primary_key]
    #[auto_inc]
    id: u64,
    message: String,
    scheduled_at: ScheduleAt,
}

#[reducer]
fn send_reminder(ctx: &ReducerContext, reminder: Reminder) {
    log::info!("Reminder: {}", reminder.message);
}
```

## Procedures

```
// In Cargo.toml: spacetimedb = { version = "1.*", features = ["unstable"] }

#[spacetimedb::procedure]
fn fetch_data(ctx: &mut ProcedureContext, url: String) -> String {
    match ctx.http.get(&url) {
        Ok(response) => {
            let body = response.body().to_string_lossy();
            ctx.with_tx(|ctx| {
                ctx.db.cache().insert(Cache { data: body.clone() });
            });
            body
        }
        Err(error) => {
            log::error!("HTTP request failed: {error:?}");
            String::new()
        }
    }
}
```

## Views

```
use spacetimedb::{view, Query, ViewContext};

// Return single row
#[view(accessor = my_player, public)]
fn my_player(ctx: &ViewContext) -> Option<Player> {
    ctx.db.player().identity().find(ctx.sender())
}

// Return multiple rows
#[view(accessor = top_players, public)]
fn top_players(ctx: &ViewContext) -> Vec<Player> {
    ctx.db.player().score().filter(1000).collect()
}

// Perform a generic filter using the query builder.
// Equivalent to `SELECT * FROM player WHERE score < 1000`.
#[view(accessor = bottom_players, public)]
fn bottom_players(ctx: &ViewContext) -> impl Query<Player> {
    ctx.from.player().r#where(|p| p.score.lt(1000))
}
```

## Context Properties

```
ctx.db                  // Database access
ctx.sender()            // Identity of caller
ctx.connection_id()     // Option<ConnectionId>
ctx.timestamp           // Timestamp
ctx.identity()          // Module's identity
ctx.rng()               // Random number generator
```

## Logging

```
log::error!("Error: {}", msg);
log::warn!("Warning: {}", msg);
log::info!("Info: {}", msg);
log::debug!("Debug: {}", msg);
```

## Common Types

```
// Primitives
bool, String, f32, f64
i8, i16, i32, i64, i128
u8, u16, u32, u64, u128

// Collections
Option<T>, Vec<T>

// SpacetimeDB types
Identity, ConnectionId, Timestamp, Duration, ScheduleAt
```
