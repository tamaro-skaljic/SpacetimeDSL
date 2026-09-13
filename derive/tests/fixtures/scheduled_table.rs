//! Covers a scheduled table: `scheduled(<reducer>)` on the `#[table]` attribute plus the
//! `scheduled_id` / `scheduled_at` column pair SpacetimeDB requires, which the generator
//! has to carry through the create argument struct unchanged.

#[spacetimedsl::dsl(plural_name = cleanup_timers, method(update = false))]
#[spacetimedb::table(accessor = cleanup_timer, scheduled(run_cleanup))]
pub struct CleanupTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,

    scheduled_at: spacetimedb::ScheduleAt,
}
