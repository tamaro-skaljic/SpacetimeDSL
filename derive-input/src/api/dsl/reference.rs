#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct ReferencingTable {
    pub path: Box<str>,
    pub table_name: Box<str>,
}
