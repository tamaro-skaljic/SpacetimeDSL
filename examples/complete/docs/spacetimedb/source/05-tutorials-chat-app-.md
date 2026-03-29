# Chat App Tutorial

In this tutorial, we'll implement a simple chat server as a SpacetimeDB module in Rust.

A SpacetimeDB module is code that gets compiled and uploaded to SpacetimeDB. This code becomes server-side logic that interfaces directly with SpacetimeDB's relational database.

Each SpacetimeDB module defines a set of **tables** and a set of **reducers**.

- Each table is defined as a Rust struct annotated with `#[table(accessor = table_name)]`. An instance of the struct represents a row, and each field represents a column.
- By default, tables are **private**. The `#[table(accessor = table_name, public)]` macro makes a table public. **Public** tables are readable by all users but can still only be modified by your server module code.
- A reducer is a function that traverses and updates the database. Each reducer call runs in its own transaction, and its updates to the database are only committed if the reducer returns successfully. Reducers may return a `Result<()>`, with an `Err` return aborting the transaction.

## Declare imports

Clear out `spacetimedb/src/lib.rs` and add these imports:

```
use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp};
```

From `spacetimedb`, we import:

- `table`, a macro used to define SpacetimeDB tables.
- `reducer`, a macro used to define SpacetimeDB reducers.
- `Table`, a rust trait which allows us to interact with tables.
- `ReducerContext`, a special argument passed to each reducer.
- `Identity`, a unique identifier for each user.
- `Timestamp`, a point in time.

## Define tables

We'll store two kinds of data: information about each user, and the messages that have been sent.

For each `User`, we'll store their `Identity` (the caller's unique identifier), an optional display name, and whether they're currently online. We'll use `Identity` as the primary key (unique and indexed).

Add to `spacetimedb/src/lib.rs`:

```
#[table(accessor = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: Option<String>,
    online: bool,
}

#[table(accessor = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}
```

## Set users' names

We'll allow users to set a display name, since raw identities aren't user-friendly. Define a reducer that validates input, looks up the caller's `User` row by primary key, and updates it.

Add to `spacetimedb/src/lib.rs`:

```
#[reducer]
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User { name: Some(name), ..user });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}
```

You can extend validation with moderation checks, Unicode normalization, max length checks, or duplicate-name rejection.

## Send messages

Define a reducer to insert a new `Message` with the caller's identity and the call timestamp.

Add to `spacetimedb/src/lib.rs`:

```
#[reducer]
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
    let text = validate_message(text)?;
    log::info!("{}", text);
    ctx.db.message().insert(Message {
        sender: ctx.sender(),
        text,
        sent: ctx.timestamp,
    });
    Ok(())
}

fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Messages must not be empty".to_string())
    } else {
        Ok(text)
    }
}
```

## Set users' online status

SpacetimeDB can invoke lifecycle reducers when clients connect/disconnect. We'll create or update a `User` row to mark the caller online on connect, and mark them offline on disconnect.

Add to `spacetimedb/src/lib.rs`:

```
#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User { online: true, ..user });
    } else {
        ctx.db.user().insert(User {
            name: None,
            identity: ctx.sender(),
            online: true,
        });
    }
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User { online: false, ..user });
    } else {
        log::warn!("Disconnect event for unknown user with identity {:?}", ctx.sender());
    }
}
```

You've just set up your first SpacetimeDB module! You can find the full code for this module:

- [Rust server module](https://github.com/clockworklabs/SpacetimeDB/tree/master/templates/chat-console-rs/spacetimedb)
