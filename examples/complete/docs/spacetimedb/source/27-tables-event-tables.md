# Event Tables

In many applications, particularly games and real-time systems, modules need to notify clients about things that happened without storing that information permanently. A combat system might need to tell clients "entity X took 50 damage" so they can display a floating damage number, but there is no reason to keep that record in the database after the moment has passed.

Event tables provide exactly this capability. An event table is a table whose rows are inserted and then immediately deleted by the database: they exist only for the duration of the transaction that created them. When the transaction commits, the rows are broadcast to subscribed clients and then deleted from the table. Between transactions, the table is always empty.

From the module's perspective, event tables behave like regular tables during a reducer's execution. You insert rows, query them, and apply constraints just as you would with any other table. The difference is purely in what happens after the transaction completes: rather than merging the rows into the committed database state, SpacetimeDB publishes them to subscribers and deletes them from the table. The inserts are still recorded in the commitlog, so a full history of events is preserved.

## Defining an Event Table

To declare a table as an event table, add the `event` attribute to the table definition. Event tables support all the same column types, constraints, indexes, and auto-increment fields as regular tables.

```
#[spacetimedb::table(accessor = damage_event, public, event)]
pub struct DamageEvent {
    pub entity_id: Identity,
    pub damage: u32,
    pub source: String,
}
```

> **Changing the event flag**
>
> Once a table has been published as an event table (or a regular table), the `event` flag cannot be changed in a subsequent module update. Attempting to convert a regular table to an event table or vice versa will produce a migration error.

## Publishing Events

To publish an event, simply insert a row into the event table from within a reducer. The insertion works exactly like inserting into a regular table. The row is visible within the current transaction and can be queried or used in constraints. When the transaction commits successfully, the row is broadcast to all subscribed clients. If the reducer panics or the transaction is rolled back, no events are sent.

```
#[spacetimedb::reducer]
fn attack(ctx: &ReducerContext, target_id: Identity, damage: u32) {
    // Game logic...

    // Publish the event
    ctx.db.damage_event().insert(DamageEvent {
        entity_id: target_id,
        damage,
        source: "melee_attack".to_string(),
    });
}
```

Because events are just table inserts, you can publish the same event type from any number of reducers. A `DamageEvent` might be inserted by a melee attack reducer, a spell reducer, and an environmental hazard reducer and clients receive the same event regardless of what triggered it.

## Constraints and Indexes

Primary keys, unique constraints, indexes, sequences, and auto-increment columns all work on event tables. The key difference is that these constraints are enforced only within a single transaction and reset between transactions.

For example, if an event table has a primary key column, inserting two rows with the same primary key within the same transaction will produce an error, just as it would for a regular table. However, inserting a row with primary key `1` in one transaction and another row with primary key `1` in a later transaction will both succeed, because the table is empty at the start of each transaction.

This behavior follows naturally from the fact that event table rows are never merged into the committed state. Each transaction begins with an empty table.

## How It Works

Conceptually, every insert into an event table is a **noop**: an insert paired with an automatic delete. The result is that the table state never changes; it is always the empty set.

## Row-Level Security

Row-level security applies to event tables with the same semantics as regular tables. This means you can use RLS rules to control which clients receive which events based on their identity. For example, you could restrict a `DamageEvent` so that only clients whose identity matches the `entity_id` field receive the event, preventing players from seeing damage dealt to other players.

## Current Limitations

Event tables are fully functional for the use cases described above, but a few capabilities are intentionally restricted for the initial release:

- **Views.** Event tables cannot currently be accessed within view functions. Although the proposal defines clear semantics for this (event-table-ness is "infectious," meaning a view that joins on an event table itself becomes an event table), this is deferred to a future release.

## Use Cases

Event tables are well-suited to any situation where the module needs to notify clients about something that happened without storing a permanent record:

- **Combat and damage events.** Floating damage numbers, hit indicators, and kill notifications.
- **Chat messages.** Real-time chat where messages are displayed on arrival but don't need server-side persistence.
- **Notifications.** Transient UI messages like "Player joined", "Achievement unlocked", or "Trade completed".
- **Sound and visual effects.** Triggering client-side effects such as explosions, particles, or audio cues at the right moment.
- **Telemetry and debugging.** Streaming diagnostic data to a connected developer client without accumulating it in the database.

## Next Steps

- Learn about [Tables](/docs/tables) for persistent data storage
- Explore [Schedule Tables](/docs/tables/schedule-tables) for time-triggered actions
- See [Row-Level Security](/docs/tables/access-permissions) for controlling data visibility
