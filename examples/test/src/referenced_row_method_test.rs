//! The methods a table adds to the wrapper type of its primary key, which look up the row a
//! foreign key column of the row with that key references: `alliance.get_server_id()
//! .get_lookup_season(dsl)` follows `lookup_server.season_id` without naming the server row.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = lookup_seasons, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_season)]
pub struct LookupSeason {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_server)]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_tournament)]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_active_membership)]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_expired_membership)]
    id: u64,

    max_alliance_level: u32,
}

#[spacetimedsl::dsl(plural_name = lookup_servers, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_server)]
pub struct LookupServer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_alliance)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    season_id: u64,
}

#[spacetimedsl::dsl(plural_name = lookup_alliances, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_alliance)]
pub struct LookupAlliance {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupServerId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_server, column = id)]
    server_id: u64,
}

/// A tournament runs from one season into another, so it references `lookup_season` twice,
/// and each of its methods takes the name of its column.
#[spacetimedsl::dsl(plural_name = lookup_tournaments, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_tournament)]
pub struct LookupTournament {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    opening_season_id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    closing_season_id: u64,
}

/// A chain of stages, each naming the one after it through a unique foreign key to its own
/// table: `LookupStageId` gets `get_lookup_stage_by_next_stage_id`, the stage before, and
/// `get_next_stage`, the stage after.
#[spacetimedsl::dsl(plural_name = lookup_stages, method(update = false, delete = false))]
#[spacetimedb::table(accessor = lookup_stage)]
pub struct LookupStage {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::referenced_row_method_test, table = lookup_stage)]
    id: u64,

    #[unique]
    #[use_wrapper(LookupStageId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_stage, column = id)]
    next_stage_id: u64,
}

/// A membership is active or expired, one table each. `LookupMembershipId` names a row in
/// both, so the struct adds no method for the referenced row; it compiles only while neither
/// of its two expansions adds one.
#[spacetimedsl::dsl(
    plural_name = lookup_active_memberships,
    table = lookup_active_membership,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = lookup_active_membership)]
#[spacetimedsl::dsl(
    plural_name = lookup_expired_memberships,
    table = lookup_expired_membership,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = lookup_expired_membership)]
pub struct LookupMembership {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LookupSeasonId)]
    #[foreign_key(path = crate::referenced_row_method_test, table = lookup_season, column = id)]
    season_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    a_chain_of_foreign_keys_is_followed_from_a_wrapper(dsl)?;
    a_column_referencing_no_row_finds_no_row(dsl)?;
    each_of_several_foreign_keys_to_one_table_is_followed(dsl)?;
    a_unique_foreign_key_to_its_own_table_is_followed_both_ways(dsl)?;
    a_struct_with_several_tables_keeps_its_methods_for_referencing_rows(dsl)?;

    Ok(())
}

fn a_chain_of_foreign_keys_is_followed_from_a_wrapper<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let season = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 30,
    })?;
    let server = dsl.create_lookup_server(CreateLookupServer {
        season_id: season.get_id(),
    })?;
    let alliance = dsl.create_lookup_alliance(CreateLookupAlliance {
        server_id: server.get_id(),
    })?;

    let max_alliance_level = *alliance
        .get_server_id()
        .get_lookup_season(dsl)?
        .get_max_alliance_level();

    if max_alliance_level != 30 {
        return Err(format!(
            "LookupServerId::get_lookup_season should find the season the server references! Got a max_alliance_level of {max_alliance_level}"
        ));
    }

    if alliance.get_id().get_lookup_server(dsl)?.get_id() != server.get_id() {
        return Err(
            "LookupAllianceId::get_lookup_server should find the server the alliance references!"
                .to_string(),
        );
    }

    Ok(())
}

/// `0` references no row, so the lookup finds none.
fn a_column_referencing_no_row_finds_no_row<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let alliance = dsl.create_lookup_alliance(CreateLookupAlliance {
        server_id: LookupServerId::new(0),
    })?;

    match alliance.get_id().get_lookup_server(dsl) {
        Err(SpacetimeDSLError::NotFoundError { table_name, .. })
            if &*table_name == "lookup_server" =>
        {
            Ok(())
        }
        other => Err(format!(
            "An alliance whose server_id is 0 should find no server! Got: {other:?}"
        )),
    }
}

fn each_of_several_foreign_keys_to_one_table_is_followed<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let opening = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 10,
    })?;
    let closing = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 20,
    })?;
    let tournament = dsl.create_lookup_tournament(CreateLookupTournament {
        opening_season_id: opening.get_id(),
        closing_season_id: closing.get_id(),
    })?;

    if tournament.get_id().get_opening_season(dsl)?.get_id() != opening.get_id() {
        return Err(
            "LookupTournamentId::get_opening_season should find the season opening_season_id references!"
                .to_string(),
        );
    }

    if tournament.get_id().get_closing_season(dsl)?.get_id() != closing.get_id() {
        return Err(
            "LookupTournamentId::get_closing_season should find the season closing_season_id references!"
                .to_string(),
        );
    }

    Ok(())
}

fn a_unique_foreign_key_to_its_own_table_is_followed_both_ways<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let last = dsl.create_lookup_stage(CreateLookupStage {
        next_stage_id: LookupStageId::new(0),
    })?;
    let middle = dsl.create_lookup_stage(CreateLookupStage {
        next_stage_id: last.get_id(),
    })?;
    let first = dsl.create_lookup_stage(CreateLookupStage {
        next_stage_id: middle.get_id(),
    })?;

    if first.get_id().get_next_stage(dsl)?.get_id() != middle.get_id() {
        return Err(
            "LookupStageId::get_next_stage should find the stage the first one names!".to_string(),
        );
    }

    if middle
        .get_id()
        .get_lookup_stage_by_next_stage_id(dsl)?
        .get_id()
        != first.get_id()
    {
        return Err(
            "LookupStageId::get_lookup_stage_by_next_stage_id should find the stage which names the middle one!"
                .to_string(),
        );
    }

    Ok(())
}

fn a_struct_with_several_tables_keeps_its_methods_for_referencing_rows<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let season = dsl.create_lookup_season(CreateLookupSeason {
        max_alliance_level: 5,
    })?;
    dsl.create_lookup_active_membership(CreateLookupActiveMembership {
        season_id: season.get_id(),
    })?;

    if season.get_id().get_lookup_active_memberships(dsl).len() != 1 {
        return Err(
            "LookupSeasonId::get_lookup_active_memberships should find the one active membership!"
                .to_string(),
        );
    }

    Ok(())
}
