use crate::spacetimedsl::prelude::*;
use spacetimedb::Timestamp;

/// A Identifier is a developer-friendly String.
#[spacetimedsl::dsl(
    plural_name = identifiers,
    method(update = true),
)]
#[spacetimedb::table(
    accessor = identifier,
    public,
)]
pub struct Identifier {
    /// The unique ID of the Identifier.
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::component::identifier, table = identifier_reference)]
    id: u128,

    /// The unique ID of the Entity the Identifier belongs to.
    #[unique]
    #[use_wrapper(crate::entity::EntityId)]
    #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
    entity_id: u128,

    // The unique value of the Identifier.
    #[unique]
    pub value: String,

    created_at: Timestamp,

    modified_at: Option<Timestamp>,
}

#[spacetimedsl::dsl(
    plural_name = identifier_references,
    method(update = false),
)]
#[spacetimedb::table(
    accessor = identifier_reference,
    public,
)]
pub struct IdentifierReference {
    #[primary_key]
    #[use_wrapper(IdentifierId)]
    #[foreign_key(path = crate::component::identifier, table = identifier, column = id, on_delete = Delete)]
    id: u128,
}

pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let ctx = dsl.ctx();

    let time = ctx.timestamp.to_system_time();

    let player = dsl.create_entity()?;

    match dsl.create_identifier(CreateIdentifier {
        entity_id: player.get_obj_id(),
        value: "cool".to_string(),
    }) {
        Ok(_) => {}
        Err(error) => {
            return Err(format!(
                "{player:?}: Should be able to add an newly created Identifier! Got:\n{error}"
            ));
        }
    };

    if dsl.count_of_all_identifiers().ne(&1) {
        return Err("Count of identifiers should be 1!".to_string());
    }

    dsl.delete_entity_by_obj_id(&player)?;

    if dsl.count_of_all_identifiers().ne(&0) {
        return Err("Count of identifiers should be 0 because the last one should be deleted through the foreign key / referenced by feature!".to_string());
    }

    let player = dsl.create_entity()?;

    let mut player_identifier;
    match dsl.create_identifier(CreateIdentifier {
        entity_id: player.get_obj_id(),
        value: "PLAYER".to_string(),
    }) {
        Ok(identifier) => {
            player_identifier = identifier;
        }
        Err(error) => {
            return Err(format!(
                "{player:?}: Should be able to add an newly created Identifier! Got:\n{error}"
            ));
        }
    };

    if player_identifier
        .get_created_at()
        .to_system_time()
        .ne(&time)
    {
        return Err(
            "The create method should have set the created_at column of the identifier!"
                .to_string(),
        );
    }

    if player_identifier.get_modified_at().is_some() {
        return Err(
            "The create method should have set the modified_at column of the identifier to None!"
                .to_string(),
        );
    }

    if let Ok(identifier) = dsl.create_identifier(CreateIdentifier {
        entity_id: player.get_obj_id(),
        value: "PLAYER".to_string(),
    }) {
        return Err(format!(
            "Entity {} ({}): Shouldn't be able to add an Identifier because it has already one!",
            player.get_obj_id().value(),
            identifier.get_value()
        ));
    };

    match dsl.get_identifier_by_value("PLAYER") {
        Ok(identifier) => {
            player_identifier = identifier;
        }
        Err(error) => {
            return Err(format!(
                "Should be able to get an Identifier by it's value! Got:\n{error}"
            ));
        }
    }

    player_identifier.set_value("PLAYER_REFLECTION".to_string());
    player_identifier.modified_at = Some(
        ctx.timestamp
            .checked_add(TimeDuration::from_micros(99999999999))
            .expect("should have worked"),
    );

    let player_reflection_identifier = match dsl.update_identifier_by_id(player_identifier) {
        Ok(i) => i,
        Err(e) => {
            return Err(format!(
                "Should have been able to update the identifier. Got: {e}"
            ));
        }
    };

    if player_reflection_identifier
        .get_modified_at()
        .unwrap()
        .to_system_time()
        .ne(&time)
    {
        return Err(
            "The update method should have set the modified_at column of the identifier!"
                .to_string(),
        );
    }

    let player_reflection = player;

    match dsl.get_identifier_by_entity_id(&player_reflection) {
        Ok(identifier) => {
            if identifier
                .get_value()
                .ne(player_reflection_identifier.get_value())
            {
                return Err(format!(
                    "The Identifier values should equal. Expected: {}, Actual: {}!",
                    player_reflection_identifier.get_value(),
                    identifier.get_value()
                ));
            }
        }
        Err(error) => {
            return Err(format!(
                "Should be able to get an Identifier by it's Entity! Got:\n{error}"
            ));
        }
    }

    match dsl.delete_entity_by_obj_id(&player_reflection) {
        Ok(_) => {}
        Err(error) => {
            return Err(format!(
                "Should be able to delete the player_reflection Entity! Got:\n{error}"
            ));
        }
    }

    if dsl.count_of_all_identifiers().ne(&0) {
        return Err("The count of Identifiers should be 0 because the player_reflection Entity was deleted and the foreign key has a Delete strategy!".to_string());
    }

    Ok(())
}
