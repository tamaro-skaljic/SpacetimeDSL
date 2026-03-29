# Procedures

A **procedure** is a function exported by a [database](/docs/databases), similar to a [reducer](/docs/functions/reducers).
Connected [clients](/docs/clients) can call procedures.
Procedures can perform additional operations not possible in reducers, including making HTTP requests to external services.
However, procedures don't automatically run in database transactions,
and must manually open and commit a transaction in order to read from or modify the database state.
For this reason, prefer defining reducers rather than procedures unless you need to use one of the special procedure operators.

> **warning**
>
> ***Procedures are currently in beta, and their API may change in upcoming SpacetimeDB releases.***

## Defining Procedures

Because procedures are unstable, Rust modules that define them must opt in to the `unstable` feature in their `Cargo.toml`:

```
[dependencies]
spacetimedb = { version = "1.*", features = ["unstable"] }
```

Define a procedure by annotating a function with `#[spacetimedb::procedure]`.

This function's first argument must be of type `&mut spacetimedb::ProcedureContext`.
By convention, this argument is named `ctx`.

A procedure may accept any number of additional arguments.
Each argument must be of a type that implements `spacetimedb::SpacetimeType`.
When defining a `struct` or `enum`, annotate it with `#[derive(spacetimedb::SpacetimeType)]`
to make it usable as a procedure argument.
These argument values will not be broadcast to clients other than the caller.

A procedure may return a value of any type that implements `spacetimedb::SpacetimeType`.
This return value will be sent to the caller, but will not be broadcast to any other clients.

```
#[spacetimedb::procedure]
fn add_two_numbers(ctx: &mut spacetimedb::ProcedureContext, lhs: u32, rhs: u32) -> u64 {
    lhs as u64 + rhs as u64
}
```

### Accessing the database

Unlike reducers, procedures don't automatically run in database transactions.
This means there's no `ctx.db` field to access the database.
Instead, procedure code must manage transactions explicitly with `ProcedureContext::with_tx`.

```
#[spacetimedb::table(accessor = my_table)]
struct MyTable {
    a: u32,
    b: String,
}

#[spacetimedb::procedure]
fn insert_a_value(ctx: &mut ProcedureContext, a: u32, b: String) {
    ctx.with_tx(|ctx| {
        ctx.my_table().insert(MyTable { a, b });
    });
}
```

`ProcedureContext::with_tx` takes a function of type `Fn(&TxContext) -> T`.
Within that function, the `&TxContext` can be used to access the database
[in all the same ways as a `ReducerContext`](https://docs.rs/spacetimedb/latest/spacetimedb/struct.ReducerContext.html).
When the function returns, the transaction will be committed,
and its changes to the database state will become permanent and be broadcast to clients.
If the function panics, the transaction will be rolled back, and its changes will be discarded.
However, for transactions that may fail,
[prefer calling `try_with_tx` and returning a `Result`](#fallible-database-operations) rather than panicking.

> **warning**
>
> The function passed to `ProcedureContext::with_tx` may be invoked multiple times,
> possibly seeing a different version of the database state each time.
>
>
>
> If invoked more than once with reference to the same database state,
> it must perform the same operations and return the same result each time.
>
>
>
> If invoked more than once with reference to different database states,
> values observed during prior runs must not influence the behavior of the function or the calling procedure.
>
>
>
> Avoid capturing mutable state within functions passed to `with_tx`.

#### Fallible database operations

For fallible database operations, instead use `ProcedureContext::try_with_tx`:

```
#[spacetimedb::procedure]
fn maybe_insert_a_value(ctx: &mut ProcedureContext, a: u32, b: String) {
    ctx.try_with_tx(|ctx| {
        if a < 10 {
            return Err("a is less than 10!");
        }
        ctx.my_table().insert(MyTable { a, b });
        Ok(())
    });
}
```

`ProcedureContext::try_with_tx` takes a function of type `Fn(&TxContext) -> Result<T, E>`.
If the function returns `Ok`, the transaction will be committed,
and its changes to the database state will become permanent and be broadcast to clients.
If that function returns `Err`, the transaction will be rolled back, and its changes will be discarded.

#### Reading values out of the database

Functions passed to
[`ProcedureContext::with_tx`](#accessing-the-database) and [`ProcedureContext::try_with_tx`](#fallible-database-operations)
may return a value, and that value will be returned to the calling procedure.

Transaction return values are never saved or broadcast to clients, and are used only by the calling procedure.

```
#[spacetimedb::table(accessor = player)]
struct Player {
    id: spacetimedb::Identity,
    level: u32,
}

#[spacetimedb::procedure]
fn find_highest_level_player(ctx: &mut ProcedureContext) {
    let highest_level_player = ctx.with_tx(|ctx| {
        ctx.db.player().iter().max_by_key(|player| player.level)
    });
    match highest_level_player {
        Some(player) => log::info!("Congratulations to {}", player.id),
        None => log::warn!("No players..."),
    }
}
```

## HTTP Requests

Procedures can make HTTP requests to external services using methods contained in `ctx.http`.

`ctx.http.get` performs simple `GET` requests with no headers:

```
#[spacetimedb::procedure]
fn get_request(ctx: &mut ProcedureContext) {
    match ctx.http.get("https://example.invalid") {
        Ok(response) => {
            let (response, body) = response.into_parts();
            log::info!(
                "Got response with status {} and body {}",
                response.status,
                body.into_string_lossy(),
            )
        },
        Err(error) => log::error!("Request failed: {error:?}"),
    }
}
```

`ctx.http.send` sends any [`http::Request`](https://docs.rs/http/latest/http/request/struct.Request.html)
whose body can be converted to `spacetimedb::http::Body`.
`http::Request` is re-exported as `spacetimedb::http::Request`.

```
#[spacetimedb::procedure]
fn post_request(ctx: &mut spacetimedb::ProcedureContext) {
    let request = spacetimedb::http::Request::builder()
        .uri("https://example.invalid/upload")
        .method("POST")
        .header("Content-Type", "text/plain")
        .body("This is the body of the HTTP request")
        .expect("Building `Request` object failed");
    match ctx.http.send(request) {
        Ok(response) => {
            let (response, body) = response.into_parts();
            log::info!(
                "Got response with status {} and body {}",
                response.status,
                body.into_string_lossy(),
            )
        }
        Err(error) => log::error!("Request failed: {error:?}"),
    }
}
```

Each of these methods returns a [`http::Response`](https://docs.rs/http/latest/http/response/struct.Response.html#method.body)
containing a `spacetimedb::http::Body`. `http::Response` is re-exported as `spacetimedb::http::Response`.

Set a timeout for a `ctx.http.send` request by including a `spacetimedb::http::Timeout` as an [`extension`](https://docs.rs/http/latest/http/request/struct.Builder.html#method.extension):

```
#[spacetimedb::procedure]
fn get_request_with_short_timeout(ctx: &mut spacetimedb::ProcedureContext) {
    let request = spacetimedb::http::Request::builder()
        .uri("https://example.invalid")
        .method("GET")
        // Set a timeout of 10 ms.
        .extension(spacetimedb::http::Timeout(std::time::Duration::from_millis(10).into()))
        // Empty body for a `GET` request.
        .body(())
        .expect("Building `Request` object failed");
    ctx.http.send(request).expect("HTTP request failed");
}
```

Procedures can't send requests at the same time as holding open a [transaction](#accessing-the-database).

## Calling Reducers from Procedures

Procedures can call reducers by invoking them within a transaction block. The reducer function runs within the transaction context:

```
#[spacetimedb::reducer]
fn process_item(ctx: &ReducerContext, item_id: u64) {
    // ... reducer logic
}

#[spacetimedb::procedure]
fn fetch_and_process(ctx: &mut ProcedureContext, url: String) -> Result<(), String> {
    // Fetch external data
    let response = ctx.http.get(&url).map_err(|e| format!("{e:?}"))?;
    let (_, body) = response.into_parts();
    let item_id: u64 = parse_id(&body.into_string_lossy());

    // Call the reducer within a transaction
    ctx.with_tx(|tx_ctx| {
        process_item(tx_ctx, item_id);
    });

    Ok(())
}
```

> **note**
>
> When you call a reducer function inside `withTx`, it executes as part of the same transaction, not as a subtransaction. The reducer's logic runs inline within your anonymous transaction block, just like calling any other helper function.

This pattern is useful when you need to:

- Fetch external data and then process it transactionally
- Reuse existing reducer logic from a procedure
- Combine side effects (HTTP) with database operations

## Example: Calling an External AI API

A common use case for procedures is integrating with external APIs like OpenAI's ChatGPT. Here's a complete example showing how to build an AI-powered chat feature.

```
use spacetimedb::{procedure, table, Identity, ProcedureContext, Table, TimeDuration, Timestamp};

#[table(accessor = ai_message, public)]
pub struct AiMessage {
    user: Identity,
    prompt: String,
    response: String,
    created_at: Timestamp,
}

#[derive(serde::Deserialize)]
struct AiResponse {
    choices: Vec<AiResponseChoice>,
    // more fields...
}

#[derive(serde::Deserialize)]
struct AiResponseChoice {
    message: AiResponseMessage,
    // more fields...
}

#[derive(serde::Deserialize)]
struct AiResponseMessage {
    content: String,
    // more fields...
}

#[procedure]
pub fn ask_ai(ctx: &mut ProcedureContext, prompt: String, api_key: String) -> Result<String, String> {
    // Build the request to OpenAI's API
    let request_body = serde_json::json!({
        "model": "gpt-4",
        "messages": [{ "role": "user", "content": prompt }]
    });

    let request = spacetimedb::http::Request::builder()
        .uri("https://api.openai.com/v1/chat/completions")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        // Give it some time to think
        .extension(spacetimedb::http::Timeout(TimeDuration::from_micros(3_000_000)))
        .body(serde_json::to_vec(&request_body).unwrap())
        .map_err(|e| format!("Failed to build request: {e}"))?;

    // Make the HTTP request
    let response = ctx.http.send(request)
        .map_err(|e| format!("HTTP request failed: {e:?}"))?;

    let (parts, body) = response.into_parts();

    if parts.status != 200 {
        return Err(format!("API returned status {}", parts.status));
    }

    let body = body.into_bytes();
    let ai_response: AiResponse =
        serde_json::from_slice(&body).map_err(|e| format!("Failed to parse AI response: {e}"))?;
    let ai_response = ai_response.choices[0].message.content.clone();

    // Store the conversation in the database
    ctx.with_tx(|tx_ctx| {
        tx_ctx.db.ai_message().insert(AiMessage {
            user: tx_ctx.sender(),
            prompt: prompt.clone(),
            response: ai_response.clone(),
            created_at: tx_ctx.timestamp,
        });
    });

    Ok(ai_response)
}
```
