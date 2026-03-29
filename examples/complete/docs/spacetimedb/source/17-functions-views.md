# Views

Views are read-only functions that compute and return results from your tables. Unlike [reducers](/docs/functions/reducers), views do not modify database state - they only query and return data. Views are useful for computing derived data, aggregations, or joining multiple tables before sending results to clients.

## Why Use Views?

Views provide several benefits:

- **Performance**: Views compute results server-side, reducing the amount of data sent to clients
- **Encapsulation**: Views can hide complex queries behind simple interfaces
- **Consistency**: Views ensure clients receive consistently formatted data
- **Real-time updates**: Like tables, views can be subscribed to and automatically update when underlying data changes

## Defining Views

Views must be declared as `public` with an explicit `name`, and they accept only a context parameter - no user-defined arguments beyond the context type.

Use the `#[spacetimedb::view]` macro on a function:

```
use spacetimedb::{view, ViewContext, AnonymousViewContext, table, SpacetimeType};
use spacetimedb_lib::Identity;

#[spacetimedb::table(accessor = player)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    #[unique]
    identity: Identity,
    name: String,
}

#[spacetimedb::table(accessor = player_level)]
pub struct PlayerLevel {
    #[unique]
    player_id: u64,
    #[index(btree)]
    level: u64,
}

#[derive(SpacetimeType)]
pub struct PlayerAndLevel {
    id: u64,
    identity: Identity,
    name: String,
    level: u64,
}

// At-most-one row: return Option<T>
#[view(accessor = my_player, public)]
fn my_player(ctx: &ViewContext) -> Option<Player> {
    ctx.db.player().identity().find(ctx.sender())
}

// Multiple rows: return Vec<T>
#[view(accessor = players_for_level, public)]
fn players_for_level(ctx: &AnonymousViewContext) -> Vec<PlayerAndLevel> {
    ctx.db
        .player_level()
        .level()
        .filter(2u64)
        .flat_map(|player| {
            ctx.db
                .player()
                .id()
                .find(player.player_id)
                .map(|p| PlayerAndLevel {
                    id: p.id,
                    identity: p.identity,
                    name: p.name,
                    level: player.level,
                })
        })
        .collect()
}
```

Views can return either `Option<T>` for at-most-one row or `Vec<T>` for multiple rows, where `T` can be a table type or any product type.

## ViewContext and AnonymousViewContext

Views use one of two context types:

- **`ViewContext`**: Provides access to the caller's `Identity` through `ctx.sender()`. Use this when the view depends on who is querying it.
- **`AnonymousViewContext`**: Does not provide caller information. Use this when the view produces the same results regardless of who queries it.

Both contexts provide read-only access to tables and indexes through `ctx.db`.

### Performance: Why AnonymousViewContext Matters

The choice between `ViewContext` and `AnonymousViewContext` has significant performance implications.

**Anonymous views can be shared across all subscribers.** When a view uses `AnonymousViewContext`, SpacetimeDB knows the result is the same for every client. The database can materialize the view once and serve that same result to all subscribers. When the underlying data changes, it recomputes the view once and broadcasts the update to everyone.

**Per-user views require separate computation for each subscriber.** When a view uses `ViewContext` and invokes `ctx.sender()`, each client potentially sees different data. SpacetimeDB must compute and track the view separately for each subscriber. With 1,000 connected users, that's 1,000 separate view computations and 1,000 separate sets of change tracking.

**Prefer `AnonymousViewContext` when possible.** Design your views to be caller-independent when the use case allows. For example:

| Use Case           | Recommended Context    | Why                                          |
| ------------------ | ---------------------- | -------------------------------------------- |
| Global leaderboard | `AnonymousViewContext` | Same top-10 for everyone                     |
| Shop inventory     | `AnonymousViewContext` | Same items available to all                  |
| My inventory       | `ViewContext`          | Different per player                         |
| My messages        | `ViewContext`          | Private to each user                         |
| World map regions  | `AnonymousViewContext` | Geographic data shared by all nearby players |

**Design around shared data when you can.** Sometimes a small design change lets you use anonymous views. For example, instead of a view that returns "entities near me" (which requires knowing who "me" is), consider views that return "entities in region X". Multiple players in the same region share a single materialized view rather than each having their own.

### Example: Per-User View

This view returns the caller's own player data. Each connected client sees different results, so SpacetimeDB must track it separately for each subscriber.

```
// Per-user: each client sees their own player
#[view(accessor = my_player, public)]
fn my_player(ctx: &ViewContext) -> Option<Player> {
    ctx.db.player().identity().find(&ctx.sender())
}
```

### Example: High Scores Leaderboard

This view returns players with scores above a threshold. Every client sees the same results, so SpacetimeDB computes it once and shares it across all subscribers. The view uses a btree index on the score column to efficiently find high-scoring players.

```
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
    #[index(btree)]
    score: u64,
}

// Shared: same high scorers for all clients
#[view(accessor = high_scorers, public)]
fn high_scorers(ctx: &AnonymousViewContext) -> Vec<Player> {
    // Get all players with score >= 1000 using the btree index
    ctx.db.player().score().filter(1000..).collect()
}
```

### Example: Region-Based Design

Instead of querying "what's near me" (per-user), design your data model so clients subscribe to shared regions. This example shows entities organized by chunk coordinates.

```
use spacetimedb::{view, AnonymousViewContext, ViewContext};

#[spacetimedb::table(accessor = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    id: u64,
    #[index(btree)]
    chunk_x: i32,
    #[index(btree)]
    chunk_y: i32,
    local_x: f32,
    local_y: f32,
    entity_type: String,
}

#[spacetimedb::table(accessor = player_chunk, public)]
pub struct PlayerChunk {
    #[primary_key]
    player_id: u64,
    chunk_x: i32,
    chunk_y: i32,
}

// Shared: all players in chunk (0,0) share this view
#[view(accessor = entities_in_origin_chunk, public)]
fn entities_in_origin_chunk(ctx: &AnonymousViewContext) -> Vec<Entity> {
    ctx.db.entity().chunk_x().filter(&0)
        .filter(|e| e.chunk_y == 0)
        .collect()
}

// Per-user: returns entities in the chunk the player is currently in
#[view(accessor = entities_in_my_chunk, public)]
fn entities_in_my_chunk(ctx: &ViewContext) -> Vec<Entity> {
    let Some(player) = ctx.db.player().identity().find(&ctx.sender()) else {
        return vec![];
    };
    let Some(chunk) = ctx.db.player_chunk().player_id().find(&player.id) else {
        return vec![];
    };

    ctx.db.entity().chunk_x().filter(&chunk.chunk_x)
        .filter(|e| e.chunk_y == chunk.chunk_y)
        .collect()
}
```

The `entities_in_origin_chunk` view is shared - if 100 players are all looking at chunk (0,0), SpacetimeDB computes it once. The `entities_in_my_chunk` view requires per-user computation since each player may be in a different chunk.

For games with many players in the same area, the shared approach scales much better. Clients can subscribe to the specific chunk views they need based on their position, and players in the same chunk automatically share the same materialized data.

## Querying Views

Views can be queried and subscribed to just like normal tables.

When subscribed to, views automatically update when their underlying tables change, providing real-time updates to clients.

## Why Views Cannot Use .iter()

You may notice that views can only access table data through indexed lookups (`.find()` and `.filter()` on indexed columns), not through `.iter()` which scans all rows. This is a deliberate design choice for performance.

**Views are black boxes.** View functions are Turing-complete code that SpacetimeDB cannot analyze or optimize. When a view reads from a table, SpacetimeDB tracks the "read set" - which rows the view accessed. If any row in that read set changes, the view's output might have changed, so SpacetimeDB must re-execute the entire view function.

**Full table scans create pessimistic read sets.** If a view used `.iter()` to scan an entire table, its read set would include every row in that table. This means any change to any row - even rows unrelated to the view's output - would trigger a complete re-evaluation of the view. For a table with millions of rows, this becomes prohibitively expensive.

**Index lookups enable targeted invalidation.** When a view uses `.find()` or `.filter()` on an indexed column, SpacetimeDB knows exactly which rows the view depends on. If a row outside that set changes, the view doesn't need to be re-evaluated. This keeps view updates fast and predictable.

**Why SQL subscriptions can scan.** You might wonder why SQL subscription queries can include full table scans while view functions cannot. The difference is that SQL queries are not black boxes - SpacetimeDB can analyze and transform them. The query engine uses **incremental evaluation**: when rows change, it computes exactly which output rows are affected without re-running the entire query. Think of it like taking the derivative of the query - given a small change in input, compute the small change in output. Since view functions are opaque code, this kind of incremental computation isn't possible.

**Why query builder subscriptions can scan.** For the same reason that SQL subscriptions can scan. Anything you can do with SQL subscriptions you can do with the query builder API and vice versa.

**The tradeoff is acceptable for indexed access.** For point lookups (`.find()`) and small range scans (`.filter()` on indexed columns), the performance difference between full re-evaluation and incremental evaluation is small. This is why views are limited to indexed access - it's the subset of operations where the black-box limitation doesn't hurt performance.

If you need to aggregate or sort entire tables, consider returning a `Query` from your view instead. Since queries can be analyzed by the query engine, they support incremental evaluation even when scanning full tables. Alternatively, design your schema so the data you need is accessible through indexes.

## Performance Considerations

Views compute results server side, which can improve performance by:

- Reducing network traffic by filtering/aggregating before sending data
- Avoiding redundant computation on multiple clients
- Leveraging server-side indexes and query optimization

However, keep in mind that:

- View functions are reevaluated when SpacetimeDB detects something they read has since changed (the read set)
- The key to optimizing a view function is keeping its read set small
- View functions can have large read sets when they read from non-unique indexes and/or multiple tables without using the query builder
- View functions are executed largely as written by the WASM/V8 runtimes with little room for global optimization
- Join-heavy view functions will incur extra row materialization costs when serializing data across the WASM/V8 boundary

If a view is primarily performing query logic like filtering and joining rows, it is recommended to use the module-side query builder,
since it pushes work into the query engine, which can optimize and evaluate the plan more efficiently.

## Query Builder in Views

### Example

```
#[view(accessor = high_scorers, public)]
fn high_scorers(ctx: &AnonymousViewContext) -> impl Query<Player> {
    ctx.from
        .player()
        .r#where(|p| p.score.gte(1000u64))
        .r#where(|p| p.name.ne("BOT"))
}
```

### Why Use the Query Builder?

Both `ViewContext` and `AnonymousViewContext` expose a builder API for expressing query logic like filters and joins.
The module-side query builder is designed for view logic that is mostly filters and joins.
These views are evaluated by the query engine instead of the WASM/V8 runtime, the benefits of which include the following:

- The query engine can apply global optimizations that WASM and V8 cannot, drastically improving the performance of your views
- These views can be updated incrementally; the database does not have to re-evaluate the entire view if its read set changes
- These views avoid the row materialization costs incurred by procedural view functions every time your code fetches/reads a row from the database

For join-heavy views, this difference is often significant.

In the example below, we have `players` and `moderators`.
The procedural version must execute as written: iterate filtered rows from `players`, then probe `moderators` row by row.
The query builder version expresses the same relationship declaratively,
which means the query engine is free to choose a better plan,
including reordering the join and starting from `moderators` if it thinks that will be more efficient.

```
// Procedural: row-by-row join in module code.
#[view(accessor = high_scoring_moderators_procedural, public)]
fn high_scoring_moderators_procedural(ctx: &AnonymousViewContext) -> Vec<Player> {
    let mut out = Vec::new();
    for p in ctx.db.player().score().filter(100..) {
        if ctx.db.moderator().player_id().find(p.id).is_some() {
            out.push(p);
        }
    }
    out
}

// Query builder: equivalent logic pushed to the query engine.
// The engine can reorder this join if it decides the smaller side is a better starting point.
#[view(accessor = high_scoring_moderators_declarative, public)]
fn high_scoring_moderators_declarative(ctx: &AnonymousViewContext) -> impl Query<Player> {
    ctx.from
        .player()
        .r#where(|p| p.score.gte(100u64))
        .left_semijoin(ctx.from.moderator(), |p, m| p.id.eq(m.player_id))
}
```

### API Reference

#### Table Accessors

The query builder is available only through `ViewContext` and `AnonymousViewContext`:
There is an accessor method/property for each table defined in your module.

```
use spacetimedb::{table, view, AnonymousViewContext, Query};

#[spacetimedb::table(name = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
    #[index(btree)]
    score: u64,
}

#[spacetimedb::table(name = player_level, public)]
pub struct PlayerLevel {
    #[unique]
    player_id: u64,
    #[index(btree)]
    level: u64,
}

#[view(accessor = all_players, public)]
fn all_players(ctx: &AnonymousViewContext) -> impl Query<Player> {
    ctx.from.player()
}

#[view(accessor = all_player_levels, public)]
fn all_player_levels(ctx: &AnonymousViewContext) -> impl Query<PlayerLevel> {
    ctx.from.player_level()
}
```

#### where / filter

Use `where` to apply predicates. Chaining multiple filters combines them with logical `AND`.

`filter` is an alias for `where`.

##### Comparison operators

| Operation             | Rust  |
| --------------------- | ----- |
| Equal                 | `eq`  |
| Not equal             | `ne`  |
| Less than             | `lt`  |
| Less than or equal    | `lte` |
| Greater than          | `gt`  |
| Greater than or equal | `gte` |

```
#[view(accessor = high_scorers, public)]
fn high_scorers(ctx: &AnonymousViewContext) -> impl Query<Player> {
    ctx.from
        .player()
        .r#where(|p| p.score.gte(1000u64))
        .filter(|p| p.name.ne("BOT"))
}
```

##### Boolean combinators

Combine conditions with `and`, `or`, and `not`:

```
ctx.from.player().r#where(|p| p.score.gte(1000u64).and(p.score.lt(5000u64)));
ctx.from.player().r#where(|p| p.name.eq("ADMIN").or(p.name.eq("BOT")));
ctx.from.player().r#where(|p| p.name.eq("BOT").not());
```

Comparisons are strongly typed, meaning invalid comparisons are rejected by the type system.
An example would be comparing an `Identity` column to an integer literal.

#### Semijoins

Semijoins keep rows from one side when a matching row exists on the other side. They are used for filtering rows in one table based on rows in the other table.

- A left semijoin returns rows from the left side that have at least one match on the right side.
- A right semijoin returns rows from the right side that have at least one match on the left side.
- Filters chained before a semijoin apply to the pre-join source side.
- Filters chained after a semijoin apply to whichever side the semijoin returns.
- Like `where/filter`, join predicates are strongly typed
- Additionally join predicates may only use indexed columns with multi-column indexes not supported at the moment

```
#[view(accessor = players_with_levels, public)]
fn players_with_levels(ctx: &AnonymousViewContext) -> impl Query<Player> {
    ctx.from
        .player()
        .left_semijoin(ctx.from.player_level(), |p, pl| p.id.eq(pl.player_id))
}

#[view(accessor = levels_for_high_scorers, public)]
fn levels_for_high_scorers(ctx: &AnonymousViewContext) -> impl Query<PlayerLevel> {
    ctx.from
        .player()
        .r#where(|p| p.score.gte(1000u64))
        .right_semijoin(ctx.from.player_level(), |p, pl| p.id.eq(pl.player_id))
        .r#where(|pl| pl.level.gte(10u64))
}
```
