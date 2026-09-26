//! Which hooks run when something other than `update_guild_member_by_id` writes a member row.
//!
//! `on_delete = SetZero` writes every row which referenced the deleted one, so the update
//! hooks of the referencing table see that write and its `set_on_update` column records it,
//! as with any other update. A soft deletion writes the row as well, but it is not an update:
//! it runs the soft-delete hooks alone, whether it is called directly or reached through
//! `on_soft_delete = SoftDelete`.

use crate::spacetimedsl::prelude::*;

/// What a locked member's before-update hook says when it refuses.
const LOCKED_GUILD_MEMBER_MESSAGE: &str = "this guild member is locked";

#[spacetimedsl::dsl(
    plural_name = guilds,
    method(update = false, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = guild)]
pub struct Guild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(GuildId)]
    #[referenced_by(path = crate::update_and_soft_delete_hook_test, table = guild_member)]
    id: u64,

    deleted: bool,
}

/// A member outlives a deleted guild and keeps a `guild_id` of 0 afterwards, but is
/// retired together with a retired one.
#[spacetimedsl::dsl(
    plural_name = guild_members,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete), after(update, soft_delete))
)]
#[spacetimedb::table(accessor = guild_member)]
pub struct GuildMember {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(GuildId)]
    #[foreign_key(
        path = crate::update_and_soft_delete_hook_test,
        table = guild,
        column = id,
        on_delete = SetZero,
        on_soft_delete = SoftDelete
    )]
    pub guild_id: u64,

    locked: bool,

    #[set_on_update]
    modified_at: Option<Timestamp>,

    deleted: bool,
}

/// One row per hook call on `guild_member`, in the order the hooks ran, naming the hook
/// and, for an update hook, the change of `guild_id` it saw.
#[spacetimedsl::dsl(
    plural_name = guild_member_hook_calls,
    method(update = false, delete = false)
)]
#[spacetimedb::table(accessor = guild_member_hook_call)]
pub struct GuildMemberHookCall {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    description: String,
}

#[spacetimedsl::hook]
fn before_guild_member_update(
    dsl: &DSL<'_, T>,
    old_guild_member: &GuildMember,
    new_guild_member: GuildMember,
) -> Result<GuildMember, SpacetimeDSLError> {
    if *old_guild_member.get_locked() {
        return Err(SpacetimeDSLError::Error(
            LOCKED_GUILD_MEMBER_MESSAGE.to_string(),
        ));
    }

    dsl.create_guild_member_hook_call(CreateGuildMemberHookCall {
        description: format!(
            "before_guild_member_update: guild_id {} -> {}",
            old_guild_member.get_guild_id().value(),
            new_guild_member.get_guild_id().value()
        ),
    })?;

    Ok(new_guild_member)
}

#[spacetimedsl::hook]
fn after_guild_member_update(
    dsl: &DSL<'_, T>,
    old_guild_member: &GuildMember,
    new_guild_member: &GuildMember,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_guild_member_hook_call(CreateGuildMemberHookCall {
        description: format!(
            "after_guild_member_update: guild_id {} -> {}",
            old_guild_member.get_guild_id().value(),
            new_guild_member.get_guild_id().value()
        ),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_guild_member_soft_delete(
    dsl: &DSL<'_, T>,
    _old_guild_member: &GuildMember,
    new_guild_member: GuildMember,
) -> Result<GuildMember, SpacetimeDSLError> {
    dsl.create_guild_member_hook_call(CreateGuildMemberHookCall {
        description: "before_guild_member_soft_delete".to_string(),
    })?;

    Ok(new_guild_member)
}

#[spacetimedsl::hook]
fn after_guild_member_soft_delete(
    dsl: &DSL<'_, T>,
    _old_guild_member: &GuildMember,
    _new_guild_member: &GuildMember,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_guild_member_hook_call(CreateGuildMemberHookCall {
        description: "after_guild_member_soft_delete".to_string(),
    })?;

    Ok(())
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    set_zero_update_hook_test(dsl)?;

    soft_delete_skips_update_hooks_test(dsl)?;

    Ok(())
}

/// Deleting a guild sets `guild_id` of its members to 0 through `on_delete = SetZero`.
/// That is an update of each member row, so it runs the update hooks of `guild_member`
/// around the write, stops at an error one of them returns, and sets `modified_at` the way
/// `update_guild_member_by_id` does.
fn set_zero_update_hook_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let guild_of_a_locked_member = dsl.create_guild()?;

    dsl.create_guild_member(CreateGuildMember {
        guild_id: guild_of_a_locked_member.get_id(),
        locked: true,
    })?;

    match dsl.delete_guild_by_id(&guild_of_a_locked_member) {
        Ok(_) => {
            return Err(
                "Deleting a guild whose member's before_update hook refuses should fail!"
                    .to_string(),
            );
        }
        Err(error) => {
            let error = error.to_string();
            if !error.contains(LOCKED_GUILD_MEMBER_MESSAGE) {
                return Err(format!(
                    "The error the before_update hook raised during on_delete = SetZero should reach the caller! Got:\n{error}"
                ));
            }
        }
    };

    let guild = dsl.create_guild()?;
    let guild_id = guild.get_id().value();

    let member = dsl.create_guild_member(CreateGuildMember {
        guild_id: guild.get_id(),
        locked: false,
    })?;

    let logged_before = guild_member_hook_calls(dsl).len();

    dsl.delete_guild_by_id(&guild)?;

    let hook_calls = guild_member_hook_calls(dsl).split_off(logged_before);

    let member = dsl.get_guild_member_by_id(&member)?;

    if member.get_guild_id().value().ne(&0) {
        return Err(
            "Deleting the Guild should have set guild_id of its GuildMember to 0 through on_delete = SetZero!"
                .to_string(),
        );
    }

    let expected_hook_calls = vec![
        format!("before_guild_member_update: guild_id {guild_id} -> 0"),
        format!("after_guild_member_update: guild_id {guild_id} -> 0"),
    ];

    if hook_calls.ne(&expected_hook_calls) {
        return Err(format!(
            "Setting guild_id to 0 through on_delete = SetZero should run the update hooks of guild_member!\n\nExpected:\n{expected_hook_calls:?}\n\nActual:\n{hook_calls:?}"
        ));
    }

    if member.get_modified_at().is_none() {
        return Err(
            "Setting guild_id to 0 through on_delete = SetZero should set modified_at, as update_guild_member_by_id does!"
                .to_string(),
        );
    }

    Ok(())
}

/// A soft deletion writes the member row as well, but it is not an update: it runs the
/// soft-delete hooks of `guild_member` and none of its update hooks, both when it is called
/// directly and when `on_soft_delete = SoftDelete` reaches it.
fn soft_delete_skips_update_hooks_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let expected_hook_calls = vec![
        "before_guild_member_soft_delete".to_string(),
        "after_guild_member_soft_delete".to_string(),
    ];

    let guild = dsl.create_guild()?;

    let member = dsl.create_guild_member(CreateGuildMember {
        guild_id: guild.get_id(),
        locked: false,
    })?;

    let logged_before = guild_member_hook_calls(dsl).len();

    dsl.soft_delete_guild_member_by_id(&member)?;

    let hook_calls = guild_member_hook_calls(dsl).split_off(logged_before);

    if hook_calls.ne(&expected_hook_calls) {
        return Err(format!(
            "soft_delete_guild_member_by_id should run the soft-delete hooks of guild_member and none of its update hooks!\n\nExpected:\n{expected_hook_calls:?}\n\nActual:\n{hook_calls:?}"
        ));
    }

    let retired_guild = dsl.create_guild()?;

    dsl.create_guild_member(CreateGuildMember {
        guild_id: retired_guild.get_id(),
        locked: false,
    })?;

    let logged_before = guild_member_hook_calls(dsl).len();

    dsl.soft_delete_guild_by_id(&retired_guild)?;

    let hook_calls = guild_member_hook_calls(dsl).split_off(logged_before);

    if hook_calls.ne(&expected_hook_calls) {
        return Err(format!(
            "Retiring a guild through on_soft_delete = SoftDelete should run the soft-delete hooks of guild_member and none of its update hooks!\n\nExpected:\n{expected_hook_calls:?}\n\nActual:\n{hook_calls:?}"
        ));
    }

    if member.get_modified_at().is_some() {
        return Err(
            "Setting deleted_at to Some(...) through on_soft_delete = SoftDelete should NOT set modified_at, as update_guild_member_by_id does!"
                .to_string(),
        );
    }

    Ok(())
}

/// What the hooks of `guild_member` logged so far, in the order they ran.
fn guild_member_hook_calls<T: WriteContext>(dsl: &DSL<'_, T>) -> Vec<String> {
    dsl.get_all_guild_member_hook_calls()
        .map(|hook_call| hook_call.get_description().to_string())
        .collect()
}
