
# SpacetimeDSL

## SpacetimeDB Basics Through the DSL Lens

> Source: `SpacetimeDSL.md`, lines 54-370

### Table Definitions

#### Vanilla SpacetimeDB Table Macros

- [ ] Define a table with `#[table(accessor = <name>, public)]` on a `pub struct`
- [ ] Import core types via `use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp}`
- [ ] Mark a column as primary key with `#[primary_key]`
- [ ] Mark a column as auto-incrementing with `#[auto_inc]` combined with `#[primary_key]`
- [ ] Mark a column as unique with `#[unique]`
- [ ] Add a single-column B-Tree index with `#[index(btree)]`
- [ ] Use `Identity` as a primary key type on a table column
- [ ] Use `u64` as a primary key type with `#[auto_inc]`
- [ ] Use `Timestamp` as a column type
- [ ] Use `String` as a column type
- [ ] Use `bool` as a column type

#### SpacetimeDSL Table Macros

- [ ] Import the DSL prelude via `use spacetimedsl::prelude::*`
- [ ] Annotate a table struct with `#[dsl(plural_name = <name>, method(update = true, delete = true))]`
- [ ] Set `method(update = true)` to generate update methods for a table
- [ ] Set `method(delete = true)` to generate delete methods for a table
- [ ] Set `method(update = false)` to suppress update method generation
- [ ] Annotate a primary key column with `#[create_wrapper]` to generate a wrapper type
- [ ] Annotate a unique column with `#[create_wrapper]` to generate a wrapper type
- [ ] Annotate an auto-inc column with `#[create_wrapper]` to generate a wrapper type
- [ ] Reuse an existing wrapper type on a column with `#[use_wrapper(UserId)]`
- [ ] Combine `#[index(btree)]` with `#[use_wrapper(UserId)]` on a column

#### Table Options

- [ ] Define a private table with `#[table(accessor = my_table)]` -- private is the default visibility
- [ ] Define a public table with `#[table(accessor = my_table, public)]` -- clients can subscribe

#### Column Attributes (Vanilla SpacetimeDB)

- [ ] Apply `#[primary_key]` to auto-index a column and enable `.find()`
- [ ] Apply `#[auto_inc]` alongside `#[primary_key]` for auto-increment behavior
- [ ] Apply `#[unique]` to enforce a unique constraint with auto-indexing
- [ ] Apply `#[index(btree)]` for B-Tree indexed range and filter queries
- [ ] Use table-level `index(accessor = ..., btree(columns = [...]))` for multi-column indices

#### Column Attributes (SpacetimeDSL Additions)

- [ ] Apply `#[create_wrapper]` to generate a wrapper type named after the column
- [ ] Apply `#[create_wrapper(Name)]` to generate a wrapper type with a custom name
- [ ] Apply `#[use_wrapper(Name)]` to reuse an existing wrapper type
- [ ] Apply `#[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]` to define a foreign key constraint
- [ ] Apply `#[referenced_by(path = self, table = position)]` to mark a primary key as referenced by another table's foreign key

### ReducerContext API

#### Vanilla SpacetimeDB ReducerContext

- [ ] Call `ctx.sender()` to obtain the `Identity` of the caller -- method call, not field access
- [ ] Access `ctx.timestamp` to obtain the current `Timestamp` -- field access, not method call
- [ ] Access `ctx.db` to obtain the database handle -- field access, not method call
- [ ] Call `ctx.rng()` to obtain a deterministic RNG -- method call, not field access

#### SpacetimeDSL ReducerContext

- [ ] Initialize DSL handle with `let dsl = spacetimedsl::dsl(ctx)`
- [ ] Call `dsl.ctx().sender()` to obtain the `Identity` of the caller
- [ ] Access `dsl.ctx().timestamp` to obtain the current `Timestamp`
- [ ] Access `dsl.ctx().db` to obtain the database handle
- [ ] Call `dsl.ctx().rng()` to obtain a deterministic RNG

### Reducers

- [ ] Annotate a function with `#[reducer]` to define a reducer
- [ ] Accept `ctx: &ReducerContext` as the first parameter -- immutable reference
- [ ] Return `Result<(), SpacetimeDSLError>` from a reducer
- [ ] Construct `SpacetimeDSLError::Error("message".to_string())` to return a custom error
- [ ] Call `spacetimedsl::dsl(ctx)` at the start of a reducer body
- [ ] Call `dsl.create_message(CreateMessage { ... })?` to insert a row using a generated create struct
- [ ] Pass `dsl.ctx().sender()` as a field value in a create struct

### Lifecycle Reducers

- [ ] Define an init reducer with `#[reducer(init)]` -- called when the module is first published
- [ ] Define a client-connected reducer with `#[reducer(client_connected)]`
- [ ] Define a client-disconnected reducer with `#[reducer(client_disconnected)]`
- [ ] Accept `ctx: &ReducerContext` and return `Result<(), SpacetimeDSLError>` in lifecycle reducers
- [ ] Call `spacetimedsl::dsl(ctx)` inside lifecycle reducers

### Scheduled Tables

- [ ] Define a scheduled table with `#[table(accessor = cleanup_job, scheduled(cleanup_expired))]`
- [ ] Include a `scheduled_at: ScheduleAt` column in a scheduled table struct
- [ ] Import `ScheduleAt` from `spacetimedb`
- [ ] Annotate the scheduled table with `#[dsl(plural_name = cleanup_jobs, method(update = false))]`
- [ ] Define the scheduled reducer accepting the job struct as `fn cleanup_expired(ctx: &ReducerContext, job: CleanupJob)`
- [ ] Compute a future timestamp with `dsl.ctx().timestamp()? + std::time::Duration::from_millis(delay_ms)`
- [ ] Schedule a job by calling `dsl.create_cleanup_job(CreateCleanupJob { scheduled_at: ScheduleAt::Time(future_time), ... })?`
- [ ] Use `ScheduleAt::Time(future_time)` to schedule at a specific timestamp
- [ ] Cancel a scheduled job by calling `dsl.delete_cleanup_job_by_scheduled_id(CleanupJobScheduledId::new(job_id))?`
- [ ] Construct a wrapper ID with `CleanupJobScheduledId::new(job_id)` for deletion

### Procedures

- [ ] Define a procedure with `#[spacetimedb::procedure]`
- [ ] Accept `ctx: &mut ProcedureContext` as the parameter -- mutable reference
- [ ] Return `Result<(), SpacetimeDSLError>` from a procedure
- [ ] Call `ctx.try_with_tx(|ctx| { ... })` to access a transactional context inside a procedure
- [ ] Call `spacetimedsl::dsl(ctx)` inside the `try_with_tx` closure for DSL database access

### Views

- [ ] Define a view with `#[spacetimedb::view(accessor = my_view, public)]`
- [ ] Accept `ctx: &ViewContext` as the parameter
- [ ] Call `spacetimedsl::read_only_dsl(ctx)` inside a view -- never use `dsl(ctx)` in views
- [ ] Access the database directly via `ctx.db` inside a view for queries

### Custom Types

- [ ] Derive `SpacetimeType` on non-table structs with `#[derive(SpacetimeType, Clone, Debug, PartialEq)]`
- [ ] Derive `SpacetimeType` on enums with `#[derive(SpacetimeType, Clone, Debug, PartialEq)]`
- [ ] Define a custom struct type using `SpacetimeType` for use as a table column or reducer parameter
- [ ] Define a custom enum type using `SpacetimeType` with unit variants

### Logging

- [ ] Call `log::trace!("...")` for detailed trace output
- [ ] Call `log::debug!("...")` for debug information
- [ ] Call `log::info!("...")` for informational messages
- [ ] Call `log::warn!("...")` for warnings
- [ ] Call `log::error!("...")` for error messages
- [ ] Import the log module via `use spacetimedb::log`

## DSL Core Concepts

> Source: `SpacetimeDSL.md`, lines 371-455

### Creating the DSL Context

- [ ] Call `spacetimedsl::dsl(ctx)` to create a DSL context with write access for reducers
- [ ] Call `spacetimedsl::read_only_dsl(ctx)` to create a DSL context with read-only access for views

### Accessing the Underlying Context

- [ ] Call `dsl.ctx()` to obtain a `&ReducerContext` reference
- [ ] Access `dsl.ctx().sender()` to retrieve the sender identity
- [ ] Access `dsl.ctx().timestamp` to retrieve the current timestamp

### Best Practice: Create DSL Once

- [ ] Create the DSL once at reducer start with `let dsl = spacetimedsl::dsl(ctx)`
- [ ] Pass `&DSL` as a reference to helper functions

### Helper Function Signatures

- [ ] Define helper functions with concrete signature `fn helper(dsl: &DSL<'_, ReducerContext>) -> Result<(), SpacetimeDSLError>`
- [ ] Define helper functions with trait-bound signature `fn helper<T: ReadContext>(dsl: &DSL<'_, T>) -> Result<(), SpacetimeDSLError>` -- enables reuse across both reducers and views
- [ ] Use `WriteContext` trait bound for write operations in reducers
- [ ] Use `ReadContext` trait bound for read-only operations in views

### Method Configuration

#### Attribute Syntax

- [ ] Specify `method(...)` configuration inside the `#[dsl(...)]` attribute
- [ ] Set `method(update = true, delete = true)` for explicit full configuration
- [ ] Set `method(update = false)` to disable updates while `delete` defaults to `true`

#### `update` Parameter

- [ ] Set `update = true` on tables that have at least one `pub` field
- [ ] Set `update = true` on tables that have a `modified_at` or `updated_at` column
- [ ] Set `update = false` on tables where all fields are private and no `modified_at`/`updated_at` column exists

#### `delete` Parameter

- [ ] Set `delete = true` on tables referenced by a foreign key with `on_delete = Delete`
- [ ] Set `delete = true` on tables where row deletion is needed in general
- [ ] Set `delete = false` for audit tables -- no delete DSL methods are generated

#### Compile-Time Validation

- [ ] Trigger a compilation error by omitting the `update` parameter from `method(...)`
- [ ] Require matching method config for hooks -- `hook(after(update))` needs `method(update = true)`

## Table Definition with DSL

> Source: `SpacetimeDSL.md`, lines 456-532

### Full Syntax

- [ ] Annotate a struct with `#[dsl(...)]` to enable DSL code generation
- [ ] Set `plural_name = entities` inside `#[dsl]` to control multi-row method name generation
- [ ] Set `method(update = true, delete = true)` inside `#[dsl]` to generate update and delete methods
- [ ] Set `unique_index(name = some_index)` inside `#[dsl]` to declare a unique index by name
- [ ] Set `hook(before(insert, update, delete), after(insert, update, delete))` inside `#[dsl]` to register before and after hooks for insert, update, and delete operations
- [ ] Pair `#[dsl(...)]` directly above `#[table(...)]` on a struct definition
- [ ] Configure `#[table(accessor = entity, public)]` to set the table accessor name and visibility

### Short Form

- [ ] Use the short form `#[dsl]` rather than `#[spacetimedsl::dsl]` in all code

### Pairing `#[dsl]` with `#[table]`

- [ ] Place `#[dsl]` directly above `#[table]` on the same struct
- [ ] Combine `plural_name` and `method(update = true, delete = true)` in a single `#[dsl]` attribute

### `plural_name` Requirement

- [ ] Provide the required `plural_name` parameter inside `#[dsl]`
- [ ] Generate `get_all_{plural_name}()` methods via `plural_name` -- e.g., `get_all_entities()`
- [ ] Generate `count_of_all_{plural_name}()` methods via `plural_name` -- e.g., `count_of_all_entities()`
- [ ] Generate `delete_{plural_name}_by_{column}()` methods via `plural_name` -- e.g., `delete_entities_by_status()`

### Multiple `#[dsl]` + `#[table]` on Same Struct

- [ ] Apply multiple `#[dsl]` + `#[table]` pairs on a single struct to generate separate tables sharing the same struct definition
- [ ] Assign distinct `plural_name` values for each `#[dsl]` pair -- e.g., `modules1` and `modules2`
- [ ] Assign distinct `accessor` values for each `#[table]` pair -- e.g., `module1` and `module2`
- [ ] Declare distinct `unique_index(name = ...)` values per `#[dsl]` pair -- e.g., `database_and_parent_id_and_name` and `database_and_name_and_parent_id`
- [ ] Configure `index(accessor = ..., btree(columns = [...]))` inside `#[table]` to define a btree index with specific column order
- [ ] Vary btree column order across table instances -- e.g., `[database_id, parent_id, name]` vs `[database_id, name, parent_id]`
- [ ] Annotate a column with `#[primary_key]` to designate it as the primary key
- [ ] Annotate a column with `#[auto_inc]` to enable auto-increment on that column
- [ ] Annotate a column with `#[create_wrapper]` to include the column in the generated wrapper type
- [ ] Share wrapper types across all table instances of the same struct

## Wrapper Types

> Source: `SpacetimeDSL.md`, lines 533-636

### Generated Wrapper Types

#### Creating and Reusing Wrappers

- [ ] Annotate a column with `#[create_wrapper]` to generate a wrapper type using the default naming convention `{SingularTableNamePascalCase}{ColumnNamePascalCase}`
- [ ] Annotate a column with `#[create_wrapper(EntityId)]` to generate a wrapper type with a custom name
- [ ] Annotate a column with `#[use_wrapper(EntityId)]` to reuse an existing wrapper type from the same module
- [ ] Annotate a column with `#[use_wrapper(crate::entity::EntityId)]` to reuse a wrapper type from another module
- [ ] Apply `#[create_wrapper]` or `#[use_wrapper]` to every `#[primary_key]`, `#[unique]`, and `#[index]` column

#### Wrapper Trait

- [ ] Implement the `Wrapper<WrappedType, WrapperType>` trait on all generated wrapper types
- [ ] Derive `Default`, `Clone`, `PartialEq`, `PartialOrd`, `spacetimedb::SpacetimeType`, and `Display` on all wrapper types
- [ ] Call `Wrapper::new(value)` to construct a wrapper from the inner type
- [ ] Call `Wrapper::value(&self)` to extract the inner type from a wrapper

#### Option<T> with Wrapper Types

- [ ] Declare an `Option<u128>` column with `#[use_wrapper(crate::entity::EntityId)]` to wrap optional values
- [ ] Declare an `Option<String>` column with `#[create_wrapper]` to wrap optional string values
- [ ] Call the getter on an optional wrapped column to receive `Option<WrapperType>`
- [ ] Call the setter with `None` to clear an optional wrapped column
- [ ] Call the setter with a `WrapperType` value to set an optional wrapped column via `impl Into<Option<WrapperType>>`
- [ ] Call the setter with a reference to the source entity to auto-extract and set the wrapped value

#### Lookup by Wrapper Type

- [ ] Call `dsl.get_entity_by_id(EntityId::new(123))` to look up an entity using a constructed wrapper
- [ ] Call `dsl.get_entity_by_id(&some_entity)` to look up an entity by auto-extracting the primary key from a reference
- [ ] Call `dsl.get_entity_by_id(some_entity.get_id())` to look up an entity using an explicit getter

## Create Structs & Create Methods

> Source: `SpacetimeDSL.md`, lines 637-714

### Field Inclusion

- [ ] Include all non-auto-defaulted fields in `Create{SingularTableNamePascalCase}` structs regardless of visibility (`pub` or private)

### Auto-Defaulted Field Exclusions

- [ ] Exclude `#[auto_inc]` columns from the create struct and default their value to `0`
- [ ] Exclude `created_at: Timestamp` columns from the create struct and default to `ctx.timestamp`
- [ ] Exclude `inserted_at: Timestamp` columns from the create struct and default to `ctx.timestamp`
- [ ] Exclude `modified_at: Option<Timestamp>` columns from the create struct and default to `None`
- [ ] Exclude `updated_at: Option<Timestamp>` columns from the create struct and default to `None`
- [ ] Exclude `modified_at: Timestamp` columns from the create struct and default to `ctx.timestamp`
- [ ] Exclude `updated_at: Timestamp` columns from the create struct and default to `ctx.timestamp`
- [ ] Recognize `created_at` and `inserted_at` as interchangeable aliases
- [ ] Recognize `modified_at` and `updated_at` as interchangeable aliases

### Timestamp Protection

- [ ] Make `created_at`/`inserted_at` columns always private with no generated setter -- prevents changes after initial creation
- [ ] Make `modified_at`/`updated_at` columns always private with no generated setter -- DSL exclusively controls these values on every update operation

### Usage Patterns

#### Create with All Fields Auto-Defaulted

- [ ] Define a struct with `#[primary_key]`, `#[auto_inc]`, `#[create_wrapper]`, `created_at: Timestamp`, and `modified_at: Option<Timestamp>`
- [ ] Call `dsl.create_entity()` with no arguments when all fields are auto-defaulted -- `Create{Name}` struct has no fields

#### Create with Explicit Fields

- [ ] Define a struct with `#[primary_key]` and `#[create_wrapper]` on a non-`#[auto_inc]` column
- [ ] Call `dsl.create_config(CreateConfig { ... })` passing a `Create{Name}` struct with explicit field values
- [ ] Pass a wrapper type value via `ConfigId::new(0)` for the `id` field in `CreateConfig`

#### Create with Wrapper Types

- [ ] Call `dsl.create_player(CreatePlayer { ... })` passing wrapper type values for identity, primitive values for `name`, and enum values for `login_status`
- [ ] Use `ConfigId::new(dsl.ctx().sender())` to wrap a context-derived value in a wrapper type
- [ ] Use `String::new()` as a default empty string field value in a create struct
- [ ] Pass an enum variant `LoginStatus::LoggedIn` as a field value in a create struct

## Get Methods (Read)

> Source: `SpacetimeDSL.md`, lines 715-781

### By Primary Key

- [ ] Call `dsl.get_entity_by_id()` with a table reference to auto-extract the primary key
- [ ] Call `dsl.get_entity_by_id()` with an explicit ID obtained via `.get_id()`
- [ ] Call `dsl.get_entity_by_id()` with a manually constructed ID wrapper via `EntityId::new()`
- [ ] Return `Result<{Table}, SpacetimeDSLError>` from primary key lookups
- [ ] Raise `NotFoundError` when a primary key lookup finds no matching row

### By Unique Column

- [ ] Call `dsl.get_identifier_by_entity_id()` with a table reference to look up by unique foreign-key column
- [ ] Call `dsl.get_identifier_by_value()` with a string literal to look up by unique value column
- [ ] Return `Result<{Table}, SpacetimeDSLError>` from unique column lookups

### By BTree Index

- [ ] Call `dsl.get_circles_by_player_id()` to retrieve an iterator over rows matching a BTree-indexed column
- [ ] Iterate over BTree index results using a `for` loop
- [ ] Call `.collect_vec()` on a BTree index iterator to collect results into a `Vec<{Table}>`

### Get All

- [ ] Call `dsl.get_all_entities()` to retrieve an iterator over all rows in a table
- [ ] Iterate over all-rows results using a `for` loop

### Count

- [ ] Call `dsl.count_of_all_entities()` to return a `u64` count of all rows in a table

### By Unique Multi-Column Index

- [ ] Call `dsl.get_entity_relationship_by_parent_child_entity_id()` with two ID references to look up by a two-column unique index
- [ ] Call `dsl.get_module1_by_database_and_parent_id_and_name()` with three arguments to look up by a three-column unique index
- [ ] Return `Result<{Table}, SpacetimeDSLError>` from unique multi-column index lookups

## Update Methods

> Source: `SpacetimeDSL.md`, lines 782-818

### Update Method Generation

- [ ] Enable update method generation with `method(update = true)` in `#[dsl]`
- [ ] Require at least one `pub` field or a `modified_at`/`updated_at` column when using `update = true`

#### By Primary Key

- [ ] Retrieve a mutable entity with `dsl.get_{table}_by_{primary_key}()` returning `Result<{Table}, SpacetimeDSLError>`
- [ ] Mutate a field on the retrieved entity with `.set_{field}()` setter method
- [ ] Apply the update with `dsl.update_{table}_by_{primary_key}()` returning `Result<{Table}, SpacetimeDSLError>`

#### By Unique Multi-Column Index

- [ ] Update a row by unique multi-column index with `dsl.update_{table}_by_{col1}_{col2}()` passing the modified struct

#### Automatic Timestamp Refresh

- [ ] Set `modified_at: Option<Timestamp>` to `Some(ctx.timestamp)` on every update
- [ ] Set `updated_at: Option<Timestamp>` to `Some(ctx.timestamp)` on every update
- [ ] Set `modified_at: Timestamp` to `ctx.timestamp` on every update
- [ ] Set `updated_at: Timestamp` to `ctx.timestamp` on every update

## Delete Methods

> Source: `SpacetimeDSL.md`, lines 819-874

### Configuration

- [ ] Enable delete methods with `method(delete = true)` in the `#[dsl]` attribute
- [ ] Rely on default behavior where `delete` is `true` if not specified in `#[dsl]`

### By Primary Key

- [ ] Call `dsl.delete_entity_by_id(&entity)` to delete a row by its primary key
- [ ] Receive `Result<DeletionResult, SpacetimeDSLError>` from a primary-key deletion

### By Unique Column

- [ ] Call `dsl.delete_identifier_by_entity_id(&entity)` to delete a row by a unique column
- [ ] Receive `Result<DeletionResult, SpacetimeDSLError>` from a unique-column deletion

### By BTree Index (Many)

- [ ] Call `dsl.delete_tests_by_btree_index(some_value)` to delete multiple rows matching a BTree index
- [ ] Receive `Result<DeletionResult, SpacetimeDSLError>` from a BTree-index deletion

### `DeletionResult`

- [ ] Access `DeletionResult.table_name` as `Box<str>` to identify the deleted table
- [ ] Access `DeletionResult.one_or_multiple` as `OneOrMultiple` to distinguish single vs. batch deletions
- [ ] Access `DeletionResult.entries` as `Vec<DeletionResultEntry>` to inspect each deleted entry

### `DeletionResultEntry`

- [ ] Access `DeletionResultEntry.table_name` as `Box<str>` to identify the entry's table
- [ ] Access `DeletionResultEntry.column_name` as `Box<str>` to identify the entry's column
- [ ] Access `DeletionResultEntry.strategy` as `OnDeleteStrategy` to identify the cascade strategy used
- [ ] Access `DeletionResultEntry.row_value` as `Box<str>` to retrieve the deleted row's value
- [ ] Access `DeletionResultEntry.child_entries` as `Vec<DeletionResultEntry>` to traverse nested cascade tracking

### CSV Audit Trail

- [ ] Call `result.to_csv()` to produce a CSV-formatted audit trail of the deletion
- [ ] Parse CSV columns `entry_id`, `parent_entry_id`, `table_name`, `column_name`, `strategy`, `row_value` from the audit output

## Accessor Methods (Getters/Setters/Mut-Getters)

> Source: `SpacetimeDSL.md`, lines 875-957

### Getters

- [ ] Generate getters for all columns when `#[dsl]` is applied
- [ ] Return wrapper types by clone from a getter (e.g., `fn get_id(&self) -> EntityId`)
- [ ] Return primitive types by reference from a getter (e.g., `fn get_name(&self) -> &String`)
- [ ] Return `Option<WrapperType>` by clone from a getter (e.g., `fn get_wrapped_option(&self) -> Option<EntityId>`)
- [ ] Return `&Option<T>` by reference from a getter when the inner type has no wrapper (e.g., `fn get_modified_at(&self) -> &Option<Timestamp>`)

### Setters

- [ ] Generate setters only for `pub` (non-private) columns
- [ ] Mark a field as `pub` to enable setter generation (e.g., `pub name: String`)
- [ ] Call a setter with the pattern `entity.set_name("new_name".to_string())`
- [ ] Omit setter generation for private fields (e.g., `id: u128`, `created_at: Timestamp`)

### Mut-Getters

- [ ] Generate mut-getters for `pub` columns that do not have a wrapper type
- [ ] Call a mut-getter with the pattern `entity.get_tags_mut()` to obtain `&mut Vec<String>`
- [ ] Mutate the returned mutable reference in place (e.g., `tags.push("new_tag".to_string())`)
- [ ] Omit mut-getter generation for wrapped columns -- use the setter instead

### impl Into Flexible Input Patterns

- [ ] Accept `impl Into` on DSL methods and setters for flexible input types
- [ ] Pass a reference to a table struct to auto-extract its primary key (e.g., `dsl.get_entity_by_id(&player)`)
- [ ] Pass an explicit wrapper value to a DSL method (e.g., `dsl.get_entity_by_id(player.get_id())`)
- [ ] Use the explicit getter for non-primary-key column lookups (e.g., `dsl.get_tests_by_wrapped_index(player.get_id())`)
- [ ] Pass either the wrapper type or the raw inner type to a setter via `impl Into` (e.g., `entity.set_some_field(value)`)

### Field Privacy

- [ ] Make all fields private automatically when `#[dsl]` is applied

## Foreign Keys & Referential Integrity

> Source: `SpacetimeDSL.md`, lines 958-1130

### Declaration

- [ ] Annotate the referenced table's primary key column with `#[referenced_by(path = self, table = position)]`
- [ ] Annotate the referencing table's foreign key column with `#[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]`
- [ ] Specify `path` parameter as `path = self` for same-module references
- [ ] Specify `path` parameter as `path = crate` for crate-root references
- [ ] Specify `path` parameter as `path = crate::entity` for specific-module references
- [ ] Specify `table` parameter as the SpacetimeDB table accessor name
- [ ] Specify `column` parameter as the primary key column of the referenced table
- [ ] Specify `on_delete` parameter with one of `Error`, `Delete`, `SetZero`, or `Ignore`

### Pairing Requirement

- [ ] Pair every `#[foreign_key]` with a corresponding `#[referenced_by]` on the referenced table's primary key

### OnDeleteStrategy

#### Error

- [ ] Use `on_delete = Error` to prevent deletion when referenced rows exist
- [ ] Propagate the `ReferenceIntegrityViolation` error from the reducer using `?`

#### Delete (Cascade)

- [ ] Use `on_delete = Delete` to cascade-delete referencing rows
- [ ] Enable `method(delete = true)` on the referencing table's `#[dsl]` attribute when using cascade delete

#### SetZero

- [ ] Use `on_delete = SetZero` to set the foreign key column to `0` on parent deletion -- numeric types only
- [ ] Enable `method(update = true)` on the referencing table's `#[dsl]` attribute when using `SetZero`
- [ ] Declare the foreign key column as `pub` when using `SetZero` -- required so a setter exists

#### Ignore

- [ ] Use `on_delete = Ignore` to allow dangling references on deletion -- intended for audit logs or append-only tables

### Full Foreign Key Example

- [ ] Apply multiple `#[referenced_by]` attributes on a single primary key column to reference multiple child tables
- [ ] Combine `#[primary_key]`, `#[auto_inc]`, `#[create_wrapper(EntityId)]`, and `#[referenced_by]` on the same column
- [ ] Combine `#[primary_key]`, `#[use_wrapper(EntityId)]`, and `#[foreign_key]` on the same column
- [ ] Combine `#[index(btree)]`, `#[use_wrapper(PlayerId)]`, and `#[foreign_key]` on a non-primary-key column
- [ ] Declare `method(update = true, delete = true)` in `#[dsl]` for tables participating in foreign key relationships

### Self-Referencing Tables

- [ ] Apply `#[referenced_by]` and `#[foreign_key]` pointing to the same table for self-referencing relationships
- [ ] Use `path = crate::entity` in both `#[referenced_by]` and `#[foreign_key]` for cross-module self-references
- [ ] Combine `#[use_wrapper(EntityRelationship3Id)]` with `#[foreign_key(..., on_delete = SetZero)]` on a self-referencing column

### Multiple Foreign Keys to Same Table

- [ ] Define multiple columns each with `#[foreign_key]` referencing the same target table
- [ ] Use different `on_delete` strategies on separate foreign key columns referencing the same table (e.g., `Error` and `Delete`)
- [ ] Combine `unique_index(name = parent_child_entity_id)` in `#[dsl]` with a multi-column btree index on the foreign key columns
- [ ] Share the same `#[use_wrapper(EntityId)]` across multiple foreign key columns referencing the same table

### DSL Method Enforcement

- [ ] Use `dsl.create_position(CreatePosition { ... })?` instead of `ctx.db.position().insert(...)` to enforce FK checks on insert
- [ ] Use `dsl.delete_position_by_id(&position)?` instead of `dsl.ctx().db.position().entity_id().delete(&id)` to enforce FK checks on delete
- [ ] Propagate errors from DSL methods using `?` to prevent partial commits on integrity violations

## Unique Multi-Column Indices

> Source: `SpacetimeDSL.md`, lines 1131-1186

### Declaration

- [ ] Declare `unique_index(name = ...)` inside the `#[dsl(...)]` attribute to define a unique multi-column index
- [ ] Match the `unique_index(name = ...)` value to a corresponding `index(accessor = ...)` on the same `#[table(...)]`
- [ ] Use the `name` parameter (not `accessor`) when specifying the index in the `#[dsl]` attribute
- [ ] Combine `unique_index` with `method(update = true, delete = true)` in `#[dsl]` to enable update and delete by the unique index
- [ ] Declare `index(accessor = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id]))` on the `#[table]` to define a composite btree index over two columns
- [ ] Annotate individual columns with `#[index(btree)]` for single-column btree indices alongside the multi-column index

### Generated Methods

- [ ] Call `dsl.get_<table>_by_<index_name>(&col1, &col2)` to retrieve a single row by unique multi-column index -- returns `Result`, not an iterator
- [ ] Call `dsl.update_<table>_by_<index_name>(row)` to update a row identified by the unique multi-column index
- [ ] Call `dsl.delete_<table>_by_<index_name>(&col1, &col2)` to delete a single row by the unique multi-column index

### Uniqueness Enforcement

- [ ] Return a `UniqueConstraintViolation` error when creating a row whose multi-column index values match an existing row
- [ ] Return a `UniqueConstraintViolation` error when updating a row whose multi-column index values match an existing row
- [ ] Invoke uniqueness checks exclusively through DSL methods -- raw SpacetimeDB calls bypass the constraint

## Hooks System

> Source: `SpacetimeDSL.md`, lines 1187-1272

### Declaration

- [ ] Declare `hook(before(insert, update, delete), after(insert, update, delete))` in the `#[dsl(...)]` attribute to register all six hook points for a table
- [ ] Declare `hook(before(insert))` to register only a subset of hook points
- [ ] Combine `hook(...)` with `method(update = true, delete = true)` in the same `#[dsl(...)]` attribute

### Hook Function Naming

- [ ] Name hook functions following the pattern `{before|after}_{table_name}_{insert|update|delete}`
- [ ] Derive the `{table_name}` segment from the struct name in snake_case (e.g., `Attribute` becomes `attribute`)

### The `#[hook]` Attribute

- [ ] Apply `#[spacetimedsl::hook]` to each hook function
- [ ] Apply `#[hook]` as a shorthand when the prelude is imported -- equivalent to `#[spacetimedsl::hook]`

### Hook Signatures

#### `before_insert`

- [ ] Define `fn before_{table}_insert(dsl: &DSL<'_, T>, create: Create{Table}) -> Result<Create{Table}, SpacetimeDSLError>` to intercept inserts before they occur
- [ ] Accept the `Create{Table}` wrapper type as the second parameter
- [ ] Return the modified `Create{Table}` from a `before_insert` hook to allow the insert to proceed

#### `after_insert`

- [ ] Define `fn after_{table}_insert(dsl: &DSL<'_, T>, row: &{Table}) -> Result<(), SpacetimeDSLError>` to react after a row is inserted
- [ ] Receive the inserted row as an immutable reference `&{Table}`

#### `before_update`

- [ ] Define `fn before_{table}_update(dsl: &DSL<'_, T>, old: &{Table}, new: {Table}) -> Result<{Table}, SpacetimeDSLError>` to intercept updates before they occur
- [ ] Receive the old row as an immutable reference `&{Table}` and the new row as an owned `{Table}`
- [ ] Return the modified `{Table}` from a `before_update` hook to allow the update to proceed

#### `after_update`

- [ ] Define `fn after_{table}_update(dsl: &DSL<'_, T>, old: &{Table}, new: &{Table}) -> Result<(), SpacetimeDSLError>` to react after an update
- [ ] Receive both old and new rows as immutable references `&{Table}`

#### `before_delete`

- [ ] Define `fn before_{table}_delete(dsl: &DSL<'_, T>, row: &{Table}) -> Result<(), SpacetimeDSLError>` to validate before deletion
- [ ] Receive the row to be deleted as an immutable reference `&{Table}`

#### `after_delete`

- [ ] Define `fn after_{table}_delete(dsl: &DSL<'_, T>, row: &{Table}) -> Result<(), SpacetimeDSLError>` to react after deletion
- [ ] Receive the deleted row as an immutable reference `&{Table}`

### Error Handling in Hooks

- [ ] Return `Err(SpacetimeDSLError)` from a `before` hook to abort the operation
- [ ] Return `Err(SpacetimeDSLError)` from an `after` hook to propagate the error and prevent transaction commit
- [ ] Use the `?` operator inside hooks to propagate errors from nested calls

### Hook-Method Compatibility

- [ ] Enable `method(update = true)` when declaring `before_update` or `after_update` hooks
- [ ] Enable `method(delete = true)` when declaring `before_delete` or `after_delete` hooks
- [ ] Use `before_insert` and `after_insert` hooks without any `method(...)` prerequisite -- create is always available

### Location Requirement

- [ ] Define hook functions in the same module as the table definition

## Error Handling

> Source: `SpacetimeDSL.md`, lines 1273-1415

### SpacetimeDSLError Variants

- [ ] Define `SpacetimeDSLError::Error(String)` variant for generic error messages
- [ ] Define `SpacetimeDSLError::NotFoundError` variant with `table_name` and `column_names_and_row_values` fields
- [ ] Define `SpacetimeDSLError::UniqueConstraintViolation` variant with `table_name`, `action`, `error_from`, `one_or_multiple`, and `column_names_and_row_values` fields
- [ ] Define `SpacetimeDSLError::AutoIncOverflow` variant with `table_name` field
- [ ] Define `SpacetimeDSLError::ReferenceIntegrityViolation` variant wrapping `ReferenceIntegrityViolationError`

### ReferenceIntegrityViolationError Variants

- [ ] Define `ReferenceIntegrityViolationError::OnCreateOrUpdate` variant with `table_name`, `create_or_update` of type `Action`, and `column_names_and_row_values` fields
- [ ] Define `ReferenceIntegrityViolationError::OnDelete` variant wrapping `DeletionResult`

### Supporting Enums

- [ ] Define `Action` enum with variants `Create`, `Get`, `Update`, `Delete`
- [ ] Define `ErrorFrom` enum with variants `SpacetimeDB`, `SpacetimeDSL`
- [ ] Define `OneOrMultiple` enum with variants `One`, `Multiple`

### Conversion Chain

- [ ] Implement `Display` trait for `SpacetimeDSLError`
- [ ] Implement `Error` trait for `SpacetimeDSLError`
- [ ] Implement `From<SpacetimeDSLError> for String` by calling `.to_string()`
- [ ] Return `Result<(), SpacetimeDSLError>` from reducers -- works because SpacetimeDB expects `Result<(), impl Into<String>>`

### Recommended Pattern: ? Propagation

- [ ] Declare reducer return type as `Result<(), SpacetimeDSLError>`
- [ ] Propagate errors from `dsl.create_entity()` using `?` operator
- [ ] Propagate errors from `dsl.get_position_by_entity_id()` using `?` operator
- [ ] Propagate errors from `dsl.delete_entity_by_id()` using `?` operator

### Explicit Matching for ReferenceIntegrityViolation

- [ ] Match `Ok(deletion_result)` from `dsl.delete_entity_by_id()` and call `deletion_result.to_csv()`
- [ ] Match `Err(SpacetimeDSLError::ReferenceIntegrityViolation(err))` to handle foreign key violations
- [ ] Re-propagate unhandled error variants with `Err(e) => return Err(e)`

### Error Messages

#### NotFoundError

- [ ] Format `NotFoundError` message including table name and column-value pairs in `{{ key : value }}` syntax

#### UniqueConstraintViolation

- [ ] Format `UniqueConstraintViolation` message from `ErrorFrom::SpacetimeDB` with all columns and values -- SpacetimeDB does not identify which column caused the violation
- [ ] Format `UniqueConstraintViolation` message from `ErrorFrom::SpacetimeDSL` with specific multi-column key in `{{ col1 : val1, col2 : val2 }}` syntax

#### AutoIncOverflow

- [ ] Format `AutoIncOverflow` message including table name

#### ReferenceIntegrityViolation

- [ ] Format `ReferenceIntegrityViolation` on create/update message including table name and column-value pairs
- [ ] Format `ReferenceIntegrityViolation` on delete message as CSV with columns `entry_id`, `parent_entry_id`, `table_name`, `column_name`, `strategy`, `row_value`

## Generated Trait Names Reference

> Source: `SpacetimeDSL.md`, lines 1416-1480

### Trait Naming Patterns

- [ ] Use PascalCase singular table name in all generated trait names
- [ ] Import `Create{Table}Row` trait to access the `create_{table}()` method
- [ ] Import `Get{Table}RowOptionBy{Column}` trait to access the `get_{table}_by_{column}()` method -- for primary key or unique column lookups
- [ ] Import `Get{Table}RowsBy{Index}` trait to access the `get_{plural}_by_{index}()` method -- returns multiple rows by index
- [ ] Import `GetAll{Table}Rows` trait to access the `get_all_{plural}()` method
- [ ] Import `CountOfAll{Table}Rows` trait to access the `count_of_all_{plural}()` method
- [ ] Import `Update{Table}RowBy{Column}` trait to access the `update_{table}_by_{column}()` method
- [ ] Import `Delete{Table}RowBy{Column}` trait to access the `delete_{table}_by_{column}()` method -- deletes one row by primary key or unique column
- [ ] Import `Delete{Table}RowsBy{Index}` trait to access the `delete_{plural}_by_{index}()` method -- deletes multiple rows by index

### Struct Naming Patterns

- [ ] Use `Create{Table}` as the create argument struct -- e.g., `CreateEntity`, `CreatePosition`
- [ ] Use default wrapper type `{Table}{Column}` -- e.g., `EntityObjId`, `PositionId`
- [ ] Specify a custom wrapper type name to override the default `{Table}{Column}` pattern -- e.g., `EntityId`, `PlayerId`

### Hook Trait Naming Patterns

- [ ] Import `Before{Table}InsertHook` trait to register a before-insert hook
- [ ] Import `After{Table}InsertHook` trait to register an after-insert hook
- [ ] Import `Before{Table}UpdateHook` trait to register a before-update hook
- [ ] Import `After{Table}UpdateHook` trait to register an after-update hook
- [ ] Import `Before{Table}DeleteHook` trait to register a before-delete hook
- [ ] Import `After{Table}DeleteHook` trait to register an after-delete hook

### Internal FK Cascade Trait Patterns

- [ ] Use `ExecuteOnDeleteStrategiesOfReferencingTablesAfterOneRowOfThe{Table}TableWasDeleted` trait for single-row FK cascade logic
- [ ] Use `ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfThe{Table}TableWereDeleted` trait for multi-row FK cascade logic

### Importing Generated Traits in Multi-Module Projects

- [ ] Import generated traits from `crate::{module}` in multi-module projects for cross-module helper functions
- [ ] Import `CreateEntityRow`, `EntityId`, `GetEntityRowOptionByObjId`, `DeleteEntityRowByObjId`, `UpdateEntityRowByObjId`, and `CountOfAllEntityRows` from `crate::entity`
- [ ] Import `CreatePosition`, `CreatePositionRow`, `PositionId`, `GetPositionRowOptionById`, `GetAllPositionRows`, `UpdatePositionRowById`, and `CountOfAllPositionRows` from `crate::component::position`
