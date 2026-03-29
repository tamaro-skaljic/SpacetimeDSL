# Table Access Permissions

SpacetimeDB controls data access through table visibility and context-based permissions. Tables can be public or private, and different execution contexts (reducers, views, clients) have different levels of access.

## Public and Private Tables

Tables are **private** by default. Private tables can only be accessed by reducers and views running on the server. Clients cannot query, subscribe to, or see private tables.

**Public** tables are exposed to clients for read access through subscriptions and queries. Clients can see public table data but can only modify it by calling reducers.

```
// Private table (default) - only accessible from server-side code
#[spacetimedb::table(accessor = internal_config)]
pub struct InternalConfig {
    #[primary_key]
    key: String,
    value: String,
}

// Public table - clients can subscribe and query
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
    score: u64,
}
```

Use private tables for:

- Internal configuration or state that clients should not see
- Sensitive data like password hashes or API keys
- Intermediate computation results

Use public tables for:

- Data that clients need to display or interact with
- Game state, user profiles, or other user-facing data

## Reducers - Read-Write Access

Reducers receive a `ReducerContext` which provides full read-write access to all tables (both public and private). They can perform all CRUD operations: insert, read, update, and delete.

```
#[spacetimedb::reducer]
fn example(ctx: &ReducerContext) -> Result<(), String> {
    // Insert
    ctx.db.user().insert(User {
        id: 0,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    });

    // Read: iterate all rows
    for user in ctx.db.user().iter() {
        log::info!("User: {}", user.name);
    }

    // Read: find by unique column
    if let Some(mut user) = ctx.db.user().id().find(123) {
        // Update
        user.name = "Bob".to_string();
        ctx.db.user().id().update(user);
    }

    // Delete
    ctx.db.user().id().delete(456);

    Ok(())
}
```

## Procedures with Transactions - Read-Write Access

Procedures receive a `ProcedureContext` and can access tables through transactions. Unlike reducers, procedures must explicitly open a transaction to read from or modify the database.

```
#[spacetimedb::procedure]
fn update_user_procedure(ctx: &mut ProcedureContext, user_id: u64, new_name: String) {
    // Must explicitly open a transaction
    ctx.with_tx(|ctx| {
        // Full read-write access within the transaction
        if let Some(mut user) = ctx.db.user().id().find(user_id) {
            user.name = new_name.clone();
            ctx.db.user().id().update(user);
        }
    });
    // Transaction is committed when the closure returns
}
```

See the [Procedures documentation](/docs/functions/procedures) for more details on using procedures, including making HTTP requests to external services.

## Views - Read-Only Access

[Views](/docs/functions/views) receive a `ViewContext` or `AnonymousViewContext` which provides read-only access to all tables (both public and private). They can query and iterate tables, but cannot insert, update, or delete rows.

```
#[spacetimedb::view(accessor = find_users_by_name, public)]
fn find_users_by_name(ctx: &ViewContext) -> Vec<User> {
    // Can read and filter
    ctx.db.user().name().filter("Alice").collect()

    // Cannot insert, update, or delete
    // ctx.db.user().insert(...) // ❌ Compile error
}
```

See the [Views documentation](/docs/functions/views) for more details on defining and querying views.

## Using Views for Fine-Grained Access Control

While table visibility controls whether clients can access a table at all, views provide fine-grained control over which rows and columns clients can see. Views can read from private tables and expose only the data appropriate for each client.

> **note**
>
> Views can only access table data through indexed lookups, not by scanning all rows. This restriction ensures views remain performant. See the [Views documentation](/docs/functions/views) for details.

### Filtering Rows by Caller

Use views with `ViewContext` to return only the rows that belong to the caller. The view accesses the caller's identity through `ctx.sender()` and uses it to look up rows via an index.

```
use spacetimedb::{Identity, Timestamp, ViewContext};

// Private table containing all messages
#[spacetimedb::table(accessor = message)]  // Private by default
pub struct Message {
    #[primary_key]
    #[auto_inc]
    id: u64,
    #[index(btree)]
    sender: Identity,
    #[index(btree)]
    recipient: Identity,
    content: String,
    timestamp: Timestamp,
}

// Public view that only returns messages the caller can see
#[spacetimedb::view(accessor = my_messages, public)]
fn my_messages(ctx: &ViewContext) -> Vec<Message> {
    // Look up messages by index where caller is sender or recipient
    let sent: Vec<_> = ctx.db.message().sender().filter(&ctx.sender()).collect();
    let received: Vec<_> = ctx.db.message().recipient().filter(&ctx.sender()).collect();
    sent.into_iter().chain(received).collect()
}
```

Clients querying `my_messages` will only see their own messages, even though all messages are stored in the same table.

### Hiding Sensitive Columns

Use views to return a custom type that omits sensitive columns. The view reads from a table with sensitive data and returns a projection containing only the columns clients should see.

```
use spacetimedb::{SpacetimeType, ViewContext, Timestamp, Identity};

// Private table with sensitive data
#[spacetimedb::table(accessor = user_account)]  // Private by default
pub struct UserAccount {
    #[primary_key]
    #[auto_inc]
    id: u64,
    #[unique]
    identity: Identity,
    username: String,
    email: String,
    password_hash: String,  // Sensitive
    api_key: String,        // Sensitive
    created_at: Timestamp,
}

// Public type without sensitive columns
#[derive(SpacetimeType)]
pub struct PublicUserProfile {
    id: u64,
    username: String,
    created_at: Timestamp,
}

// Public view that returns the caller's profile without sensitive data
#[spacetimedb::view(accessor = my_profile, public)]
fn my_profile(ctx: &ViewContext) -> Option<PublicUserProfile> {
    // Look up the caller's account by their identity (unique index)
    let user = ctx.db.user_account().identity().find(&ctx.sender())?;
    Some(PublicUserProfile {
        id: user.id,
        username: user.username,
        created_at: user.created_at,
        // email, password_hash, and api_key are not included
    })
}
```

Clients can query `my_profile` to see their username and creation date, but never see their email address, password hash, or API key.

### Combining Both Techniques

Views can combine row filtering and column projection. This example returns colleagues in the same department as the caller, with salary information hidden:

```
use spacetimedb::{SpacetimeType, Identity, ViewContext};

// Private table with all employee data
#[spacetimedb::table(accessor = employee)]
pub struct Employee {
    #[primary_key]
    id: u64,
    #[unique]
    identity: Identity,
    name: String,
    #[index(btree)]
    department: String,
    salary: u64,           // Sensitive
}

// Public type for colleagues (no salary)
#[derive(SpacetimeType)]
pub struct Colleague {
    id: u64,
    name: String,
    department: String,
}

// View that returns colleagues in the caller's department, without salary info
#[spacetimedb::view(accessor = my_colleagues, public)]
fn my_colleagues(ctx: &ViewContext) -> Vec<Colleague> {
    // Find the caller's employee record by identity (unique index)
    let Some(me) = ctx.db.employee().identity().find(&ctx.sender()) else {
        return vec![];
    };

    // Look up employees in the same department
    ctx.db.employee().department().filter(&me.department)
        .map(|emp| Colleague {
            id: emp.id,
            name: emp.name.clone(),
            department: emp.department.clone(),
            // salary is not included
        })
        .collect()
}
```

## Client Access - Read-Only Access

Clients connect to databases and can access public tables and views through subscriptions and queries. They cannot access private tables directly. See the [Subscriptions documentation](/docs/clients/subscriptions) for details on client-side table access.
