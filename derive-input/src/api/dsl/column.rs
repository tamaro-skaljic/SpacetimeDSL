use super::{foreign_key::ForeignKey, getter::Getter, setter::Setter, wrapper::WrapperType};
use crate::api::dsl::{method::SpacetimeDSLMethod, mut_getter::MutGetter};

#[derive(Clone)]
pub struct SpacetimeDSLColumn {
    pub is_option: bool,
    // Only `Some(T)` if it has `#[create_wrapper(MyTableId)]` or `#[use_wrapper(path = path::to::MyTableId)]`.
    pub wrapper_type: Option<WrapperType>,
    // Only `Some(T)` if it has `#[foreign_key(table = my_table, column = my_column, on_delete = OnDeleteStrategy)]`.
    pub foreign_key: Option<ForeignKey>,
    pub getter: Option<Getter>,
    // Only `Some(T)` if mutable
    pub mut_getter: Option<MutGetter>,
    // Only `Some(T)` if mutable
    pub setter: Option<Setter>,
}

#[derive(Clone)]
pub enum SpacetimeDSLColumnMethods {
    ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex),
    ForIndex(SpacetimeDSLColumnMethodsForIndex),
}

#[derive(Clone)]
pub struct SpacetimeDSLColumnMethodsForUniqueIndex {
    pub get_one_option: SpacetimeDSLMethod,
    pub update: Option<SpacetimeDSLMethod>,
    pub delete_one: SpacetimeDSLMethod,
}

#[derive(Clone)]
pub struct SpacetimeDSLColumnMethodsForIndex {
    pub get_many: SpacetimeDSLMethod,
    pub delete_many: SpacetimeDSLMethod,
}
