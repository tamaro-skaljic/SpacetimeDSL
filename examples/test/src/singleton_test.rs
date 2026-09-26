use crate::spacetimedsl::prelude::*;

/// A singleton table representing global game configuration.
#[spacetimedsl::dsl(singleton, method(update = true, delete = true,))]
#[spacetimedb::table(
    accessor = game_config,
    public,
)]
pub struct GameConfig {
    pub max_players: u32,

    pub game_name: String,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let game_config = dsl
        .create_game_config(CreateGameConfig {
            max_players: 10,
            game_name: "SpacetimeDSL Test".to_string(),
        })
        .map_err(|e| format!("Should be able to create a GameConfig! Got:\n{e}"))?;

    if game_config.get_max_players().ne(&10) {
        return Err(format!(
            "max_players should be 10, got: {}",
            game_config.get_max_players()
        ));
    }

    if game_config.get_game_name().ne("SpacetimeDSL Test") {
        return Err(format!(
            "game_name should be 'SpacetimeDSL Test', got: {}",
            game_config.get_game_name()
        ));
    }

    let retrieved = dsl
        .get_game_config()
        .map_err(|e| format!("Should be able to get the GameConfig! Got:\n{e}"))?;

    if retrieved.get_max_players().ne(&10) {
        return Err(format!(
            "Retrieved max_players should be 10, got: {}",
            retrieved.get_max_players()
        ));
    }

    let mut updated_config = retrieved;
    updated_config.set_max_players(20);
    updated_config.set_game_name("Updated Game".to_string());

    let updated = dsl
        .update_game_config(updated_config)
        .map_err(|e| format!("Should be able to update the GameConfig! Got:\n{e}"))?;

    if updated.get_max_players().ne(&20) {
        return Err(format!(
            "Updated max_players should be 20, got: {}",
            updated.get_max_players()
        ));
    }

    dsl.delete_game_config()
        .map_err(|e| format!("Should be able to delete the GameConfig! Got:\n{e}"))?;

    dsl.get_game_config()
        .expect_err("GameConfig should have been deleted");

    Ok(())
}
