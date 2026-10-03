//! `#[disallow(decreasing)]` and `#[disallow(increasing)]`: a column whose value an update must
//! not lower, or must not raise.
//!
//! `update_<table>_by_<key>` and the update path of `upsert_<table>` compare the value they
//! write with the stored one; the insert path of `upsert_<table>` has nothing to compare with.
//! A float compares through `partial_cmp`: a change to or from NaN breaks both rules, and an
//! unchanged value, NaN included, breaks neither.

use crate::{disallow_zero_test::expect_disallowed, spacetimedsl::prelude::*};

#[spacetimedsl::dsl(plural_name = disallow_change_scores, method(update = true))]
#[spacetimedb::table(accessor = disallow_change_score)]
pub struct DisallowChangeScore {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(decreasing)]
    pub points: i64,

    #[disallow(increasing)]
    pub remaining_attempts: u8,

    #[disallow(decreasing)]
    pub rating: f64,

    pub label: String,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = disallow_change_season)]
pub struct DisallowChangeSeason {
    #[disallow(decreasing)]
    pub number: u32,
}

impl DefaultSingleton for DisallowChangeSeason {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<DisallowChangeSeason, SpacetimeDSLError> {
        Ok(DisallowChangeSeason { id: 0, number: 1 })
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    update_refuses_a_decrease(dsl)?;
    update_refuses_an_increase(dsl)?;
    update_accepts_the_allowed_changes(dsl)?;
    an_unchanged_nan_is_no_change(dsl)?;
    a_change_to_or_from_nan_is_refused(dsl)?;
    upsert_compares_only_while_the_row_exists(dsl)?;

    Ok(())
}

fn create_score<T: WriteContext>(
    dsl: &DSL<'_, T>,
    rating: f64,
) -> Result<DisallowChangeScore, SpacetimeDSLError> {
    dsl.create_disallow_change_score(CreateDisallowChangeScore {
        points: 10,
        remaining_attempts: 3,
        rating,
        label: "first".to_string(),
    })
}

/// The message of an update of `score` which moved `column` from `stored` to `written`.
fn change_message(
    score: &DisallowChangeScore,
    column: &str,
    verb: &str,
    stored: impl std::fmt::Display,
    written: impl std::fmt::Display,
    rule: &str,
) -> String {
    format!(
        "Disallowed Value Error while trying to update the row `{{ id : {} }}` in the `disallow_change_score` table because `{column}` would {verb} from `{stored}` to `{written}`, which `#[disallow({rule})]` forbids!",
        score.get_id().value()
    )
}

fn update_refuses_a_decrease<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, 1.5)?;
    let expected_message = change_message(&score, "points", "decrease", 10, 5, "decreasing");
    score.set_points(5);

    expect_disallowed(
        dsl.update_disallow_change_score_by_id(score),
        &expected_message,
    )
}

fn update_refuses_an_increase<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, 1.5)?;
    let expected_message =
        change_message(&score, "remaining_attempts", "increase", 3, 4, "increasing");
    score.set_remaining_attempts(4);

    expect_disallowed(
        dsl.update_disallow_change_score_by_id(score),
        &expected_message,
    )
}

fn update_accepts_the_allowed_changes<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, 1.5)?;
    score.set_points(15);
    score.set_remaining_attempts(2);
    score.set_rating(2.5);

    let mut score = dsl
        .update_disallow_change_score_by_id(score)
        .map_err(|error| {
            format!("Raising points and rating and lowering remaining_attempts should be allowed! Got:\n{error}")
        })?;

    score.set_label("second".to_string());

    dsl.update_disallow_change_score_by_id(score)
        .map_err(|error| {
            format!("An update which changes none of the guarded columns should be allowed! Got:\n{error}")
        })?;

    Ok(())
}

/// A NaN compares unordered with itself, but the stored and the written value are the same
/// bits, so the column did not change.
fn an_unchanged_nan_is_no_change<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut score = create_score(dsl, f64::NAN)?;
    score.set_label("second".to_string());

    dsl.update_disallow_change_score_by_id(score)
        .map_err(|error| {
            format!("An update which keeps a NaN rating should be allowed! Got:\n{error}")
        })?;

    Ok(())
}

fn a_change_to_or_from_nan_is_refused<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut from_nan = create_score(dsl, f64::NAN)?;
    let expected_message =
        change_message(&from_nan, "rating", "decrease", f64::NAN, 1.0, "decreasing");
    from_nan.set_rating(1.0);

    expect_disallowed(
        dsl.update_disallow_change_score_by_id(from_nan),
        &expected_message,
    )?;

    let mut to_nan = create_score(dsl, 1.0)?;
    let expected_message =
        change_message(&to_nan, "rating", "decrease", 1.0, f64::NAN, "decreasing");
    to_nan.set_rating(f64::NAN);

    expect_disallowed(
        dsl.update_disallow_change_score_by_id(to_nan),
        &expected_message,
    )
}

/// The insert path of `upsert_disallow_change_season` writes the first row, which no stored
/// row constrains; the update path compares with the stored one.
fn upsert_compares_only_while_the_row_exists<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let mut season = dsl.get_disallow_change_season()?;
    season.set_number(5);
    let mut season = dsl.upsert_disallow_change_season(season)?;

    season.set_number(3);
    expect_disallowed(
        dsl.upsert_disallow_change_season(season.clone()),
        "Disallowed Value Error while trying to update the row `{ id : 0 }` in the `disallow_change_season` table because `number` would decrease from `5` to `3`, which `#[disallow(decreasing)]` forbids!",
    )?;

    season.set_number(6);
    dsl.upsert_disallow_change_season(season)
        .map_err(|error| format!("Raising the season number should be allowed! Got:\n{error}"))?;

    Ok(())
}
