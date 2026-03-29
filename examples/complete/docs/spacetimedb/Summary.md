# SpacetimeDB Feature Reference

This document catalogs every feature, concept, and capability of SpacetimeDB with concise summaries, organized into thematic categories. Each section describes what a feature is, how it works, and when to use it. Each heading includes a keyword regex in an HTML comment — use these patterns with grep or search tools against `docs/spacetimedb/` to locate full documentation with code examples and implementation details.

## Table of Contents

<!-- keywords: table\s+of\s+contents|toc|feature\s+list -->

- [Architecture and Design](#architecture-and-design)
- [Tables](#tables)
- [Row Operations](#row-operations)
- [Indexes](#indexes)
- [Transactions](#transactions)
- [Storage and Persistence](#storage-and-persistence)
- [State Synchronization](#state-synchronization)
- [Reducers](#reducers)
- [Procedures](#procedures)
- [Views](#views)
- [Schedule Tables](#schedule-tables)
- [Identity and Authentication](#identity-and-authentication)
- [Schema Migrations](#schema-migrations)
- [Logging and Utilities](#logging-and-utilities)

## Architecture and Design
<!-- keywords: architect(ure)?.*design|design\s+philosoph(y|ies)|core\s+princip(le|les)|module\s+component -->

### Architecture
<!-- keywords: architect(ure)?|database.*(server|inside)|server.*(database|inside) -->

SpacetimeDB is a relational database that runs application logic directly inside the database. No separate web server, REST API, GraphQL layer, microservices, containers, or additional infrastructure needed — clients connect directly to the database.

- **Client interaction:** Clients call reducers (RPC endpoints) and subscribe to real-time data updates.
- **Authorization:** Written within the module, same as in a traditional server.

#### Host
<!-- keywords: host|server\s+that\s+hosts|maincloud|run\s+your\s+own -->

A SpacetimeDB host is a server that hosts databases. You can run your own host or use the SpacetimeDB maincloud. Many databases can run on a single host.

#### Database and Module
<!-- keywords: module|WebAssembly|wasm|spacetime\s+publish|deploy.*binary|schema.*business\s+logic -->

```sh
spacetime publish my-database
```

- **Module** = the code you write: table definitions, reducers, views, procedures, and business logic. Compiled to WebAssembly.
- **Database** = a running instance of a module with its own stored data and active connections.
- One module can be deployed to multiple databases (e.g., testing, staging, production), each with independent data.
- Re-publishing updates schema and logic; existing data remains intact (complex schema changes may require migration handling).

### Module Components
<!-- keywords: module\s+contain|tables.*reducers.*procedures.*views|three\s+categor(y|ies)|server.?side\s+function|function\s+type(s)?|reducer(s)?\s+vs|procedure(s)?\s+vs|view(s)?\s+vs|three\s+types\s+of\s+function|export(ed)?\s+function -->

A module contains four component types:

|                     | **Reducer**                 | **Procedure**                            | **View**                               |
| ------------------- | --------------------------- | ---------------------------------------- | -------------------------------------- |
| **Purpose**         | Modify data transactionally | Perform external operations (e.g., HTTP) | Read-only computed queries             |
| **Read tables**     | Yes                         | Yes                                      | Yes                                    |
| **Write tables**    | Yes                         | Yes                                      | No                                     |
| **Transaction**     | Automatic                   | Manual                                   | Automatic                              |
| **Deterministic**   | Yes                         | No                                       | Yes                                    |
| **External I/O**    | No                          | Yes                                      | No                                     |
| **Schedulable**     | Yes                         | Yes                                      | No                                     |
| **Client-callable** | Yes                         | Yes                                      | Yes                                    |
| **Context type**    | `ReducerContext`            | `ProcedureContext`                       | `ViewContext` / `AnonymousViewContext` |
| **Private tables**  | Read/Write                  | Read/Write (in tx)                       | Read                                   |

Tables define data structure and storage. Reducers, procedures, and views are the three exported function types clients interact with.

### Design Philosophy
<!-- keywords: zen|core\s+princip(le|les)|design\s+philosoph(y|ies)|everything\s+is -->

SpacetimeDB is built on five core design principles:

1. Everything is a table
2. Everything is persistent
3. Everything is real-time
4. Everything is transactional
5. Everything is programmable

These eliminate the need for separate backend servers, caching layers, sync code, and rollback logic.

### Supported Languages
<!-- keywords: supported\s+language|Rust.*fully\s+supported|server\s+module(s)?|performance.?critical -->

Rust is the fully supported language for writing SpacetimeDB server modules. The Rust Module SDK documentation is hosted on docs.rs.

### Database Names
<!-- keywords: database\s+name|name.*regex|lowercase.*dash|unique\s+identity|hex\s+string -->

Database names must match `/^[a-z0-9]+(-[a-z0-9]+)*$/` (lowercase ASCII, numbers, dashes). Examples: `my-game-server`, `chat-app-production`, `test123`. Each database also receives a unique identity (hex string) as an alternative reference.

### Version Compatibility
<!-- keywords: version\s+mismatch|same\s+version|module\s+library|spacetimedb.*crate|confusing.*error -->

The SpacetimeDB module library version (e.g., the `spacetimedb` Rust crate) must match the target host version. Version mismatches cause compilation or publish errors.

### Energy
<!-- keywords: energy|currency.*pay|data\s+storage.*compute|compute\s+operation -->

Energy is the currency used to pay for data storage and compute operations in a SpacetimeDB host.

### Database Resource Limits
<!-- keywords: size\s+limit|available\s+RAM|resource\s+limit|Maincloud.*plan|self.?hosted -->

SpacetimeDB holds all data in memory, so the practical size limit is the available RAM on the host.

- **Maincloud:** Resource limits depend on the user's plan.
- **Self-hosted:** The developer controls hardware and resource limits.

### Room-Based and Match-Based Architecture
<!-- keywords: room.?based|match.?based|lightweight.*database|orchestrat(e|ion)|create.*destroy.*database|independent\s+instance -->

SpacetimeDB databases are lightweight and fast to create, making them suitable for room-based or match-based multiplayer architectures. Use an external orchestration service to create and destroy databases per room/match — each runs an independent module instance with isolated state.

## Tables
<!-- keywords: table(s)?|#\[spacetimedb::table|accessor\s*=|public\s+table|private\s+table -->

```rust
#[spacetimedb::table(accessor = person, public)]
pub struct Person {
    #[primary_key]
    #[auto_inc]
    id: u32,
    name: String,
    #[unique]
    email: String,
}
```

A table is a SQL table declared via the `#[spacetimedb::table(accessor = ..., {public|private})]` macro. Each struct instance = a row, each field = a column. `accessor` names the handle for code access (e.g., `ctx.db.person()`).

- **Storage:** All data in memory (low-latency); automatically persisted to disk.

- **Features:** Constraints (`#[primary_key]`, `#[unique]`), `#[auto_inc]`, indexes, scheduling columns, default values.

### Table Visibility
<!-- keywords: table\s+visibilit(y|ies)|public\s+table|private\s+table|default.*private|client\s+read\s+access|view\s+function -->

```rust
#[spacetimedb::table(accessor = position, {private|public})]
struct Position { /* ... */ }
```

**Private (default):**

- Server can read and write, clients can't access.

- If not **all** clients should be able to access **all** data of this table, make it private.

- E.g.: Internal configuration, sensitive data like password hashes or API keys, intermediate computation results.

**Public:**

- Server can read and write, clients can read.

- If **all** clients should be able to access **all** data of this table, you can make it public.

**Regardless of visibility:**

- Clients modify data only by calling reducers — no direct write access to tables.

- For fine-grained access control, views can expose computed subsets of private table data (row filtering, column selection, joins).

- Rust `pub` modifier has no effect on table visibility — it only controls Rust module-level access.

### Table Naming and Accessor Conventions
<!-- keywords: accessor\s+name|naming\s+convention|lower_snake_case|ctx\.db\.\w+\(\)|accessor\s+match -->

```rust
#[spacetimedb::table(accessor = player_score, public)]
pub struct PlayerScore { /* ... */ }

// Generated accessor:
ctx.db.player_score()  // insert, find, update, delete, filter, iter, count
```

- Accessor name exactly matches the `accessor` attribute value.

- Convention: `lower_snake_case` (idiomatic Rust).

### Multiple Tables for the Same Type
<!-- keywords: multiple\s+table(s)?.*same\s+type|multiple.*#\[spacetimedb::table|shared\s+schema|move.*row.*between -->

```rust
#[spacetimedb::table(accessor = player, public)]
#[spacetimedb::table(accessor = logged_out_player)]
pub struct Player {
    #[primary_key]
    identity: Identity,
    name: String,
}

// Each table has independent rows; move rows between them:
let p = ctx.db.player().identity().delete(caller);
ctx.db.logged_out_player().insert(p);
```

Use cases: state separation (active vs. inactive entities), archiving, staging pending records.

#### Shared Constraints Across Multi-Table Types
<!-- keywords: shared\s+constraint|column\s+attribute.*all\s+tables|independent\s+primary\s+key|independent\s+unique -->

Column attributes (`#[primary_key]`, `#[unique]`, `#[auto_inc]`, `#[index]`) apply to **all** tables on the same struct, but each table enforces its own **independent** constraints and sequences. A value unique in one table may also appear in the other.

### Table Decomposition
<!-- keywords: table\s+decompos(e|ition)|access\s+pattern|update\s+frequenc(y|ies)|bandwidth|cache\s+efficien|schema\s+evolution|denormaliz -->

Favor many smaller tables over fewer large ones — organize by **access pattern**, not by entity. Joins in SpacetimeDB reduce to in-memory index lookups (nanoseconds), so splitting tables is cheap.

**Benefits:**

- **Reduced bandwidth:** Clients subscribing to high-frequency data don't receive low-frequency updates.

- **Cache efficiency:** Data with similar update frequencies resides in contiguous memory.

- **Semantic clarity:** Each table has a single responsibility.

- **Schema evolution:** Add columns to one table without affecting others.

**Guiding principle:** Keep data you read together in the same table; separate data you read at different times or frequencies.

### Data-Oriented Design and Relational Model
<!-- keywords: data.?oriented\s+design|relational\s+model|entity\s+component|ECS|relational\s+capabilit -->

SpacetimeDB's relational model is the logical endpoint of data-oriented design. ECS implements a strict subset of relational capabilities — tables provide indexes, constraints, relational queries, and real-time subscriptions out of the box.

### Physical and Logical Independence
<!-- keywords: physical.*logical\s+independence|logical\s+access|physical\s+representation|subscription\s+quer(y|ies).*unchanged -->

Queries express **what** data you need, not **how** to retrieve it. SpacetimeDB can change physical representation (storage formats, memory layouts, indexes) without requiring query rewrites. E.g., adding an index accelerates lookups automatically — existing queries work unchanged.

### System Tables
<!-- keywords: system\s+table|st_table|st_column|describes?\s+itself|meta.?data -->

SpacetimeDB stores its own schema as relational data in system tables (`st_table`, `st_column`, etc.) — no separate metadata layer needed. Do not modify system tables directly; use module code for schema changes.

### Supported Column Types
<!-- keywords: column\s+type|primitive\s+type|supported\s+type|composite\s+type|special\s+type|i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|bool|String|Vec<|Option< -->

| Category             | Types                                                                      |
| -------------------- | -------------------------------------------------------------------------- |
| Integers             | `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`       |
| Floats               | `f32`, `f64`                                                               |
| Other primitives     | `bool`, `String`                                                           |
| Composite            | `struct` / `enum` (with `#[derive(SpacetimeType)]`), `Vec<T>`, `Option<T>` |
| SpacetimeDB-specific | `Identity`, `ConnectionId`, `Timestamp`, `TimeDuration`, `ScheduleAt`      |

Any type deriving `SpacetimeType` can be used as a column type.

#### Custom Types with SpacetimeType
<!-- keywords: SpacetimeType|#\[derive\(SpacetimeType\)\]|custom\s+type|enum.*table|struct.*column -->

```rust
#[derive(SpacetimeType)]
pub struct Coordinates { x: f64, y: f64, z: f64 }

#[derive(SpacetimeType)]
pub enum Status {
    Active,
    Inactive,
    Suspended { reason: String },
}
```

Types deriving `SpacetimeType` can be used as column types, reducer arguments, or view results.

#### Optional Columns
<!-- keywords: Option<|optional\s+column|nullable|None -->

Wrap a column type in `Option<T>` for nullable fields. `None` indicates no value (e.g., `display_name: Option<String>` for users who haven't chosen a name yet).

#### Column Type Performance
<!-- keywords: type\s+performance|fixed.?size\s+type|variable.?size\s+type|column\s+order(ing)?|alignment|padding|serializ(e|ation)|cache\s+efficien -->

- **Use smallest fitting type:** `u8` for 0–255 uses less memory/bandwidth than `u64` for the same range.

- **Prefer fixed-size types:** Fixed types (`u32`, `f64`) allow direct offset computation. Use `[u8; 32]` over `Vec<u8>` for fixed-length data; use enums over `String` for categorical data.

- **Order columns largest-to-smallest alignment:** Reduces padding. `(u64, u8, u8)` = 16 bytes vs. `(u8, u64, u8)` = 24 bytes due to alignment padding.

### Primary Key
<!-- keywords: primary.?key|#\[primary_key\]|unique.*indexed|row\s+identity|immutable\s+identity -->

```rust
#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    id: u64,
    name: String,
}
```

- At most one primary key per table. Uniquely identifies each row.

- Automatically creates a unique index for efficient lookups.

- Changing a primary key value = delete old row + insert new row (not an in-place update).

#### Common Primary Key Patterns
<!-- keywords: auto.?increment.*id|identity\s+as\s+primary|Identity.*primary\s+key|user.?specific\s+data|caller.*identity -->

##### Auto-Incrementing ID Pattern
<!-- keywords: auto.?increment.*id|#\[primary_key\].*#\[auto_inc\]|auto.*assigned.*unique -->

```rust
#[primary_key]
#[auto_inc]
id: u64,
```

Most common pattern for tables needing stable unique row identifiers without caller-supplied IDs.

##### Identity as Primary Key Pattern
<!-- keywords: identity\s+as\s+primary|Identity.*primary\s+key|caller.*identity|user.?specific\s+data|one.*per.*identity -->

```rust
#[primary_key]
identity: Identity,
```

Ensures one row per user identity. Ideal for per-user data (profiles, settings).

#### Multi-Column Primary Keys
<!-- keywords: multi.?column\s+primary|composite\s+primary|compound\s+primary|multi.?column.*key -->

Not yet supported. Workaround: use a multi-column btree index for lookups + `#[auto_inc]` primary key for identity.

```rust
#[spacetimedb::table(accessor = inventory, public,
    index(accessor = inventory_index, btree(columns = [user_id, item_id])))]
pub struct Inventory {
    #[primary_key]
    #[auto_inc]
    id: u64,
    user_id: u64,
    item_id: u64,
    quantity: u32,
}
```

#### Update Behavior with Primary Keys
<!-- keywords: update.*primary\s+key|same\s+primary\s+key|different\s+primary\s+key|delete.*insert|update\s+in\s+place|subscriber.*event -->

- **Same primary key value:** Row updated in place → subscribers see an **update** event.

- **Different primary key value:** Old row deleted, new row inserted → subscribers see **delete + insert** events.

#### Tables Without Primary Keys
<!-- keywords: without\s+primary\s+key|no\s+primary\s+key|entire\s+row.*identity|set\s+semantics|duplicate\s+row|omit.*primary\s+key -->

Without a primary key, the **entire row** acts as the identity — rows are identified by complete content, and duplicate rows cannot exist (set semantics). Updates require matching all fields. Omitting a primary key avoids indexing overhead, useful for tables only accessed via full iteration.

### Unique Constraint
<!-- keywords: #\[unique\]|unique\s+constraint|unique\s+column|no\s+two\s+rows -->

```rust
#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    id: u32,
    #[unique]
    email: String,
    #[unique]
    username: String,
}
```

- Enforces no duplicate values for the column. Inserting a duplicate fails the transaction.

- Unlike `#[primary_key]`: multiple `#[unique]` columns allowed per table.

- Each unique column automatically creates an index for efficient lookups.

#### Primary Keys vs Unique Columns
<!-- keywords: primary\s+key\s+vs\s+unique|row\s+identity\s+vs\s+data\s+integrity|one\s+per\s+table|multiple\s+allowed|delete.*insert\s+vs\s+in.?place -->

| Aspect        | `#[primary_key]` | `#[unique]`     |
| ------------- | ---------------- | --------------- |
| Purpose       | Row identity     | Data integrity  |
| Per table     | At most 1        | Any number      |
| On key change | Delete + insert  | Update in place |
| Required      | No               | No              |

### Auto-Increment
<!-- keywords: auto.?inc(rement)?|#\[auto_inc\]|automatic.*id|sequence\s+number -->

```rust
#[spacetimedb::table(accessor = post, public)]
pub struct Post {
    #[primary_key]
    #[auto_inc]
    id: u64,
    title: String,
}

#[spacetimedb::reducer]
fn add_post(ctx: &ReducerContext, title: String) {
    let inserted = ctx.db.post().insert(Post { id: 0, title });
    log::info!("Assigned id: {}", inserted.id);
}
```

- Must be an integer type (`i8`–`i128`, `u8`–`u128`).

- Insert with value `0` → auto-assigned next value. Non-zero → used as-is (useful for data migration).

- `insert()` returns the row with the assigned value populated.

#### Auto-Increment Attribute Combinations
<!-- keywords: auto_inc.*primary_key|auto_inc.*unique|auto_inc.*default|combining.*auto_inc|cannot.*default -->

- `#[auto_inc]` + `#[primary_key]` → auto-generated unique row IDs (most common).

- `#[auto_inc]` + `#[unique]` → auto-generated unique values on non-PK column.

#### Sequences
<!-- keywords: sequence(s)?|internal\s+counter|PostgreSQL\s+sequence|sequence\s+parameter|start|min_value|max_value|increment -->

Auto-increment uses internal sequences (inspired by PostgreSQL). Configurable parameters: `start`, `min_value`, `max_value`, `increment` (can be negative). SpacetimeDB sets sensible defaults based on column type (e.g., `u64` → start=1, max=2^64−1). Internally uses a 128-bit counter.

##### Sequence Wrapping Behavior
<!-- keywords: wrap(s|ping)?|cycle|reach.*max|min_value.*max_value|negative\s+increment -->

When a sequence reaches its max, it wraps to min and continues cycling. Negative increments wrap in the opposite direction. After wrapping, a column with `#[primary_key]` or `#[unique]` will fail on duplicate insertion.

##### Sequence Crash Recovery
<!-- keywords: crash\s+recover|batch\s+allocat|4096|persist.*allocation|never\s+reused|skip.*values -->

Sequences allocate values in batches of 4096 and persist the allocation boundary. On crash/restart, the sequence resumes from the next boundary — values allocated but unused are skipped, but no value is ever assigned twice. Trades potential gaps for durability and performance.

##### Sequence Exhaustion
<!-- keywords: exhaust(ion|ed)|small(er)?\s+type|u8|u16|high\s+insert\s+volume|range.*large\s+enough -->

64-bit integers effectively never wrap. Smaller types (`u8` max 255, `u16` max 65535) or very high insert volumes can exhaust the range. On wrap with `#[primary_key]`/`#[unique]`, duplicate insertions fail.

#### Auto-Increment Sequence Gaps
<!-- keywords: auto.?inc(rement)?.*transactional|sequence\s+gap|sequence.*not\s+transactional|allocated.*roll(s|ed)?\s+back -->

Sequences are **not transactional** — rolled-back transactions still consume sequence numbers. Gaps also arise from batch allocation (4096 at a time) and future concurrent transactions. Consecutive inserts within a single reducer are not guaranteed consecutive values.

If strictly sequential numbering is required, maintain an explicit counter in a separate table and increment it transactionally.

### Default Values
<!-- keywords: default\s+value(s)?|#\[default\(|default\s+attribute -->

```rust
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
    #[default(0)]
    score: u32,
    #[default(true)]
    is_active: bool,
}
```

- Enables adding new columns during migration — existing rows auto-populated with the default.

- Expression must be const-evaluable (usable in Rust `const` context).

- New columns with defaults must be added at the **end** of the struct.

#### Default Value Const-Evaluable Restriction
<!-- keywords: const.?evalua|const\s+(fn|context)|String.*default|\.to_string\(\).*not.*const -->

Only const-constructible types (primitives, enums, etc.) can have defaults. Heap-allocated types like `String` cannot — e.g., `#[default("".to_string())]` is invalid because `.to_string()` is not `const fn`.

#### Default Value Attribute Restrictions
<!-- keywords: default.*cannot.*combin|default.*primary\s+key|default.*unique|default.*auto.?inc|conflict.*static\s+default -->

`#[default(...)]` cannot combine with `#[primary_key]`, `#[unique]`, or `#[auto_inc]` — these attributes manage column values themselves, conflicting with a static default.

#### Default Value Use Cases
<!-- keywords: schema\s+evolution|optional\s+field|feature\s+flag|default\(false\)|sensible\s+default -->

- **Schema evolution:** Add new columns without losing existing data.

- **Optional fields:** Sensible defaults for previously untracked fields.

- **Feature flags:** `#[default(false)]` to enable new functionality gradually.

### Event Tables
<!-- keywords: event\s+table(s)?|event\s+attribute|transient\s+row|noop|broadcast.*delet -->

```rust
#[spacetimedb::table(accessor = damage_event, public, event)]
pub struct DamageEvent {
    entity_id: Identity,
    damage: u32,
    source: String,
}
```

Event table rows exist only for the duration of the inserting transaction. On commit, rows are broadcast to subscribed clients then immediately deleted — the table is always empty between transactions.

- During a reducer, event tables behave like regular tables (insert, query, constraints).

- Inserts are recorded in the commitlog (full history preserved).

- All column types, constraints, indexes, and auto-increment work normally.

- The `event` flag cannot be changed after initial publication (migration error).

#### Publishing Events
<!-- keywords: publish\s+event|insert.*event\s+table|broadcast.*commit|rolled?\s+back.*no\s+event -->

```rust
#[spacetimedb::reducer]
fn attack(ctx: &ReducerContext, target_id: Identity, damage: u32) {
    ctx.db.damage_event().insert(DamageEvent {
        entity_id: target_id,
        damage,
        source: "melee_attack".to_string(),
    });
}
```

Insert a row into the event table from a reducer. On commit → broadcast to subscribers. On rollback → no events sent. The same event type can be published from any reducer.

#### Event Table Constraints and Indexes
<!-- keywords: event\s+table.*constraint|event\s+table.*index|constraint.*single\s+transaction|reset\s+between\s+transaction -->

Constraints (`#[primary_key]`, `#[unique]`, indexes, `#[auto_inc]`) work but are enforced **only within a single transaction** and reset between transactions. Duplicate primary keys across different transactions succeed because the table is empty at the start of each transaction.

#### Event Table Internal Mechanism
<!-- keywords: noop|insert.*automatic\s+delete|empty\s+set|table\s+state\s+never\s+change -->

Every insert is conceptually a noop: insert paired with automatic delete. Committed table state is always the empty set. Inserts are recorded in the commitlog for historical purposes only.

#### Event Table Row-Level Security
<!-- keywords: event.*row.?level\s+security|event.*RLS|control.*which\s+client.*event -->

RLS applies to event tables with the same semantics as regular tables — control which clients receive which events based on identity. E.g., restrict damage events so only the targeted player receives them.

#### Event Table Limitations
<!-- keywords: event\s+table.*limit|event.*view\s+function|infectious|deferred\s+to\s+a\s+future -->

Event tables cannot currently be accessed within view functions. A view joining on an event table would itself become an event view ("infectious" semantics), but this is deferred to a future release.

#### Event Table Use Cases
<!-- keywords: event\s+table.*use\s+case|damage\s+(number|event)|chat\s+message.*transient|notification.*transient|sound.*visual\s+effect|telemetry.*debug -->

For notifying clients about something that happened without storing a permanent record:

- Combat/damage events (floating damage numbers, hit indicators, kill notifications)

- Chat messages displayed on arrival without server-side persistence

- Transient notifications (player joined, achievement unlocked, trade completed)

- Client-side effects (explosions, particles, sounds)

- Streaming telemetry/diagnostics to a developer client

### Table Growth Management
<!-- keywords: table\s+growth|unbounded\s+growth|cleanup\s+reducer|expir(e|ation)|archive|temporary\s+data|pagination|large\s+result -->

Tables grow without bound unless actively managed. Strategies:

- **Cleanup reducers:** Periodically remove stale/temporary data.

- **Scheduled expiration:** Use schedule tables to trigger deletion of aged rows.

- **Archiving:** Delete or move old records no longer needed.

- **Pagination:** Limit data processed per operation for large result sets.

Unbounded growth increases memory consumption (all data in memory), degrades query performance, and increases client sync bandwidth for public tables.

### Collection Column vs. Separate Table
<!-- keywords: collection\s+column|Vec<.*>.*column|separate\s+table|atomic\s+unit|independent\s+identity|access\s+pattern -->

**Use `Vec<T>` column when:**

- Items form an atomic unit always read/written together

- Order is semantically important

- Collection is small and bounded

- Items are values without independent identity

**Use a separate table when:**

- Items have independent identity and lifecycle

- You need to query, filter, or index individual items

- Collection can grow unbounded

- Clients should receive per-item updates (not whole-collection)

- You need referential integrity between items and other data

### Binary Data Storage
<!-- keywords: binary\s+data|Vec<u8>|blob|file(s)?\s+in\s+table|image(s)?\s+in\s+column|file\s+storage -->

Store binary data in `Vec<u8>` columns — updates are atomic with metadata and auto-broadcast to subscribers. For files over ~100MB or data changing independently of other fields, use external storage with a reference in the table, or a hybrid approach (inline thumbnails + external originals).

#### Inline Binary Storage
<!-- keywords: inline\s+storage|Vec<u8>|data\s+stored\s+inline|files?\s+up\s+to -->

```rust
#[spacetimedb::table(accessor = user_avatar, public)]
pub struct UserAvatar {
    #[primary_key]
    user_id: u64,
    mime_type: String,
    data: Vec<u8>,
    uploaded_at: Timestamp,
}
```

Recommended for files up to ~100MB that change together with other row fields or need real-time subscription delivery. Trade-off: large binary data increases memory usage, network bandwidth, and commit times.

#### External Storage with References
<!-- keywords: external\s+storage|storage_url|storage\s+reference|large\s+file|separate.*bulk\s+storage -->

```rust
#[spacetimedb::table(accessor = document, public)]
pub struct Document {
    #[primary_key]
    #[auto_inc]
    id: u64,
    owner_id: Identity,
    filename: String,
    mime_type: String,
    size_bytes: u64,
    storage_url: String,  // Reference to external storage
    uploaded_at: Timestamp,
}
```

Store files in external storage, keep only metadata + URL in SpacetimeDB. Recommended for files over 100MB or when external blob storage is more economical (~$1/GB in SpacetimeDB).

#### External Storage Upload Flow
<!-- keywords: upload\s+flow|pre.?signed\s+url|client\s+upload|register\s+metadata|confirm_upload -->

1. Client calls a procedure/reducer to get a pre-signed upload URL.
2. Client uploads file directly to external storage.
3. Client calls a reducer with the storage URL + metadata to register the upload.
4. Database stores the reference.

Pre-signed URLs are preferred: files never pass through SpacetimeDB, clients transfer directly to storage, and SpacetimeDB handles only lightweight metadata. Procedures can make HTTP requests to external services (e.g., S3) and store metadata via transactions.

#### Hybrid Storage Strategy
<!-- keywords: hybrid|thumbnail|inline.*thumbnail|original.*external|small\s+preview -->

```rust
#[spacetimedb::table(accessor = image, public)]
pub struct Image {
    #[primary_key]
    #[auto_inc]
    id: u64,
    owner_id: Identity,
    thumbnail: Vec<u8>,      // Small preview inline
    original_url: String,    // Full-size in external storage
    width: u32,
    height: u32,
    uploaded_at: Timestamp,
}
```

Thumbnails arrive instantly via subscriptions; originals fetched on demand from external storage.

#### File Storage Strategy Selection
<!-- keywords: choosing\s+a\s+strategy|storage\s+cost|when\s+to\s+use.*storage|scenario.*approach -->

| Scenario                                                 | Approach                                      |
| -------------------------------------------------------- | --------------------------------------------- |
| Avatars (<10MB), attachments (<50MB), documents (<100MB) | Inline `Vec<u8>`                              |
| Large files (>100MB), video                              | External storage + DB reference               |
| Images needing previews                                  | Hybrid (inline thumbnail + external original) |
| Static assets needing CDN                                | External storage + CDN                        |

SpacetimeDB storage: ~$1/GB. For large files not needing atomic updates, external blob storage is more economical.

## Row Operations
<!-- keywords: row\s+operat(ion|ions)|insert|delete|update|filter|iter(ate)?|CRUD -->

### Table Trait and Row Operations
<!-- keywords: Table\s+trait|ctx\.db\.|\.insert\(|\.find\(|\.update\(|row\s+operat -->

```rust
use spacetimedb::Table; // Required — without it, table methods won't compile

ctx.db.<accessor>()          // accessor from #[table(accessor = ...)]
    .insert(...)             // add a row
    .try_insert(...)         // add a row, return Result
    .<column>().find(...)    // lookup by unique/primary key → Option
    .<column>().update(...)  // update by primary key
    .<column>().delete(...)  // delete by indexed column
    .<column>().filter(...)  // filter by indexed column → iterator
    .iter()                  // iterate all rows (full table scan)
    .count()                 // row count without iteration
```

### Row Insert
<!-- keywords: \.insert\(|insert\s+row|insert\s+a\s+new|ctx\.db\..*\.insert -->

```rust
ctx.db.user().insert(User {
    id: 0,  // auto-increment assigns the actual value
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
});
```

Inserts a row into the table within the current transaction.

### Row Lookup by Unique Column
<!-- keywords: \.find\(|find\s+by\s+(primary|unique)|lookup\s+by\s+(primary|unique)|identity\(\)\.find -->

```rust
// Find by primary key
if let Some(user) = ctx.db.user().id().find(123) {
    log::info!("Found: {}", user.name);
}

// Find by unique column
let by_email = ctx.db.user().email().find("alice@example.com");
```

`find` is available on any column marked `#[primary_key]` or `#[unique]`. Returns `Option<Row>`. Lookups are indexed.

### Row Update
<!-- keywords: \.update\(|update\s+row|identity\(\)\.update|struct\s+update\s+syntax|\.\.user -->

```rust
if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
    ctx.db.user().identity().update(User { name: Some(name), ..user });
}
```

Updates by primary key column. Use Rust struct update syntax (`..existing`) to copy unchanged fields.

### Row Delete
<!-- keywords: \.delete\(|delete\s+row|delete\s+by|remove\s+row -->

```rust
// Delete by primary key (single row)
ctx.db.user().id().delete(123);

// Delete by non-unique indexed column (all matching rows)
let deleted = ctx.db.user().name().delete("Alice");

// Delete by range
let deleted = ctx.db.user().age().delete(..18);
```

- **Unique/primary key column:** removes the single matching row.
- **Non-unique indexed column:** removes all matching rows, returns count.
- **Range expressions** (`..18`, `18..`, `18..=65`): bulk deletion without full table scan.

### Row Filter
<!-- keywords: \.filter\(|filter\s+by|filter\s+row|query\s+by\s+column -->

```rust
// Exact match
for user in ctx.db.user().name().filter("Alice") {
    log::info!("User {}: {}", user.id, user.email);
}

// Range queries
for user in ctx.db.user().age().filter(18..=65) { /* inclusive range */ }
for user in ctx.db.user().age().filter(18..)    { /* 18 and above */ }
for user in ctx.db.user().age().filter(..18)    { /* below 18 */ }

// Multi-column index: prefix exact + trailing range
for score in ctx.db.score().by_player_and_level().filter((123u32, 1u32..=10u32)) {
    log::info!("Level {}: {} points", score.level, score.points);
}
```

Returns an iterator over all matching rows. Supports exact values, range expressions, and multi-column index tuples (prefix columns exact, trailing column optionally a range).

### Row Iteration
<!-- keywords: \.iter\(\)|iterate\s+all|all\s+rows|scan\s+table -->

```rust
for user in ctx.db.user().iter() {
    log::info!("{}: {}", user.id, user.name);
}
```

Full table scan — iterates every row. Prefer indexed lookups or filters for large tables.

### Row Count
<!-- keywords: \.count\(\)|count\s+row|number\s+of\s+rows|total\s+rows -->

```rust
let total = ctx.db.user().count();
```

Returns the total row count without iterating.

### Batch Operations
<!-- keywords: batch\s+operation|batch.*insert|batch.*update|single\s+reducer\s+call|multiple\s+(insert|update|row)|reduce.*overhead|round\s+trip -->

```rust
#[reducer]
pub fn create_npcs(ctx: &ReducerContext) {
    for npc in generate_npcs() {
        ctx.db.npc().insert(npc);
    }
    // All inserts share one transaction — no per-row overhead.
}
```

Batch multiple row operations in a single reducer call. Each reducer runs in one transaction, so batching avoids repeated transaction and network overhead.

## Indexes
<!-- keywords: #\[index\(|btree|index\s+type|indexed\s+column|multi.?column\s+index|direct\s+index -->

```rust
#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    id: u32,
    #[index(btree)]        // field-level index
    name: String,
    #[index(btree)]
    age: u8,
}
```

Indexes maintain sorted data structures alongside tables to locate matching rows directly instead of scanning every row. Primary keys and unique constraints automatically create indexes. Indexes cost memory and slow writes — add them based on actual query patterns, not speculatively.

### When to Use Indexes
<!-- keywords: when\s+to\s+(use|add)\s+index|filter(ing)?\s+by\s+foreign\s+key|range\s+quer|sort(ing)?\s+column|index.*memory|index.*slow.*insert -->

**Add an index for:** foreign key columns (e.g., `player_id` in an inventory table), range query columns (e.g., `age`), and sort columns.

**Don't add an index for:** columns with primary key or unique constraints (already indexed automatically).

```rust
// Good: indexed lookup — O(log n)
ctx.db.player().name().filter("Alice")

// Bad: full table scan — O(n)
ctx.db.player().iter().find(|p| p.name == "Alice")
```

### B-tree Indexes
<!-- keywords: btree|b.?tree|sorted\s+order|equality\s+lookup|range\s+quer|prefix\s+match|default\s+index -->

Default index type. Maintains sorted order — supports equality lookups, range queries, and prefix matching on multi-column indexes. Works with any key type, single or multi-column.

### Direct Indexes
<!-- keywords: direct\s+index|#\[index\(direct\)\]|O\(1\)\s+lookup|array\s+index(ing)?|unsigned\s+integer\s+key|dense\s+integer -->

```rust
#[spacetimedb::table(accessor = position, public)]
pub struct Position {
    #[primary_key]
    #[index(direct)]
    id: u32,
    x: f32, y: f32, z: f32,
}
```

O(1) lookups using the key value as an array offset. Rust only.

- **Restrictions:** single-column, unsigned integer types only (`u8`, `u16`, `u32`, `u64`)
- **Works well:** dense keys starting near zero, sequential inserts (e.g., auto-increment primary keys)
- **Works poorly:** sparse keys, large first key, random insert patterns
- **Default to B-tree** unless profiling shows index lookups are a bottleneck and keys are dense/sequential.

### Single-Column Index Syntax
<!-- keywords: single.?column\s+index|field.?level\s+syntax|table.?level\s+syntax|#\[index\(btree\)\]|index\(accessor -->

**Field-level** (concise):

```rust
#[index(btree)]
name: String,
```

**Table-level** (named accessor):

```rust
#[spacetimedb::table(accessor = user, public, index(accessor = idx_age, btree(columns = [age])))]
pub struct User {
    #[primary_key]
    id: u32,
    name: String,
    age: u8,
}
```

Both produce the same index. Table-level gives the index its own named accessor.

### Multi-Column Indexes
<!-- keywords: multi.?column\s+index|composite\s+index|full\s+match|prefix\s+match|range\s+on\s+trailing|column\s+order -->

```rust
#[spacetimedb::table(accessor = score, public,
    index(accessor = by_player_and_level, btree(columns = [player_id, level])))]
pub struct Score {
    player_id: u32,
    level: u32,
    points: i64,
}
```

Rows sorted by first column, then second within equal first, etc. Supported query patterns:

| Pattern                | Example on `(player_id, level)`       | Accelerated? |
| ---------------------- | ------------------------------------- | ------------ |
| Full match             | `player_id = 123 AND level = 5`       | Yes          |
| Prefix match           | `player_id = 123`                     | Yes          |
| Range on trailing      | `player_id = 123 AND level IN 1..=10` | Yes          |
| Non-prefix column only | `level = 5`                           | No           |

### Index Query Methods
<!-- keywords: index\s+query|type.?safe\s+accessor|\.filter\(.*\.\.|range\s+syntax|multi.?column\s+query|equality\s+quer -->

Pattern: `ctx.db.<table>().<index_name>().filter(...)`

```rust
// Equality
for user in ctx.db.user().name().filter("Alice") { /* ... */ }

// Range queries
for user in ctx.db.user().age().filter(18..=65) { /* ... */ }  // inclusive
for user in ctx.db.user().age().filter(18..) { /* ... */ }     // from value
for user in ctx.db.user().age().filter(..18) { /* ... */ }     // up to value

// Multi-column: prefix match
for score in ctx.db.score().by_player_and_level().filter(&123u32) { /* ... */ }

// Multi-column: equality + range on trailing
for score in ctx.db.score().by_player_and_level().filter((123u32, 1u32..=10u32)) { /* ... */ }

// Multi-column: full match
for score in ctx.db.score().by_player_and_level().filter((123u32, 5u32)) { /* ... */ }
```

### Index-Accelerated Deletion
<!-- keywords: delet(e|ing)\s+with\s+index|delete\s+by\s+index|delete.*range|index.*accelerat.*delet -->

Same syntax as `filter()`, but deletes matching rows and returns the count deleted:

```rust
let deleted = ctx.db.user().name().delete("Alice");   // equality
let deleted = ctx.db.user().age().delete(..18);        // range
```

### Index Design Guidelines
<!-- keywords: index\s+design|choose\s+column|column\s+order.*index|redundant\s+index|read.*write\s+balance|selective\s+column -->

- **Index columns used in filters and joins** — unused indexes waste memory.
- **Multi-column order:** most selective column first, range columns after.
- **Avoid redundant indexes:** index on `(a, b)` makes a separate index on `(a)` redundant (prefix queries). Index on `(b)` alone is not redundant if queried independently.
- **Read/write tradeoff:** each index speeds reads but slows writes. High-write/low-read tables benefit from fewer indexes.

## Transactions
<!-- keywords: transaction(s|al)?|ACID|atomicit(y)?|isolat(ion)?|durabilit(y)?|consistenc(y)? -->

### ACID Transaction Guarantees
<!-- keywords: ACID|atomicit(y)?|consistenc(y)?|isolat(e|ion)|durabilit(y)?|transaction(al)?\\s+guarantee -->

Every reducer invocation runs as a single ACID transaction.

#### Atomicity
<!-- keywords: atomicit(y)?|all.or.nothing|partial\\s+state|commit(ted)?|roll(ed)?\\s+back -->

All-or-nothing execution per reducer. Successful completion → all changes committed. Error or exception → all changes rolled back as if the reducer never ran.

#### Consistency
<!-- keywords: consistenc(y)?|valid\\s+state|constraint(s)?\\s+enforc|unique\\s+constraint|constraint.*violat -->

All constraints (unique keys, indexes, module-enforced relationships) checked before commit. Violation → entire transaction rolled back.

#### Isolation
<!-- keywords: isolat(e|ion)|consistent\\s+snapshot|race\\s+condition|partial\\s+change|snapshot\\s+of\\s+the\\s+database -->

Each reducer sees the database state as of transaction start. Changes from concurrent reducers are invisible until their transaction commits.

#### Durability
<!-- keywords: durabilit(y)?|persist(ed|ent)?\\s+to\\s+disk|survive\\s+server\\s+restart|permanent\\s+change -->

Committed changes are persisted to disk and survive server restarts.

### Transaction Scope
<!-- keywords: transaction\\s+scope|automatic\\s+transaction|transaction\\s+start|transaction\\s+commit|reducer.*transaction\\s+lifecycle -->

**Reducers** — automatic transaction per invocation:

```rust
#[reducer]
pub fn parent_reducer(ctx: &ReducerContext) -> Result<(), String> {
    ctx.db.table_a().insert(RowA { /* ... */ });
    child_reducer(ctx)?; // Same transaction — not a nested transaction
    Ok(())
    // Success → commit all changes; Err → rollback all changes
}
```

- Starts on call, commits on success, rolls back on error — no manual setup needed.
- Nested reducer calls share the parent's transaction (no nested transactions supported).

**Procedures** — manual transactions via `ctx.with_tx`:

```rust
#[spacetimedb::procedure]
fn insert_value(ctx: &mut ProcedureContext, a: u32, b: String) {
    ctx.with_tx(|ctx| {
        ctx.my_table().insert(MyTable { a, b });
    });
    // Each with_tx call creates and commits an independent transaction
}
```

- Must call `ctx.with_tx(|ctx| { ... })` to access the database.
- Can open multiple separate transactions and perform I/O between them.

### Transaction Best Practices
<!-- keywords: transaction\\s+best\\s+practice|keep\\s+transaction.*short|shorter\\s+transaction|contention|throughput|error.*graceful -->

- **Keep transactions short** — only necessary DB operations in reducers; move external I/O to procedures. Shorter transactions reduce contention and improve throughput.
- **Handle errors gracefully** — return descriptive messages via `Result<(), String>`; any error rolls back all changes.

## Storage and Persistence
<!-- keywords: storage.*persist(ence)?|in.?memory|commit\s+log|hot.?swap|time\s+travel -->

### In-Memory Storage with Commit Log Persistence
<!-- keywords: (in.?memory|commit\s+log|persist(ence)?|recover|restart|crash|latency.*throughput|SSD.*bandwidth|15\s+GB) -->

All state is held in memory for sub-microsecond access. A commit log (write-ahead log) persists changes and replays on restart to recover exact state.

- Optimized for real-time apps (games, chat, collaboration), not batch/analytical workloads.
- 100–1,000x faster than traditional databases in benchmarks.
- Durability increases latency but not throughput — SSDs write ~15 GB/s (~4x less than DRAM bandwidth), so persistence cost is minimal.

### Row History and Time Travel
<!-- keywords: row\s+histor(y|ies)|full\s+history|time.?travel(ing)?\s+debug|every\s+position|historical\s+data -->

Full row change history is retained by default, enabling time-traveling debugging (inspect exact state at any past point).

- History must be explicitly deleted; never silently discarded.
- Storage example: 1M players × 10 updates/sec × 1 year ≈ 1–2 PB after 5–10x compression.

### Hot-Swap Module Updates
<!-- keywords: hot.?swap|swap.*code|without\s+disconnect|code.*without.*client -->

Server code can be hot-swapped without disconnecting clients. All state lives in tables (not ephemeral memory), so module logic is replaced while the database retains state and clients continue seamlessly.

## State Synchronization
<!-- keywords: state\s+(sync|mirror)|subscription|client.?side\s+(data|cache)|code\s+gen|RLS|row.?level\s+security -->

### State Mirroring
<!-- keywords: state\s+mirror(ing)?|mirror\s+state|live\s+update|stream\s+of\s+(live\s+)?update -->

SpacetimeDB mirrors database state to connected clients in real-time.

- Clients define subscriptions (via query builder or raw SQL) specifying what data they need.
- The server pushes incremental updates whenever subscribed data changes.
- The client-side mirror is **read-only** — clients modify the database only through reducer calls validated on the server.

### Client-Side Data View
<!-- keywords: (client.?side\s+read|data\s+view|local(ly)?\s+cache|cached\s+locally) -->

All client-side reads query a locally cached data view — never the server directly. Active subscriptions keep this cache in sync automatically (no polling).

### Client Code Generation
<!-- keywords: (client\s+code\s+gen|generat(e|ed|ing)\s+client|client\s+librar(y|ies)) -->

The `spacetime` CLI generates a typed client library from your database schema. The generated code provides:

- Strongly-typed data structures matching your tables (no raw query results)
- Interfaces for connecting, calling reducers, and receiving state updates

## Reducers
<!-- keywords: reducer(s)?|async\s+RPC|request.*sent|changes.*database\s+directly -->

```rust
#[spacetimedb::reducer]
pub fn create_user(ctx: &ReducerContext, name: String, email: String) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string()); // Rolls back transaction
    }
    ctx.db.user().insert(User { id: 0, name, email });
    Ok(()) // Commits transaction
}
```

Reducers are transactional RPC calls — the primary way clients modify server-side state. No HTTP overhead or JSON serialization.

- **Atomic:** Each reducer runs in its own transaction. Success → commit; error/panic → full rollback (all-or-nothing).
- **Return types:** `()`, `Result<(), String>`, or `Result<(), E: Display>`. Returning `Err` aborts the transaction.
- **Parameters:** First parameter must be `&ReducerContext`. All additional parameters must be serializable.
- **Isolated:** Only database operations allowed — no network, filesystem, or system calls.
- **Deterministic:** Behavior depends solely on inputs and current database state.
- **Subscription-aware:** If data changes pass the client's active subscriptions, the client sees the updated state in its local cache.

### ReducerContext
<!-- keywords: ReducerContext|reducer.*context|caller.*identity|authenticate.*caller -->

The `ReducerContext` (`&ReducerContext`) is the mandatory first parameter. All other parameters are user-defined and correspond to arguments clients pass when calling the reducer.

| Property/Method       | Type                   | Description                                                                        |
| --------------------- | ---------------------- | ---------------------------------------------------------------------------------- |
| `ctx.db`              | `Local`                | Read-write access to all table accessors                                           |
| `ctx.sender()`        | `Identity`             | Caller's identity (for authentication/access control)                              |
| `ctx.timestamp`       | `Timestamp`            | Time of reducer call (consistent throughout execution)                             |
| `ctx.connection_id()` | `Option<ConnectionId>` | Client connection ID (`None` for system-invoked reducers like scheduled/lifecycle) |
| `ctx.identity()`      | `Identity`             | Module's own identity                                                              |
| `ctx.rng()`           | `&StdbRng`             | Deterministic random number generator                                              |
| `ctx.random::<T>()`   | `T`                    | Generate single random value                                                       |
| `ctx.sender_auth()`   | `&AuthCtx`             | Authorization context (JWT claims, internal call detection)                        |

### Nested Reducer Calls
<!-- keywords: nested\s+reducer|child\s+transaction|nested\s+transaction|called\s+reducer|reducer.*call.*another -->

```rust
#[spacetimedb::reducer]
pub fn parent(ctx: &ReducerContext) -> Result<(), String> {
    ctx.db.table_a().insert(/* ... */);
    if child(ctx).is_err() {
        // Child error caught — parent continues, ALL changes (including pre-error) commit
    }
    Ok(())
}
```

No nested transactions. Direct reducer calls share the parent's transaction:

- **Error caught by parent:** Parent continues → entire transaction commits (including changes before the nested error).
- **Error propagated (`?`):** Parent fails → entire transaction rolls back (both parent and child changes).
- **For independent transactions:** Use scheduled reducers instead of direct calls.

### Reducer Isolation Constraints
<!-- keywords: reducer\s+isolat(e|ed|ion)|no\s+network|no\s+file\s+system|no\s+system\s+call|only\s+database\s+operation|cannot.*outside -->

Reducers can **only** perform database reads/writes through `ReducerContext`. No network requests, no filesystem access, no system calls.

This isolation ensures determinism, replay safety, and concurrent execution support. For external I/O (e.g., HTTP requests), use **procedures** instead.

### Global and Static Variable Prohibition
<!-- keywords: global\s+variable|static\s+variable|static\s+mut|module.?level\s+state|undefined\s+behavior|persist\s+across\s+reducer -->

```rust
// ❌ Undefined behavior — may not persist across reducer calls
static mut COUNTER: u64 = 0;

// ✅ Store state in tables instead
#[spacetimedb::table(accessor = counter)]
pub struct Counter { #[primary_key] id: u32, value: u64 }
```

Relying on global/static/module-level state across reducer calls is **undefined behavior**. Reasons:

- Fresh WASM instance may be created per reducer invocation
- Module publish creates a fresh execution environment (hot-swapping)
- Concurrent execution in separate environments (MVCC)
- Instance memory not persisted across crash recovery restarts
- Transaction rollback doesn't revert global state modifications (non-transactional)
- Serializability anomaly detection may re-execute reducers, causing duplicate global mutations

**All persistent state must be stored in tables.**

### Deterministic Random Number Generation
<!-- keywords: rng\(\)|random|StdbRng|deterministic.*random|reproducible|non.?deterministic|breaking\s+consensus -->

```rust
// ✅ Deterministic — identical results across all nodes
let value = ctx.random::<u32>();
let rng = ctx.rng(); // &StdbRng for multiple values

// ❌ Never use external RNG — breaks consensus across nodes
// use rand::Rng; let v = rand::thread_rng().gen::<u32>();
```

The context-provided RNG is deterministic and reproducible, ensuring consistent results across all nodes in a distributed system. External RNG (std library, third-party crates) is non-deterministic and **breaks consensus**.

### Reducer Error Categories
<!-- keywords: sender\s+error|programmer\s+error|error\s+handling|two\s+types?\s+of\s+error|error\s+categor -->

Two error categories — both cause full transaction rollback:

#### Sender Errors
<!-- keywords: sender\s+error|invalid\s+client\s+input|expected.*error|return.*Err\(|handled\s+graceful -->

Expected failures from invalid client input. Return via `Result<(), String>` — the error message is communicated back to the client for corrective action.

```rust
return Err("Insufficient credits".to_string()); // Sender error → client can handle
```

#### Programmer Errors
<!-- keywords: programmer\s+error|unexpected\s+error|bug.*module|panic|uncaught|assert!|\.expect\(|project\s+dashboard|alert(ing)? -->

Unexpected failures from bugs — panics (`assert!`, `.expect()`), uncaught errors. Logged in the project dashboard; set up alerting. Fix the code, don't handle at runtime.

### Lifecycle Reducers
<!-- keywords: lifecycle\s+reducer|client_connected|client_disconnected|connect.*disconnect|#\[reducer\(client_|#\[reducer\(init -->

Runtime-invoked reducers at key moments. Only take `&ReducerContext` — no additional parameters, not callable by clients.

| Attribute                         | Trigger                                    |
| --------------------------------- | ------------------------------------------ |
| `#[reducer(init)]`                | Module first published or database cleared |
| `#[reducer(client_connected)]`    | Client establishes connection              |
| `#[reducer(client_disconnected)]` | Client disconnects (close, timeout, error) |

#### Init Lifecycle Reducer
<!-- keywords: #\[reducer\(init\)\]|init\s+reducer|module\s+initializ(e|ation)|first\s+publish|database\s+clear -->

```rust
#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    ctx.db.settings().try_insert(Settings {
        key: "welcome_message".to_string(),
        value: "Hello, SpacetimeDB!".to_string(),
    })?;
    Ok(())
}
```

Runs once on first publish or database clear. Use for seeding default values and one-time configuration. If it fails, the publish/clear operation is **prevented**.

#### Client Connected
<!-- keywords: client_connected|on\s*connect|user.*connect|first\s+connect -->

```rust
#[reducer(client_connected)]
pub fn on_connect(ctx: &ReducerContext) -> Result<(), String> {
    let conn_id = ctx.connection_id().unwrap(); // Guaranteed Some(...)
    ctx.db.sessions().try_insert(Session {
        connection_id: conn_id,
        identity: ctx.sender(),
        connected_at: ctx.timestamp,
    })?;
    Ok(())
}
```

Fires per connection (WebSocket or HTTP). `ctx.connection_id()` is guaranteed `Some(...)`. If the reducer fails, the **client is disconnected**.

##### Rejecting Client Connections
<!-- keywords: reject.*client|disconnect.*client_connected|client_connected.*Result|client_connected.*Err|connection\s+was\s+rejected -->

Return `Err` from `client_connected` to reject: client is immediately disconnected and an ERROR-level log entry is written. Use for allowlists, banlists, capacity limits, or any custom gating logic. JWT/OIDC validation (documented in the Identity and Authentication section) is a common rejection reason.

#### Client Disconnected
<!-- keywords: client_disconnected|on\s*disconnect|user.*disconnect|mark.*offline -->

```rust
#[reducer(client_disconnected)]
pub fn on_disconnect(ctx: &ReducerContext) {
    let conn_id = ctx.connection_id().unwrap(); // Guaranteed Some(...)
    ctx.db.sessions().connection_id().delete(&conn_id);
}
```

Fires on connection close, timeout, or error. `ctx.connection_id()` is guaranteed `Some(...)`. Use for cleanup (mark offline, remove session records). Unlike `client_connected`, failure is **logged but does not prevent disconnection**.

## Procedures
<!-- keywords: procedure(s)?|#\[spacetimedb::procedure|ProcedureContext|unstable\s+feature|HTTP\s+request.*external -->

Procedures are like reducers but can perform operations not possible in reducers (e.g., HTTP requests to external services). Key differences from reducers:

- **No automatic transaction** — must manually open transactions via `with_tx` / `try_with_tx` to access the database.
- **HTTP capable** — can make HTTP requests to external services.
- **Beta** — requires `features = ["unstable"]` in `Cargo.toml`.

Use reducers for most cases; use procedures when you need to interact with the outside world.

### Procedure Declaration and Context
<!-- keywords: #\[spacetimedb::procedure\]|ProcedureContext|&mut\s+.*ProcedureContext -->

```rust
// Cargo.toml: spacetimedb = { version = "1.*", features = ["unstable"] }

#[spacetimedb::procedure]
fn add_two_numbers(ctx: &mut spacetimedb::ProcedureContext, lhs: u32, rhs: u32) -> u64 {
    lhs as u64 + rhs as u64
}
```

- Additional arguments must implement `SpacetimeType` (derive it for custom structs/enums).
- Arguments and return values are sent only to the caller, not broadcast to other clients.
- Clients call procedures and receive results via callbacks:

```rust
// Client-side
ctx.procedures.add_two_numbers_then(|ctx, res| {
    match res {
        Ok(value) => log::info!("Got {value}"),
        Err(e) => log::error!("Failed: {e:?}"),
    }
});
```

### Procedure Manual Transactions
<!-- keywords: with_tx|manual\s+transaction|procedure.*transaction|multiple\s+transaction|outside\s+transaction -->

Procedures do not automatically run in a transaction. Use `ProcedureContext::with_tx` to open one:

```rust
#[spacetimedb::procedure]
fn insert_a_value(ctx: &mut ProcedureContext, a: u32, b: String) {
    ctx.with_tx(|ctx| {
        ctx.db.my_table().insert(MyTable { a, b });
    });
}
```

- **`with_tx(Fn(&TxContext) -> T)`** — `&TxContext` provides the same database access as `ReducerContext` (`ctx.db` table accessors).
- **On return:** transaction commits, changes become permanent and are broadcast to clients.
- **On panic:** transaction rolls back, changes are discarded.
- **Multiple transactions:** each `with_tx` call creates an independent transaction. The procedure as a whole is not atomic — a later failure does not roll back earlier committed transactions.
- **Idempotency requirement:** the closure may be invoked multiple times against different database state versions. It must produce the same result for the same state. Do not capture mutable state.

#### Fallible Procedure Transactions
<!-- keywords: try_with_tx|fallible\s+(database\s+)?operation|procedure.*Result|Err.*roll\s*back -->

Use `ProcedureContext::try_with_tx` for fallible operations — `Ok` commits, `Err` rolls back:

```rust
ctx.try_with_tx(|ctx| {
    if a < 10 {
        return Err("a is less than 10!");
    }
    ctx.db.my_table().insert(MyTable { a, b });
    Ok(())
});
```

Preferred over panicking inside `with_tx` — provides explicit error handling without stack unwinding.

#### Transaction Return Values
<!-- keywords: return\s+value.*transaction|reading\s+values?\s+out|transaction.*return|with_tx.*return -->

`with_tx` and `try_with_tx` return the closure's return value to the calling procedure code. Return values are not saved to the database or broadcast to clients.

```rust
#[spacetimedb::procedure]
fn find_highest_level_player(ctx: &mut ProcedureContext) {
    let highest = ctx.with_tx(|ctx| {
        ctx.db.player().iter().max_by_key(|p| p.level)
    });
    // Use `highest` outside the transaction (e.g., for HTTP responses, logging)
}
```

### Procedure HTTP Requests
<!-- keywords: ctx\.http|http\.get|http\.send|HTTP\s+request.*procedure|spacetimedb::http|http::Body|http::Request|http::Response|http::Timeout -->

**`ctx.http.get(url)`** — simple GET, no headers:

```rust
match ctx.http.get("https://example.invalid") {
    Ok(response) => {
        let (response, body) = response.into_parts();
        log::info!("Status: {}, Body: {}", response.status, body.into_string_lossy());
    }
    Err(error) => log::error!("Request failed: {error:?}"),
}
```

**`ctx.http.send(request)`** — any HTTP method, custom headers, request body:

```rust
let request = spacetimedb::http::Request::builder()
    .uri("https://example.invalid/upload")
    .method("POST")
    .header("Content-Type", "text/plain")
    .body("This is the body of the HTTP request")
    .expect("Building `Request` object failed");
let response = ctx.http.send(request)?;
let (parts, body) = response.into_parts();
// body.into_string_lossy() or body.into_bytes()
```

Types `http::Request` and `http::Response` are re-exported as `spacetimedb::http::Request` and `spacetimedb::http::Response`.

#### HTTP Request Timeouts
<!-- keywords: http::Timeout|Timeout\(|extension\(|timeout.*procedure|Duration.*timeout -->

Set a timeout via `spacetimedb::http::Timeout` extension on the request builder:

```rust
let request = spacetimedb::http::Request::builder()
    .uri("https://example.invalid")
    .method("GET")
    .extension(spacetimedb::http::Timeout(std::time::Duration::from_millis(10).into()))
    .body(())
    .expect("Building `Request` object failed");
ctx.http.send(request).expect("HTTP request failed");
```

#### HTTP and Transaction Exclusivity
<!-- keywords: can't\s+send.*transaction|HTTP.*transaction.*same\s+time|transaction.*HTTP.*exclusive -->

HTTP requests and `with_tx`/`try_with_tx` blocks are mutually exclusive — close any open transaction before making an HTTP request, and complete any HTTP request before opening a transaction. Alternate between HTTP and transaction blocks; do not nest them.

### Calling Reducers from Procedures
<!-- keywords: call(ing)?\s+reducer.*procedure|reducer.*within.*transaction|procedure.*invoke.*reducer|reuse.*reducer\s+logic -->

```rust
#[spacetimedb::reducer]
fn process_item(ctx: &ReducerContext, item_id: u64) { /* ... */ }

#[spacetimedb::procedure]
fn fetch_and_process(ctx: &mut ProcedureContext, url: String) -> Result<(), String> {
    let response = ctx.http.get(&url).map_err(|e| format!("{e:?}"))?;
    let (_, body) = response.into_parts();
    let item_id: u64 = parse_id(&body.into_string_lossy());

    // Call the reducer within a transaction — runs inline, not as a subtransaction.
    ctx.with_tx(|tx_ctx| {
        process_item(tx_ctx, item_id);
    });

    Ok(())
}
```

Calling a reducer inside `with_tx` executes it as part of the same transaction (like any other function call). Useful for reusing reducer logic and combining HTTP + database operations.

### Scheduling Procedures from Reducers
<!-- keywords: schedul(e|ing)\s+procedure|reducer.*procedure|procedure.*schedule\s+table|ScheduleAt::Interval|Duration::ZERO|cannot\s+call\s+procedure -->

Reducers cannot call procedures directly (side effects are incompatible with transactional execution). Instead, insert a row into a schedule table that triggers the procedure. Use `ScheduleAt::Interval(Duration::ZERO.into())` to fire immediately after the reducer's transaction commits. See the Schedule Tables section for full details and code examples.

## Views
<!-- keywords: view(s)?|#\[spacetimedb::view|ViewContext|read.?only\s+function|derived\s+data|aggregation -->

```rust
#[spacetimedb::view(accessor = my_player, public)]
fn my_player(ctx: &ViewContext) -> Option<Player> {
    ctx.db.player().identity().find(ctx.sender())
}
```

A view is a read-only function declared with `#[spacetimedb::view]` that computes and returns results from tables. Views do not modify database state.

- **Declaration:** Must be `public` with an explicit `accessor` name. Accepts only a context parameter -- no user-defined arguments.
- **Transactional:** Runs within a transaction; never sees partial updates from concurrent reducers.
- **Real-time:** Auto-updates subscribed clients when underlying data changes.
- **Server-side computation:** Reduces data sent to clients, encapsulates queries, ensures consistent formatting.

### View Return Types and Subscriptions
<!-- keywords: view.*return|Option<|Vec<|subscribe.*view|SELECT\s+\*\s+FROM -->

| Return type     | Semantics                          |
| --------------- | ---------------------------------- |
| `Option<T>`     | At-most-one row                    |
| `Vec<T>`        | Multiple rows (procedural)         |
| `impl Query<T>` | Query builder result (incremental) |

`T` can be a table type or any product type (`#[derive(SpacetimeType)]`). Views are subscribable via SQL (`SELECT * FROM view_name`). `impl Query<T>` views use the query builder (`ctx.from.<table>().r#where(|row| ...)`) for type-safe filters evaluated by the query engine.

### ViewContext and AnonymousViewContext
<!-- keywords: ViewContext|AnonymousViewContext|view\s+context|ctx\.sender\(\)|caller.?independent|per.?user\s+view -->

| Context                | `ctx.sender()` | Use when                                               |
| ---------------------- | -------------- | ------------------------------------------------------ |
| `ViewContext`          | Yes            | Result depends on caller (e.g., "my inventory")        |
| `AnonymousViewContext` | No             | Result is the same for all callers (e.g., leaderboard) |

Both provide read-only table access via `ctx.db` and the query builder API via `ctx.from`.

#### AnonymousViewContext Performance Advantage
<!-- keywords: AnonymousViewContext|shared\s+across\s+all\s+subscriber|materialize.*once|per.?user\s+computation|anonymous\s+view -->

| Context                | Materialization                                                       | Scaling cost                             |
| ---------------------- | --------------------------------------------------------------------- | ---------------------------------------- |
| `AnonymousViewContext` | Once for all subscribers; recomputed once on change, broadcast to all | O(1)                                     |
| `ViewContext`          | Separate per subscriber (each may see different data)                 | O(N) -- 1,000 users = 1,000 computations |

Prefer `AnonymousViewContext` whenever possible. Consider design changes that make views caller-independent (e.g., "entities in chunk X" instead of "entities near me").

### View Query Builder
<!-- keywords: ctx\.from|r#where|impl\s+Query|Query<|query\s+builder|type.?safe\s+filter -->

```rust
#[view(accessor = high_scorers, public)]
fn high_scorers(ctx: &AnonymousViewContext) -> impl Query<Player> {
    ctx.from.player()
        .r#where(|p| p.score.gte(1000u64))
        .r#where(|p| p.name.ne("BOT"))  // chained where = logical AND
}
```

**Access:** `ctx.from.<table>()` on both `ViewContext` and `AnonymousViewContext`. Use `.r#where(|row| ...)` (alias: `.filter(...)`) with a closure returning a boolean condition. Return `impl Query<T>` instead of `Vec<T>`.

**Benefits over procedural views (returning `Vec<T>`):**

- Query engine applies global optimizations WASM/V8 cannot
- Incremental updates -- no full re-evaluation when the read set changes
- No row materialization costs at the WASM/V8 boundary
- For joins, the engine can reorder and choose better plans (e.g., start from the smaller table)

#### Query Builder Comparison Operators
<!-- keywords: comparison\s+operator|\.eq\(|\.ne\(|\.lt\(|\.lte\(|\.gt\(|\.gte\(|strongly\s+typed -->

| Operator | Meaning               |
| -------- | --------------------- |
| `eq`     | Equal                 |
| `ne`     | Not equal             |
| `lt`     | Less than             |
| `lte`    | Less than or equal    |
| `gt`     | Greater than          |
| `gte`    | Greater than or equal |

Comparisons are strongly typed -- invalid comparisons (e.g., `Identity` column vs. integer) produce compile-time errors.

#### Query Builder Boolean Combinators
<!-- keywords: boolean\s+combinator|\.and\(|\.or\(|\.not\(\)|combine\s+condition -->

```rust
ctx.from.player().r#where(|p| p.score.gte(1000u64).and(p.score.lt(5000u64)));
ctx.from.player().r#where(|p| p.name.eq("ADMIN").or(p.name.eq("BOT")));
ctx.from.player().r#where(|p| p.name.eq("BOT").not());
```

Combine conditions within a `where` closure using `and`, `or`, and `not`. Chain these on comparison operator results.

#### Query Builder Semijoins
<!-- keywords: semijoin|left_semijoin|right_semijoin|join\s+predicate|filtering\s+rows.*based\s+on -->

```rust
// Left semijoin: players that have a level entry
ctx.from.player()
    .left_semijoin(ctx.from.player_level(), |p, pl| p.id.eq(pl.player_id))

// Right semijoin: levels for high-scoring players
ctx.from.player()
    .r#where(|p| p.score.gte(1000u64))
    .right_semijoin(ctx.from.player_level(), |p, pl| p.id.eq(pl.player_id))
    .r#where(|pl| pl.level.gte(10u64))
```

Semijoins filter rows in one table based on the existence of matching rows in another.

- **`.left_semijoin()`** -- returns left (source) rows with at least one match on the right.
- **`.right_semijoin()`** -- returns right rows with at least one match on the left.
- Filters before the semijoin apply to the source side; filters after apply to the returned side.
- Join predicates are strongly typed and may only use indexed columns (multi-column indexes not supported).

### View Read Set and Invalidation
<!-- keywords: read\s+set|invalidat(e|ion)|iter\(\)|full\s+table\s+scan|indexed\s+lookup|re.?evaluat(e|ion)|black\s+box -->

Procedural view functions are opaque "black boxes" -- SpacetimeDB cannot analyze them. It tracks the **read set** (rows accessed) and re-executes the entire view if any row in that set changes.

- **Keep the read set small** to minimize reevaluation frequency.
- **`.iter()` is prohibited** in views -- a full table scan makes the read set include every row, so any change to any row triggers re-evaluation.
- **Only indexed lookups allowed:** `.find()` and `.filter()` on indexed columns enable targeted invalidation -- SpacetimeDB knows exactly which rows the view depends on.

#### Why SQL and Query Builder Subscriptions Can Scan
<!-- keywords: SQL\s+subscription|incremental\s+evaluation|derivative|query\s+engine|not\s+black\s+box|opaque\s+code -->

SQL and query builder subscriptions are not black boxes -- SpacetimeDB can analyze and transform them. The query engine uses **incremental evaluation**: when rows change, it computes exactly which output rows are affected without re-running the entire query (analogous to taking the derivative). Procedural view functions are opaque code, so incremental evaluation is impossible for them.

- For indexed access (`.find()`, `.filter()` on indexed columns), the cost difference between full re-evaluation and incremental evaluation is small -- hence the restriction to indexed access for procedural views.
- To aggregate or sort entire tables, return `impl Query<T>` to use the query builder, which supports incremental evaluation even on full scans.

### View Performance Considerations
<!-- keywords: view\s+performance|reevaluat(e|ed|ion)|row\s+materialization|WASM.*boundary|V8.*boundary|join.?heavy\s+view -->

**Server-side benefits:** Reduces network traffic, avoids redundant client computation, leverages server indexes.

**Reevaluation triggers:** Views re-execute when any row in their read set changes. Procedural views run as-is in WASM/V8 with no global optimization.

**Large read sets cause frequent reevaluation** -- non-unique index reads or multi-table joins without the query builder widen the read set.

**Join-heavy views** incur extra row materialization costs at the WASM/V8 boundary.

### Fine-Grained Access Control with Views
<!-- keywords: fine.?grained\s+access|row\s+filter.*view|column\s+projection|sensitive\s+column|view.*access\s+control|RLS|row.?level\s+security -->

Table visibility (public/private) is all-or-nothing. Views add fine-grained access control, also known as Row-Level Security (RLS): they read from private tables and expose only specific rows and/or columns per client. Sensitive data stays private; clients receive filtered, safe subsets through public views.

#### Row Filtering by Caller Identity
<!-- keywords: filter.*row.*caller|ctx\.sender\(\).*filter|my_messages|message.*sender.*recipient|caller.*identity.*row -->

```rust
#[spacetimedb::view(accessor = my_messages, public)]
fn my_messages(ctx: &ViewContext) -> Vec<Message> {
    let sent: Vec<_> = ctx.db.message().sender().filter(&ctx.sender()).collect();
    let received: Vec<_> = ctx.db.message().recipient().filter(&ctx.sender()).collect();
    sent.into_iter().chain(received).collect()
}
```

Use `ctx.sender()` with indexed lookups to return only rows belonging to the caller. Clients see only their own data (e.g., messages where they are sender or recipient).

#### Column Projection for Sensitive Data
<!-- keywords: hiding\s+sensitive|column\s+projection|omit.*sensitive|PublicUserProfile|SpacetimeType.*projection|password_hash|api_key -->

```rust
#[derive(SpacetimeType)]
pub struct PublicUserProfile { id: u64, username: String, created_at: Timestamp }

#[spacetimedb::view(accessor = my_profile, public)]
fn my_profile(ctx: &ViewContext) -> Option<PublicUserProfile> {
    let user = ctx.db.user_account().identity().find(&ctx.sender())?;
    Some(PublicUserProfile { id: user.id, username: user.username, created_at: user.created_at })
    // email, password_hash, api_key are never exposed
}
```

Return a custom `SpacetimeType` struct that omits sensitive columns. The view reads from a private table and returns only the safe subset of columns.

#### Combined Row and Column Filtering
<!-- keywords: combin(e|ed|ing)\s+(row|both)|row\s+filtering.*column\s+projection|colleague|department.*salary -->

```rust
#[spacetimedb::view(accessor = my_colleagues, public)]
fn my_colleagues(ctx: &ViewContext) -> Vec<Colleague> {
    let Some(me) = ctx.db.employee().identity().find(&ctx.sender()) else {
        return vec![];
    };
    ctx.db.employee().department().filter(&me.department)
        .map(|emp| Colleague { id: emp.id, name: emp.name.clone(), department: emp.department.clone() })
        .collect()
}
```

Combine row filtering and column projection in one view: identify the caller, filter rows by a shared attribute (e.g., same department), then map results to a projection type that omits sensitive columns (e.g., salary).

## Schedule Tables
<!-- keywords: schedul(e|ed)\s+table|#\[table\(.*scheduled|ScheduleAt|scheduled\s+reducer|schedul(e|ed|ing)\s+(another\s+)?reducer|run\s+at\s+an?\s+interval|specific\s+time -->

A schedule table automatically triggers a designated reducer (or procedure) for each row at a specified time.

```rust
#[spacetimedb::table(accessor = reminder_schedule, scheduled(send_reminder))]
pub struct Reminder {
    #[primary_key]
    #[auto_inc]
    id: u64,
    user_id: u32,
    message: String,
    scheduled_at: ScheduleAt,  // required column
}

#[spacetimedb::reducer]
fn send_reminder(ctx: &ReducerContext, reminder: Reminder) -> Result<(), String> {
    // Process the scheduled reminder -- receives the full row as second argument
    Ok(())
}
```

- **Declaration:** `#[table(accessor = ..., scheduled(reducer_name))]` -- the attribute uses `scheduled` (with "d") because it names the scheduled reducer.
- **Required column:** must include a `ScheduleAt` column specifying when to fire.
- **Procedures:** reference the procedure name in `scheduled` the same way.
- **Transactions:** each scheduled call runs in its own independent transaction, providing separate transactional boundaries when nested transactions are unavailable.
- **Sender identity:** `ctx.sender()` is the module's own identity; `ctx.connection_id()` is `None` (calls originate from SpacetimeDB, not a client).

### Schedule Table Execution Lifecycle
<!-- keywords: schedul(e|ed)\s+table.*lifecycle|monitor(s|ing)\s+the\s+schedule|row\s+is.*deleted|time\s+arrives -->

1. Insert a row with a `ScheduleAt` value into the schedule table.
2. SpacetimeDB monitors the table continuously.
3. When the time arrives, the designated reducer/procedure is called with the full row as a parameter.
4. The reducer must delete or update the row -- the runtime does **not** automatically remove it.

### ScheduleAt::Interval
<!-- keywords: ScheduleAt::Interval|Duration::from_(secs|millis)|interval\s+scheduling|periodic|repeating|recurring -->

Schedules a reducer to execute **repeatedly** at a fixed interval. Accepts a `Duration` converted via `.into()`.

```rust
ScheduleAt::Interval(Duration::from_secs(5).into())    // every 5 seconds
ScheduleAt::Interval(Duration::from_millis(100).into()) // every 100ms
ScheduleAt::Interval(Duration::ZERO.into())             // fire immediately after commit
```

Suited for game ticks, heartbeats, recurring maintenance. `Duration::ZERO` is useful for executing a procedure as soon as possible from a reducer.

### ScheduleAt::Time
<!-- keywords: ScheduleAt::Time|specific\s+time|one.?shot|absolute\s+timestamp|ctx\.timestamp\s*\+ -->

Schedules a reducer to execute **once** at an absolute timestamp.

```rust
// 10 seconds from now
ScheduleAt::Time(ctx.timestamp + Duration::from_secs(10))
// Immediate execution
ScheduleAt::Time(ctx.timestamp.clone())
```

Suited for one-shot actions: reminders, content expiration, auction endings.

### Schedule Table Security
<!-- keywords: schedul(ed)?\s+reducer.*callable|is_internal\(\)|only\s+.*scheduler|prevent.*external\s+invocation -->

Scheduled reducers are normal reducers — external clients can invoke them directly. To restrict to scheduler-only execution, add an authentication guard using `is_internal()` or `ctx.sender() == ctx.identity()` (documented in the Identity and Authentication section).

### Schedule Table Use Cases
<!-- keywords: reminder|expir(e|ing)\s+content|delayed\s+action|periodic\s+task|game\s+mechanic|timer.?based -->

- **Reminders/notifications** -- fire at specific times
- **Content expiration** -- auto-remove or archive old data
- **Delayed actions** -- execute after a timeout
- **Periodic maintenance** -- recurring cleanup at fixed intervals
- **Game mechanics** -- building timers, energy regeneration

Use `ScheduleAt::Interval` for repeating actions, `ScheduleAt::Time` for one-shot actions.

## Identity and Authentication
<!-- keywords: identity|authenticat(e|ion)|OIDC|ConnectionId|sender_auth|JWT -->

### Identity
<!-- keywords: Identity|identif(y|ier|ies)|issuer.*subject|JWT|blake3|globally\s+valid -->

`Identity` is a long-lived, public, globally valid 32-byte identifier for someone interacting with a database. Persists across connections and sessions — the recommended primary user identifier.

- Attached to every reducer call for authorization decisions
- Modules also have Identities (issued on `spacetime publish`); clients must provide the module's `Identity` when connecting
- Derived from OIDC provider tokens — same user always gets the same `Identity`

#### Identity Derivation
<!-- keywords: identity.*deriv(e|ed|ation)|issuer.*subject.*hash|blake3_hash|checksum|OpenID\s+Connect -->

Identity is derived by hashing the JWT's `iss` and `sub` fields with blake3. Database developers are responsible for issuing Identities via OIDC.

```python
def identity_from_claims(issuer: str, subject: str) -> [u8; 32]:
    hash1: [u8; 32] = blake3_hash(issuer + "|" + subject)
    id_hash: [u8; 26] = hash1[:26]
    checksum_hash: [u8; 32] = blake3_hash([0xC2, 0x00, *id_hash])
    return [0xC2, 0x00, *checksum_hash[:4], *id_hash]
```

### ConnectionId
<!-- keywords: ConnectionId|connection\s+id|multiple\s+connection(s)?|unique.*connection -->

A `ConnectionId` identifies a specific client connection. A user has one `Identity` but may open multiple connections, each with a unique `ConnectionId`. Useful for tracking sessions or per-connection state.

- **Access:** `ctx.connection_id()` returns `Option<ConnectionId>`
- **`None`** for system-invoked reducers (scheduled, lifecycle)

### Authentication
<!-- keywords: authenticat(e|ion|ing)|OIDC|OpenID\s+Connect|identity\s+provider|SpacetimeAuth|auth -->

SpacetimeDB uses OpenID Connect (OIDC) identity tokens for authentication, compatible with most OIDC providers. Modules are exposed to the open internet — authentication is critical. Every reducer call includes the caller's `Identity` for authorization logic.

- **Supported providers:** SpacetimeAuth (managed) or any third-party OIDC-compliant provider
- Do not store/verify passwords in modules — use OIDC-based authentication

#### SpacetimeAuth
<!-- keywords: SpacetimeAuth|managed\s+OIDC|built\s+specifically|user\s+management|token\s+issuance -->

Fully managed OIDC provider built for SpacetimeDB. Handles user management, authentication flows, and token issuance. Production-ready for common use cases; use a third-party OIDC provider for advanced features or customization.

#### Service-to-Service Authentication
<!-- keywords: service.?to.?service|client\s+credentials\s+flow|service\s+account(s)?|authenticate\s+your\s+service|server.*API.*auth -->

OIDC tokens also authenticate servers, APIs, and other services (not just end-user clients). Two approaches:

- **Client credentials flow** — service obtains an access token using its own client ID and secret
- **Service accounts** — special non-human accounts from OIDC providers for automated services

#### Authorization in Modules
<!-- keywords: authoriz(e|ation)\s+in\s+.*module|identity\s+claims|validates.*OIDC\s+token|extract.*claims|claims.*context -->

Authentication (OIDC token validation) is only the first step; modules must implement authorization to control what authenticated users can do. The server validates OIDC tokens and makes identity claims available via the context object.

```rust
const OIDC_CLIENT_ID: &str = "client_XXXXXXXXXXXXXXXXXXXXXX";

#[reducer(client_connected)]
pub fn connect(ctx: &ReducerContext) -> Result<(), String> {
    let jwt = ctx.sender_auth().jwt().ok_or("Authentication required")?;
    if jwt.issuer() != "https://auth.spacetimedb.com/oidc" {
        return Err("Invalid issuer".to_string());
    }
    if !jwt.audience().iter().any(|a| a == OIDC_CLIENT_ID) {
        return Err("Invalid audience".to_string());
    }
    Ok(())
}
```

**Best practices:**

- Validate `iss` claim in `client_connected` to accept only expected providers
- Validate `aud` claim to prevent token reuse from other applications sharing the same issuer
- Deserialize JWT payload for custom claims not parsed by default

#### Localhost Identity
<!-- keywords: localhost\s+identity|POST\s+/v1/identity|development\s+only|token.*lost|single\s+token -->

Create via `POST /v1/identity`, which returns a new identity and token. **Development only** — localhost identities are tied to a single non-expiring token with no recovery if lost. Production applications should use an external OIDC provider with proper token lifecycle management.

### Sender Authorization Context
<!-- keywords: sender_auth|AuthCtx|authorization\s+context|JWT\s+claims|internal\s+call\s+detection -->

`ctx.sender_auth()` returns `&AuthCtx` with the caller's authorization context for fine-grained access control beyond `Identity`.

**`AuthCtx` methods:**

- **`sender_auth.jwt()`** — returns `Option` with parsed JWT claims (`None` when no JWT is present, e.g., localhost identities)
- **`sender_auth.is_internal()`** — returns `true` for system-originated calls (scheduled reducers) rather than external clients

#### JWT Claims API
<!-- keywords: jwt\(\)|claims\.subject|claims\.issuer|claims\.audience|aud\s+claim|sub\s+claim|iss\s+claim|raw_payload -->

The JWT claims object from `sender_auth.jwt()` provides:

| Method              | Claim | Description                                   |
| ------------------- | ----- | --------------------------------------------- |
| `claims.subject()`  | `sub` | Unique user identifier from the issuer        |
| `claims.issuer()`   | `iss` | Authentication provider that issued the token |
| `claims.audience()` | `aud` | Intended recipients (iterable collection)     |

- **`sub` + `iss`** are required claims used together to compute the user's `Identity`
- **`jwt.raw_payload()`** returns the full JWT payload as a string for deserializing non-standard claims via serde

#### Module Identity
<!-- keywords: ctx\.identity\(\)|module.?s?\s+own\s+identity|system.?initiated|sender.*!=.*identity -->

`ctx.identity()` returns the module's own `Identity`. Comparing `ctx.sender()` with `ctx.identity()` distinguishes system-initiated calls (equal) from client-initiated calls (differ) — an alternative to `is_internal()` for detecting system calls.

```rust
#[spacetimedb::reducer]
fn scheduled_task(ctx: &ReducerContext) -> Result<(), String> {
    // Reject external clients — only allow system scheduler
    if ctx.sender() != ctx.identity() {
        return Err("Only the system can call this reducer".to_string());
    }
    // ... system-only logic
    Ok(())
}
```

#### Authorization Patterns
<!-- keywords: is_internal\(\)|internal\s+call|scheduled\s+reducer.*trust|system\s+caller|bypass.*auth|custom\s+claims|raw_payload|deserializ(e|ing)\s+JWT|roles?\s+claim -->

**Internal call detection:** `sender_auth.is_internal()` returns `true` for system-initiated calls (scheduled reducers, lifecycle reducers) rather than external client connections. Check `is_internal()` before JWT validation — system calls carry no JWT but should be trusted.

**Custom claims:** Access non-standard JWT claims by deserializing `jwt.raw_payload()` into a custom struct.

```rust
#[derive(serde::Deserialize)]
pub struct CustomClaims {
    pub roles: Vec<String>,
}

fn ensure_admin_access(sender_auth: &spacetimedb::AuthCtx) -> Result<(), String> {
    if sender_auth.is_internal() {
        return Ok(()); // Trust system-initiated calls (scheduled, lifecycle)
    }
    let jwt = sender_auth.jwt().ok_or("Authentication required")?;
    if jwt.issuer() != "https://auth.spacetimedb.com/oidc" {
        return Err("Invalid issuer".to_string());
    }
    if !jwt.audience().iter().any(|a| a == OIDC_CLIENT_ID) {
        return Err("Invalid audience".to_string());
    }
    let claims: CustomClaims = serde_json::from_slice(jwt.raw_payload().as_bytes())
        .map_err(|e| format!("Invalid JWT: {e}"))?;
    if claims.roles.iter().any(|r| r == "admin") {
        return Ok(());
    }
    Err("Admin role required".to_string())
}
```

## Schema Migrations
<!-- keywords: migrat(e|ion|ions)|schema\s+change|auto(matic)?\s+migrat|publish.*schema|compatible\s+change -->

No built-in general schema-modifying migrations. On republish, SpacetimeDB compares the new schema (tables, reducers, procedures, views, and their types) against the existing one and attempts automatic migration.

- **Compatible:** adding tables, changing reducers, adding columns with defaults, adding/removing indexes
- **Incompatible (publish fails):** removing/reordering columns, changing column types

External migration tools are possible but require downtime and simultaneous client updates.

### Adding Columns
<!-- keywords: add(ing)?\s+column|new\s+column|default\s+value(s)?|end\s+of\s+(a\s+)?table -->

New columns can be added to the **end** of an existing table if they have default values (handled automatically on republish). Columns cannot be added in the middle of a table.

```rust
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    id: u64,
    name: String,
    #[default(0)]    // new column -- appended at end with default
    score: u32,
}
```

Without a default value, publish fails: `"Database update rejected: Adding a column <name> to table <table> requires a manual migration"`.

### Removing Columns
<!-- keywords: remov(e|ing)\s+column|drop\s+column|not\s+supported.*migrat -->

Not supported through automatic migration. Use the incremental migration pattern: create a new table with the desired schema (without the unwanted column) and migrate data lazily.

### Incremental Migration Pattern
<!-- keywords: incremental\s+migrat|laz(y|ily)\s+migrat|new\s+table.*desired\s+schema|migrat.*row(s)?\s+as -->

For changes unsupported by automatic migration (removing/reordering columns, changing types), add a new table (e.g., `character_v2`) with the desired schema alongside the original. On row access:

1. Check new table -- if present, already migrated
2. If absent, look up in old table, transform and insert into new table
3. If absent in both, row does not exist

```rust
fn find_character_for_player(ctx: &ReducerContext) -> CharacterV2 {
    if let Some(c) = ctx.db.character_v2().player_id().find(ctx.sender()) {
        return c; // already migrated
    }
    let old = ctx.db.character().player_id().find(ctx.sender())
        .expect("Player has not created a character");
    ctx.db.character_v2().insert(CharacterV2 {
        player_id: old.player_id, nickname: old.nickname,
        level: old.level, class: old.class, alliance: Alliance::Neutral,
    })
}
```

This achieves zero-downtime updates via module hotswapping and amortizes migration cost across transactions.

#### Dual-Write for New Records
<!-- keywords: dual.?write|insert.*both|new\s+record.*both\s+table|create.*both -->

Insert new rows into both old and new tables so old clients subscribing to the original table continue to see new data. The old-table row omits new columns; the new-table row includes the full schema.

```rust
#[spacetimedb::reducer]
fn create_character(ctx: &ReducerContext, class: Class, nickname: String) {
    ctx.db.character().insert(Character {
        player_id: ctx.sender(), nickname: nickname.clone(), level: 1, class,
    });
    ctx.db.character_v2().insert(CharacterV2 {
        player_id: ctx.sender(), nickname, level: 1, class, alliance: Alliance::Neutral,
    });
}
```

#### Backward Sync on Update
<!-- keywords: backward\s+sync|update.*old\s+table|propagat.*old|outdated\s+client.*function -->

When updating a row in the new table, also update the corresponding old-table row using the same translation logic as creation/migration. This keeps outdated clients functional (though they will not see new columns).

```rust
fn update_character(ctx: &ReducerContext, character: CharacterV2) {
    ctx.db.character().player_id().update(Character {
        player_id: character.player_id,
        nickname: character.nickname.clone(),
        level: character.level,
        class: character.class,
    });
    ctx.db.character_v2().player_id().update(character);
}
```

#### Client Coexistence During Incremental Migration
<!-- keywords: old\s+client.*new\s+client|coexist|outdated\s+client|update.*own\s+pace|roll\s+out.*client -->

Old and new clients coexist because the old table stays in sync. The module update publishes without disconnecting clients, and users update at their own pace.

#### Amortized Migration Cost
<!-- keywords: amortiz(e|ed)|cost.*transform|rows.*only.*needed|laz(y|ily)\s+populat -->

Rows are only migrated to the new table when accessed, spreading transformation cost across many transactions instead of a single bulk migration. Suitable for large tables where upfront migration would be expensive.

### Adding and Removing Indexes
<!-- keywords: add(ing)?\s+index|remov(e|ing)\s+index|break.*subscription|depend.*index -->

Add or remove indexes by updating the table definition and republishing. Removing an index can invalidate semijoin subscription queries (e.g., left semijoin) where SpacetimeDB requires indexes on both join columns -- clients with such subscriptions receive runtime errors.

### Safe Migration Changes
<!-- keywords: safe\s+change|always\s+allowed|non.?breaking|add(ing)?\s+new\s+table|add(ing)?\s+index|auto.?inc.*annotation|private\s+to\s+public|add(ing)?\s+new\s+reducer|remov(e|ing)\s+unique -->

Always allowed, will not break existing clients (no client coordination needed):

- Adding new tables (non-updated clients cannot see them)
- Adding indexes
- Adding or removing `Auto Inc` annotations
- Changing tables from private to public
- Adding new reducers
- Removing `Unique` constraints

### Potentially Breaking Migration Changes
<!-- keywords: breaking\s+change|potentially\s+break|runtime\s+error|non.?updated\s+client|public\s+to\s+private|remov(e|ing)\s+primary\s+key|remov(e|ing)\s+reducer|chang(e|ing)\s+reducer -->

Allowed by automatic migration but may cause runtime errors for non-updated clients:

| Change                       | Impact on Non-Updated Clients                                        |
| ---------------------------- | -------------------------------------------------------------------- |
| Adding columns with defaults | Clients unaware of new column                                        |
| Changing/removing reducers   | Runtime errors when calling old/removed reducers                     |
| Public to private table      | Runtime errors for clients subscribed to the now-private table       |
| Removing `Primary Key`       | Non-deterministic local cache behavior (old PK used as unique key)   |
| Removing indexes             | Breaks semijoin subscriptions requiring indexes on both join columns |

### Forbidden Migration Changes
<!-- keywords: forbidden\s+change|cannot.*automatic\s+migrat|publish.*fail|remov(e|ing)\s+table|reorder(ing)?\s+column|chang(e|ing)\s+column\s+type|add(ing)?\s+unique|add(ing)?\s+primary\s+key|scheduling.*change -->

The following changes cause the publish to fail (use incremental migration instead):

| Forbidden Change                        | Reason                                       |
| --------------------------------------- | -------------------------------------------- |
| Removing tables                         | Data loss                                    |
| Removing/modifying/reordering columns   | Incompatible with existing rows              |
| Adding columns without a default value  | Existing rows cannot be populated            |
| Adding columns in the middle of a table | New columns must be appended at the end      |
| Changing scheduling usage of a table    | Structural incompatibility                   |
| Adding `Unique` or `Primary Key`        | Existing data may violate the new constraint |

### Migration Best Practices
<!-- keywords: migration\s+best\s+practice|plan\s+schema|coordinate.*client|feature\s+flag|backwards?\s+compat|additive\s+change|dual.?write|staged\s+rollout -->

**Development:** Test migrations with sample data before production. Use separate databases for dev/staging/production.

**Production:**

- Review migration compatibility rules before making changes
- Coordinate client updates for breaking changes
- Use feature flags in reducers for gradual rollouts
- Prefer adding new tables/reducers over modifying existing ones
- Document breaking changes in a changelog for client teams

**Staged migration approach** for unsupported automatic changes:

1. **Additive changes** -- add new tables and columns
2. **Dual-write period** -- write to both old and new schema
3. **Staged rollout** -- clients read from new schema, old schema still supported
4. **Remove old schema** -- once all clients are updated

### Client Compatibility During Migrations
<!-- keywords: client\s+compat(ibility)?|active\s+connect|subscription.*continu|brief\s+interrupt|regenerate.*binding|client\s+binding -->

Active client connections and subscriptions are maintained during automatic migrations. Caveats:

- **Scheduled reducer interruptions** -- brief pauses in game loops or timers
- **Removed/changed reducers** -- runtime errors for clients calling old signatures
- **Schema-unaware clients** -- client bindings must be regenerated to reflect new tables/reducers

## Logging and Utilities
<!-- keywords: log(ging)?|timestamp|utilit(y|ies)|diagnostic -->

### Logging
<!-- keywords: log::|log::info|log::warn|logging|server.?side\s+log -->

```rust
#[reducer]
pub fn process_data(ctx: &ReducerContext, value: u32) -> Result<(), String> {
    log::info!("Processing data with value: {}", value);
    if value > 100 { log::warn!("Value {} exceeds threshold", value); }
    if value == 0 {
        log::error!("Invalid value: 0");
        return Err("Value cannot be zero".to_string());
    }
    log::debug!("Debug information: ctx.sender = {:?}", ctx.sender());
    Ok(())
}
```

Uses the standard Rust `log` crate macros. Log messages are private to the database owner and never visible to clients.

#### Log Levels
<!-- keywords: log\s+level|error!|warn!|info!|debug!|trace!|log::trace -->

| Level         | Purpose                                                     |
| ------------- | ----------------------------------------------------------- |
| `log::error!` | Errors that prevent operations from completing              |
| `log::warn!`  | Problematic situations that do not prevent execution        |
| `log::info!`  | Important application events (user actions, state changes)  |
| `log::debug!` | Detailed diagnostic info for development                    |
| `log::trace!` | Very detailed diagnostics, typically disabled in production |

#### Logging Performance
<!-- keywords: log(ging)?\s+performance|log(ging)?\s+overhead|tight\s+loop|high.?frequency|excessive\s+log -->

Minimal overhead, but avoid logging in tight loops or high-frequency operations. Use `log::debug!`/`log::trace!` for verbose output so it can be filtered in production.

#### Log Privacy and Security
<!-- keywords: log(s)?\s+(private|visible|owner)|sensitive\s+information|PII|password|auth.*token -->

Logs are only visible to the database owner, never exposed to clients. Avoid logging passwords, auth tokens, or PII.

#### Structured Logging
<!-- keywords: structured\s+log|key.?value\s+pair|log\s+analysis|format.*log -->

```rust
log::info!(
    "Credit transfer: from={:?}, to={}, amount={}",
    ctx.sender(), to_user, amount
);
```

Embed key-value pairs in log messages for searchability and cross-invocation correlation.

### Timestamp
<!-- keywords: Timestamp|point\s+in\s+time|ctx\.timestamp|sent\s+time|time\s+of\s+call -->

```rust
ctx.db.message().insert(Message {
    sender: ctx.sender(),
    text,
    sent: ctx.timestamp,
});
```

Built-in SpacetimeDB type representing a point in time (microseconds since Unix epoch). Usable as a table column type. Access the current reducer invocation time via `ctx.timestamp` on `ReducerContext`.

- **Server-authoritative** -- not derived from client clocks
- **Constant within a reducer** -- does not change between start and end of execution
- Reliable for timestamping events and time-based logic within a single reducer call
