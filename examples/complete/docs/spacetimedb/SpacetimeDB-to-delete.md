# SpacetimeDB

## Tables

> Source: `SpacetimeDB.md`, lines 118-683

### Table Declaration

- [ ] Declare a table with `#[spacetimedb::table(accessor = ..., public)]` macro on a `pub struct`
- [ ] Declare a table with `#[spacetimedb::table(accessor = ..., private)]` macro on a `pub struct`
- [ ] Specify the `accessor` attribute to name the handle for code access via `ctx.db.<accessor>()`
- [ ] Mark a column with `#[primary_key]` to uniquely identify each row
- [ ] Mark a column with `#[auto_inc]` to auto-assign incrementing values
- [ ] Mark a column with `#[unique]` to enforce no duplicate values

### Table Visibility

- [ ] Set table visibility to `private` (default) to restrict client access entirely
- [ ] Set table visibility to `public` to allow all clients read access
- [ ] Expose computed subsets of private table data using views -- for fine-grained access control with row filtering, column selection, and joins

### Table Naming and Accessor Conventions

- [ ] Name the `accessor` attribute value in `lower_snake_case`
- [ ] Access the generated table handle via `ctx.db.<accessor>()` for insert, find, update, delete, filter, iter, and count operations

### Multiple Tables for the Same Type

- [ ] Apply multiple `#[spacetimedb::table(accessor = ...)]` attributes to a single struct to create independent tables sharing the same schema
- [ ] Delete a row from one table and insert it into another to move rows between tables sharing the same struct -- e.g., `ctx.db.player().identity().delete(caller)` followed by `ctx.db.logged_out_player().insert(p)`

#### Shared Constraints Across Multi-Table Types

- [ ] Apply column attributes (`#[primary_key]`, `#[unique]`, `#[auto_inc]`, `#[index]`) to a multi-table struct where each table enforces its own independent constraints and sequences

### Table Decomposition

- [ ] Organize tables by access pattern rather than by entity -- joins reduce to in-memory index lookups
- [ ] Split tables to reduce bandwidth so clients subscribing to high-frequency data do not receive low-frequency updates
- [ ] Split tables to improve cache efficiency by co-locating data with similar update frequencies

### Supported Column Types

- [ ] Use integer types `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128` as column types
- [ ] Use float types `f32`, `f64` as column types
- [ ] Use `bool` and `String` as column types
- [ ] Use `Vec<T>` as a column type for collection data
- [ ] Use `Option<T>` as a column type for nullable fields
- [ ] Use SpacetimeDB-specific types `Identity`, `ConnectionId`, `Timestamp`, `TimeDuration`, `ScheduleAt` as column types
- [ ] Use any type deriving `SpacetimeType` as a column type

#### Custom Types with `SpacetimeType`

- [ ] Derive `SpacetimeType` on a struct to use it as a column type -- e.g., `#[derive(SpacetimeType)] pub struct Coordinates { x: f64, y: f64, z: f64 }`
- [ ] Derive `SpacetimeType` on an enum to use it as a column type -- e.g., `#[derive(SpacetimeType)] pub enum Status { Active, Inactive, Suspended { reason: String } }`
- [ ] Use `SpacetimeType`-derived types as reducer arguments or view results

#### Optional Columns

- [ ] Wrap a column type in `Option<T>` for nullable fields where `None` indicates no value

#### Column Type Performance

- [ ] Use the smallest fitting integer type to reduce memory and bandwidth -- e.g., `u8` for 0-255 instead of `u64`
- [ ] Prefer fixed-size types (`u32`, `f64`) over variable-size types for direct offset computation
- [ ] Use `[u8; 32]` instead of `Vec<u8>` for fixed-length binary data
- [ ] Use enums instead of `String` for categorical data
- [ ] Order columns largest-to-smallest alignment to reduce padding -- e.g., `(u64, u8, u8)` = 16 bytes vs. `(u8, u64, u8)` = 24 bytes

### Primary Key

- [ ] Annotate a column with `#[primary_key]` to create a unique index for efficient lookups
- [ ] Limit to at most one `#[primary_key]` per table

#### Common Primary Key Patterns

- [ ] Combine `#[primary_key]` with `#[auto_inc]` on a `u64` column for auto-generated unique row IDs
- [ ] Use `Identity` type with `#[primary_key]` to ensure one row per user identity -- ideal for per-user data

#### Multi-Column Primary Keys

- [ ] Simulate multi-column primary keys using a multi-column btree index for lookups combined with an `#[auto_inc]` primary key -- e.g., `index(accessor = inventory_index, btree(columns = [user_id, item_id]))`

#### Update Behavior with Primary Keys

- [ ] Insert a row with the same primary key value to trigger an in-place update -- subscribers see an update event
- [ ] Insert a row with a different primary key value to trigger delete + insert -- subscribers see delete + insert events

#### Tables Without Primary Keys

- [ ] Omit `#[primary_key]` so the entire row acts as the identity with set semantics -- duplicate rows cannot exist

### Unique Constraint

- [ ] Annotate a column with `#[unique]` to enforce no duplicate values and automatically create an index
- [ ] Apply multiple `#[unique]` attributes to different columns in the same table

### Auto-Increment

- [ ] Annotate an integer column with `#[auto_inc]` to auto-assign incrementing values
- [ ] Insert a row with `#[auto_inc]` column value `0` to trigger auto-assignment of the next value
- [ ] Insert a row with a non-zero `#[auto_inc]` column value to use it as-is -- useful for data migration
- [ ] Read the returned row from `insert()` to obtain the auto-assigned value

#### Auto-Increment Attribute Combinations

- [ ] Combine `#[auto_inc]` with `#[primary_key]` for auto-generated unique row IDs
- [ ] Combine `#[auto_inc]` with `#[unique]` for auto-generated unique values on a non-primary-key column

#### Sequences

- [ ] Configure sequence parameters `start`, `min_value`, `max_value`, `increment` for auto-increment columns -- inspired by PostgreSQL sequences
- [ ] Use negative `increment` for descending sequences

#### Sequence Wrapping Behavior

- [ ] Allow a sequence to wrap from `max_value` to `min_value` and continue cycling when it reaches its maximum

#### Sequence Crash Recovery

- [ ] Rely on batch allocation of 4096 values for sequence crash recovery -- sequences resume from the next allocation boundary after restart

#### Auto-Increment Sequence Gaps

- [ ] Accept non-transactional sequence behavior where rolled-back transactions still consume sequence numbers
- [ ] Maintain an explicit counter in a separate table for strictly sequential numbering -- increment it transactionally

### Default Values

- [ ] Annotate a column with `#[default(0)]` to set a default integer value
- [ ] Annotate a column with `#[default(true)]` or `#[default(false)]` to set a default boolean value
- [ ] Add new columns with `#[default(...)]` at the end of the struct for schema migration -- existing rows auto-populated with the default
- [ ] Use only const-evaluable expressions in `#[default(...)]`

#### Default Value Attribute Restrictions

- [ ] Avoid combining `#[default(...)]` with `#[primary_key]`, `#[unique]`, or `#[auto_inc]` -- these attributes conflict with a static default

### Event Tables

- [ ] Declare an event table with `#[spacetimedb::table(accessor = ..., public, event)]` to create a table whose rows exist only for the duration of the inserting transaction
- [ ] Insert rows into an event table from a reducer to broadcast them to subscribed clients on commit
- [ ] Use all standard column types, constraints, indexes, and auto-increment on event tables

#### Publishing Events

- [ ] Insert a row into an event table via `ctx.db.<accessor>().insert(...)` from any reducer -- broadcast occurs on transaction commit
- [ ] Rely on rollback semantics where no events are sent if the transaction rolls back

#### Event Table Constraints and Indexes

- [ ] Apply `#[primary_key]`, `#[unique]`, indexes, and `#[auto_inc]` to event tables where constraints are enforced only within a single transaction and reset between transactions

#### Event Table Row-Level Security

- [ ] Apply row-level security (RLS) to event tables to control which clients receive which events based on identity

### Table Growth Management

- [ ] Implement cleanup reducers to periodically remove stale or temporary data
- [ ] Use schedule tables to trigger deletion of aged rows -- scheduled expiration
- [ ] Delete or move old records for archiving
- [ ] Limit data processed per operation using pagination for large result sets

### Collection Column vs. Separate Table

- [ ] Use a `Vec<T>` column when items form an atomic unit always read and written together, order is important, the collection is small and bounded, and items lack independent identity
- [ ] Use a separate table when items have independent identity and lifecycle, need individual querying or indexing, can grow unbounded, or require per-item subscription updates

### Binary Data Storage

- [ ] Store binary data in `Vec<u8>` columns for atomic updates with metadata and auto-broadcast to subscribers

#### Inline Binary Storage

- [ ] Declare a `Vec<u8>` column for inline binary storage -- recommended for files up to ~100MB that change together with other row fields

#### External Storage with References

- [ ] Store a `storage_url: String` column referencing external storage for files over 100MB -- keep only metadata and URL in SpacetimeDB

#### External Storage Upload Flow

- [ ] Call a reducer to obtain a pre-signed upload URL for direct client-to-storage transfer
- [ ] Register uploaded file metadata by calling a reducer with the storage URL and metadata after upload completes

#### Hybrid Storage Strategy

- [ ] Store a small `thumbnail: Vec<u8>` inline and an `original_url: String` referencing external storage -- thumbnails arrive via subscriptions, originals fetched on demand

#### File Storage Strategy Selection

- [ ] Use inline `Vec<u8>` for avatars (<10MB), attachments (<50MB), and documents (<100MB)
- [ ] Use external storage with a DB reference for large files (>100MB) and video
- [ ] Use hybrid inline thumbnail plus external original for images needing previews
- [ ] Use external storage with CDN for static assets needing content delivery

## Row Operations

> Source: `SpacetimeDB.md`, lines 684-817

### Table Trait and Row Operations

- [ ] Import `spacetimedb::Table` trait to enable table method compilation
- [ ] Access a table via `ctx.db.<accessor>()` using the name defined in `#[table(accessor = ...)]`
- [ ] Call `.insert(...)` on a table accessor to add a row
- [ ] Call `.try_insert(...)` on a table accessor to add a row and return a `Result`
- [ ] Call `.<column>().find(...)` to look up a row by unique or primary key column
- [ ] Call `.<column>().update(...)` to update a row by primary key column
- [ ] Call `.<column>().delete(...)` to delete rows by indexed column
- [ ] Call `.<column>().filter(...)` to filter rows by indexed column and return an iterator
- [ ] Call `.iter()` on a table accessor to iterate all rows via full table scan
- [ ] Call `.count()` on a table accessor to get the row count without iteration

### Row Insert

- [ ] Call `ctx.db.<accessor>().insert(StructName { ... })` to insert a row within the current transaction
- [ ] Pass `0` for an `#[auto_inc]` column in `.insert()` to let auto-increment assign the actual value

### Row Lookup by Unique Column

- [ ] Call `ctx.db.<accessor>().<column>().find(<value>)` to look up a row by primary key
- [ ] Call `ctx.db.<accessor>().<column>().find(<value>)` to look up a row by `#[unique]` column
- [ ] Handle the `Option<Row>` return type of `.find()` with `if let Some(...)`

### Row Update

- [ ] Call `ctx.db.<accessor>().<column>().update(StructName { ... })` to update a row by primary key column
- [ ] Use Rust struct update syntax (`..existing`) in `.update()` to copy unchanged fields from the existing row

### Row Delete

- [ ] Call `ctx.db.<accessor>().<column>().delete(<value>)` on a unique or primary key column to delete a single row
- [ ] Call `ctx.db.<accessor>().<column>().delete(<value>)` on a non-unique indexed column to delete all matching rows and return a count
- [ ] Call `.delete(..18)` with an exclusive upper-bound range expression for bulk deletion
- [ ] Call `.delete(18..)` with a lower-bound range expression for bulk deletion
- [ ] Call `.delete(18..=65)` with an inclusive range expression for bulk deletion

### Row Filter

- [ ] Call `ctx.db.<accessor>().<column>().filter(<value>)` with an exact value for exact-match filtering
- [ ] Call `.filter(18..=65)` with an inclusive range expression
- [ ] Call `.filter(18..)` with a lower-bound range expression
- [ ] Call `.filter(..18)` with an exclusive upper-bound range expression
- [ ] Call `.filter((<exact>, <range>))` on a multi-column index with prefix columns exact and trailing column as a range
- [ ] Iterate the return value of `.filter()` with a `for` loop

### Row Iteration

- [ ] Call `ctx.db.<accessor>().iter()` to iterate every row via full table scan
- [ ] Iterate the return value of `.iter()` with a `for` loop

### Row Count

- [ ] Call `ctx.db.<accessor>().count()` to get the total row count without iterating

### Batch Operations

- [ ] Annotate a function with `#[reducer]` to batch multiple row operations in a single transaction
- [ ] Call `.insert()` in a loop inside a single reducer to batch inserts without per-row transaction overhead

## Indexes

> Source: `SpacetimeDB.md`, lines 818-962

### Index Declaration

- [ ] Declare a field-level B-tree index with `#[index(btree)]` on a struct field
- [ ] Declare a table-level index with `index(accessor = <name>, btree(columns = [<col>]))` inside `#[spacetimedb::table(...)]`
- [ ] Assign a named accessor to a table-level index via the `accessor` parameter

### When to Use Indexes

- [ ] Apply `#[index(btree)]` to foreign key columns used in filtered lookups
- [ ] Apply `#[index(btree)]` to columns used in range queries
- [ ] Apply `#[index(btree)]` to columns used for sorting
- [ ] Use indexed lookup via `ctx.db.<table>().<index_name>().filter(...)` for O(log n) access
- [ ] Avoid `ctx.db.<table>().iter().find(|row| ...)` full table scans when an index exists

### B-tree Indexes

- [ ] Use `#[index(btree)]` to create a B-tree index -- default index type, maintains sorted order
- [ ] Perform equality lookups with a B-tree index
- [ ] Perform range queries with a B-tree index
- [ ] Perform prefix matching on multi-column B-tree indexes

### Direct Indexes

- [ ] Declare a direct index with `#[index(direct)]` on a single unsigned integer column
- [ ] Use `#[index(direct)]` with `u8`, `u16`, `u32`, or `u64` column types for O(1) lookups
- [ ] Combine `#[primary_key]` and `#[index(direct)]` on the same field

### Single-Column Index Syntax

- [ ] Use field-level syntax `#[index(btree)]` directly above the field declaration
- [ ] Use table-level syntax `index(accessor = idx_age, btree(columns = [age]))` in `#[spacetimedb::table(...)]`

### Multi-Column Indexes

- [ ] Declare a multi-column index with `btree(columns = [<col1>, <col2>])` in `#[spacetimedb::table(...)]`
- [ ] Query a multi-column index with full match using a tuple `filter((<val1>, <val2>))`
- [ ] Query a multi-column index with prefix match using a single value `filter(&<val>)`
- [ ] Query a multi-column index with equality on leading column plus range on trailing column using `filter((<val>, <range>))`

### Index Query Methods

- [ ] Filter by equality using `ctx.db.<table>().<index>().filter(<value>)`
- [ ] Filter by inclusive range using `.filter(<start>..=<end>)`
- [ ] Filter by lower-bounded range using `.filter(<start>..)`
- [ ] Filter by upper-bounded range using `.filter(..<end>)`
- [ ] Filter a multi-column index by prefix using `.filter(&<leading_value>)`
- [ ] Filter a multi-column index by leading equality plus trailing range using `.filter((<leading_value>, <start>..=<end>))`
- [ ] Filter a multi-column index by full tuple match using `.filter((<val1>, <val2>))`

### Index-Accelerated Deletion

- [ ] Delete rows by index equality using `ctx.db.<table>().<index>().delete(<value>)`
- [ ] Delete rows by index range using `ctx.db.<table>().<index>().delete(..<end>)`

### Index Design Guidelines

- [ ] Place the most selective column first in multi-column index column order
- [ ] Place range-queried columns after equality-queried columns in multi-column indexes
- [ ] Rely on `(a, b)` composite index to serve prefix queries on `(a)` instead of creating a redundant single-column index
- [ ] Create a separate index on `(b)` when it is queried independently of `(a)`

## Transactions

> Source: `SpacetimeDB.md`, lines 963-1029

### ACID Transaction Guarantees

- [ ] Execute every reducer invocation as a single ACID transaction
- [ ] Apply atomicity so all changes commit on reducer success or all changes roll back on reducer error
- [ ] Enforce consistency by checking all constraints (unique keys, indexes, module-enforced relationships) before commit
- [ ] Isolate each reducer so it sees a consistent database snapshot as of transaction start
- [ ] Persist committed changes to disk via durability guarantees that survive server restarts

### Transaction Scope

#### Reducer Automatic Transactions

- [ ] Use `#[reducer]` to run the entire reducer body as one automatic transaction
- [ ] Insert rows via `ctx.db.table_a().insert(...)` within the reducer's implicit transaction
- [ ] Call a child reducer function directly (e.g., `child_reducer(ctx)?`) to share the parent's transaction -- not a nested transaction
- [ ] Commit all changes automatically on `Ok(())` return
- [ ] Roll back all changes automatically on `Err(...)` return

#### Procedure Manual Transactions

- [ ] Annotate a function with `#[spacetimedb::procedure]` to define a procedure
- [ ] Accept `ctx: &mut ProcedureContext` as the procedure's context parameter
- [ ] Call `ctx.with_tx(|ctx| { ... })` to open a manual transaction and access the database
- [ ] Insert rows inside `ctx.with_tx` via `ctx.my_table().insert(...)`
- [ ] Open multiple separate `ctx.with_tx` calls within a single procedure to create independent transactions
- [ ] Perform I/O between separate `ctx.with_tx` transaction blocks

### Transaction Best Practices

- [ ] Keep transactions short by placing only necessary DB operations in reducers
- [ ] Move external I/O to procedures to reduce transaction contention
- [ ] Return descriptive error messages via `Result<(), String>` to handle errors gracefully

## Storage and Persistence

> Source: `SpacetimeDB.md`, lines 1030-1054

### In-Memory Storage with Commit Log Persistence

- [ ] Store all state in memory for sub-microsecond access
- [ ] Persist changes via a commit log (write-ahead log)
- [ ] Replay the commit log on restart to recover exact state
- [ ] Target real-time workloads (games, chat, collaboration) with in-memory storage
- [ ] Achieve 100-1,000x faster performance than traditional databases via in-memory design
- [ ] Leverage SSD write bandwidth (~15 GB/s) for durable commits with minimal throughput impact

### Row History and Time Travel

- [ ] Retain full row change history by default
- [ ] Perform time-traveling debugging to inspect exact state at any past point
- [ ] Delete history explicitly -- history is never silently discarded

### Hot-Swap Module Updates

- [ ] Hot-swap server code without disconnecting clients
- [ ] Store all state in tables rather than ephemeral memory to enable hot-swap
- [ ] Replace module logic while the database retains state and clients continue seamlessly

## State Synchronization

> Source: `SpacetimeDB.md`, lines 1055-1079

### State Mirroring

- [ ] Mirror database state to connected clients in real-time
- [ ] Define subscriptions via query builder to specify needed data
- [ ] Define subscriptions via raw SQL to specify needed data
- [ ] Push incremental updates from server whenever subscribed data changes
- [ ] Enforce read-only access on the client-side mirror -- clients cannot mutate the mirror directly
- [ ] Modify the database exclusively through reducer calls validated on the server

### Client-Side Data View

- [ ] Query the locally cached data view for all client-side reads -- reads never hit the server directly
- [ ] Keep the local cache in sync automatically via active subscriptions -- no polling required

### Client Code Generation

- [ ] Generate a typed client library from the database schema using the `spacetime` CLI
- [ ] Provide strongly-typed data structures matching table definitions -- no raw query results
- [ ] Provide interfaces for connecting to the database
- [ ] Provide interfaces for calling reducers
- [ ] Provide interfaces for receiving state updates

## Reducers

> Source: `SpacetimeDB.md`, lines 1080-1264

### Reducer Definition and Invocation

- [ ] Annotate a function with `#[spacetimedb::reducer]` to define a reducer
- [ ] Accept `&ReducerContext` as the mandatory first parameter of every reducer
- [ ] Accept additional serializable parameters after `&ReducerContext` for client-supplied arguments
- [ ] Return `()` from a reducer to commit the transaction unconditionally
- [ ] Return `Result<(), String>` from a reducer to commit on `Ok(())` or roll back on `Err`
- [ ] Return `Result<(), E: Display>` from a reducer using any error type that implements `Display`
- [ ] Insert a row via `ctx.db.user().insert(...)` inside a reducer

### ReducerContext

- [ ] Access read-write table accessors via `ctx.db` of type `Local`
- [ ] Retrieve the caller's identity via `ctx.sender()` returning `Identity`
- [ ] Read the reducer invocation time via `ctx.timestamp` of type `Timestamp`
- [ ] Retrieve the client connection ID via `ctx.connection_id()` returning `Option<ConnectionId>` -- returns `None` for system-invoked reducers
- [ ] Retrieve the module's own identity via `ctx.identity()` returning `Identity`
- [ ] Obtain a deterministic RNG via `ctx.rng()` returning `&StdbRng`
- [ ] Generate a single random value via `ctx.random::<T>()` returning `T`
- [ ] Access the authorization context via `ctx.sender_auth()` returning `&AuthCtx` -- contains JWT claims and internal call detection

### Nested Reducer Calls

- [ ] Call a child reducer directly by passing the parent's `ctx` as an argument
- [ ] Catch a child reducer error with `.is_err()` to allow the parent transaction to continue and commit
- [ ] Propagate a child reducer error with `?` to roll back the entire parent transaction
- [ ] Use scheduled reducers instead of direct calls to achieve independent transactions

### Reducer Isolation Constraints

- [ ] Perform only database reads and writes through `ReducerContext` inside reducers -- no network, filesystem, or system calls
- [ ] Use procedures instead of reducers for external I/O such as HTTP requests

### Global and Static Variable Prohibition

- [ ] Store all persistent state in tables instead of `static mut` variables
- [ ] Define a table with `#[spacetimedb::table(accessor = counter)]` to replace module-level state
- [ ] Define a `#[primary_key]` field on a state-tracking table struct

### Deterministic Random Number Generation

- [ ] Generate a deterministic `u32` via `ctx.random::<u32>()`
- [ ] Obtain `&StdbRng` via `ctx.rng()` for generating multiple random values
- [ ] Avoid external RNG crates such as `rand::thread_rng()` -- breaks consensus across nodes

### Reducer Error Categories

#### Sender Errors

- [ ] Return `Err("message".to_string())` to signal an expected failure from invalid client input
- [ ] Communicate the error message back to the client via `Result<(), String>`

#### Programmer Errors

- [ ] Use `assert!` to surface unexpected bugs as panics
- [ ] Use `.expect()` to surface unexpected bugs as panics
- [ ] Monitor programmer errors in the project dashboard and configure alerting

### Lifecycle Reducers

- [ ] Annotate a reducer with `#[reducer(init)]` to run on first module publish or database clear
- [ ] Annotate a reducer with `#[reducer(client_connected)]` to run when a client establishes a connection
- [ ] Annotate a reducer with `#[reducer(client_disconnected)]` to run when a client disconnects
- [ ] Accept only `&ReducerContext` as the parameter for lifecycle reducers -- no additional parameters

#### Init Lifecycle Reducer

- [ ] Define an `#[reducer(init)]` function returning `Result<(), String>`
- [ ] Seed default values via `ctx.db.settings().try_insert(...)` inside `init`
- [ ] Return `Err` from `init` to prevent the publish or database clear operation

#### Client Connected

- [ ] Define a `#[reducer(client_connected)]` function returning `Result<(), String>`
- [ ] Unwrap `ctx.connection_id().unwrap()` inside `client_connected` -- guaranteed `Some(...)`
- [ ] Insert a session record via `ctx.db.sessions().try_insert(...)` using `ctx.sender()` and `ctx.timestamp`
- [ ] Return `Err` from `client_connected` to reject the client and disconnect them immediately
- [ ] Implement allowlists, banlists, or capacity limits by returning `Err` from `client_connected`

#### Client Disconnected

- [ ] Define a `#[reducer(client_disconnected)]` function with no return type
- [ ] Unwrap `ctx.connection_id().unwrap()` inside `client_disconnected` -- guaranteed `Some(...)`
- [ ] Delete a session record via `ctx.db.sessions().connection_id().delete(&conn_id)` for cleanup

## Procedures

> Source: `SpacetimeDB.md`, lines 1265-1433

### Procedure Declaration and Context

- [ ] Enable procedures with `features = ["unstable"]` in `Cargo.toml`
- [ ] Annotate a function with `#[spacetimedb::procedure]` to declare a procedure
- [ ] Accept `&mut ProcedureContext` as the first parameter of a procedure
- [ ] Accept additional typed arguments that implement `SpacetimeType`
- [ ] Return a value from a procedure to send it only to the calling client
- [ ] Call a procedure from the client via `ctx.procedures.<name>_then(|ctx, res| { ... })` callback pattern
- [ ] Match on `Ok(value)` and `Err(e)` in the client-side procedure callback

### Procedure Manual Transactions

- [ ] Open a manual transaction with `ProcedureContext::with_tx(|ctx| { ... })`
- [ ] Access `ctx.db` table accessors inside `with_tx` via the `&TxContext` parameter
- [ ] Insert a row inside `with_tx` using `ctx.db.<table>().insert(...)` to commit on return
- [ ] Open multiple independent transactions by calling `with_tx` multiple times within a single procedure

#### Fallible Procedure Transactions

- [ ] Open a fallible transaction with `ProcedureContext::try_with_tx(|ctx| { ... })`
- [ ] Return `Ok(())` from `try_with_tx` to commit the transaction
- [ ] Return `Err(...)` from `try_with_tx` to roll back the transaction
- [ ] Perform conditional validation inside `try_with_tx` and return `Err` on failure

#### Transaction Return Values

- [ ] Capture the return value of `with_tx` in the calling procedure code
- [ ] Capture the return value of `try_with_tx` in the calling procedure code
- [ ] Use `ctx.db.<table>().iter()` inside `with_tx` to query data and return it outside the transaction
- [ ] Chain `.max_by_key()` on an iterator inside `with_tx` to compute aggregated results

### Procedure HTTP Requests

- [ ] Perform a simple GET request with `ctx.http.get(url)`
- [ ] Decompose the response with `response.into_parts()` into response metadata and body
- [ ] Convert the response body to a string with `body.into_string_lossy()`
- [ ] Convert the response body to bytes with `body.into_bytes()`
- [ ] Build a custom HTTP request with `spacetimedb::http::Request::builder()`
- [ ] Set the URI with `.uri(...)` on the request builder
- [ ] Set the HTTP method with `.method("POST")` on the request builder
- [ ] Set a header with `.header("Content-Type", "text/plain")` on the request builder
- [ ] Attach a string body with `.body("...")` on the request builder
- [ ] Finalize the request builder with `.expect(...)` to unwrap the built `Request`
- [ ] Send a custom request with `ctx.http.send(request)`
- [ ] Use `spacetimedb::http::Request` and `spacetimedb::http::Response` as re-exported types

#### HTTP Request Timeouts

- [ ] Set a request timeout with `.extension(spacetimedb::http::Timeout(duration))` on the request builder
- [ ] Construct the timeout duration with `std::time::Duration::from_millis(...)` converted via `.into()`
- [ ] Pass an empty body with `.body(())` when no request body is needed

#### HTTP and Transaction Exclusivity

- [ ] Alternate between HTTP request blocks and `with_tx`/`try_with_tx` blocks -- HTTP and transactions cannot be active simultaneously

### Calling Reducers from Procedures

- [ ] Call a `#[spacetimedb::reducer]` function directly inside `with_tx` to execute it within the same transaction
- [ ] Combine HTTP response parsing with a reducer call by performing HTTP outside `with_tx` and invoking the reducer inside `with_tx`
- [ ] Parse an HTTP response body outside the transaction and pass the result into `with_tx`

### Scheduling Procedures from Reducers

- [ ] Insert a row into a schedule table from a reducer to trigger a procedure indirectly
- [ ] Use `ScheduleAt::Interval(Duration::ZERO.into())` to fire a scheduled procedure immediately after the reducer's transaction commits

## Views

> Source: `SpacetimeDB.md`, lines 1434-1630

### View Declaration

- [ ] Declare a view with `#[spacetimedb::view(accessor = <name>, public)]` on a function
- [ ] Accept `&ViewContext` as the sole parameter to access caller identity via `ctx.sender()`
- [ ] Accept `&AnonymousViewContext` as the sole parameter for caller-independent views
- [ ] Access tables read-only via `ctx.db.<table>()` inside a view function
- [ ] Specify the `accessor` name in the `#[spacetimedb::view]` attribute to set the client-facing identifier
- [ ] Mark the view `public` in the `#[spacetimedb::view]` attribute

### View Return Types and Subscriptions

- [ ] Return `Option<T>` from a view to represent at-most-one row
- [ ] Return `Vec<T>` from a view to return multiple rows procedurally
- [ ] Return `impl Query<T>` from a view to use the query builder with incremental evaluation
- [ ] Use a table type as `T` in the view return type
- [ ] Use a custom product type derived with `#[derive(SpacetimeType)]` as `T` in the view return type
- [ ] Subscribe to a view via SQL using `SELECT * FROM <view_name>`

### ViewContext and AnonymousViewContext

- [ ] Use `ViewContext` when the view result depends on the caller via `ctx.sender()`
- [ ] Use `AnonymousViewContext` when the view result is identical for all callers
- [ ] Access read-only table data via `ctx.db` on both `ViewContext` and `AnonymousViewContext`
- [ ] Access the query builder API via `ctx.from` on both `ViewContext` and `AnonymousViewContext`

#### AnonymousViewContext Performance Advantage

- [ ] Prefer `AnonymousViewContext` to materialize the view once for all subscribers -- O(1) scaling
- [ ] Note `ViewContext` materializes separately per subscriber -- O(N) scaling

### View Query Builder

- [ ] Access a table in the query builder via `ctx.from.<table>()`
- [ ] Apply a filter with `.r#where(|row| ...)` using a closure returning a boolean condition
- [ ] Apply a filter with `.filter(...)` as an alias for `.r#where(...)`
- [ ] Chain multiple `.r#where()` calls to combine filters with logical AND
- [ ] Return `impl Query<T>` instead of `Vec<T>` to enable query engine optimizations

#### Query Builder Comparison Operators

- [ ] Use `.eq()` for equal comparison
- [ ] Use `.ne()` for not-equal comparison
- [ ] Use `.lt()` for less-than comparison
- [ ] Use `.lte()` for less-than-or-equal comparison
- [ ] Use `.gt()` for greater-than comparison
- [ ] Use `.gte()` for greater-than-or-equal comparison

#### Query Builder Boolean Combinators

- [ ] Combine conditions with `.and()` chained on a comparison result inside a `.r#where()` closure
- [ ] Combine conditions with `.or()` chained on a comparison result inside a `.r#where()` closure
- [ ] Negate a condition with `.not()` chained on a comparison result inside a `.r#where()` closure

#### Query Builder Semijoins

- [ ] Use `.left_semijoin(ctx.from.<table>(), |left, right| <predicate>)` to return source rows with at least one match
- [ ] Use `.right_semijoin(ctx.from.<table>(), |left, right| <predicate>)` to return right-side rows with at least one match
- [ ] Apply `.r#where()` before a semijoin to filter the source side
- [ ] Apply `.r#where()` after a semijoin to filter the returned side
- [ ] Use `.eq()` on indexed columns inside the semijoin join predicate

### View Read Set and Invalidation

- [ ] Use `.find()` on indexed columns for targeted invalidation in procedural views
- [ ] Use `.filter()` on indexed columns for targeted invalidation in procedural views
- [ ] Return `impl Query<T>` to enable incremental evaluation even on full table scans -- avoids the `.iter()` prohibition in procedural views

### Fine-Grained Access Control with Views

- [ ] Read from private tables inside a public view to implement Row-Level Security (RLS)

#### Row Filtering by Caller Identity

- [ ] Filter rows by `ctx.sender()` with indexed lookups to return only the caller's rows
- [ ] Combine `.sender().filter()` and `.recipient().filter()` with `.chain()` to union multiple filtered sets

#### Column Projection for Sensitive Data

- [ ] Define a custom `#[derive(SpacetimeType)]` struct that omits sensitive columns
- [ ] Map a full table row to the projection struct inside the view to exclude sensitive fields
- [ ] Return `Option<ProjectionType>` from a view for single-row column projection

#### Combined Row and Column Filtering

- [ ] Identify the caller with `ctx.db.<table>().identity().find(&ctx.sender())`
- [ ] Filter rows by a shared attribute using `ctx.db.<table>().<column>().filter(&value)`
- [ ] Map filtered rows to a projection type with `.map(|row| ProjectionType { ... })` to omit sensitive columns
- [ ] Return `Vec<ProjectionType>` from a view combining row filtering and column projection

## Schedule Tables

> Source: `SpacetimeDB.md`, lines 1631-1710

### Schedule Table Declaration

- [ ] Annotate a struct with `#[table(accessor = ..., scheduled(reducer_name))]` to create a schedule table
- [ ] Define a `ScheduleAt` column in the schedule table struct -- required column specifying when to fire
- [ ] Mark an `id` column with `#[primary_key]` and `#[auto_inc]` in the schedule table
- [ ] Define the scheduled reducer with `#[spacetimedb::reducer]` accepting `(&ReducerContext, RowType)` -- receives the full row as the second argument
- [ ] Reference a procedure name in the `scheduled(...)` attribute to schedule a procedure instead of a reducer

### Schedule Table Execution Lifecycle

- [ ] Insert a row with a `ScheduleAt` value into the schedule table to schedule execution
- [ ] Rely on SpacetimeDB to monitor the schedule table continuously
- [ ] Accept the full row as a parameter in the designated reducer/procedure when the scheduled time arrives
- [ ] Delete or update the row inside the reducer -- the runtime does not automatically remove it

### `ScheduleAt::Interval`

- [ ] Use `ScheduleAt::Interval(Duration::from_secs(N).into())` to schedule repeating execution at a fixed interval in seconds
- [ ] Use `ScheduleAt::Interval(Duration::from_millis(N).into())` to schedule repeating execution at a fixed interval in milliseconds
- [ ] Use `ScheduleAt::Interval(Duration::ZERO.into())` to fire immediately after transaction commit
- [ ] Convert `Duration` to the interval representation via `.into()`

### `ScheduleAt::Time`

- [ ] Use `ScheduleAt::Time(ctx.timestamp + Duration::from_secs(N))` to schedule one-shot execution at an absolute timestamp offset from now
- [ ] Use `ScheduleAt::Time(ctx.timestamp.clone())` to schedule immediate one-shot execution

### Schedule Table Security

- [ ] Guard a scheduled reducer with `is_internal()` to prevent external client invocation
- [ ] Guard a scheduled reducer with `ctx.sender() == ctx.identity()` to restrict to scheduler-only execution
- [ ] Access `ctx.sender()` inside a scheduled reducer to obtain the module's own identity
- [ ] Access `ctx.connection_id()` inside a scheduled reducer -- returns `None` since calls originate from SpacetimeDB

### Schedule Table Transactions

- [ ] Rely on each scheduled call running in its own independent transaction

## Identity and Authentication

> Source: `SpacetimeDB.md`, lines 1711-1870

### Identity

- [ ] Use `Identity` as a 32-byte, globally valid, long-lived public identifier for a database user
- [ ] Attach `Identity` to every reducer call for authorization decisions
- [ ] Derive module `Identity` on `spacetime publish`
- [ ] Provide the module's `Identity` from the client when connecting
- [ ] Derive `Identity` from OIDC provider tokens so the same user always resolves to the same `Identity`

#### Identity Derivation

- [ ] Derive `Identity` by hashing the JWT `iss` and `sub` fields with `blake3`
- [ ] Compute `blake3_hash(issuer + "|" + subject)` to produce the initial 32-byte hash
- [ ] Extract the first 26 bytes of the initial hash as `id_hash`
- [ ] Compute a checksum via `blake3_hash([0xC2, 0x00, *id_hash])`
- [ ] Assemble the final 32-byte `Identity` as `[0xC2, 0x00, *checksum_hash[:4], *id_hash]`

### ConnectionId

- [ ] Use `ConnectionId` to identify a specific client connection
- [ ] Call `ctx.connection_id()` to obtain `Option<ConnectionId>`
- [ ] Handle `None` from `ctx.connection_id()` for system-invoked reducers -- scheduled and lifecycle reducers return `None`

### Authentication

- [ ] Authenticate using OpenID Connect (OIDC) identity tokens
- [ ] Use `SpacetimeAuth` as a managed OIDC provider
- [ ] Use any third-party OIDC-compliant provider for authentication

#### SpacetimeAuth

- [ ] Use `SpacetimeAuth` for managed user management, authentication flows, and token issuance

#### Service-to-Service Authentication

- [ ] Authenticate services using OIDC tokens via the client credentials flow -- service obtains an access token using its own client ID and secret
- [ ] Authenticate services using OIDC service accounts -- special non-human accounts for automated services

#### Authorization in Modules

- [ ] Call `ctx.sender_auth()` to access identity claims from the context object
- [ ] Call `ctx.sender_auth().jwt()` to obtain the parsed JWT
- [ ] Call `jwt.issuer()` to validate the `iss` claim in `#[reducer(client_connected)]`
- [ ] Call `jwt.audience().iter()` to validate the `aud` claim against a known `OIDC_CLIENT_ID`
- [ ] Return `Err` from a `client_connected` reducer to reject unauthenticated or unauthorized connections
- [ ] Deserialize the JWT payload for custom claims not parsed by default

#### Localhost Identity

- [ ] Create a localhost identity via `POST /v1/identity` -- development only, returns a new identity and non-expiring token

### Sender Authorization Context

- [ ] Call `ctx.sender_auth()` to obtain `&AuthCtx` with the caller's authorization context
- [ ] Call `sender_auth.jwt()` to get `Option` with parsed JWT claims -- returns `None` when no JWT is present
- [ ] Call `sender_auth.is_internal()` to detect system-originated calls -- returns `true` for scheduled and lifecycle reducers

#### JWT Claims API

- [ ] Call `claims.subject()` to retrieve the `sub` claim -- unique user identifier from the issuer
- [ ] Call `claims.issuer()` to retrieve the `iss` claim -- authentication provider that issued the token
- [ ] Call `claims.audience()` to retrieve the `aud` claim as an iterable collection
- [ ] Combine `sub` and `iss` claims to compute the user's `Identity`
- [ ] Call `jwt.raw_payload()` to get the full JWT payload as a string for deserializing non-standard claims

#### Module Identity

- [ ] Call `ctx.identity()` to obtain the module's own `Identity`
- [ ] Compare `ctx.sender()` with `ctx.identity()` to distinguish system-initiated calls from client-initiated calls
- [ ] Guard a reducer by returning `Err` when `ctx.sender() != ctx.identity()` to restrict it to system-only invocation

#### Authorization Patterns

- [ ] Check `sender_auth.is_internal()` before JWT validation to trust system calls that carry no JWT
- [ ] Define a custom struct with `#[derive(serde::Deserialize)]` to model non-standard JWT claims
- [ ] Deserialize `jwt.raw_payload().as_bytes()` via `serde_json::from_slice` into a custom claims struct
- [ ] Extract a `roles: Vec<String>` field from custom claims for role-based access control
- [ ] Implement a reusable authorization function accepting `&spacetimedb::AuthCtx` for fine-grained access control
- [ ] Combine `is_internal()` check, issuer validation, audience validation, and custom claims parsing in a single authorization function

## Schema Migrations

> Source: `SpacetimeDB.md`, lines 1871-2045

### Automatic Migration on Republish

- [ ] Rely on automatic migration when adding tables, changing reducers, adding columns with defaults, or adding/removing indexes
- [ ] Expect publish to fail when removing/reordering columns or changing column types

### Adding Columns

- [ ] Append a new column to the end of an existing table with `#[default(0)]` to enable automatic migration
- [ ] Declare `#[spacetimedb::table(accessor = player, public)]` on the struct containing the new column
- [ ] Annotate the new column with `#[default(...)]` to supply a value for existing rows
- [ ] Expect publish failure with `"Database update rejected: Adding a column <name> to table <table> requires a manual migration"` when omitting a default value

### Removing Columns

- [ ] Use the incremental migration pattern to remove a column -- automatic migration does not support column removal

### Incremental Migration Pattern

- [ ] Create a new table (e.g., `CharacterV2`) with the desired schema alongside the original table
- [ ] Look up a row in the new table first via `ctx.db.character_v2().player_id().find(ctx.sender())`
- [ ] Fall back to the old table via `ctx.db.character().player_id().find(ctx.sender())` when the row is not yet migrated
- [ ] Transform and insert the old row into the new table via `ctx.db.character_v2().insert(CharacterV2 { ... })`
- [ ] Return the migrated row directly from `ctx.db.character_v2().insert(...)` as its return value

#### Dual-Write for New Records

- [ ] Insert new rows into both old and new tables via separate `ctx.db.character().insert(...)` and `ctx.db.character_v2().insert(...)` calls
- [ ] Annotate the dual-write reducer with `#[spacetimedb::reducer]`
- [ ] Omit new-schema-only fields (e.g., `alliance`) from the old-table insert
- [ ] Include the full schema with default values (e.g., `alliance: Alliance::Neutral`) in the new-table insert

#### Backward Sync on Update

- [ ] Update the old-table row via `ctx.db.character().player_id().update(Character { ... })` when modifying the new table
- [ ] Update the new-table row via `ctx.db.character_v2().player_id().update(character)` in the same function
- [ ] Translate new-table fields back to old-table fields by cloning shared fields (e.g., `character.nickname.clone()`)

#### Client Coexistence During Incremental Migration

- [ ] Keep the old table in sync so old and new clients coexist without disconnection
- [ ] Publish module updates without disconnecting active clients

#### Amortized Migration Cost

- [ ] Migrate rows lazily on access to spread transformation cost across many transactions -- avoids a single bulk migration

### Adding and Removing Indexes

- [ ] Add an index by updating the table definition and republishing
- [ ] Remove an index by updating the table definition and republishing -- may invalidate semijoin subscription queries requiring indexes on both join columns

### Safe Migration Changes

- [ ] Add new tables -- non-updated clients cannot see them
- [ ] Add indexes to existing tables
- [ ] Add or remove `#[auto_inc]` annotations
- [ ] Change a table from private to public
- [ ] Add new reducers
- [ ] Remove `#[unique]` constraints

### Potentially Breaking Migration Changes

- [ ] Add columns with defaults -- non-updated clients are unaware of the new column
- [ ] Change or remove reducers -- causes runtime errors for clients calling old/removed signatures
- [ ] Change a table from public to private -- causes runtime errors for clients subscribed to the now-private table
- [ ] Remove `#[primary_key]` -- causes non-deterministic local cache behavior on non-updated clients
- [ ] Remove indexes -- breaks semijoin subscriptions requiring indexes on both join columns

### Forbidden Migration Changes

- [ ] Avoid removing tables -- publish fails due to data loss
- [ ] Avoid removing, modifying, or reordering columns -- publish fails due to incompatibility with existing rows
- [ ] Avoid adding columns without a default value -- publish fails because existing rows cannot be populated
- [ ] Avoid adding columns in the middle of a table -- publish fails because new columns must be appended at the end
- [ ] Avoid changing scheduling usage of a table -- publish fails due to structural incompatibility
- [ ] Avoid adding `#[unique]` or `#[primary_key]` to an existing table -- publish fails because existing data may violate the new constraint

### Migration Best Practices

- [ ] Test migrations with sample data before production
- [ ] Use separate databases for dev, staging, and production environments
- [ ] Coordinate client updates for breaking changes
- [ ] Use feature flags in reducers for gradual rollouts
- [ ] Prefer adding new tables and reducers over modifying existing ones
- [ ] Document breaking changes in a changelog for client teams

#### Staged Migration Approach

- [ ] Apply additive changes first -- add new tables and columns
- [ ] Enter a dual-write period -- write to both old and new schema simultaneously
- [ ] Perform a staged rollout -- clients read from new schema while old schema remains supported
- [ ] Remove old schema once all clients are updated

### Client Compatibility During Migrations

- [ ] Expect brief interruptions to scheduled reducers during automatic migrations -- pauses in game loops or timers
- [ ] Expect runtime errors for clients calling removed or changed reducer signatures
- [ ] Regenerate client bindings to reflect new tables and reducers after schema changes

## Logging and Utilities

> Source: `SpacetimeDB.md`, lines 2046-2116

### Logging

- [ ] Call `log::info!` to log important application events such as user actions and state changes
- [ ] Call `log::warn!` to log problematic situations that do not prevent execution
- [ ] Call `log::error!` to log errors that prevent operations from completing
- [ ] Call `log::debug!` to log detailed diagnostic information for development
- [ ] Call `log::trace!` to log very detailed diagnostics -- typically disabled in production
- [ ] Use `log::info!` with format string interpolation inside a `#[reducer]` function
- [ ] Use `log::warn!` with conditional logic to flag values exceeding a threshold
- [ ] Use `log::error!` combined with returning `Err(String)` to signal invalid input
- [ ] Use `log::debug!` with `{:?}` formatting to inspect `ctx.sender()`

#### Structured Logging

- [ ] Embed key-value pairs in `log::info!` format strings for searchability -- e.g., `"from={:?}, to={}, amount={}"`
- [ ] Correlate log entries across invocations by including contextual fields like `ctx.sender()` in structured messages

#### Logging Performance

- [ ] Use `log::debug!` or `log::trace!` for verbose output so it can be filtered in production

### Timestamp

- [ ] Use `Timestamp` as a table column type -- built-in SpacetimeDB type representing microseconds since Unix epoch
- [ ] Access `ctx.timestamp` on `ReducerContext` to obtain the current reducer invocation time
- [ ] Insert a row using `ctx.timestamp` as the value for a `Timestamp`-typed column via `ctx.db.message().insert()`
