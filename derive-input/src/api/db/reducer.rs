use syn::Ident;

#[derive(Debug, Clone)]
pub struct ScheduledReducer {
    pub reducer_name: Ident,
}
