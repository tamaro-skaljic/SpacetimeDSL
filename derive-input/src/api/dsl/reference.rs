#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct ReferencingTable {
    pub path: Box<str>,
    pub table_name: Box<str>,
}

/*
 * TODO: Reference
 *   - Use the function
 *     - in delete_one:
 *       - for_single_column_index:
 *         - Change `return [...].delete()` to `[...].delete()?;`
 *         - perform_actions_after_{table_name}_deletion(#column_value)?;
 *         - Ok(true)
 *       - for_multi_column_index:
 *         - Change `return [...].delete()` to `[...].delete()?;`
 *         - perform_actions_after_{table_name}_deletion(#field_name_for_found_value.unwrap().#primary_key_column_name)?;
 *         - Ok(true)
 *     - in delete_many:
 *       - for_single_column_index:
 *         - After `#into_option`
 *           - let #plural_table_name = self.ctx().db().#table_name().#column_name().filter(#column_value).map(|#table_name| => #table_name.#primary_key_column_name).collect();
 *         - Change `return [...].delete()` to `let count = [...].delete()?;`
 *         - perform_actions_after_{table_name}_deletions(#plural_table_name)?;
 *         - Ok(count)
 *       - for_multi_column_index:
 *         - After `#into_option`
 *           - let #plural_table_name = self.ctx().db().#table_name().#index_name().filter((#(#column_values),*)).map(|#table_name| => #table_name.#primary_key_column_name).collect();
 *         - Change `return [...].delete()` to `let count = [...].delete()?;`
 *         - perform_actions_after_{table_name}_deletions(#plural_table_name)?;
 *         - Ok(count)
 */

/*
 * TODO: MUST USE THE DSL METHODS FOR ON DELETION ACTIONS BECAUSE THE DELETION OF A ROW CAN TRIGGER ACTIONS IN OTHER CLASSES
 *
 * - in column.try_parse all #[foreign_key]'s must be parsed before the SpacetimeDSLColumnMethods are created (currently they are parsed one column by one before in SpacetimeDSLColumn, meaning that a column only knows all foreign keys of itself and the columns parsed before)
 * - If there are any foreign keys
 *   - There must be a data structure like HashMap<TableName, HashMap<OnDeleteStrategy, Vec<ColumnName>>>
 * - For each table
 *   - (the dsl reference is passed as argument to any function as well, though it isn't written below)
 *   - Generate a function (for delete_one) in the same module.
 *     - Name: perform_{table_name}_actions_after_{foreign_table_name}_deletion
 *     - Arg: #foreign_table_name: &#column_type
 *     - Return Type: Result<(), UniqueConstraintViolationError>
 *     - Impl:
 *       - For each OnDeleteStrategy (Sort Order: Error, Cascade, SetNone, SetZero):
 *         - For Unique Indices
```
match dsl.ctx().db().#table_name().#column_name().find(#foreign_table_name){
    Some(#table_name) => {
        #on_delete_action
    },
    None => {
    }
};
```
*          - For Indices
```
match dsl.ctx().db().#table_name().#column_name().filter(#foreign_table_name){
    Some(#plural_table_name) => {
        #on_delete_action
    },
    None => {
    }
};
```
 *       - Ok(())
 *   - Generate another function (delete_many) in the same module.
 *     - Name: perform_{table_name}_actions_after_{foreign_table_name}_deletions
 *     - Arg: #plural_table_name: Vec<&#column_type>
 *     - Return Type: Result<(), UniqueConstraintViolationError>
 *     - Impl:
 *       - For each OnDeleteStrategy (Sort Order: Error, Cascade, SetNone, SetZero):
 *         - TODO
 *       - Ok(())
 */
