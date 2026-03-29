# Indexes

Indexes accelerate queries by maintaining sorted data structures alongside your tables. Without an index, finding rows that match a condition requires scanning every row. With an index, the database locates matching rows directly.

## When to Use Indexes

Add an index when you frequently query a column with equality or range conditions. Common scenarios include:

- **Filtering by foreign key**: A `player_id` column in an inventory table benefits from an index when you query items belonging to a specific player.
- **Range queries**: An `age` column benefits from an index when you query users within an age range.
- **Sorting**: Columns used for ordering results benefit from indexes that maintain sort order.

Indexes consume additional memory and slow down inserts and updates, since the database must maintain the index structure. Add indexes based on your actual query patterns rather than speculatively.

Primary keys and unique constraints automatically create indexes. You do not need to add a separate index for columns that already have these constraints.

## Index Types

SpacetimeDB supports two index types:

| Type   | Use Case                | Key Types                 | Multi-Column |
| ------ | ----------------------- | ------------------------- | ------------ |
| B-tree | General purpose         | Any                       | Yes          |
| Direct | Dense integer sequences | `u8`, `u16`, `u32`, `u64` | No           |

### B-tree Indexes

B-trees maintain data in sorted order, enabling both equality lookups (`x = 5`) and range queries (`x > 5`, `x >= 1 && x <= 10`). The sorted structure also supports prefix matching on multi-column indexes. B-tree is the default and most commonly used index type.

### Direct Indexes

Direct indexes use array indexing instead of tree traversal, providing O(1) lookups for unsigned integer keys. SpacetimeDB uses the key value directly as an array offset, eliminating the need to search through a tree structure.

Direct indexes perform well when:

- Keys are dense (few gaps between values)
- Keys start near zero
- Insert patterns are sequential rather than random

Direct indexes perform poorly when:

- Keys are sparse (large gaps between values)
- The first key inserted is a large number
- Insert patterns are highly random

Direct indexes only support single-column indexes on unsigned integer types. Use them for auto-increment primary keys or other dense sequential identifiers where you need maximum lookup performance.

> **note**
>
> Direct indexes are currently available in Rust.

```
#[spacetimedb::table(accessor = position, public)]
pub struct Position {
    #[primary_key]
    #[index(direct)]
    id: u32,
    x: f32,
    y: f32,
    z: f32,
}
```

This example from the SpacetimeDB benchmarks uses direct indexes for a million entities with sequential IDs starting at 0, enabling O(1) lookups when joining position and velocity data by entity ID.

For most use cases, B-tree indexes provide good performance without these restrictions. Consider direct indexes only when profiling reveals that index lookups are a bottleneck and your key distribution matches the ideal pattern.

## Single-Column Indexes

A single-column index accelerates queries that filter on one column. You can define the index at the field level or the table level.

### Field-Level Syntax

The field-level syntax places the index declaration directly on the column:

```
#[spacetimedb::table(accessor = user, public)]
pub struct User {
    #[primary_key]
    id: u32,
    #[index(btree)]
    name: String,
    #[index(btree)]
    age: u8,
}
```

### Table-Level Syntax

The table-level syntax defines indexes separately from columns. This approach allows you to name the index explicitly:

```
#[spacetimedb::table(accessor = user, public, index(accessor = idx_age, btree(columns = [age])))]
pub struct User {
    #[primary_key]
    id: u32,
    name: String,
    age: u8,
}
```

## Multi-Column Indexes

A multi-column index (also called a composite index) spans multiple columns. The index maintains rows sorted by the first column, then by the second column within equal values of the first, and so on.

Multi-column indexes support:

- **Full match**: Queries that specify all indexed columns
- **Prefix match**: Queries that specify the leftmost columns in order
- **Range on trailing column**: A prefix of equality conditions followed by a range on the next column

A multi-column index on `(player_id, level)` accelerates these lookups:

- `player_id` equals 123 (prefix match on first column)
- `player_id` equals 123 and `level` equals 5 (full match)
- `player_id` equals 123 and `level` greater than 5 (prefix match with range)

The same index does not accelerate a lookup on `level` alone, since `level` is not a prefix of the index.

```
#[spacetimedb::table(accessor = score, public, index(accessor = by_player_and_level, btree(columns = [player_id, level])))]
pub struct Score {
    player_id: u32,
    level: u32,
    points: i64,
}
```

## Querying with Indexes

SpacetimeDB generates type-safe accessor methods for each index. These methods accept filter arguments and return matching rows.

### Equality Queries

Pass a single value to find rows where the indexed column equals that value:

```
// Find users with a specific name
for user in ctx.db.user().name().filter("Alice") {
    log::info!("Found user: {}", user.id);
}
```

### Range Queries

Pass a range to find rows where the indexed column falls within bounds:

```
// Find users aged 18 to 65 (inclusive)
for user in ctx.db.user().age().filter(18..=65) {
    log::info!("{} is {}", user.name, user.age);
}

// Find users aged 18 or older
for user in ctx.db.user().age().filter(18..) {
    log::info!("{} is an adult", user.name);
}

// Find users younger than 18
for user in ctx.db.user().age().filter(..18) {
    log::info!("{} is a minor", user.name);
}
```

### Multi-Column Queries

For multi-column indexes, pass a tuple of values. You can specify exact values for prefix columns and optionally a range for the trailing column:

```
// Find all scores for player 123 (prefix match)
for score in ctx.db.score().by_player_and_level().filter(&123u32) {
    log::info!("Level {}: {} points", score.level, score.points);
}

// Find scores for player 123 at levels 1-10
for score in ctx.db.score().by_player_and_level().filter((123u32, 1u32..=10u32)) {
    log::info!("Level {}: {} points", score.level, score.points);
}

// Find the exact score for player 123 at level 5
for score in ctx.db.score().by_player_and_level().filter((123u32, 5u32)) {
    log::info!("Points: {}", score.points);
}
```

## Deleting with Indexes

Indexes also accelerate deletions. Instead of scanning the entire table to find rows to delete, you can delete directly by index value:

```
// Delete all users named "Alice"
let deleted = ctx.db.user().name().delete("Alice");
log::info!("Deleted {} user(s)", deleted);

// Delete users in an age range
let deleted = ctx.db.user().age().delete(..18);
log::info!("Deleted {} minor(s)", deleted);
```

## Index Design Guidelines

**Choose columns based on query patterns.** Index the columns that appear in your filter conditions and join lookups. An unused index wastes memory.

**Consider column order in multi-column indexes.** Place the most selective column (the one that narrows results most) first, followed by columns used in range conditions. An index on `(country, city)` works for lookups on `country` alone or both `country` and `city`, but not for lookups on `city` alone.

**Avoid redundant indexes.** A multi-column index on `(a, b)` makes a separate index on `(a)` redundant, since the multi-column index handles prefix queries. However, an index on `(b)` is not redundant if you query `b` independently.

**Balance read and write performance.** Each index speeds up reads but slows down writes. Tables with high write volume and few reads may benefit from fewer indexes.

## Next Steps

- Learn about [Constraints](/docs/tables/constraints) for primary keys and unique indexes
- See [Access Permissions](/docs/tables/access-permissions) for querying tables from reducers
