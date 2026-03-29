# Constraints

Constraints enforce data integrity rules on your tables. SpacetimeDB supports primary key and unique constraints.

## Primary Keys

A primary key uniquely identifies each row in a table. It represents the identity of a row and determines how updates and deletes are handled.

```
#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    id: u64,
    name: String,
    email: String,
}
```

Use the `#[primary_key]` attribute to mark a field as the primary key.

### Primary Key Rules

- **One per table**: A table can have at most one primary key column.
- **Immutable identity**: The primary key defines the row's identity. Changing a primary key value is treated as deleting the old row and inserting a new one.
- **Unique by definition**: Primary keys are automatically unique. No two rows can have the same primary key value.

Because of the unique constraint, SpacetimeDB implements primary keys using a **unique index**. This index is created automatically.

### Multi-Column Primary Keys

SpacetimeDB does not yet support multi-column (composite) primary keys. If you need to look up rows by multiple columns, use a multi-column btree index combined with an auto-increment primary key:

```
#[spacetimedb::table(accessor = inventory, public, index(accessor = inventory_index, btree(columns = [user_id, item_id])))]
pub struct Inventory {
    #[primary_key]
    #[auto_inc]
    id: u64,
    user_id: u64,
    item_id: u64,
    quantity: u32,
}
```

This gives you efficient lookups by the column combination while using a simple auto-increment value as the primary key.

### Updates and Primary Keys

When you update a row, SpacetimeDB uses the primary key to determine whether it's a modification or a replacement:

- **Same primary key**: The row is updated in place. Subscribers see an update event.
- **Different primary key**: The old row is deleted and a new row is inserted. Subscribers see a delete event followed by an insert event.

```
#[spacetimedb::reducer]
fn update_user_name(ctx: &ReducerContext, id: u64, new_name: String) -> Result<(), String> {
    if let Some(mut user) = ctx.db.user().id().find(id) {
        // This is an update — primary key (id) stays the same
        user.name = new_name;
        ctx.db.user().id().update(user);
    }
    Ok(())
}
```

### Tables Without Primary Keys

Tables don't require a primary key. Without one, the entire row acts as the primary key:

- Rows are identified by their complete content
- Updates require matching all fields
- Duplicate rows are not possible. Inserting an identical row has no effect

SpacetimeDB always maintains set semantics regardless of whether you define a primary key. The difference is what defines uniqueness: a primary key column, or the entire row.

Primary keys add indexing overhead. If your table is only accessed by iterating over all rows (no lookups by key), omitting the primary key can improve performance.

### Common Primary Key Patterns

**Auto-incrementing IDs**: Combine `primaryKey()` with `autoInc()` for automatically assigned unique identifiers:

```
#[spacetimedb::table(accessor = post, public)]
pub struct Post {
    #[primary_key]
    #[auto_inc]
    id: u64,
    title: String,
    content: String,
}
```

**Identity as primary key**: Use the caller's identity as the primary key for user-specific data:

```
#[spacetimedb::table(accessor = user_profile, public)]
pub struct UserProfile {
    #[primary_key]
    identity: Identity,
    display_name: String,
    bio: String,
}
```

This pattern ensures each identity can only have one profile and makes lookups by identity efficient.

## Unique Columns

Mark columns as unique to ensure no two rows can have the same value for that column.

```
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

Use the `#[unique]` attribute.

Unlike primary keys, you can have multiple unique columns on a single table. Unique columns also create an index that enables efficient lookups.

## Primary Keys vs Unique Columns

Both primary keys and unique columns enforce uniqueness, but they serve different purposes:

| Aspect          | Primary Key     | Unique Column    |
| --------------- | --------------- | ---------------- |
| Purpose         | Row identity    | Data integrity   |
| Count per table | One             | Multiple allowed |
| Update behavior | Delete + Insert | In-place update  |
| Required        | No              | No               |
