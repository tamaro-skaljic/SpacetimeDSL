use syn::Ident;

/// An index SpacetimeDB maintains for the table.
#[derive(Clone)]
pub struct Index {
    /// The accessor of the index: the `accessor` of an `index(...)` of the table, or the name
    /// of the field for an index declared on the field.
    pub name: Ident,
    /// Whether no two rows share a value of the index: a `#[primary_key]` or `#[unique]`
    /// field, or a multi-column index named in `#[dsl(unique_index(name = ...))]`.
    pub is_unique: bool,
    /// The kind of the index and the columns it covers.
    pub index_type: IndexType,
}

/// The kind of an index and the columns it covers, in the order the index declares them.
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
