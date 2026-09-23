use super::{
    auto_gen::UUIDVersion, foreign_key::ForeignKey, getter::Getter, setter::Setter,
    wrapper::WrapperType,
};
use crate::api::dsl::{method::SpacetimeDSLMethod, mut_getter::MutGetter};

#[derive(Clone)]
pub struct SpacetimeDSLColumn {
    pub is_option: bool,
    // Only `Some(T)` if it has `#[create_wrapper(MyTableId)]` or `#[use_wrapper(path = path::to::MyTableId)]`.
    pub wrapper_type: Option<WrapperType>,
    // Only `Some(T)` if it has `#[foreign_key(table = my_table, column = my_column, on_delete = OnDeleteStrategy)]`.
    pub foreign_key: Option<ForeignKey>,
    // Only `Some(T)` if it has `#[auto_gen(v4)]` or `#[auto_gen(v7)]`.
    pub auto_generated_uuid_version: Option<UUIDVersion>,
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
    // Only `Some(T)` if the table has an update method and this index is the primary key.
    pub update: Option<SpacetimeDSLMethod>,
    // Only `Some(T)` if the table has a delete method.
    pub delete_one: Option<SpacetimeDSLMethod>,
    // Only `Some(T)` if the table is soft-deletable.
    pub soft_delete_one: Option<SpacetimeDSLMethod>,
}

#[derive(Clone)]
pub struct SpacetimeDSLColumnMethodsForIndex {
    pub get_many: SpacetimeDSLMethod,
    // Only `Some(T)` if the table has a delete method.
    pub delete_many: Option<SpacetimeDSLMethod>,
    // Only `Some(T)` if the table is soft-deletable.
    pub soft_delete_many: Option<SpacetimeDSLMethod>,
}
