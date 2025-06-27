#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct ScheduledReducer {
    pub reducer_name: Box<str>,
}