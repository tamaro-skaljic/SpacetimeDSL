#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct Index {
    pub name: Box<str>,
    pub is_unique: bool,
    pub index_type: IndexType,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub enum IndexType {
    /// Available from `SpacetimeDBTable.multi_column_indices`
    BTreeMultiColumn { columns: Vec<Box<str>> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    BTreeSingleColumn { column: Box<str> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    Direct { column: Box<str> },
}