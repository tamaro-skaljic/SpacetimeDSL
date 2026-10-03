//! Covers `#[disallow(...)]` in the writes which are neither a create nor an update:
//! `soft_delete_member_by_id` and `soft_delete_members_by_guild_id` check the row their before
//! hook hands back, and the cascades of `member` and `guest` check each row they write — the
//! `SetZero` arm as an update, the `SoftDelete` arm as a soft deletion — in the shape of a
//! table other tables reference (`member`, referenced by `badge`) and of one nothing
//! references (`guest`). `guest` has no hooks, so only its change rule makes the cascade keep
//! the stored row. A broken rule stops a cascade the way an error of a hook does.

#[spacetimedsl::dsl(
    plural_name = guilds,
    method(update = false, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = guild, public)]
pub struct Guild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = member)]
    #[referenced_by(path = self, table = guest)]
    id: u64,

    deleted: bool,
}

#[spacetimedsl::dsl(
    plural_name = members,
    method(update = true, delete = false, soft_delete = true),
    hook(before(update, soft_delete))
)]
#[spacetimedb::table(accessor = member, public)]
pub struct Member {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = badge)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(GuildId)]
    #[foreign_key(path = self, table = guild, column = id, on_delete = SetZero, on_soft_delete = SoftDelete)]
    pub guild_id: u64,

    #[disallow(zero, decreasing)]
    pub experience: u32,

    deleted: bool,
}

#[spacetimedsl::dsl(plural_name = badges, method(update = false, delete = true))]
#[spacetimedb::table(accessor = badge, public)]
pub struct Badge {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(MemberId)]
    #[foreign_key(path = self, table = member, column = id, on_soft_delete = Ignore)]
    member_id: u64,
}

#[spacetimedsl::dsl(
    plural_name = guests,
    method(update = true, delete = false, soft_delete = true)
)]
#[spacetimedb::table(accessor = guest, public)]
pub struct Guest {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(GuildId)]
    #[foreign_key(path = self, table = guild, column = id, on_delete = SetZero, on_soft_delete = SoftDelete)]
    pub guild_id: u64,

    #[disallow(increasing)]
    pub remaining_visits: u8,

    deleted: bool,
}
