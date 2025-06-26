#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct Getter {
    pub method_name: Box<str>,
    pub return_type: Box<str>,
    pub method_impl: Box<str>,
}
