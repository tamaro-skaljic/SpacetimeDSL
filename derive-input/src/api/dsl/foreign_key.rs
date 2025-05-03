#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct ForeignKey {
    pub table_name: Box<str>,
    pub column_name: Box<str>,
    // TODO: Implement
    pub on_delete_strategy: OnDeleteStrategy,
}

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub enum OnDeleteStrategy {
    /// Available independent from the column type.
    Cascade,
    /// Available only for columns with type `Option<T>`.
    SetNone,
    /// Available only for columns with a numeric type.
    SetZero,
}
