use syn::Ident;

#[derive(Clone)]
pub struct Index {
    pub name: Ident,
    pub is_unique: bool,
    pub index_type: IndexType,
}

#[derive(Clone)]
pub enum IndexType {
    /// Available from `SpacetimeDBTable.multi_column_indices`
    BTreeMultiColumn { columns: Vec<Ident> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    BTreeSingleColumn { column: Ident },
    /// Available from `SpacetimeDBTable.multi_column_indices`
    HashMultiColumn { columns: Vec<Ident> },
    /// Available from `SpacetimeDBColumn.single_column_index`
    HashSingleColumn { column: Ident },
    /// Available from `SpacetimeDBColumn.single_column_index`
    Direct { column: Ident },
}
