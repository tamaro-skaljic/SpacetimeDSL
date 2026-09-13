//! Covers two `#[dsl]` attributes on one struct, which the compiler expands one after the
//! other. The second pass finds the `derive(SpacetimeDSL)` helper the first pass added, so
//! `first_dsl_attribute` is false and the wrapper types and the accessors are emitted only
//! once.
//!
//! Snapshotted as `pass_1` and `pass_2`; the difference between the two directories is
//! exactly what `first_dsl_attribute` controls.

#[spacetimedsl::dsl(
    plural_name = modules1,
    method(update = true),
    unique_index(name = database_and_name),
)]
#[spacetimedb::table(
    accessor = module1,
    index(accessor = database_and_name, btree(columns = [database_id, name])),
    public,
)]
#[spacetimedsl::dsl(
    plural_name = modules2,
    method(update = true),
    unique_index(name = name_and_database),
)]
#[spacetimedb::table(
    accessor = module2,
    index(accessor = name_and_database, btree(columns = [name, database_id])),
    public,
)]
pub struct Module {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub database_id: u64,

    pub name: String,
}
