<!-- markdownlint-disable MD024 -->
# SpacetimeDB Features

> INFO: This document should be created by writing the deduplicated content of [SpacetimeDB-Raw.md](SpacetimeDB-Raw.md) in a more structured and organized way.

## General

- [ ] Target real-time workloads (games, chat, collaboration)

## Table Declarations

### Visibility & Type

- [ ] `#[table(private)]`
- [ ] `#[table(public)]`
- [ ] `#[table(event)]`

### Accessor

- [ ] `#[table(accessor = singular_noun)]`

### Multi-Table Structs

#### Usage

- [ ] Apply multiple `#[table]` attributes on the same struct type
- [ ] Move rows from one table to another using `ctx.db().online_player().id().delete(p.id())` followed by `ctx.db().offline_player().insert(p)`
- [ ] Apply column attributes (`#[primary_key]`, `#[unique]`, `#[auto_inc]`, `#[index]`) to a multi-table struct where each table enforces its own independent constraints and sequences
- [ ] Add a table-level index `#[table(index(accessor = ..., btree(columns = [...])))]` to allow one table having an index which the other table doesn't have, even though they share the same struct definition

### Scheduled Tables

#### Declaration

- [ ] Annotate a struct with `#[table(accessor = ..., scheduled(function_name))]` to create a schedule table
- [ ] Define a `ScheduleAt` column in the schedule table struct — required column specifying when to fire
- [ ] Define the scheduled reducer with `#[spacetimedb::reducer]` accepting `(&ReducerContext, RowType)` — receives the full row as the second argument
- [ ] Reference a procedure name in the `scheduled(...)` attribute to schedule a procedure instead of a reducer

#### Timing Variants

- [ ] At intervals - Execute repeatedly at fixed time intervals (e.g., every 5 seconds) for periodic tasks like game ticks, heartbeats, or recurring maintenance (they are re-inserted by SpacetimeDB after each execution since they are repeating)
- [ ] At specific times - Execute once at an absolute timestamp for one-shot actions like sending a reminder at a particular moment or expiring content (they are deleted by SpacetimeDB when they run automatically since they are one-shot)

#### Triggering

- [ ] Insert a row with a `ScheduleAt` value into the schedule table to schedule execution
- [ ] Insert a row into a schedule table from a reducer to trigger a procedure indirectly
- [ ] Use `ScheduleAt::Time(ctx.timestamp + Duration::from_secs(N))` to schedule one-shot execution at an absolute timestamp offset from now
- [ ] Use `ScheduleAt::Interval(Duration::from_secs(N).into())` to schedule repeating execution at a fixed interval in seconds
- [ ] Use `ScheduleAt::Interval(Duration::from_millis(N).into())` to schedule repeating execution at a fixed interval in milliseconds
- [ ] Use `ScheduleAt::Interval(Duration::ZERO.into())` to fire a reducer or procedure immediately after the transaction commits

### Event Tables

#### Declaration

- [ ] Annotate a table with `#[table(accessor = ..., event, public)]` to create an event table
- [ ] Use all the same column types, constraints and indexes as regular tables in an event table

#### Publishing Events

- [ ] Insert a row into an event table from a reducer or procedure transaction to publish an event — rows are broadcasted to subscribed clients on commit and then deleted automatically
- [ ] Publish the same event type from multiple reducers — clients receive the same event regardless of the trigger

#### Row-Level Security

- [ ] Apply row-level security rules to event tables to control which clients receive which events based on their identity

## Column Types

### Primitives

- [ ] Use smallest fitting signed integer types `i8`, `i16`, `i32`, `i64`, `i128` as column types
- [ ] Use smallest fitting unsigned integer types `u8`, `u16`, `u32`, `u64`, `u128` as column types
- [ ] Use float types `f32`, `f64` as column types
- [ ] Use `bool` as column type
- [ ] Use `String` as column type

### Collections & Optionals

- [ ] Use `Vec<T>` as a column type for collection data when items form an atomic unit always read and written together, order is important, the collection is small and bounded, and items lack independent identity
- [ ] Use `Option<T>` as a column type for nullable fields

### SpacetimeDB Types

- [ ] Use SpacetimeDB identity types `Identity`, `ConnectionId` as column types
- [ ] Use SpacetimeDB time types `Timestamp`, `TimeDuration`, `ScheduleAt` as column types

### Custom Types

- [ ] Derive `SpacetimeType` on a struct to use it as a column type
- [ ] Derive `SpacetimeType` on an enum to use it as a column type
- [ ] Use any type deriving `SpacetimeType` as a column type
- [ ] Use `SpacetimeType`-derived types as reducer arguments or view results

## Indexes

### Primary Keys

- [ ] `#[primary_key]` with `#[auto_inc]` on `u64` column
- [ ] `#[primary_key]` with `#[index(direct)]` on `u64` column
- [ ] `#[primary_key]` with `Identity` type
- [ ] Table without `#[primary_key]` so the entire row acts as identity with set semantics — duplicate rows can't exist
- [ ] Simulate multi-column primary keys using a multi-column btree index for lookups combined with an `#[auto_inc]` primary key — e.g., `index(accessor = inventory_index, btree(columns = [user_id, item_id]))`

### Unique Constraints

- [ ] `#[unique]` with `#[auto_inc]`
- [ ] `#[unique]` without `#[auto_inc]`
- [ ] Multiple `#[unique]` columns in the same table

### B-Tree

- [ ] Single-Column B-Tree index with O(log n) lookups: `#[index(btree)]`
- [ ] Multi-Column B-Tree index with O(log n) lookups: `#[index(accessor = {{multi-column-index-accessor}}, btree(columns = [{{column-names}}]))]`

### Hash

- [ ] `#[index(hash)]`

### Direct

- [ ] Single-Column Direct index with O(1) lookups: `#[index(direct)]`

## Row Operations

### Prerequisites

- [ ] Import `spacetimedb::Table` trait to enable table method compilation

### Insert

- [ ] `ctx.db.{{table-accessor}}().try_insert()`
- [ ] Insert a row with `#[auto_inc]` column value `0` to trigger auto-assignment of the next value

### Read

- [ ] `ctx.db.{{table-accessor}}().iter()`
- [ ] `ctx.db.{{table-accessor}}().{{single-column-unique-index-accessor}}().find()`
- [ ] Handle the `Option<Row>` return type of `.find()` with `if let Some(...)`
- [ ] `ctx.db.{{table-accessor}}().{{single-column-index-accessor}}().filter()`
- [ ] `ctx.db.{{table-accessor}}().{{multi-column-index-accessor}}().filter()`

### Count

- [ ] `ctx.db.{{table-accessor}}().count()`

### Update

- [ ] `ctx.db.{{table-accessor}}().{{primary-key-accessor}}().update()`
- [ ] Use Rust struct update syntax (`..existing`) in `.update()` to copy unchanged fields from the existing row

### Delete

- [ ] `ctx.db.{{table-accessor}}().{{primary-key-accessor}}().delete()`
- [ ] `ctx.db.{{table-accessor}}().{{single-column-unique-index-accessor}}().delete()`
- [ ] `ctx.db.{{table-accessor}}().{{single-column-index-accessor}}().delete()`
- [ ] `ctx.db.{{table-accessor}}().{{multi-column-index-accessor}}().delete()`

### Iteration

- [ ] Iterate the return value of `ctx.db.{{table-accessor}}().iter()` with a `for` loop to iterate over every row via full table scan
- [ ] Iterate the return value of `ctx.db.{{table-accessor}}().{{single-column-index-accessor}}.filter()` with a `for` loop to iterate over every row via filtered table scan
- [ ] Iterate the return value of `ctx.db.{{table-accessor}}().{{multi-column-index-accessor}}.filter()` with a `for` loop to iterate over every row via filtered table scan

## Reducers

### Declaration

- [ ] `#[reducer]` with `&ReducerContext`
- [ ] Return `Result<(), String>` to commit on `Ok(())` in reducers (which don't allow returning values in `Ok`)
- [ ] Return `Err(String)` from a reducer to roll back the entire transaction

### Context Properties

- [ ] Access read-write table accessors via `ctx.db` of type `Local`
- [ ] Retrieve the caller's identity via `ctx.sender()` returning `Identity` - a 32-byte, globally valid, long-lived public identifier for a database user (or the database itself for system-invoked functions)
- [ ] Retrieve the module's own identity via `ctx.identity()` returning `Identity`
- [ ] Retrieve the client connection ID via `ctx.connection_id()` returning `Option<ConnectionId>` — returns `None` for system-invoked (scheduled functions and lifecycle reducers) functions
- [ ] Read the reducer invocation time via `ctx.timestamp` of type `Timestamp`
- [ ] Access the authorization context via `ctx.sender_auth()` returning `&AuthCtx` — contains JWT claims and internal call detection
- [ ] Obtain a deterministic RNG via `ctx.rng()` returning `&StdbRng` for generating multiple random values
- [ ] Generate a single random value via `ctx.random::<numeric-type>()` returning `numeric-type`

### Error Handling

- [ ] Return descriptive error messages via `Result<(), String>` to handle errors gracefully

### Composition

- [ ] Call a child reducer directly by passing the parent's `ctx` as an argument. Propagate the child reducer error with `?` to roll back the entire parent transaction.

### Lifecycle Reducers

- [ ] Annotate a reducer with `#[reducer(init)]` to run on first module publish or database clear to perform initial setup such as seeding tables with default data
- [ ] Annotate a reducer with `#[reducer(client_connected)]` to run when a client establishes a connection
- [ ] Return `Err` from `client_connected` to reject the client and disconnect them immediately — implement allowlists, banlists, or capacity limits
- [ ] Annotate a reducer with `#[reducer(client_disconnected)]` to run when a client disconnects

## Identity & Authentication

- [ ] Derive `Identity` from OIDC provider tokens so the same user always resolves to the same `Identity`
- [ ] Use `SpacetimeAuth` as a managed OIDC provider
- [ ] Use any third-party OIDC-compliant provider for authentication

## Procedures

### Declaration

- [ ] `#[procedure]` with `ctx: &mut ProcedureContext` and multiple `ctx.try_with_tx(|ctx| { ... })` (use `&TxContext` like `&ReducerContext`, it transparently wraps it)
- [ ] Return a value from a procedure in `Ok` (`Result<impl SpacetimeType, String>`) to send it only to the calling client

### Transaction Control

- [ ] Return `Ok(())` from `try_with_tx` to commit the transaction
- [ ] Return `Err(...)` from `try_with_tx` to roll back the transaction
- [ ] Capture the return value of `try_with_tx` in the calling procedure code

### Use Cases

- [ ] Use a procedure to upload files to s3 and insert metadata into the database in a transaction
- [ ] Implement scheduled tables calling a cleanup procedure to periodically archive old data into s3

### HTTP Client

#### Simple Requests

- [ ] Perform a simple GET request with `ctx.http.get(url)`
- [ ] Use `spacetimedb::http::Request` and `spacetimedb::http::Response` as re-exported types

#### Request Builder

- [ ] Build a custom HTTP request with `spacetimedb::http::Request::builder()`
- [ ] Set the URI with `.uri(...)` on the request builder
- [ ] Set the HTTP method with `.method("POST")` on the request builder
- [ ] Set a header with `.header("Content-Type", "application/json")` on the request builder
- [ ] Attach a string body with `.body("...")` on the request builder (serde_json can be used to serialize JSON strings)
- [ ] Pass an empty body with `.body(())` when no request body is needed
- [ ] Finalize the request builder with `.expect(...)` to unwrap the built `Request`
- [ ] Send a custom request with `ctx.http.send(request)`

#### Response Handling

- [ ] Decompose the response with `response.into_parts()` into response metadata and body
- [ ] Convert the response body to a string with `body.into_string_lossy()`
- [ ] Convert the response body to bytes with `body.into_bytes()`

#### Configuration

- [ ] Set a request timeout with `.extension(spacetimedb::http::Timeout(duration))` on the request builder
- [ ] Construct the timeout duration with `std::time::Duration::from_millis(...)` converted via `.into()`

## Views

### Declaration

- [ ] Declare a view with `#[spacetimedb::view(accessor = <name>, public)]` on a function
- [ ] Accept `&ViewContext` as the sole parameter when the view result depends on the caller via `ctx.sender()` to materialize separately per subscriber — O(N) scaling
- [ ] Accept `&AnonymousViewContext` as the sole parameter when the view result is identical for all callers and materialize the view once for all subscribers — O(1) scaling

### Return Types

- [ ] Return `Option<T>` from a view to represent at-most-one row
- [ ] Return `Vec<T>` from a view to return multiple rows procedurally
- [ ] Return `impl Query<T>` instead of `Vec<T>` to enable query engine optimizations
- [ ] Use a table type as `T` in the view return type
- [ ] Use a custom product type derived with `#[derive(SpacetimeType)]` as `T` in the view return type

### Table & Row Access

- [ ] Access tables read-only via `ctx.db.<table>()` inside a view function
- [ ] Read from private tables inside a view to implement Row-Level Security (RLS)
- [ ] Combine `.sender().filter()` and `.recipient().filter()` with `.chain()` to union multiple filtered sets
- [ ] Event tables cannot be accessed within view functions (deferred to a future release)

### Query Builder

#### Filters

- [ ] Access the query builder API via `ctx.from` on both `ViewContext` and `AnonymousViewContext`
- [ ] Access a table in the query builder via `ctx.from.<table>()`
- [ ] Apply a filter with `.filter(|row| ...)` using a closure returning a boolean condition
- [ ] Chain multiple `.filter()` calls to combine filters with logical AND

#### Comparison Operators

- [ ] Use `.eq()` for equal comparison
- [ ] Use `.ne()` for not-equal comparison
- [ ] Use `.lt()` for less-than comparison
- [ ] Use `.lte()` for less-than-or-equal comparison
- [ ] Use `.gt()` for greater-than comparison
- [ ] Use `.gte()` for greater-than-or-equal comparison

#### Boolean Combinators

- [ ] Combine conditions with `.and()` chained on a comparison result inside a `.filter()` closure
- [ ] Combine conditions with `.or()` chained on a comparison result inside a `.filter()` closure
- [ ] Negate a condition with `.not()` chained on a comparison result inside a `.filter()` closure

#### Semijoins

- [ ] Use `.left_semijoin(ctx.from.<table>(), |left, right| <predicate>)` to return source rows with at least one match
- [ ] Use `.right_semijoin(ctx.from.<table>(), |left, right| <predicate>)` to return right-side rows with at least one match
- [ ] Apply `.filter()` before a semijoin to filter the source side
- [ ] Apply `.filter()` after a semijoin to filter the returned side

### Column Projection

- [ ] Define a custom `#[derive(SpacetimeType)]` struct that omits sensitive columns
- [ ] Map a full table row to the projection struct inside the view to exclude sensitive fields
- [ ] Return `Option<ProjectionType>` from a view for single-row column projection

## Logging

- [ ] Call `log::error!` to log errors that prevent operations from completing
- [ ] Call `log::warn!` to log problematic situations that do not prevent execution
- [ ] Call `log::info!` to log important application events such as user actions and state changes
- [ ] Call `log::debug!` to log detailed diagnostic information for development
- [ ] Call `log::trace!` to log very detailed diagnostics — typically disabled in production

## Data Management

- [ ] Limit data processed per operation using pagination for large result sets
- [ ] Store binary data in `Vec<u8>` columns with metadata columns for small or changing files
- [ ] Store storage URLs in `String` columns with metadata columns referencing external storage for large or not changing files

## Best Practices

- [ ] Never use `static mut` variables, always store all state in tables
- [ ] Use a separate table instead of a `Vec<T>` column for related data when items have independent identity and lifecycle, need individual querying or indexing, can grow unbounded, or require per-item subscription updates
- [ ] Use scheduled reducers or procedures instead of direct calls to achieve independent transactions
