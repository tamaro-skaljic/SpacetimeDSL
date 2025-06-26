#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct ReferencingTable {
    pub path: Box<str>,
    pub table_name: Box<str>,
}

/*
 * TODO: Reference
 *   - Use the function
 *     - in delete_one:
 *       - for_multi_column_index:
 *         - Change `return [...].delete()` to `[...].delete()?;`
 *         - perform_actions_after_{table_name}_deletion(#field_name_for_found_value.unwrap().#primary_key_column_name)?;
 *         - Ok(true)
 *     - in delete_many:
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
 * TODO: Check if below is true
 * - in column.try_parse all #[foreign_key]'s must be parsed before the SpacetimeDSLColumnMethods are created (currently they are parsed one column by one before in SpacetimeDSLColumn, meaning that a column only knows all foreign keys of itself and the columns parsed before)
*/
