# The Database Module

A **module** is a collection of functions and schema definitions written in Rust. Modules define the structure of your database and the server-side logic that processes and handles client requests.

A **database** is a running instance of a module. While a module is the code you write (schema and reducers), a database is the actual deployed entity running on a SpacetimeDB **host** with stored data and active connections.

## Module vs Database

Understanding this distinction is important:

- A **module** is the code you write; it defines your schema (tables) and business logic (reducers, procedures, and views). Modules are compiled to WebAssembly and deployed to SpacetimeDB.
- A **database** is a *running instance* of a module; it has the module's schema and logic, plus actual stored data.

You can deploy the same module to multiple databases (e.g. separate environments for testing, staging, production), each with its own independent data. When you update your module code and re-publish, SpacetimeDB will update the database's schema/logic — the existing data remains (though for complicated schema changes you may need to handle migrations carefully).

## What's in a Module?

A module contains:

- **[Tables](/docs/tables)** - Define your data structure and storage.
- **[Reducers](/docs/functions/reducers)** - Server-side functions that modify your data transactionally.
- **[Procedures](/docs/functions/procedures)** - Functions that can perform external operations like HTTP requests and return results.
- **[Views](/docs/functions/views)** - Read-only computed queries over your data.

The logic is contained within these three categories of server-side functions: reducers (transactional state changes), procedures (functions with external capabilities), and views (read-only queries).

## Supported Languages

Rust is fully supported for server modules. Rust is a great choice for performance-critical applications.

- The Rust Module SDK docs are [hosted on docs.rs](https://docs.rs/spacetimedb/latest/spacetimedb/).
- [Rust Quickstart Guide](/docs/quickstarts/rust)

## Database Names

When you publish a module, you give the database a name. Database names must match the regex `/^[a-z0-9]+(-[a-z0-9]+)*$/`, i.e. only lowercase ASCII letters and numbers, separated by dashes.

**Examples of valid names:**

- `my-game-server`
- `chat-app-production`
- `test123`

Each database also receives a unique **identity** (a hex string) when created.

## Schema Migrations

When you republish to an existing database, SpacetimeDB attempts to automatically migrate the schema. For details on what changes are supported and migration strategies:

- [Automatic Migrations](/docs/databases/automatic-migrations) - Learn which schema changes are safe, breaking, or forbidden.
- [Incremental Migrations](/docs/databases/incremental-migrations) - Advanced pattern for complex schema changes.

## Learning Path

### Getting Started

If you're new to SpacetimeDB, follow this recommended learning path:

1. **[Define Tables](/docs/tables)** - Structure your data with tables, columns, and indexes
2. **[Write Reducers](/docs/functions/reducers)** - Create transactional functions that modify your database

### Core Concepts

Once you have the basics down, explore these essential topics:

- **[Error Handling](/docs/functions/reducers/error-handling)** - Handle errors gracefully in reducers
- **[Lifecycle Reducers](/docs/functions/reducers/lifecycle)** - Respond to system events like initialization and client connections
- **[Automatic Migrations](/docs/databases/automatic-migrations)** - Understand how schema changes work
- **[Logging](/docs/how-to/logging)** - Debug and monitor your module with logging

### Advanced Features

Ready to level up? Dive into these advanced capabilities:

- **[Procedures](/docs/functions/procedures)** - Make HTTP requests and interact with external services
- **[Views](/docs/functions/views)** - Create computed, subscribable queries
- **[Schedule Tables](/docs/tables/schedule-tables)** - Schedule reducers to run at specific times
- **[Incremental Migrations](/docs/databases/incremental-migrations)** - Handle complex schema changes

## Next Steps

- Learn about [Tables](/docs/tables) to define your database schema
- Create [Reducers](/docs/functions/reducers) to modify database state
