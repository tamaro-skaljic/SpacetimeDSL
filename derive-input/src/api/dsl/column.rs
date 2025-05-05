use super::{foreign_key::ForeignKey, getter::Getter, setter::Setter, wrapper::WrapperType};

#[cfg_attr(feature = "clone", derive(Clone))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "partial-eq", derive(PartialEq))]
#[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
#[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
pub struct SpacetimeDSLColumn {
    pub is_option: bool,
    // Only `Some(T)` if it has `#[wrap(name = MyTableId)]` or `#[wrapped(path = path::to::MyTableId)]`.
    pub wrapper_type: Option<WrapperType>,
    // Only `Some(T)` if it has `#[foreign_key(table = my_table, column = my_column, on_delete = OnDeleteStrategy)]`.
    pub foreign_key: Option<ForeignKey>,
    pub getter: Getter,
    // Only `Some(T)` if mutable
    pub setter: Option<Setter>,
}
