# Overview

Reducers are functions that modify database state in response to client requests or system events. They are the **only** way to mutate tables in SpacetimeDB - all database changes must go through reducers.

## Defining Reducers

Reducers are defined in your module code and automatically exposed as callable functions to connected clients.

Use the `#[spacetimedb::reducer]` macro on a function:

```
use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
pub fn create_user(ctx: &ReducerContext, name: String, email: String) -> Result<(), String> {
    // Validate input
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Modify tables
    ctx.db.user().insert(User {
        id: 0, // auto-increment will assign
        name,
        email,
    });

    Ok(())
}
```

Reducers must take `&ReducerContext` as their first parameter. Additional parameters must be serializable types. Reducers can return `()`, `Result<(), String>`, or `Result<(), E>` where `E: Display`.

> **Rust: Importing the Table Trait**
>
> Table operations like `insert`, `try_insert`, `iter`, and `count` are provided by the `Table` trait. You must import this trait for these methods to be available:
>
>
>
> ```
> use spacetimedb::Table;
> ```
>
>
>
> If you see errors like "no method named `try_insert` found", add this import.

## Transactional Execution

Every reducer runs inside a database transaction. This provides important guarantees:

- **Isolation**: Reducers don't see changes from other concurrent reducers
- **Atomicity**: Either all changes succeed or all are rolled back
- **Consistency**: Failed reducers leave the database unchanged

If a reducer throws an exception or returns an error, all of its changes are automatically rolled back.

## Accessing Tables

Reducers have full read-write access to all tables (both public and private) through the `ReducerContext`. The examples below assume a `user` table with `id` (primary key), `name` (indexed), and `email` (unique) columns.

### Inserting Rows

```
ctx.db.user().insert(User {
    id: 0,  // auto-increment will assign
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
});
```

### Finding Rows by Unique Column

Use `find` on a unique or primary key column to retrieve a single row:

```
if let Some(user) = ctx.db.user().id().find(123) {
    log::info!("Found: {}", user.name);
}

let by_email = ctx.db.user().email().find("alice@example.com");
```

### Filtering Rows by Indexed Column

Use `filter` on an indexed column to retrieve multiple matching rows:

```
for user in ctx.db.user().name().filter("Alice") {
    log::info!("User {}: {}", user.id, user.email);
}
```

### Updating Rows

Find a row, modify it, then call `update` on the same unique column:

```
if let Some(mut user) = ctx.db.user().id().find(123) {
    user.name = "Bob".to_string();
    ctx.db.user().id().update(user);
}
```

### Deleting Rows

Delete by unique column value or by indexed column value:

```
// Delete by primary key
ctx.db.user().id().delete(123);

// Delete all matching an indexed column
let deleted = ctx.db.user().name().delete("Alice");
log::info!("Deleted {} row(s)", deleted);
```

### Iterating All Rows

Use `iter` to iterate over all rows in a table:

```
for user in ctx.db.user().iter() {
    log::info!("{}: {}", user.id, user.name);
}
```

### Counting Rows

Use `count` to get the number of rows in a table:

```
let total = ctx.db.user().count();
log::info!("Total users: {}", total);
```

For more details on querying with indexes, including range queries and multi-column indexes, see [Indexes](/docs/tables/indexes).

## Reducer Isolation

Reducers run in an isolated environment and **cannot** interact with the outside world:

- ❌ No network requests
- ❌ No file system access
- ❌ No system calls
- ✅ Only database operations

If you need to interact with external systems, use [Procedures](/docs/functions/procedures) instead. Procedures can make network calls and perform other side effects, but they have different execution semantics and limitations.

> **Global and Static Variables Are Undefined Behavior**
>
> Relying on global variables, static variables, or module-level state to persist across reducer calls is **undefined behavior**. SpacetimeDB does not guarantee that values stored in these locations will be available in subsequent reducer invocations.
>
>
>
> This is undefined for several reasons:
>
>
>
> 1. **Fresh execution environments.** SpacetimeDB may run each reducer in a fresh WASM instance.
> 2. **Module updates.** Publishing a new module creates a fresh execution environment. This is necessary for hot-swapping modules while transactions are in flight.
> 3. **Concurrent execution.** SpacetimeDB reserves the right to execute multiple reducers concurrently in separate execution environments (e.g., with MVCC).
> 4. **Crash recovery.** Instance memory is not persisted across restarts.
> 5. **Non-transactional updates.** If you modify global state and then roll back the transaction, the modified value may remain for subsequent transactions.
> 6. **Replay safety.** If a serializability anomaly is detected, SpacetimeDB may re-execute your reducer with the same arguments, causing modifications to global state to occur multiple times.
>
>
>
> Reducers are designed to be free of side effects. They should only modify tables. Always store state in tables to ensure correctness and durability.
>
>
>
> ```
> // ❌ Undefined behavior: may or may not persist or correctly update across reducer calls
> static mut COUNTER: u64 = 0;
>
> // ✅ Store state in a table instead
> #[spacetimedb::table(accessor = counter)]
> pub struct Counter {
>     #[primary_key]
>     id: u32,
>     value: u64,
> }
> ```

## Scheduling Procedures

Reducers cannot call procedures directly (procedures may have side effects incompatible with transactional execution). Instead, schedule a procedure to run by inserting into a [schedule table](/docs/tables/schedule-tables):

```
use spacetimedb::{ScheduleAt, ReducerContext, ProcedureContext, Table};
use std::time::Duration;

#[spacetimedb::table(accessor = fetch_schedule, scheduled(fetch_external_data))]
pub struct FetchSchedule {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: ScheduleAt,
    url: String,
}

#[spacetimedb::procedure]
fn fetch_external_data(ctx: &mut ProcedureContext, schedule: FetchSchedule) {
    if let Ok(response) = ctx.http.get(&schedule.url) {
        // Process response...
    }
}

// From a reducer, schedule the procedure
#[spacetimedb::reducer]
fn queue_fetch(ctx: &ReducerContext, url: String) {
    ctx.db.fetch_schedule().insert(FetchSchedule {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(Duration::ZERO.into()),
        url,
    });
}
```

See [Schedule Tables](/docs/tables/schedule-tables) for more scheduling options.

## Next Steps

- Learn about [Tables](/docs/tables) to understand data storage
- Explore [Procedures](/docs/functions/procedures) for side effects beyond the database
