#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct ReferencingTable {
    pub path: Box<str>,
    pub table_name: Box<str>,
}

/*
 * TODO: MUST USE THE DSL METHODS FOR ON DELETION ACTIONS BECAUSE THE DELETION OF A ROW CAN TRIGGER ACTIONS IN OTHER CLASSES
 *
 * TODO: Check if below is true
 * - in column.try_parse all #[foreign_key]'s must be parsed before the SpacetimeDSLColumnMethods are created (currently they are parsed one column by one before in SpacetimeDSLColumn, meaning that a column only knows all foreign keys of itself and the columns parsed before)
*/
