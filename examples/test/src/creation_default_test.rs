//! `#[creation_default(...)]`: the value `create_<table>` fills a column with, so
//! `Create<Table>` does not ask the caller for it.
//!
//! `create_creation_default_player` is called with `name` alone, which compiles only while
//! every defaulted column is left out of `CreateCreationDefaultPlayer`. `team_id` defaults to
//! `0`, which references no row, so the reference check of the create method skips it.

use crate::spacetimedsl::prelude::*;

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum CreationDefaultMembership {
    Trial,
    Paid,
}

#[spacetimedsl::dsl(
    plural_name = creation_default_teams,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = creation_default_team)]
pub struct CreationDefaultTeam {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::creation_default_test, table = creation_default_player)]
    id: u64,
}

#[spacetimedsl::dsl(
    plural_name = creation_default_players,
    method(update = true, delete = true)
)]
#[spacetimedb::table(accessor = creation_default_player)]
pub struct CreationDefaultPlayer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub name: String,

    #[creation_default(100)]
    pub coins: u32,

    #[creation_default(-1)]
    rank: i32,

    #[creation_default(CreationDefaultMembership::Trial)]
    membership: CreationDefaultMembership,

    #[creation_default(String::from("rookie"))]
    pub title: String,

    #[index(btree)]
    #[use_wrapper(CreationDefaultTeamId)]
    #[foreign_key(
        path = crate::creation_default_test,
        table = creation_default_team,
        column = id
    )]
    #[creation_default(0)]
    pub team_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let player = dsl.create_creation_default_player(CreateCreationDefaultPlayer {
        name: "Ada".to_string(),
    })?;

    if *player.get_coins() != 100 {
        return Err(format!(
            "`coins` should hold its creation default 100! Got: {}",
            player.get_coins()
        ));
    }

    if *player.get_rank() != -1 {
        return Err(format!(
            "`rank` should hold its creation default -1! Got: {}",
            player.get_rank()
        ));
    }

    if *player.get_membership() != CreationDefaultMembership::Trial {
        return Err(format!(
            "`membership` should hold its creation default Trial! Got: {:?}",
            player.get_membership()
        ));
    }

    if player.get_title() != "rookie" {
        return Err(format!(
            "`title` should hold its creation default \"rookie\"! Got: {}",
            player.get_title()
        ));
    }

    if player.get_team_id().value() != 0 {
        return Err(format!(
            "`team_id` should hold its creation default 0, which references no row! Got: {}",
            player.get_team_id()
        ));
    }

    Ok(())
}
