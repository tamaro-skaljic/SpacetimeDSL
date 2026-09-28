use syn::Ident;

/// The reducer a scheduled table calls, from `#[spacetimedb::table(scheduled(...))]`.
#[derive(Clone)]
pub struct ScheduledReducer {
    /// The name of the reducer.
    pub reducer_name: Ident,
}
