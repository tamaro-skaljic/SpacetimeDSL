use syn::Path;

/// The reducer or procedure a scheduled table calls, from
/// `#[spacetimedb::table(scheduled(...))]`.
///
/// SpacetimeDSL does not read it itself; it is kept for crates building on this one.
#[derive(Clone)]
pub struct ScheduledReducer {
    /// The path of the reducer or procedure as written, such as `run_cleanup` or
    /// `crate::timers::send_reminder`.
    pub reducer_path: Path,
}
