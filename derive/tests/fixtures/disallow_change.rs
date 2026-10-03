//! Covers `#[disallow(decreasing)]` and `#[disallow(increasing)]` on an unsigned integer, a
//! signed integer and a float, one of them next to `zero`. `update_score_by_id` compares each
//! with the stored row, which it looks up by the primary key itself because no other check
//! does; `update_season_by_id` reuses the row its before hook looked up. The update path of
//! `upsert_league` compares with the row it found, and its insert path compares nothing. A
//! float compares through `partial_cmp`.

#[spacetimedsl::dsl(plural_name = scores, method(update = true))]
#[spacetimedb::table(accessor = score, public)]
pub struct Score {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(zero, decreasing)]
    pub points: u64,

    #[disallow(increasing)]
    pub remaining_attempts: i8,

    #[disallow(decreasing)]
    pub rating: f64,
}

#[spacetimedsl::dsl(plural_name = seasons, method(update = true), hook(before(update)))]
#[spacetimedb::table(accessor = season, public)]
pub struct Season {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(decreasing)]
    pub number: u32,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = league, public)]
pub struct League {
    #[disallow(increasing)]
    pub remaining_rounds: u16,
}
