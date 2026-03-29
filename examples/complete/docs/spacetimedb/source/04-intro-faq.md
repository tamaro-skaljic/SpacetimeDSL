# FAQ

## General

### What is SpacetimeDB?

SpacetimeDB is a database that is also a server. You upload your application logic (called a "module") directly into the database, and clients connect to it without any server in between. It is a relational database with tables, queries, and transactions, but your business logic runs inside it as stored procedures on steroids.

### How is SpacetimeDB different from a traditional backend?

In a traditional stack, you deploy a web server (Node, Django, Rails, etc.) that sits between your clients and your database. You write API endpoints, manage connection pooling, handle caching, and deploy infrastructure.

With SpacetimeDB, you skip all of that. Your module is the backend. Clients connect directly to the database, call reducers (like RPC endpoints), and subscribe to real-time data updates. No separate server, no REST API, no GraphQL layer.

```
Traditional:                    SpacetimeDB:
Client → Server → Database      Client → SpacetimeDB (database + logic)
```

### Is SpacetimeDB only for games?

No. SpacetimeDB is designed for any real-time application: games, chat apps, collaboration tools, dashboards, IoT, or anything that benefits from low-latency state synchronization. Games are a demanding use case that proves SpacetimeDB's performance. Our own MMORPG [BitCraft Online](https://bitcraftonline.com) runs entirely on SpacetimeDB. The same architecture works for any application.

### Is SpacetimeDB open source?

SpacetimeDB is source-available under the [Business Source License 1.1 (BSL)](https://github.com/clockworklabs/SpacetimeDB/blob/master/LICENSE.txt). It converts to the AGPL v3.0 (with a linking exception) after a few years. The linking exception means you are **not** required to open-source your own code if you use SpacetimeDB. You only need to contribute back changes to SpacetimeDB itself.

---

## How does SpacetimeDB compare to

### How is SpacetimeDB different from Mirror / Netcode / Photon / other networking libraries?

Networking libraries like Mirror, Netcode for GameObjects, and Photon handle the transport layer: sending messages between clients and a server you build and deploy yourself. You are still responsible for writing server logic, managing state, handling persistence, and deploying infrastructure.

SpacetimeDB replaces the entire server. Your game state lives in tables, your game logic lives in reducers, and SpacetimeDB automatically synchronizes state to clients in real-time. You do not write networking code, serialization code, or deploy servers.

|                      | Networking Libraries    | SpacetimeDB                 |
| -------------------- | ----------------------- | --------------------------- |
| **Server logic**     | You write and deploy it | Runs inside the database    |
| **State management** | You manage it           | Tables with auto-sync       |
| **Persistence**      | You add a database      | Built in                    |
| **Real-time sync**   | You implement it        | Automatic via subscriptions |
| **Infrastructure**   | You deploy and scale it | Managed or self-hosted      |

### How is SpacetimeDB different from Firebase / Supabase?

Firebase and Supabase are Backend-as-a-Service platforms. They give you a database with an API layer on top, but your application logic still runs elsewhere (cloud functions, edge functions, or your own server). Complex business logic is awkward to express as database triggers or serverless functions.

SpacetimeDB lets you write your entire application as a module in a real programming language (Rust) that runs inside the database. You get full transactional guarantees, direct table access, and real-time subscriptions without the cold starts, execution limits, or awkward abstractions of serverless functions.

### How is SpacetimeDB different from a regular database (PostgreSQL, MySQL)?

A traditional database stores data and lets you query it. Your application logic runs in a separate server that connects to the database over the network.

SpacetimeDB holds all data in memory for sub-microsecond access, runs your application logic inside the database as WebAssembly modules, and pushes real-time updates to connected clients automatically. It is purpose-built for real-time applications, not batch processing or analytics. For certain benchmarks, SpacetimeDB can be between 100 and 1000 times faster than a traditional database.

---

## Core Concepts

### What is a "module"?

A module is your application's server-side code compiled to WebAssembly. It defines your tables, reducers, views, and procedures. It runs inside the database. Think of it as your entire backend in a single deployable unit.

### What are tables?

Tables are the core data storage in SpacetimeDB. You define them in your module using your language's type system. Tables support primary keys, unique constraints, and indexes. Clients can subscribe to tables and receive real-time updates when rows change.

### What are reducers?

Reducers are functions that modify your database state. They run inside a database transaction: either all changes commit or none do. Clients call reducers through auto-generated, type-safe bindings. Think of them as transactional RPC endpoints.

### What are views?

Views are read-only functions that compute derived data from your tables. They are like SQL views but written in your module's language. Clients can subscribe to views just like tables, and they update automatically when the underlying data changes.

### What are procedures?

Procedures are similar to reducers but with additional capabilities. They can make HTTP requests to external services and manually manage transactions. Clients can also call procedures over HTTP. Procedures are currently in beta. Use reducers for most cases; use procedures when you need to interact with the outside world. In the future, procedures will be configurable as HTTP endpoints.

---

## Architecture

### How does data persistence work?

SpacetimeDB holds all data in memory for fast access, but persists everything to a commit log (similar to a write-ahead log). On restart, the database replays the commit log to recover its exact state. You get the speed of in-memory computing with the durability of a traditional database.

### How does real-time sync work?

SpacetimeDB evaluates subscriptions and pushes incremental updates whenever the underlying data changes.

### Are reducers like REST endpoints?

Reducers are more like transactional RPC calls. A reducer runs inside a database transaction, and either all changes commit or none do. Unlike REST, there is no HTTP overhead and no JSON serialization.

### What happens if a reducer fails?

The entire transaction rolls back. No partial updates, no corrupted state. The database remains exactly as it was before the reducer was called. You can throw errors or return `Err` freely. SpacetimeDB handles the cleanup.

---

## Authentication & Authorization

### How do I implement auth in my app?

SpacetimeDB uses [OpenID Connect (OIDC)](https://openid.net/developers/how-connect-works/) for authentication. Every reducer call includes the caller's `Identity`, which you can use for authorization logic in your module. You have several options for identity providers:

- **[SpacetimeAuth](/docs/core-concepts/authentication/spacetimeauth/)**: A fully managed OIDC provider built specifically for SpacetimeDB. The easiest way to get started.
- **Third-party providers**: Any OIDC-compliant provider works, including [Auth0](https://auth0.com/), [Clerk](https://clerk.com/), [Keycloak](https://www.keycloak.org/), Google, GitHub, and others.

See [Authentication](/docs/core-concepts/authentication) for full details.

### Can I use Identity as a 1-to-1 mapping with users?

Yes, and this is the recommended approach. Each authenticated user receives a stable `Identity` that persists across sessions. You can store user profiles keyed by `Identity` and use it as your primary user identifier. If you use an OIDC provider (such as SpacetimeAuth, Auth0, or Clerk), the `Identity` is derived from the provider's tokens, so the same user always gets the same `Identity`.

### Can I store passwords in a private table and have users log in by calling a reducer?

This is not recommended. While SpacetimeDB 2.0 no longer exposes reducer arguments to subscribers (the old reducer callback system has been replaced by [Event Tables](/docs/tables/event-tables)), storing and verifying passwords in your module means implementing your own authentication logic, which is error-prone and unnecessary. Use OIDC-based authentication instead, where the identity provider handles credential verification and issues tokens that SpacetimeDB validates.

### How do I create a new localhost identity?

Send a `POST /v1/identity` request to your SpacetimeDB instance. The response includes a new identity and token. See the [HTTP API reference](/docs/http/identity#post-v1identity) for details.

> **warning**
>
> Identities issued by SpacetimeDB acting as its own identity provider are tied to a single token that does not expire. If that token is lost, there is no way to recover or re-authenticate as that identity. This approach is recommended for **development only**. For production applications, use an external OIDC provider (SpacetimeAuth, Auth0, Clerk, etc.) which provides proper token lifecycle management.

---

## Schema & Migrations

### How do I handle schema migrations?

When you publish an updated module, SpacetimeDB compares the new schema with the existing one and performs automatic migrations for compatible changes (adding tables, adding columns with defaults, etc.). See [Automatic Migrations](/docs/databases/automatic-migrations) for details on what changes are supported.

### How do I add a column to an existing table?

SpacetimeDB supports adding new columns to the end of a table, provided the new columns have [default values](/docs/tables/default-values). This is handled automatically when you republish your module. For more complex changes (reordering columns, changing types), use the [incremental migration pattern](/docs/databases/incremental-migrations): create a new table with the desired schema and lazily migrate rows from the old table as they are accessed.

### How do I remove a column from an existing table?

Removing columns is not supported through automatic migration. Use [incremental migrations](/docs/databases/incremental-migrations) instead: create a new table without the column and migrate data incrementally.

### How do I add a new table?

You can simply add the table definition to your module and republish. SpacetimeDB creates new tables automatically during publish.

### How do I add or remove indexes?

You can add or remove indexes by updating the table definition and republishing. Note that removing an index may break client subscription queries that depend on it.

---

## Deployment & Operations

### Can I update my module without downtime?

Yes. SpacetimeDB hot-swaps the module code. Connected clients are not disconnected. They seamlessly continue with the new logic. This is possible because all state lives in tables, not in the server process.

### Is there a size limit for databases?

SpacetimeDB holds all data in memory, so the practical limit is the available RAM on the host. On Maincloud, resource limits depend on your plan. For self-hosted deployments, you control the hardware.

### How do I build a room-based or match-based game?

SpacetimeDB databases are lightweight and fast to create. The recommended pattern is to use an external orchestration service that creates and destroys SpacetimeDB databases for each room or match. Each database runs an independent instance of your module with its own state.

---

## Troubleshooting

### I got a weird error when compiling my module

Make sure you are using the same version of the SpacetimeDB module library (e.g., `spacetimedb` Rust crate) as the version of the SpacetimeDB host you are publishing to. Version mismatches between the module library and the host are a common source of confusing compilation or publish errors.
