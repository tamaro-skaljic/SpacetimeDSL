<!-- markdownlint-disable MD033 -->
# ✨ *SpacetimeDSL*

*SpacetimeDSL* provides you a high-level [*D*omain *S*pecific *L*anguage (DSL)](https://en.wikipedia.org/wiki/Domain-specific_language) in Rust to interact in an ergonomic, more developer-friendly and type-safe way with the data in your [*SpacetimeDB*](https://spacetimedb.com/) instances.

🚀 **Try SpacetimeDSL for yourself**, by adding it to your server modules `Cargo.toml`:

```toml
# https://crates.io/crates/spacetimedsl Ergonomic DSL for SpacetimeDB
spacetimedsl = { version = "*" }
```

📖 **Get started** by adding `#[spacetimedsl::dsl]` as well as it's helper attributes `#[create_wrapper]`, `#[use_wrapper]`,\
`#[foreign_key]` and `#[referenced_by]` to your structs with `#[spacetimedb::table]`!

💬 **Need help?**

- Consult the [FAQ](#-faq)
- Join the [SpacetimeDSL channel of the SpacetimeDB Discord Server](https://discord.com/channels/1037340874172014652/1395832638966726726)

## 🔧 Vanilla SpacetimeDB

Let's start with a ordinary SpacetimeDB schema:

```rust
#[spacetimedb::table(name = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    id: u128,

    created_at: spacetimedb::Timestamp,
}

#[spacetimedb::table(name = position, public, index(name = x_y, btree(columns = [x, y])))]
pub struct Position {
    #[primary_key]
    #[auto_inc]
    id: u128,

    #[unique]
    entity_id: u128,

    x: i128,

    y: i128,

    modified_at: spacetimedb::Timestamp,
}
```

We have two tables:

- 📋 The `Entity` table - holds no data per row except an unique machine-readable identifier
- 📍 The `Position` table - holds an `entity_id` and `x`, `y` values per row

### ⚠️ Problems with Vanilla SpacetimeDB

Even with this small data model, there are fundamental issues:

**Boilerplate Code:**

- You must create an `Entity` first, then pass it to `insert`/`try_insert`
- Must manually set sensible defaults:
  - `0` for `id` column (for auto-increment)
  - `ctx.timestamp` for timestamps
- Repetitive code that could be avoided ✂️

**Data Integrity Issues:**

- ❌ Can change `created_at` after creation and persist with `update`
- ❌ Column name `entity_id` doesn't enforce it only accepts `Entity` IDs
- ❌ No guarantee referenced `Entities` actually exist
- ❌ `Positions` won't auto-delete when `Entities` are deleted

**Missing Constraints:**

- 🎮 For tile-based 2D games: each tile should contain max one `Entity`
- No unique multi-column indices - must check manually
- Must manually update `modified_at` with `ctx.timestamp`

**The Problem:**
> SpacetimeDB is great technology, but has weaknesses that prevent developers from utilizing its full potential — sometimes you work *against* the database.

## ⚡ SpacetimeDB with SpacetimeDSL

Let's see what happens when adding **SpacetimeDSL**:

```rust
#[spacetimedsl::dsl(plural_name = entities, method(update = false))] // Added
#[spacetimedb::table(name = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]                                                // Added
    #[referenced_by(path = crate, table = position)]                 // Added
    id: u128,

    created_at: spacetimedb::Timestamp,
}

#[spacetimedsl::dsl(plural_name = positions, method(update = false), unique_index(name = x_y))] // Added
#[spacetimedb::table(name = position, public, index(name = x_y, btree(columns = [x, y])))]
pub struct Position {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]                                                                           // Added
    id: u128,

    #[unique]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = crate, table = entity, on_delete = Delete)]                            // Added
    entity_id: u128,

    x: i128,

    y: i128,

    modified_at: spacetimedb::Timestamp,
}
```

> 📝 **Note:** For clarity, **DB** = **SpacetimeDB**, **DSL** = **SpacetimeDSL**

Looks simple, but unlocks powerful capabilities! 🎯

### 🎨 The `Create` DSL method

```rust
#[spacetimedb::reducer]
pub fn create_example(ctx: &spacetimedb::ReducerContext) -> Result<(), String> {
    // Vanilla SpacetimeDB
    use spacetimedb::Table;

    // Without the question mark it would return a
    // Result<Entity, spacetimedb::TryInsertError<entity__TableHandle>>
    let entity: Entity = ctx.db.entity().try_insert(
        Entity {
            id: 0,
            created_at: ctx.timestamp,
        }
    )?;

    // SpacetimeDB with SpacetimeDSL
    let dsl: spacetimedsl::DSL<'_> = spacetimedsl::dsl(ctx);

    // Without the question mark it would return a Result<Entity, spacetimedsl::SpacetimeDSLError>
    let entity: Entity = dsl.create_entity()?;

    Ok(())
}
```

**✨ What's different?**

**Cleaner Code:**

- 🎯 Much nicer to read and write!
- No manual `Entity` construction required

**Smart Defaults:**

- 🤖 `id` column: automatically set to `0` (DB generates ID)
- ⏰ `created_at`: automatically set to `ctx.timestamp`
- 🔄 `modified_at`: supports both `Timestamp` and `Option<Timestamp>` types

**Better API:**

- SpacetimeDSL wraps `&spacetimedb::ReducerContext`
- Provides more ergonomic API with added capabilities
- Reduces boilerplate code significantly

**💡 Best Practices:**

- Create DSL instance once at reducer start: `spacetimedsl::dsl(ctx)`
- Pass only `&DSL` to functions (not `&ReducerContext`)
- Use `dsl.ctx()` method if you really need the context

Here is the implementation:

```rust
pub trait CreateEntityRow : spacetimedsl::DSLContext {
    fn create_entity<'a>(&'a self) -> Result<Entity, spacetimedsl::SpacetimeDSLError> {
        use spacetimedsl::Wrapper;
        use spacetimedb::{DbContext, Table};

        let id = u128::default();
        let created_at = self.ctx().timestamp;

        let entity = Entity { id, created_at };

        match self.ctx().db().entity().try_insert(entity) {
            Ok(entity) => Ok(entity),
            Err(error) => {
                match error {
                    spacetimedb::TryInsertError::UniqueConstraintViolation(_) => {
                        Err(spacetimedsl::SpacetimeDSLError::UniqueConstraintViolation {
                            table_name: "entity".into(),
                            action: spacetimedsl::Action::Create,
                            error_from: spacetimedsl::ErrorFrom::SpacetimeDB,
                            one_or_multiple: spacetimedsl::OneOrMultiple::One,
                            column_names_and_row_values: format!(
                                "{{ entity : {:?} }}",
                                entity
                            ).into(),
                        })
                    }
                    spacetimedb::TryInsertError::AutoIncOverflow(_) => {
                        Err(spacetimedsl::SpacetimeDSLError::AutoIncOverflow {
                            table_name: "entity".into(),
                        })
                    }
                }
            }
        }
    }
}

impl CreateEntityRow for spacetimedsl::DSL<'_> {}
```

### 🚨 The `SpacetimeDSLError` Type

Unlike DB errors, DSL errors include **metadata for better debugging**! 🔍

**Transformation:**

- `spacetimedb::TryInsertError` → `SpacetimeDSLError`
- `bool` (Delete One) → `SpacetimeDSLError`
- `Option` (Get One) → `SpacetimeDSLError`
- `u64` (Delete Many) → `SpacetimeDSLError`

**Error Variants:**

```rust
pub enum SpacetimeDSLError {
    Error,                       // Not available in vanilla SpacetimeDB
    NotFoundError,               // Not available in vanilla SpacetimeDB
    UniqueConstraintViolation,
    AutoIncOverflow,
    ReferenceIntegrityViolation, // Not available in vanilla SpacetimeDB
}
```

#### ⚠️ The `Error` variant

- Used when DSL can't help with DB issues
- **Never return `Ok(())` in reducer if this occurs!**

#### 🔍 The `NotFoundError` variant

**What DB gives:** Simple `Option<T>`  
**What DSL gives:** `NotFound` error with table name + column values

**Example log:**

```txt
Not Found Error while trying to find a row in the position table with {{ entity_id : 1 }}!
```

#### 🔒 The `UniqueConstraintViolation` variant

**Origins:**

- 📊 DB: unique **single**-column indices
- 🎯 DSL: unique **multi**-column indices

**Example log:**

```txt
Unique Constraint Violation Error while trying to create a row in the entity table! 
Unfortunately SpacetimeDB doesn't provide more information, so here are all columns and their values: 
{{ entity : Entity { id: EntityId { id: 1 }, created_at: /* omitted */ } }}
```

#### 📈 The `AutoIncOverflow` variant

**Improvements over DB:**

- DB: No column name
- DSL: Includes table name ✅

**Example log:**

```txt
Auto Inc Overflow Error on the entity table! 
Unfortunately SpacetimeDB doesn't provide more information.
```

#### 🔗 The `ReferenceIntegrityViolation` variant

**Two scenarios:**

1️⃣ **Creating/Updating rows:**

- Tables with foreign keys (`#[foreign_key]`)
- DSL checks if referenced rows exist

**Example log:**

```txt
Reference Integrity Violation Error while trying to create a row in the position table 
because of {{ entity_id : 1 }}!
```

2️⃣ **Deleting referenced rows:**

- Tables with `#[referenced_by]` on primary key
- More complex - see [DeletionResult](#-the-deletionresultentry-type)

### 📊 The `DeletionResult[Entry]` Type

**The Problem (DB Conversation):**

**Developer:** "Hi DB! I need an audit log for every deletion. How?"

**DB:** "I can give you:

- `bool` for Delete One (deleted or not)
- `u64` for Delete Many (count of deleted rows)
  
Is that enough?"

**DSL:** "May I answer for you, developer?"

**Developer:** "Yes, please!"

**DSL:** "No, that's not enough! But don't worry, I have a solution:" 🎉

**The Solution:**

```rust
pub struct DeletionResult {
    pub table_name: Box<str>,
    pub one_or_multiple: OneOrMultiple,
    pub entries: Vec<DeletionResultEntry>,
}

pub struct DeletionResultEntry {
    pub table_name: Box<str>,
    pub column_name: Box<str>,
    pub strategy: OnDeleteStrategy,
    pub row_value: Box<str>,
    pub child_entries: Vec<DeletionResultEntry>,
}
```

**Features:**

- ✅ Get `DeletionResult` on both success AND failure
- 📝 Process programmatically or use `to_csv()` method
- 📊 Import into spreadsheet applications
- 🔍 Complete audit trail

See [Foreign Keys and Referential Integrity](#-foreign-keys--referential-integrity) for more details.

### 🎯 The `OnDeleteStrategy` Type

Found in `#[foreign_key]` attribute and `DeletionResultEntry` type.

**Controls deletion behavior** when referenced rows are deleted:

```rust
pub enum OnDeleteStrategy {
    /**
     * Available independent from the column type.
     * 
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys ...
     * ... of other tables the deletion fails with a Reference Integrity Violation Error.
     */
    Error,

    /**
     * Available independent from the column type.
     * 
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys ...
     * ... of other tables, it's checked whether any primary key value of rows to delete is referenced
     * in a foreign key with `OnDeleteStrategy::Error`.
     * 
     * If true, the deletion fails with a Reference Integrity Violation Error and
     * no other OnDeleteStrategy is executed (especially: no row is deleted).
     * 
     * If false, the on delete strategies of all affected rows are executed and rows are deleted.
     */
    Delete,

    /**
     * Available only for columns with a numeric type.
     * 
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys ...
     * ... of other tables the value of the foreign key column is set to `0`.
     */
    SetZero,

    /**
     * Available independent from the column type.
     * 
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys ...
     * ... of other tables nothing happens, which means the referencing rows will reference a primary
     * key value which doesn't exist anymore. The referential integrity is only enforced while creating
     * a row or if a row is updated and the foreign key column value is changed.
     */
    Ignore,
}
```

**Strategies Explained:**

🛑 **`Error`**

- Available for all column types
- Deletion fails if referenced by other tables
- Returns Reference Integrity Violation Error

🗑️ **`Delete`**

- Available for all column types
- Cascading delete of referencing rows
- Fails only if any reference uses `Error` strategy
- All-or-nothing: either all delete or none

0️⃣ **`SetZero`**

- Numeric types only
- Sets foreign key column to `0`
- Referenced value becomes non-existent

🤷 **`Ignore`**

- Available for all column types
- Breaks referential integrity
- Creates dangling references
- Integrity only enforced on create/update

### 🏷️ The `#[create_wrapper]` and `#[use_wrapper]` attributes - aka Wrapper Types

**Requirements:**

Every column with these attributes needs a wrapper:

- `#[primary_key]` ✅
- `#[unique]` ✅
- `#[index]` ✅

Must have either:

- `#[create_wrapper]` - creates new wrapper type
- `#[use_wrapper]` - uses existing wrapper type

**Benefits:**

🎯 **Reduces Primitive Obsession:**

- [Unique, auto-generated alias types](https://medium.com/unil-ci-software-engineering/clean-ddd-lessons-modeling-identity-ff8bc17e0ae6#:~:text=We%20should%20not%20use%20primitives)
- Wraps primitive column types
- [Makes logical dependencies physical](https://medium.com/@gara.mohamed/domain-driven-design-the-identifier-type-pattern-d86fd3c128b3#:~:text=Make%20logical%20dependencies%20physical)

🔒 **Type Safety:**

- [Compile-time errors instead of runtime errors](https://medium.com/@gara.mohamed/domain-driven-design-the-identifier-type-pattern-d86fd3c128b3#:~:text=Suppose%20that%20we,at%20compile%20time.)
- Field order mistakes caught at compilation

**API:**

```rust
pub trait Wrapper<WrappedType: Clone + Default, WrapperType>: Default +
    Clone + PartialEq + PartialOrd + spacetimedb::SpacetimeType + Display
{
    fn new(value: WrappedType) -> WrapperType;
    fn value(&self) -> WrappedType;
}
```

**Usage Examples:**

```rust
#[spacetimedsl::dsl(plural_name = entities, method(update = false))]
#[spacetimedb::table(name = entity, public)]
pub struct Entity {
    // Default Name Strategy: EntityId
    // format!("{}{}", singular_table_name_pascal_case, column_name_pascal_case)
    #[create_wrapper]

    //  Custom Name Strategy: EntityID
    #[create_wrapper(EntityID)]

    id: u128,

    // Use a wrapper type from the same module
    #[use_wrapper(EntityId)]

    // Use a wrapper type from another module
    #[use_wrapper(crate::entity::EntityId)]

    parent_entity_id: u128,
}
```

**⚠️ Common Error:**

```txt
The trait bound `WrapperType: From<NumericType>` is not satisfied.
```

**What this means:**

- You provided `NumericType` (like `u128`) where `WrapperType` is required
- SpacetimeDB CLI and Admin Panel have primitive obsession limitations
- They lack feature-parity with server modules

**Solution:**

- Don't create Wrapper Types manually (`WrapperType::new(wrapped_type)`)
- Use Wrapper Types in your entire ecosystem API
- Include them in reducer arguments
- Embrace full SpacetimeDB capabilities! 💪

### 🔄 Accessors (Getters and Setters)

**Getters (for all columns):**

- 📖 Public getter for every column
- Returns reference for normal types
- Returns cloned Wrapper Type for wrapped types

**Setters (for non-private columns):**

- ✏️ Generated based on column visibility
- Visibility = field visibility
- Use field visibility to make fields immutable after creation

**🔒 Automatic Field Privacy Enforcement:**
SpacetimeDSL **automatically makes all struct fields private** when processing DSL attributes.

**Why?**

- ✅ Prevents direct field access
- ✅ Forces use of wrapper types, getters, setters
- ✅ Prevents unauthorized modifications
- ✅ Enforces proper encapsulation

**Use Cases for Private Columns:**
🔑 **Never-changing fields:**

- Primary and foreign key columns
- `created_at` timestamps
- Event/audit table data

**No `Update` Method:**

- All columns private = no `Update` DSL method
- Indicates row should never change after insertion

**Future Enhancement:**
> When [rust-lang/rust#105077](https://github.com/rust-lang/rust/issues/105077) releases, DSL will use field mutability restrictions instead of visibility.

### 🎲 Unique multi-column indices

**Achievement Unlocked:** 🏆

- [SpacetimeDSL implemented it first](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/25)
- [SpacetimeDB implementation pending](https://github.com/clockworklabs/SpacetimeDB/issues/976)

**Example:**

```rust
#[dsl(
    plural_name = entity_relationships,
    method(update = false),
    unique_index(name = parent_child_entity_id)
)]
#[table(
    name = entity_relationship,
    public,
    index(name = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id]))
)]
pub struct EntityRelationship {
    #[primary_key]
    #[auto_inc]
    id: u128,

    parent_entity_id: u128,

    child_entity_id: u128,
}
```

**Setup:**

1. Add `unique_index(name = parent_child_entity_id)` to `#[spacetimedsl::dsl]`
2. Have matching multi-column index in `#[spacetimedb::table]`

**You get:**

- ✅ `Get One` instead of `Get Many`
- ✅ `Update` method
- ✅ `Delete One` instead of `Delete Many`
- ✅ Automatic uniqueness checks on create/update

**⚠️ Important:**

- Only enforced when using DSL methods
- Don't call DB state-mutating methods directly on `&ReducerContext`
- Always use `&spacetimedsl::DSL` methods

**Status:**
> ⚠️ Unstable - will be removed when SpacetimeDB implements native support

### 🔗 Foreign Keys / Referential Integrity

Add `#[foreign_key]` and `#[referenced_by]` to enforce referential integrity and apply on delete strategies.

**Example:**

```rust
pub mod entity {
    #[dsl(plural_name = entities, method(update = false))]
    #[table(name = entity, public)]
    pub struct Entity {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate, table = identifier)] // Added
        id: u128,

        created_at: Timestamp,
    }
}

pub mod identifier {
    #[dsl(plural_name = identifiers, method(update = true))]
    #[table(name = identifier, public)]
    pub struct Identifier {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate, table = identifier_reference)]     // Added
        id: u128

        #[unique]
        #[use_wrapper(crate::EntityId)]
        #[foreign_key(path = crate, table = entity, on_delete = Delete)] // Added
        entity_id: u128

        #[unique]
        pub value: String

        created_at: Timestamp

        modified_at: Timestamp,
    }
}

#[dsl(plural_name = identifier_references, method(update = true))]
#[table(name = identifier_reference, public)]
pub struct IdentifierReference {
    #[primary_key]
    #[use_wrapper(IdentifierId)]
    #[foreign_key(path = crate, table = identifier, on_delete = Error)]   // Added
    id: u128,

    #[unique]
    #[use_wrapper(IdentifierId)]
    #[foreign_key(path = crate, table = identifier, on_delete = Delete)]  // Added
    id2: u128,

    #[unique]
    #[use_wrapper(IdentifierId)]
    #[foreign_key(path = crate, table = identifier, on_delete = SetZero)] // Added
    pub id3: u128,

    #[unique]
    #[use_wrapper(IdentifierId)]
    #[foreign_key(path = crate, table = identifier, on_delete = Ignore)]  // Added
    id4: u128,
}
```

**📋 `#[referenced_by]` Attribute:**

**Requirements:**

- Values for `path` and `table` fields
- Only on `#[primary_key]` columns
- Requires `#[create_wrapper]`/`#[use_wrapper]`

**Features:**

- Multiple `#[referenced_by]` per primary key allowed
- One for each referencing table
- Controls Delete One/Many DSL methods
- Calls `OnDeleteStrategy` of referencing tables

**🔑 `#[foreign_key]` Attribute:**

**Requirements:**

- Only on: `#[primary_key]`, `#[index]`, or `#[unique]` columns
- Requires `#[use_wrapper]`
- Values for: `path`, `table`, `column`, `on_delete`
- One per column maximum

**Features:**

- Referential integrity checks on create/update
- Executes `OnDeleteStrategy` when referenced row deleted

**⚠️ Important:**

- Only enforced when using DSL methods
- Don't call DB methods directly on `&ReducerContext`
- Always use `&spacetimedsl::DSL` methods

**Status:**
> ⚠️ Unstable
>
> - Will be removed when SpacetimeDB implements native support
> - Tests exist but may not cover all cases
> - **Backup your data before testing!**
> - Found a bug? [Create a GitHub issue](https://github.com/tamaro-skaljic/SpacetimeDSL/issues)! 🐛

<details>

<summary>📄 Here is the `Delete One` DSL method of the Entity table</summary>

```rust
pub trait DeleteEntityRowById: spacetimedsl::DSLContext {
    fn delete_entity_by_id(
        &self,
        id: impl Into<EntityId> + Clone,
    ) -> Result<spacetimedsl::DeletionResult, spacetimedsl::SpacetimeDSLError> {
        use spacetimedsl::Wrapper;
        use spacetimedb::{DbContext, Table};
        use spacetimedsl::itertools::Itertools;

        let id = id.clone().into().value();

        let row_to_delete = self.ctx().db().entity().id().find(id);

        let primary_key_value_of_a_row_to_delete = match row_to_delete {
            None => {
                return Err(spacetimedsl::SpacetimeDSLError::NotFoundError {
                    table_name: "entity".into(),
                    column_names_and_row_values: format!("{{ , id : {0}  }}", &id).into(),
                });
            }
            Some(row_to_delete) => row_to_delete.id,
        };

        let mut deletion_result_entry = spacetimedsl::DeletionResultEntry {
            table_name: "entity".into(),
            column_name: "id".into(),
            strategy: spacetimedsl::OnDeleteStrategy::Delete,
            row_value: format!("{0}", EntityId::new(primary_key_value_of_a_row_to_delete.clone())).into(),
            child_entries: vec![],
        };

        match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_entity_table_was_deleted(
            self.ctx(),
            spacetimedsl::OnDeleteStrategy::Error,
            &primary_key_value_of_a_row_to_delete,
        ) {
            Err(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
                let error = spacetimedsl::DeletionResult {
                    table_name: "entity".into(),
                    one_or_multiple: spacetimedsl::OneOrMultiple::One,
                    entries: vec![deletion_result_entry],
                };
                return Err(
                    spacetimedsl::SpacetimeDSLError::ReferenceIntegrityViolation(
                        spacetimedsl::ReferenceIntegrityViolationError::OnDelete(
                            error,
                        ),
                    ),
                );
            }
            Ok(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
            }
        };

        match self
            .ctx()
            .db()
            .entity()
            .id()
            .delete(primary_key_value_of_a_row_to_delete)
        {
            false => return Err(spacetimedsl::SpacetimeDSLError::Error("Delete One Error: `count_of_rows_to_delete ( 1 ) != ( 0 ) count_of_deleted_rows`!".to_string())),
            true => {}
        };

        match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_entity_table_was_deleted(
            self.ctx(),
            spacetimedsl::OnDeleteStrategy::Delete,
            &primary_key_value_of_a_row_to_delete,
        ) {
            Err(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
                let error = spacetimedsl::DeletionResult {
                    table_name: "entity".into(),
                    one_or_multiple: spacetimedsl::OneOrMultiple::One,
                    entries: vec![deletion_result_entry],
                };
                return Err(
                    spacetimedsl::SpacetimeDSLError::Error(
                        format!("Delete One Error: An unknown error occurred after changing the database state! If the reducer running this doesn\'t return an error, the state changes are persisted and you have problems now! Here is the deletion result: {0}", error),
                    ),
                );
            }
            Ok(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
            }
        };

        match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_entity_table_was_deleted(
            self.ctx(),
            spacetimedsl::OnDeleteStrategy::SetZero,
            &primary_key_value_of_a_row_to_delete,
        ) {
            Err(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
                let error = spacetimedsl::DeletionResult {
                    table_name: "entity".into(),
                    one_or_multiple: spacetimedsl::OneOrMultiple::One,
                    entries: vec![deletion_result_entry],
                };
                return Err(
                    spacetimedsl::SpacetimeDSLError::Error(
                        format!("Delete One Error: An unknown error occurred after changing the database state! If the reducer running this doesn\'t return an error, the state changes are persisted and you have problems now! Here is the deletion result: {0}", error),
                    ),
                );
            }
            Ok(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
            }
        };

        match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_one_row_of_the_entity_table_was_deleted(
            self.ctx(),
            spacetimedsl::OnDeleteStrategy::Ignore,
            &primary_key_value_of_a_row_to_delete,
        ) {
            Err(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
                let error = spacetimedsl::DeletionResult {
                    table_name: "entity".into(),
                    one_or_multiple: spacetimedsl::OneOrMultiple::One,
                    entries: vec![deletion_result_entry],
                };
                return Err(
                    spacetimedsl::SpacetimeDSLError::Error(
                        format!("Delete One Error: An unknown error occurred after changing the database state! If the reducer running this doesn\'t return an error, the state changes are persisted and you have problems now! Here is the deletion result: {0}"error),
                    ),
                );
            }
            Ok(mut child_entries) => {
                deletion_result_entry.child_entries.append(&mut child_entries);
            }
        };

        return Ok(spacetimedsl::DeletionResult {
            table_name: "entity".into(),
            one_or_multiple: spacetimedsl::OneOrMultiple::One,
            entries: vec![deletion_result_entry],
        });
    }
}
impl DeleteEntityRowById for spacetimedsl::DSL<'_> {}
```

</details>

Delete One/Many DSL methods call internal functions generated by tables with `#[referenced_by]`.

#### 🔧 Internals

> 💡 **Optional reading** - skip to [plural name](#-the-plural-name-dsl-attribute-field) if not interested.

**Execution:**

- Called multiple times to ensure `OnDeleteStrategy::Error` processes first
- Separate implementations for one vs. multiple rows

**Key Differences (Multiple vs. One Row):**

- Parameter: `&'a [PrimaryKeyType]` vs. `&PrimaryKeyType`
- Return: `HashMap<&'a u128, Vec<DeletionResultEntry>>` vs. `Vec<DeletionResultEntry>`

<details>

<summary>📄 Function for multiple rows</summary>

```rust
pub trait ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfTheEntityTableWereDeleted {
    fn execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_entity_table_were_deleted<'a>(
        ctx: &spacetimedb::ReducerContext,
        strategy: spacetimedsl::OnDeleteStrategy,
        primary_key_values_of_rows_to_delete: &'a [u128],
    ) -> Result<
        std::collections::HashMap<&'a u128, Vec<spacetimedsl::DeletionResultEntry>>,
        std::collections::HashMap<&'a u128, Vec<spacetimedsl::DeletionResultEntry>>,
    > {
        use spacetimedsl::Wrapper;
        use spacetimedb::{DbContext, Table};

        let mut entries = std::collections::HashMap::new();

        for primary_key_value_of_a_row_to_delete in primary_key_values_of_rows_to_delete {
            entries.insert(primary_key_value_of_a_row_to_delete, vec![]);
        }

        let mut error = false;

        use crate::component::identifier::ExecuteOnDeleteStrategiesOfTheIdentifierTableAfterMultipleRowsOfTheEntityTableWereDeleted;
        match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_the_identifier_table_after_multiple_rows_of_the_entity_table_were_deleted(
            ctx,
            &strategy,
            primary_key_values_of_rows_to_delete,
        ) {
            Err(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                    entries.get_mut(&primary_key_value_of_a_row_to_delete).unwrap().append(&mut child_entries);
                }
                error = true;
            }
            Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                    entries.get_mut(&primary_key_value_of_a_row_to_delete).unwrap().append(&mut child_entries);
                }
            }
        };

        match error {
            false => Ok(entries),
            true => Err(entries),
        }
    }
}
impl ExecuteOnDeleteStrategiesOfReferencingTablesAfterMultipleRowsOfTheEntityTableWereDeleted for spacetimedsl::internal::DSLInternals {}
```

</details>

Calls another internal function generated by the `Identifier` table (has `#[foreign_key]`).

<details>

<summary>📄 Function created by <code>Identifier</code> table</summary>

```rust
pub trait ExecuteOnDeleteStrategiesOfTheIdentifierTableAfterMultipleRowsOfTheEntityTableWereDeleted {
    fn execute_on_delete_strategies_of_the_identifier_table_after_multiple_rows_of_the_entity_table_were_deleted<'a>(
        ctx: &spacetimedb::ReducerContext,
        strategy: &spacetimedsl::OnDeleteStrategy,
        primary_key_values_of_rows_of_another_table_to_delete: &'a [u128],
    ) -> Result<
        std::collections::HashMap<&'a u128, Vec<spacetimedsl::DeletionResultEntry>>,
        std::collections::HashMap<&'a u128, Vec<spacetimedsl::DeletionResultEntry>>,
        >,
    > {
        use spacetimedsl::Wrapper;
        use spacetimedb::{DbContext, Table};
        use spacetimedsl::itertools::Itertools;

        let mut entries = std::collections::HashMap::new();

        for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
            entries.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
        }

        let mut error = false;

        match &strategy {
            spacetimedsl::OnDeleteStrategy::Ignore => {}
            spacetimedsl::OnDeleteStrategy::Delete => {
                let mut child_entries_by_primary_key_value_of_row_to_delete = std::collections::HashMap::new();

                let mut primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete = std::collections::HashMap::new();
                
                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete
                        .insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
                }

                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    match ctx.db().identifier().entity_id().find(primary_key_value_of_a_row_of_another_table_to_delete)
                    {
                        None => {}
                        Some(row) => {
                            if !child_entries_by_primary_key_value_of_row_to_delete.contains_key(&row.id) {
                                primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete
                                    .get_mut(primary_key_value_of_a_row_of_another_table_to_delete)
                                    .unwrap()
                                    .push(row.id);
                                child_entries_by_primary_key_value_of_row_to_delete.insert(row.id, vec![]);
                            }
                        }
                    };
                }

                let primary_key_values_of_rows_to_delete = child_entries_by_primary_key_value_of_row_to_delete
                    .keys()
                    .cloned()
                    .collect_vec();
                
                match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_identifier_table_were_deleted(
                    ctx,
                    spacetimedsl::OnDeleteStrategy::Error,
                    &primary_key_values_of_rows_to_delete[..],
                ) {
                    Err(
                        child_entries_by_primary_key_value_of_a_row_to_delete,
                    ) => {
                        error = true;
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                    Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                };
                match error {
                    false => {}
                    true => {
                        for (
                            primary_key_value_of_a_row_of_another_table_to_delete,
                            primary_key_values_of_rows_to_delete,
                        ) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                            for id in &primary_key_values_of_rows_to_delete {
                                let child_entries = child_entries_by_primary_key_value_of_row_to_delete
                                    .remove(&id)
                                    .unwrap();
                                entries
                                    .get_mut(
                                        primary_key_value_of_a_row_of_another_table_to_delete,
                                    )
                                    .unwrap()
                                    .push(spacetimedsl::DeletionResultEntry {
                                        table_name: "identifier".into(),
                                        column_name: "entity_id".into(),
                                        strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                        row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                        child_entries,
                                    });
                            }
                        }
                        return Err(entries);
                    }
                };
                for id in &primary_key_values_of_rows_to_delete {
                    ctx.db().identifier().id().delete(id);
                }
                match error {
                    false => {}
                    true => {
                        for (
                            primary_key_value_of_a_row_of_another_table_to_delete,
                            primary_key_values_of_rows_to_delete,
                        ) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                            for id in &primary_key_values_of_rows_to_delete {
                                let child_entries = child_entries_by_primary_key_value_of_row_to_delete
                                    .remove(&id)
                                    .unwrap();
                                entries
                                    .get_mut(
                                        primary_key_value_of_a_row_of_another_table_to_delete,
                                    )
                                    .unwrap()
                                    .push(spacetimedsl::DeletionResultEntry {
                                        table_name: "identifier".into(),
                                        column_name: "entity_id".into(),
                                        strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                        row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                        child_entries,
                                    });
                            }
                        }
                        return Err(entries);
                    }
                };
                match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_identifier_table_were_deleted(
                    ctx,
                    spacetimedsl::OnDeleteStrategy::Delete,
                    &primary_key_values_of_rows_to_delete[..],
                ) {
                    Err(
                        child_entries_by_primary_key_value_of_a_row_to_delete,
                    ) => {
                        error = true;
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                    Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                };
                match error {
                    false => {}
                    true => {
                        for (
                            primary_key_value_of_a_row_of_another_table_to_delete,
                            primary_key_values_of_rows_to_delete,
                        ) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                            for id in &primary_key_values_of_rows_to_delete {
                                let child_entries = child_entries_by_primary_key_value_of_row_to_delete
                                    .remove(&id)
                                    .unwrap();
                                entries
                                    .get_mut(
                                        primary_key_value_of_a_row_of_another_table_to_delete,
                                    )
                                    .unwrap()
                                    .push(spacetimedsl::DeletionResultEntry {
                                        table_name: "identifier".into(),
                                        column_name: "entity_id".into(),
                                        strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                        row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                        child_entries,
                                    });
                            }
                        }
                        return Err(entries);
                    }
                };
                match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_identifier_table_were_deleted(
                    ctx,
                    spacetimedsl::OnDeleteStrategy::SetZero,
                    &primary_key_values_of_rows_to_delete[..],
                ) {
                    Err(
                        child_entries_by_primary_key_value_of_a_row_to_delete,
                    ) => {
                        error = true;
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                    Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                };
                match error {
                    false => {}
                    true => {
                        for (
                            primary_key_value_of_a_row_of_another_table_to_delete,
                            primary_key_values_of_rows_to_delete,
                        ) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                            for id in &primary_key_values_of_rows_to_delete {
                                let child_entries = child_entries_by_primary_key_value_of_row_to_delete
                                    .remove(&id)
                                    .unwrap();
                                entries
                                    .get_mut(
                                        primary_key_value_of_a_row_of_another_table_to_delete,
                                    )
                                    .unwrap()
                                    .push(spacetimedsl::DeletionResultEntry {
                                        table_name: "identifier".into(),
                                        column_name: "entity_id".into(),
                                        strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                        row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                        child_entries,
                                    });
                            }
                        }
                        return Err(entries);
                    }
                };
                match spacetimedsl::internal::DSLInternals::execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_the_identifier_table_were_deleted(
                    ctx,
                    spacetimedsl::OnDeleteStrategy::Ignore,
                    &primary_key_values_of_rows_to_delete[..],
                ) {
                    Err(
                        child_entries_by_primary_key_value_of_a_row_to_delete,
                    ) => {
                        error = true;
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                    Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (
                            primary_key_value_of_a_row_to_delete,
                            mut child_entries,
                        ) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            child_entries_by_primary_key_value_of_a_row_to_delete
                                .get_mut(primary_key_value_of_a_row_to_delete)
                                .unwrap()
                                .append(&mut child_entries);
                        }
                    }
                };
                for (
                    primary_key_value_of_a_row_of_another_table_to_delete,
                    primary_key_values_of_rows_to_delete,
                ) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                    for id in &primary_key_values_of_rows_to_delete {
                        let child_entries = child_entries_by_primary_key_value_of_row_to_delete
                            .remove(&id)
                            .unwrap();
                        entries
                            .get_mut(
                                primary_key_value_of_a_row_of_another_table_to_delete,
                            )
                            .unwrap()
                            .push(spacetimedsl::DeletionResultEntry {
                                table_name: "identifier".into(),
                                column_name: "entity_id".into(),
                                strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                row_value: ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!("{0}", IdentifierId::new(id.clone())),
                                        )
                                    })
                                    .into(),
                                child_entries,
                            });
                    }
                }
            }
            spacetimedsl::OnDeleteStrategy::Error => {}
            spacetimedsl::OnDeleteStrategy::SetZero => {}
        };
        match error {
            false => Ok(entries),
            true => Err(entries),
        }
    }
}
impl ExecuteOnDeleteStrategiesOfTheIdentifierTableAfterMultipleRowsOfTheEntityTableWereDeleted for spacetimedsl::internal::DSLInternals {}
```

</details>

Because the `Identifier` table is referenced by the `Identifier Reference` table, it does much during execution of the `OnDeleteStrategy::Delete` strategy.

It calls it's own function generated because it has at least one `#[referenced_by]`.

<details>

<summary>📄 Let's have a look into it (which doesn't do that much stuff in the <code>OnDeleteStrategy::Delete</code> match arm as the one for the <code>Identifier</code> table)</summary>

```rust
pub trait ExecuteOnDeleteStrategiesOfTheIdentifierReferenceTableAfterMultipleRowsOfTheIdentifierTableWereDeleted {
    fn execute_on_delete_strategies_of_the_identifier_reference_table_after_multiple_rows_of_the_identifier_table_were_deleted<'a>(
        ctx: &spacetimedb::ReducerContext,
        strategy: &spacetimedsl::OnDeleteStrategy,
        primary_key_values_of_rows_of_another_table_to_delete: &'a [u128],
    ) -> Result<
        std::collections::HashMap<&'a u128, Vec<spacetimedsl::DeletionResultEntry>>,
        std::collections::HashMap<&'a u128, Vec<spacetimedsl::DeletionResultEntry>>,
    > {
        use spacetimedsl::Wrapper;
        use spacetimedb::{DbContext, Table};
        use spacetimedsl::itertools::Itertools;

        let mut entries = std::collections::HashMap::new();

        for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
            entries.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
        }

        let mut error = false;

        match &strategy {
            spacetimedsl::OnDeleteStrategy::Error => {
                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    match ctx.db().identifier_reference().id().find(primary_key_value_of_a_row_of_another_table_to_delete) {
                        None => {}
                        Some(row) => {
                            error = true;

                            let child_entries = vec![];

                            let id = &row.id;

                            entries.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).unwrap().push(spacetimedsl::DeletionResultEntry {
                                table_name: "identifier_reference".into(),
                                column_name: "id".into(),
                                strategy: spacetimedsl::OnDeleteStrategy::Error,
                                row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                child_entries,
                            });
                        }
                    };
                }
            }
            spacetimedsl::OnDeleteStrategy::SetZero => {
                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    match ctx.db().identifier_reference().id3().find(primary_key_value_of_a_row_of_another_table_to_delete) {
                        None => {}
                        Some(mut row) => {
                            row.id3 = 0;

                            let child_entries = vec![];

                            let id = &row.id;

                            entries.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).unwrap().push(spacetimedsl::DeletionResultEntry {
                                table_name: "identifier_reference".into(),
                                column_name: "id3".into(),
                                strategy: spacetimedsl::OnDeleteStrategy::SetZero,
                                row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                child_entries,
                            });

                            ctx.db().identifier_reference().id().update(row);
                        }
                    };
                }
            }
            spacetimedsl::OnDeleteStrategy::Delete => {
                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    match ctx.db().identifier_reference().id2().find(primary_key_value_of_a_row_of_another_table_to_delete) {
                        None => {}
                        Some(row) => {
                            let child_entries = vec![];

                            let id = &row.id;

                            entries.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).unwrap().push(spacetimedsl::DeletionResultEntry {
                                table_name: "identifier_reference".into(),
                                column_name: "id2".into(),
                                strategy: spacetimedsl::OnDeleteStrategy::Delete,
                                row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                child_entries,
                            });

                            ctx.db().identifier_reference().id().delete(row.id);
                        }
                    };
                }
            }
            spacetimedsl::OnDeleteStrategy::Ignore => {
                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                    match ctx.db().identifier_reference().id4().find(primary_key_value_of_a_row_of_another_table_to_delete) {
                        None => {}
                        Some(row) => {
                            let child_entries = vec![];
                            let id = &row.id;
                            entries.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).unwrap().push(spacetimedsl::DeletionResultEntry {
                                table_name: "identifier_reference".into(),
                                column_name: "id4".into(),
                                strategy: spacetimedsl::OnDeleteStrategy::Ignore,
                                row_value: format!("{0}", IdentifierId::new(id.clone())).into(),
                                child_entries,
                            });
                        }
                    };
                }
            }
        };

        match error {
            false => Ok(entries),
            true => Err(entries),
        }
    }
}
impl ExecuteOnDeleteStrategiesOfTheIdentifierReferenceTableAfterMultipleRowsOfTheIdentifierTableWereDeleted for spacetimedsl::internal::DSLInternals {}
```

</details>

### 📝 The `plural name` DSL attribute field

**Found in:** `#[spacetimedsl::dsl(plural_name = entities)]`

**Required:** ✅  
**Used in:** DSL method names for `#[index(btree)]` columns

- `Get Many` methods
- `Delete Many` methods

### 🎯 Other DSL methods

**Complete Method Coverage:**
Every DB method has an equivalent DSL method! 🎉

![DSL methods generated by the example project](example_dsl_methods.png)

![Example usage of the generated dsl methods](example_dsl_usage.png)

**Ready to try?** 🚀

Add to your server modules `Cargo.toml`:

```toml
# https://crates.io/crates/spacetimedsl Ergonomic DSL for SpacetimeDB
spacetimedsl = { version = "*" }
```

**Get started** with `#[spacetimedsl::dsl]` and helper attributes:

- ✨ `#[create_wrapper]`
- 🔄 `#[use_wrapper]`
- 🔗 `#[foreign_key]`
- 📌 `#[referenced_by]`

## ⚠️ Current limitations

**Not Yet Considered:**

- [Using IndexScanRangeBounds / FilterableValue](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21) 🔄

## ❓ FAQ

**❔ Why must `#[primary_key]` columns be private?**

- 🔒 They should never change after insertion
- DSL generates [setters](#-accessors-getters-and-setters) for non-private columns
- Making them public would:
  - ❌ Allow changes after creation via setters
  - ❌ Allow direct struct member access
  - ❌ Bypass [wrapped types](#️-the-create_wrapper-and-use_wrapper-attributes---aka-wrapper-types)

## 📜 Licensing

SpacetimeDSL is dual-licensed under:

- ⚖️ [MIT License](https://choosealicense.com/licenses/mit/)
- ⚖️ [Apache License (Version 2.0)](https://choosealicense.com/licenses/apache-2.0/)

**Open Source** ❤️

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
