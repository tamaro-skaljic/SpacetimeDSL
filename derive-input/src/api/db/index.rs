#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct Index {
    pub name: Box<str>,
    pub is_unique: bool,
    pub index_type: IndexType,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub enum IndexType {
    /// Available from `SpacetimeDBTable.multi_column_indices`
    BTreeMultiColumn { columns: Vec<Box<str>> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    BTreeSingleColumn { column: Box<str> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    Direct { column: Box<str> },
}