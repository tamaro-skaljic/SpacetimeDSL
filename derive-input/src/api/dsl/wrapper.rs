#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub enum WrapperType {
    Wrap(Wrap),
    Wrapped(Wrapped),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct Wrap {
    pub wrapper_struct_name: Box<str>,
    pub wrapped_type_name_or_path: Box<str>,
    pub wrapper_impl: Box<str>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct Wrapped {
    pub wrapper_struct_name_or_path: Box<str>,
    pub wrapped_type_name_or_path: Box<str>,
}
