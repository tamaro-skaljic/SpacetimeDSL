use crate::spacetimedsl::prelude::*;
use spacetimedb::Timestamp;

/// A Position in the World.
#[spacetimedsl::dsl(
    plural_name = positions,
    method(update = true),
)]
#[spacetimedb::table(
    accessor = position,
    index(accessor = x_y_z, btree(columns = [x, y, z])),
    public,
)]
pub struct Position {
    /// The unique ID of the Position.
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    /// The unique ID of the Entity the Position belongs to.
    #[unique]
    #[use_wrapper(crate::entity::EntityId)]
    #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = SetZero)]
    pub entity_id: u128,

    pub x: i128,

    pub y: i128,

    pub z: i128,

    #[use_wrapper(crate::component::position::PositionId)]
    mirrored_position_id: Option<u128>,

    created_at: Timestamp,

    modified_at: Timestamp,
}

/// A unique Position in the World.
#[spacetimedsl::dsl(
    plural_name = unique_positions,
    method(update = true),
    unique_index(name = x_y_z),
)]
#[spacetimedb::table(
    accessor = unique_position,
    index(accessor = x_y_z, btree(columns = [x, y, z])),
    public,
)]
pub struct UniquePosition {
    /// The unique ID of the unique Position.
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    /// The unique ID of the Entity the unique Position belongs to.
    #[unique]
    #[use_wrapper(crate::entity::EntityId)]
    #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
    entity_id: u128,

    pub z: i128,

    pub y: i128,

    pub x: i128,

    created_at: Timestamp,

    modified_at: Timestamp,
}

pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let ctx = dsl.ctx();

    let time = ctx.timestamp.to_system_time();

    let player_reflection = dsl.create_entity()?;

    let player_reflection_position = match dsl.create_position(CreatePosition {
        entity_id: player_reflection.get_obj_id(),
        x: 1,
        y: 1,
        z: 1,
        mirrored_position_id: None,
    }) {
        Ok(position) => position,
        Err(_) => {
            return Err(format!(
                "{player_reflection:?}: Should be able to add an newly created Position!"
            ));
        }
    };

    if player_reflection_position
        .get_modified_at()
        .to_system_time()
        .ne(&time)
    {
        return Err(
            "The create method should have set the modified_at column of the position to the current time!"
                .to_string(),
        );
    }

    let player = match dsl.create_entity() {
        Ok(entity) => entity,
        Err(error) => {
            return Err(format!("Should be able to create an Entity! Got:\n{error}"));
        }
    };

    let mut player_position = match dsl.create_position(CreatePosition {
        entity_id: player.get_obj_id(),
        x: 1,
        y: 1,
        z: -1,
        mirrored_position_id: Some(player_reflection_position.get_id().clone()),
    }) {
        Ok(p) => p,
        Err(_) => {
            return Err(format!(
                "{player:?}: Should be able to add an newly created Position!"
            ));
        }
    };

    if player
        .get_obj_id()
        .get_position(dsl)?
        .get_id()
        .ne(&player_position.get_id())
    {
        return Err("EntityId::get_position should find the Position of the player!".to_string());
    }

    player_position.set_x(0);
    player_position.set_y(0);
    player_position.set_z(0);

    let _ = match dsl.update_position_by_id(player_position) {
        Ok(p) => p,
        Err(_) => {
            return Err(format!("{player:?}: Should be able to update an Position!"));
        }
    };

    let positions_iter = dsl.get_all_positions();
    let position_count_two: usize = dsl
        .count_of_all_positions()
        .try_into()
        .expect("should have worked");

    let mut position_count_one = 0;
    let mut position_ids = vec![];
    let mut positions = vec![];

    for position in positions_iter {
        position_count_one += 1;
        position_ids.push(position.get_id());
        positions.push(Some(position));
    }
    position_ids.push(PositionId::new(
        1 + position_ids
            .last()
            .expect("Should have a position in it")
            .value(),
    ));
    positions.push(None);

    if position_count_one != position_count_two {
        return Err("The count of Positions should equal!".to_string());
    }

    let _ = match dsl.create_unique_position(CreateUniquePosition {
        entity_id: player_reflection.get_obj_id(),
        z: 1,
        y: 2,
        x: 3,
    }) {
        Ok(p) => p,
        Err(_) => {
            return Err(format!(
                "{player_reflection:?}: Should be able to add an newly created unique Position!"
            ));
        }
    };

    if dsl.delete_unique_position_by_x_y_z(&3, &2, &1).is_err() {
        return Err(format!(
            "{player_reflection:?}: Should be able to delete an unique Position by x, y, z!"
        ));
    }

    let _ = match dsl.create_unique_position(CreateUniquePosition {
        entity_id: player_reflection.get_obj_id(),
        z: 3,
        y: 4,
        x: 1,
    }) {
        Ok(p) => p,
        Err(_) => {
            return Err(format!(
                "{player_reflection:?}: Should be able to add an newly created unique Position!"
            ));
        }
    };

    if dsl
        .create_unique_position(CreateUniquePosition {
            entity_id: player.get_obj_id(),
            z: 3,
            y: 4,
            x: 1,
        })
        .is_ok()
    {
        return Err(format!(
            "{player_reflection:?}: Shouldn't be able to add an newly created unique Position which does already exist!"
        ));
    }

    let mut unique_player_position = match dsl.create_unique_position(CreateUniquePosition {
        entity_id: player.get_obj_id(),
        z: 1,
        y: 1,
        x: 1,
    }) {
        Ok(p) => p,
        Err(_) => {
            return Err(format!(
                "{player_reflection:?}: Should be able to add an newly created unique Position!"
            ));
        }
    };

    if player
        .get_obj_id()
        .get_unique_position(dsl)?
        .get_id()
        .ne(&unique_player_position.get_id())
    {
        return Err(
            "EntityId::get_unique_position should find the UniquePosition of the player!"
                .to_string(),
        );
    }

    unique_player_position.set_x(1);
    unique_player_position.set_y(4);
    unique_player_position.set_z(3);

    if dsl
        .update_unique_position_by_id(unique_player_position)
        .is_ok()
    {
        return Err(format!(
            "{player_reflection:?}: Shouldn't be able to update an unique Position to a value in x_y_z which does already exist!"
        ));
    }

    let unique_positions_iter = dsl.get_all_unique_positions();
    let unique_position_count_two: usize = dsl
        .count_of_all_unique_positions()
        .try_into()
        .expect("should have worked");

    let mut unique_position_count_one = 0;
    let mut unique_position_ids = vec![];
    let mut unique_positions = vec![];

    for unique_position in unique_positions_iter {
        unique_position_count_one += 1;
        unique_position_ids.push(unique_position.get_id());
        unique_positions.push(Some(unique_position));
    }
    unique_position_ids.push(UniquePositionId::new(
        1 + unique_position_ids
            .last()
            .expect("Should have a unique position in it")
            .value(),
    ));
    unique_positions.push(None);

    if unique_position_count_one != unique_position_count_two {
        return Err("The count of unique Positions should equal!".to_string());
    }

    match dsl.delete_entity_by_obj_id(&player_reflection) {
        Ok(_) => {}
        Err(error) => {
            return Err(format!(
                "Should be able to delete the player_reflection Entity! Got:\n{error}"
            ));
        }
    }

    if dsl
        .get_position_by_id(&player_reflection_position.get_id())
        .expect("should exist")
        .get_entity_id()
        .value()
        .ne(&0)
    {
        return Err("The entity_id of the position which was previously for the player_reflection entity should be 0 because the entity was deleted and the foreign key has a SetZero strategy!".to_string());
    }

    Ok(())
}
