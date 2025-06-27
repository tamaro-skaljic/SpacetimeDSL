use super::{foreign_key::ForeignKey, getter::Getter, setter::Setter, wrapper::WrapperType};
use crate::api::dsl::method::SpacetimeDSLMethod;

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
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

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub enum SpacetimeDSLColumnMethods {
    ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex),
    ForIndex(SpacetimeDSLColumnMethodsForIndex),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct SpacetimeDSLColumnMethodsForUniqueIndex {
    pub get_one_option: SpacetimeDSLMethod,
    pub update: Option<SpacetimeDSLMethod>,
    pub delete_one: SpacetimeDSLMethod,
    pub delete_one_result_type: SpacetimeDSLDeletionResult,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct SpacetimeDSLColumnMethodsForIndex {
    pub get_many: SpacetimeDSLMethod,
    pub delete_many: SpacetimeDSLMethod,
    pub delete_many_result_type: SpacetimeDSLDeletionResult,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct SpacetimeDSLDeletionResult {}
