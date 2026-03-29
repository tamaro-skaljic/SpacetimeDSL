# Lifecycle Reducers

Special reducers handle system events during the database lifecycle.

## Init Reducer

Runs once when the module is first published or when the database is cleared.

```
#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Database initializing...");

    // Set up default data
    if ctx.db.settings().count() == 0 {
        ctx.db.settings().try_insert(Settings {
            key: "welcome_message".to_string(),
            value: "Hello, SpacetimeDB!".to_string(),
        })?;
    }

    Ok(())
}
```

The `init` reducer:

- Cannot take arguments beyond `ReducerContext`
- Runs when publishing for the first time or publishing while clearing the database data
- Failure prevents publishing or clearing

## Client Connected

Runs when a client establishes a connection.

```
#[reducer(client_connected)]
pub fn on_connect(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Client connected: {}", ctx.sender());

    // ctx.connection_id() is guaranteed to be Some(...)
    let conn_id = ctx.connection_id().unwrap();

    // Initialize client session
    ctx.db.sessions().try_insert(Session {
        connection_id: conn_id,
        identity: ctx.sender(),
        connected_at: ctx.timestamp,
    })?;

    Ok(())
}
```

The `client_connected` reducer:

- Cannot take arguments beyond `ReducerContext`
- `ctx.connection_id()` is guaranteed to be present
- Failure disconnects the client
- Runs for each distinct connection (WebSocket, HTTP call)

## Client Disconnected

Runs when a client connection terminates.

```
#[reducer(client_disconnected)]
pub fn on_disconnect(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Client disconnected: {}", ctx.sender());

    // ctx.connection_id() is guaranteed to be Some(...)
    let conn_id = ctx.connection_id().unwrap();

    // Clean up client session
    ctx.db.sessions().connection_id().delete(&conn_id);

    Ok(())
}
```

The `client_disconnected` reducer:

- Cannot take arguments beyond `ReducerContext`
- `ctx.connection_id()` is guaranteed to be present
- Failure is logged but doesn't prevent disconnection
- Runs when connection ends (close, timeout, error)

## Scheduled Reducers

Reducers can be triggered at specific times using schedule tables. See [Schedule Tables](/docs/tables/schedule-tables) for details on:

- Defining schedule tables
- Triggering reducers at specific timestamps
- Running reducers periodically
- Canceling scheduled executions

> **Scheduled Reducer Context**
>
> Scheduled reducer calls originate from SpacetimeDB itself, not from a client. Therefore:
>
>
>
> - `ctx.sender()` will be the module's own identity
> - `ctx.connection_id()` will be `None`
