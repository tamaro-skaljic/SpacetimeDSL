//! Covers a table whose only update hook runs after the write. The hook receives the stored
//! row, so `update_<table>_by_<key>` looks that row up before the write, although no
//! before-update hook, foreign key check or `#[disallow]` rule does.

#[spacetimedsl::dsl(
    plural_name = thermostats,
    method(update = true),
    hook(after(update)),
)]
#[spacetimedb::table(accessor = thermostat, public)]
pub struct Thermostat {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub target_temperature: u32,
}
