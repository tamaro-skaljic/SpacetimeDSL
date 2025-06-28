use syn::Ident;

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ScheduledReducer {
    pub reducer_name: Ident,
}