//! `#[disallow(...)]` in the writes which are neither a create nor an update: a soft deletion,
//! and the rows an `on_delete = SetZero` or an `on_soft_delete = SoftDelete` cascade writes.
//!
//! The before hooks of `disallow_member` and `disallow_guest` lower `experience` when the row
//! asks for it, which `#[disallow(decreasing)]` forbids. `soft_delete_*` then fails with the
//! rule's error, and a cascade stops with it as the error which stopped the cascade. Badges
//! reference members, so a member's soft deletion cascades further; nothing references a
//! guest.

use crate::{disallow_zero_test::expect_disallowed, spacetimedsl::prelude::*};

#[spacetimedsl::dsl(
    plural_name = disallow_guilds,
    method(update = false, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = disallow_guild)]
pub struct DisallowGuild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(DisallowGuildId)]
    #[referenced_by(path = crate::disallow_soft_delete_and_cascade_test, table = disallow_member)]
    #[referenced_by(path = crate::disallow_soft_delete_and_cascade_test, table = disallow_guest)]
    id: u64,

    deleted: bool,
}

#[spacetimedsl::dsl(
    plural_name = disallow_members,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete))
)]
#[spacetimedb::table(accessor = disallow_member)]
pub struct DisallowMember {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(DisallowMemberId)]
    #[referenced_by(path = crate::disallow_soft_delete_and_cascade_test, table = disallow_badge)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DisallowGuildId)]
    #[foreign_key(
        path = crate::disallow_soft_delete_and_cascade_test,
        table = disallow_guild,
        column = id,
        on_delete = SetZero,
        on_soft_delete = SoftDelete
    )]
    pub guild_id: u64,

    #[disallow(decreasing)]
    pub experience: u32,

    /// Whether the before hooks lower `experience`.
    lowers_experience: bool,

    deleted: bool,
}

#[spacetimedsl::dsl(plural_name = disallow_badges, method(update = false, delete = true))]
#[spacetimedb::table(accessor = disallow_badge)]
pub struct DisallowBadge {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DisallowMemberId)]
    #[foreign_key(
        path = crate::disallow_soft_delete_and_cascade_test,
        table = disallow_member,
        column = id,
        on_soft_delete = Ignore
    )]
    member_id: u64,
}

#[spacetimedsl::dsl(
    plural_name = disallow_guests,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete))
)]
#[spacetimedb::table(accessor = disallow_guest)]
pub struct DisallowGuest {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DisallowGuildId)]
    #[foreign_key(
        path = crate::disallow_soft_delete_and_cascade_test,
        table = disallow_guild,
        column = id,
        on_delete = SetZero,
        on_soft_delete = SoftDelete
    )]
    pub guild_id: u64,

    #[disallow(decreasing)]
    pub experience: u32,

    /// Whether the before hooks lower `experience`.
    lowers_experience: bool,

    deleted: bool,
}

#[spacetimedsl::hook]
fn before_disallow_member_update(
    _dsl: &DSL<'_, T>,
    old_member: &DisallowMember,
    mut new_member: DisallowMember,
) -> Result<DisallowMember, SpacetimeDSLError> {
    if *old_member.get_lowers_experience() {
        new_member.set_experience(*old_member.get_experience() - 1);
    }

    Ok(new_member)
}

#[spacetimedsl::hook]
fn before_disallow_member_soft_delete(
    _dsl: &DSL<'_, T>,
    old_member: &DisallowMember,
    mut new_member: DisallowMember,
) -> Result<DisallowMember, SpacetimeDSLError> {
    if *old_member.get_lowers_experience() {
        new_member.set_experience(*old_member.get_experience() - 1);
    }

    Ok(new_member)
}

#[spacetimedsl::hook]
fn before_disallow_guest_update(
    _dsl: &DSL<'_, T>,
    old_guest: &DisallowGuest,
    mut new_guest: DisallowGuest,
) -> Result<DisallowGuest, SpacetimeDSLError> {
    if *old_guest.get_lowers_experience() {
        new_guest.set_experience(*old_guest.get_experience() - 1);
    }

    Ok(new_guest)
}

#[spacetimedsl::hook]
fn before_disallow_guest_soft_delete(
    _dsl: &DSL<'_, T>,
    old_guest: &DisallowGuest,
    mut new_guest: DisallowGuest,
) -> Result<DisallowGuest, SpacetimeDSLError> {
    if *old_guest.get_lowers_experience() {
        new_guest.set_experience(*old_guest.get_experience() - 1);
    }

    Ok(new_guest)
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    soft_delete_refuses_what_its_hook_breaks(dsl)?;
    set_zero_cascade_stops_at_a_broken_rule(dsl)?;
    soft_delete_cascade_stops_at_a_broken_rule(dsl)?;
    writes_which_keep_the_rules_pass(dsl)?;

    Ok(())
}

fn create_member<T: WriteContext>(
    dsl: &DSL<'_, T>,
    guild: &DisallowGuild,
    lowers_experience: bool,
) -> Result<DisallowMember, SpacetimeDSLError> {
    dsl.create_disallow_member(CreateDisallowMember {
        guild_id: guild.get_id(),
        experience: 10,
        lowers_experience,
    })
}

fn create_guest<T: WriteContext>(
    dsl: &DSL<'_, T>,
    guild: &DisallowGuild,
    lowers_experience: bool,
) -> Result<DisallowGuest, SpacetimeDSLError> {
    dsl.create_disallow_guest(CreateDisallowGuest {
        guild_id: guild.get_id(),
        experience: 10,
        lowers_experience,
    })
}

/// The message of the broken rule, written by `write` into the row `row_id` of `table`.
fn decrease_message(write: &str, row_id: u64, table: &str) -> String {
    format!(
        "Disallowed Value Error while trying to {write} the row `{{ id : {row_id} }}` in the `{table}` table because `experience` would decrease from `10` to `9`, which `#[disallow(decreasing)]` forbids!"
    )
}

/// Checks that `removal` failed with an error which contains `expected_part`.
fn expect_failure_containing(
    removal: Result<DeletionResult, SpacetimeDSLError>,
    expected_part: &str,
) -> Result<(), String> {
    match removal {
        Ok(deletion_result) => Err(format!(
            "The removal should have failed with \"{expected_part}\"! Got:\n{deletion_result}"
        )),
        Err(error) if error.to_string().contains(expected_part) => Ok(()),
        Err(error) => Err(format!(
            "The error should contain \"{expected_part}\"! Got:\n{error}"
        )),
    }
}

fn soft_delete_refuses_what_its_hook_breaks<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let guild = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild, true)?;

    expect_disallowed(
        dsl.soft_delete_disallow_member_by_id(&member),
        &decrease_message("soft delete", member.get_id().value(), "disallow_member"),
    )
}

/// Deleting a guild clears `guild_id` of its members, an update of each member row.
fn set_zero_cascade_stops_at_a_broken_rule<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let guild = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild, true)?;

    expect_failure_containing(
        dsl.delete_disallow_guild_by_id(&guild),
        &format!(
            "Error which stopped the cascade: {}",
            decrease_message("update", member.get_id().value(), "disallow_member")
        ),
    )
}

/// Retiring a guild retires its members and guests; a member's retirement cascades further,
/// a guest's does not.
fn soft_delete_cascade_stops_at_a_broken_rule<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let guild_of_a_member = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild_of_a_member, true)?;

    expect_failure_containing(
        dsl.soft_delete_disallow_guild_by_id(&guild_of_a_member),
        &format!(
            "Error which stopped the cascade: {}",
            decrease_message("soft delete", member.get_id().value(), "disallow_member")
        ),
    )?;

    let guild_of_a_guest = dsl.create_disallow_guild()?;
    let guest = create_guest(dsl, &guild_of_a_guest, true)?;

    expect_failure_containing(
        dsl.soft_delete_disallow_guild_by_id(&guild_of_a_guest),
        &format!(
            "Error which stopped the cascade: {}",
            decrease_message("soft delete", guest.get_id().value(), "disallow_guest")
        ),
    )
}

fn writes_which_keep_the_rules_pass<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let guild = dsl.create_disallow_guild()?;
    let member = create_member(dsl, &guild, false)?;
    dsl.soft_delete_disallow_member_by_id(&member)
        .map_err(|error| {
            format!("A soft deletion which keeps the rules should pass! Got:\n{error}")
        })?;

    let deleted_guild = dsl.create_disallow_guild()?;
    create_member(dsl, &deleted_guild, false)?;
    dsl.delete_disallow_guild_by_id(&deleted_guild)
        .map_err(|error| {
            format!("A SetZero cascade which keeps the rules should pass! Got:\n{error}")
        })?;

    let retired_guild = dsl.create_disallow_guild()?;
    create_guest(dsl, &retired_guild, false)?;
    dsl.soft_delete_disallow_guild_by_id(&retired_guild)
        .map_err(|error| {
            format!("A SoftDelete cascade which keeps the rules should pass! Got:\n{error}")
        })?;

    Ok(())
}
