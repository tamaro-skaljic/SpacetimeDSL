use syn::Ident;

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Index {
    pub name: Ident,
    pub is_unique: bool,
    pub index_type: IndexType,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum IndexType {
    /// Available from `SpacetimeDBTable.multi_column_indices`
    BTreeMultiColumn { columns: Vec<Ident> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    BTreeSingleColumn { column: Ident },
    /// Available from `SpacetimeDBColumn.single_column_index`
    Direct { column: Ident },
}