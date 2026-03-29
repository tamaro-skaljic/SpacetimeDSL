# Default Values

Default values allow you to add new columns to existing tables during [automatic migrations](/docs/databases/automatic-migrations). When you republish a module with a new column that has a default value, existing rows are automatically populated with that default.

> **note**
>
> New columns with default values must be added at the **end** of the table definition. Adding columns in the middle of a table is not supported.

## Defining Default Values

```
#[spacetimedb::table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    id: u64,
    name: String,
    // New columns added with defaults
    #[default(0)]
    score: u32,
    #[default(true)]
    is_active: bool,
}
```

The `#[default(value)]` attribute specifies the default value. The expression must be const-evaluable (usable in a `const` context).

> **Rust Limitation**
>
> Default values in Rust must be const-evaluable. This means you **cannot** use `String` defaults like `#[default("".to_string())]` because `.to_string()` is not a const fn. Only primitive types, enums, and other const-constructible types can have defaults.

## Restrictions

Default values **cannot** be combined with:

- Primary keys
- Unique constraints
- [Auto-increment](/docs/tables/auto-increment)

This restriction exists because these attributes require the database to manage the column values, which conflicts with providing a static default.

## Use Cases

- **Schema evolution**: Add new features to your application without losing existing data
- **Optional fields**: Provide sensible defaults for fields that may not have been tracked historically
- **Feature flags**: Add boolean columns with `default(false)` to enable new functionality gradually
