//! A singleton table which always has a row, because it answers with a default while the
//! table is empty.
//!
//! Covers the whole of `#[dsl(singleton(with_default))]`: the `DefaultSingleton`
//! implementation, the `upsert_<table>` which replaces `create_<table>` and
//! `update_<table>`, and the two timestamp columns whose two write paths differ.

use crate::spacetimedsl::prelude::*;
use spacetimedb::Timestamp;

/// The world settings a module always has, whether or not anybody wrote them.
///
/// It carries the insert and update hooks as well, because a table with a default is the
/// one shape whose `before_insert` hook takes the whole row rather than a create
/// request, and only a module which is built proves that signature compiles.
#[spacetimedsl::dsl(
    singleton(with_default),
    method(update = true, delete = true),
    hook(before(insert, update), after(insert, update))
)]
#[spacetimedb::table(
    accessor = world_settings,
    public,
)]
pub struct WorldSettings {
    pub maximum_player_count: u32,

    pub world_name: String,

    #[set_on_create]
    created_at: Option<Timestamp>,

    modified_at: Option<Timestamp>,
}

/// What each hook wrote into `world_name`, so the reducer can tell which path ran.
const INSERTED_SUFFIX: &str = " [inserted]";
const UPDATED_SUFFIX: &str = " [updated]";

#[spacetimedsl::hook]
fn before_world_settings_insert(
    dsl: &DSL<'_, T>,
    mut new_world_settings: WorldSettings,
) -> Result<WorldSettings, SpacetimeDSLError> {
    let _ = dsl;

    let world_name = format!("{}{INSERTED_SUFFIX}", new_world_settings.get_world_name());
    new_world_settings.set_world_name(world_name);

    Ok(new_world_settings)
}

#[spacetimedsl::hook]
fn after_world_settings_insert(
    dsl: &DSL<'_, T>,
    new_world_settings: &WorldSettings,
) -> Result<(), SpacetimeDSLError> {
    let _ = (dsl, new_world_settings);

    Ok(())
}

#[spacetimedsl::hook]
fn before_world_settings_update(
    dsl: &DSL<'_, T>,
    _old_world_settings: &WorldSettings,
    mut new_world_settings: WorldSettings,
) -> Result<WorldSettings, SpacetimeDSLError> {
    let _ = dsl;

    let world_name = format!("{}{UPDATED_SUFFIX}", new_world_settings.get_world_name());
    new_world_settings.set_world_name(world_name);

    Ok(new_world_settings)
}

#[spacetimedsl::hook]
fn after_world_settings_update(
    dsl: &DSL<'_, T>,
    _old_world_settings: &WorldSettings,
    new_world_settings: &WorldSettings,
) -> Result<(), SpacetimeDSLError> {
    let _ = (dsl, new_world_settings);

    Ok(())
}

impl DefaultSingleton for WorldSettings {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<WorldSettings, SpacetimeDSLError> {
        Ok(WorldSettings {
            // The injected primary key is a field like any other, so the default has to
            // name it. `get_<table>` overwrites it in any case.
            id: 0,
            maximum_player_count: 8,
            world_name: "Default World".to_string(),
            created_at: None,
            modified_at: None,
        })
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let default_settings = dsl
        .get_world_settings()
        .map_err(|e| format!("Getting the WorldSettings should give its default! Got:\n{e}"))?;

    if default_settings.get_maximum_player_count().ne(&8) {
        return Err(format!(
            "The default maximum_player_count should be 8, got: {}",
            default_settings.get_maximum_player_count()
        ));
    }

    if dsl.get_world_settings().is_err() {
        return Err(
            "Getting the WorldSettings a second time should still give its default!".to_string(),
        );
    }

    let mut new_settings = default_settings;
    new_settings.set_maximum_player_count(64);

    let inserted = dsl
        .upsert_world_settings(new_settings)
        .map_err(|e| format!("Upserting the WorldSettings should insert it! Got:\n{e}"))?;

    if inserted.get_maximum_player_count().ne(&64) {
        return Err(format!(
            "The inserted maximum_player_count should be 64, got: {}",
            inserted.get_maximum_player_count()
        ));
    }

    let created_at = inserted
        .get_created_at()
        .as_ref()
        .copied()
        .ok_or("An inserted row should have a created_at timestamp!")?;

    // The insert path of an upsert runs the insert hooks, never the update hooks.
    if !inserted.get_world_name().ends_with(INSERTED_SUFFIX) {
        return Err(format!(
            "The before_insert hook should have run on the insert path, got world_name: {}",
            inserted.get_world_name()
        ));
    }

    if inserted.get_modified_at().is_some() {
        return Err(
            "An inserted row was never updated, so modified_at should be None!".to_string(),
        );
    }

    let mut changed_settings = inserted;
    changed_settings.set_world_name("Changed World".to_string());

    let updated = dsl
        .upsert_world_settings(changed_settings)
        .map_err(|e| format!("Upserting the WorldSettings again should update it! Got:\n{e}"))?;

    if !updated.get_world_name().starts_with("Changed World") {
        return Err(format!(
            "The updated world_name should start with 'Changed World', got: {}",
            updated.get_world_name()
        ));
    }

    // The update path of an upsert runs the update hooks, never the insert hooks.
    if !updated.get_world_name().ends_with(UPDATED_SUFFIX) {
        return Err(format!(
            "The before_update hook should have run on the update path, got world_name: {}",
            updated.get_world_name()
        ));
    }

    if updated.get_modified_at().is_none() {
        return Err("An update should set modified_at!".to_string());
    }

    if updated.get_created_at().as_ref().ne(&Some(&created_at)) {
        return Err("An update should preserve created_at!".to_string());
    }

    let mut settings_from_the_default = WorldSettings::get_default(&read_only_dsl(dsl.ctx()))
        .map_err(|e| format!("The default should be available! Got:\n{e}"))?;
    settings_from_the_default.set_maximum_player_count(128);

    let updated = dsl
        .upsert_world_settings(settings_from_the_default)
        .map_err(|e| {
            format!(
                "Upserting a row built from the default should update the stored one! Got:\n{e}"
            )
        })?;

    if updated.get_created_at().as_ref().ne(&Some(&created_at)) {
        return Err("Upserting the default should preserve created_at!".to_string());
    }

    dsl.delete_world_settings()
        .map_err(|e| format!("Should be able to delete the WorldSettings! Got:\n{e}"))?;

    let default_again = dsl
        .get_world_settings()
        .map_err(|e| format!("After the delete the default should be back! Got:\n{e}"))?;

    if default_again.get_maximum_player_count().ne(&8) {
        return Err(format!(
            "After the delete the maximum_player_count should be the default 8, got: {}",
            default_again.get_maximum_player_count()
        ));
    }

    if default_again.get_created_at().is_some() {
        return Err("The default created_at should remain None!".to_string());
    }

    Ok(())
}
