pub mod entity {
    use spacetimedb::Timestamp;
    use spacetimedsl::derive::SpacetimeDSL;

    /// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.
    #[derive(Debug, SpacetimeDSL, Clone, PartialEq)]
    #[spacetimedb::table(name = entity, public)]
    #[plural_table_name(entities)]
    pub struct Entity {
        /// The unique ID of the Entity.
        #[primary_key]
        #[auto_inc]
        #[wrap]
        id: u128,

        created_at: Timestamp,
    }
}

pub mod component {
    pub mod identifier {
        use spacetimedb::Timestamp;
        use spacetimedsl::derive::SpacetimeDSL;

        /// A Identifier is a developer-friendly String.
        #[derive(Debug, SpacetimeDSL, Clone, PartialEq)]
        #[spacetimedb::table(name = identifier, public)]
        #[plural_table_name(identifiers)]
        pub struct Identifier {
            /// The unique ID of the Identifier.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the Identifier belongs to.
            #[unique]
            #[wrap(crate::entity::EntityId)]
            entity_id: u128,

            // The unique value of the Identifier.
            #[unique]
            pub value: String,

            created_at: Timestamp,

            pub modified_at: Timestamp,
        }
    }

    pub mod position {
        use spacetimedb::Timestamp;
        use spacetimedsl::derive::SpacetimeDSL;

        /// A Position in the World.
        #[derive(Debug, SpacetimeDSL, Clone, PartialEq)]
        #[spacetimedb::table(name = position, public, index(name = x_y_z, btree(columns = [x, y, z])))]
        #[plural_table_name(positions)]
        pub struct Position {
            /// The unique ID of the Position.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the Position belongs to.
            #[unique]
            #[wrap(crate::entity::EntityId)]
            entity_id: u128,

            pub x: i128,

            pub y: i128,

            pub z: i128,

            created_at: Timestamp,

            pub modified_at: Timestamp,
        }
    }

    pub mod test {
        use spacetimedb::Timestamp;
        use spacetimedsl::derive::SpacetimeDSL;

        /// A Position in the World.
        #[derive(Debug, SpacetimeDSL, Clone, PartialEq)]
        #[spacetimedb::table(name = test, public)]
        #[plural_table_name(tests)]
        pub struct Test {
            /// The unique ID of the World.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            #[wrap(crate::entity::EntityId)]
            pub wrapped_option: Option<u128>,

            // TODO: Add #[unique] if it's allowed by SpacetimeDB
            pub option: Option<u128>,

            #[unique]
            #[wrap(crate::entity::EntityId)]
            pub wrapped_unique: u128,

            #[index(btree)]
            #[wrap(crate::entity::EntityId)]
            pub wrapped_index: u128,

            #[index(btree)]
            pub btree_index: u128,

            // TODO: Add when https://github.com/tamaro-skaljic/SpacetimeDSL/issues/20 is fixed
            //#[index(direct)]
            //#[unique]
            //pub direct_index: u8,

            pub created_at: Timestamp,

            pub modified_at: Timestamp,
        }
    }
}

pub mod test {
    use std::iter::zip;

    use log::info;
    use spacetimedb::{ReducerContext, TimeDuration, reducer};
    use spacetimedsl::{Wrapper, dsl};

    use crate::{
        component::{
            identifier::{
                CreateIdentifier, GetIdentifierRowOptionByEntityId, GetIdentifierRowOptionByValue,
                UpdateIdentifierRowById,
            },
            position::{
                CreatePosition, GetAllPositionRows, GetCountOfPositionRows,
                GetPositionRowOptionsById, PositionId,
            },
            test::{CreateTest, Test, test__TableHandle},
            test::{
                CreateTest, DeleteTestRowsByBtreeIndex, DeleteTestRowsByWrappedIndex,
                GetTestRowsByBtreeIndex, GetTestRowsByWrappedIndex, Test, test__TableHandle,
            },
        },
        entity::{CreateEntity, DeleteEntityRowById, EntityId, GetEntityRowOptionById},
    };

    #[reducer]
    fn tester(ctx: &ReducerContext) -> Result<(), String> {
        let dsl = dsl(ctx);

        let mut player;
        match dsl.create_entity() {
            Ok(entity) => {
                player = entity;
            }
            Err(_) => {
                return Err("Should be able to create an Entity!".to_string());
            }
        };

        let time = ctx.timestamp.to_system_time();
        if player.get_created_at().to_system_time().ne(&time) {
            return Err(
                "The create method should have set the created_at column of the entity."
                    .to_string(),
            );
        }

        match dsl.get_entity_by_id(&player) {
            Some(entity) => {
                player = entity;
            }
            None => {
                return Err("Should be able to get an Entity by it's ID!".to_string());
            }
        };
        if !dsl.delete_entity_by_id(&player) {
            return Err("Should be able to delete an Entity by it's ID!".to_string());
        }
        if dsl.get_entity_by_id(&player).is_some() {
            return Err(
                "Shouldn't be able to get an Entity by an ID which doesn't exist!".to_string(),
            );
        }
        match dsl.create_entity() {
            Ok(entity) => {
                player = entity;
            }
            Err(_) => {
                return Err("Should be able to create an Entity!".to_string());
            }
        };

        let mut player_identifier;
        match dsl.create_identifier(&player, "PLAYER".to_string()) {
            Ok(identifier) => {
                player_identifier = identifier;
            }
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to add an newly created Identifier.",
                    player
                ));
            }
        };

        if player_identifier
            .get_created_at()
            .to_system_time()
            .ne(&time)
        {
            return Err(
                "The create method should have set the created_at column of the identifier."
                    .to_string(),
            );
        }

        if player_identifier
            .get_modified_at()
            .to_system_time()
            .ne(&time)
        {
            return Err(
                "The create method should have set the modified_at column of the identifier."
                    .to_string(),
            );
        }

        /* TODO: Uncomment if https://github.com/clockworklabs/SpacetimeDB/pull/2610 is fixed
        player_identifier.set_value("ENEMY".to_string());
        match dsl.create_identifier(player_identifier) {
            Ok(identifier) => {
                return Err(format!(
                    "Entity {} ({}): Shouldn't be able to add an Identifier because it has already one.",
                    player.get_id().value(),
                    identifier.get_value()
                ));
            }
            Err(_) => {}
        };
         */
        match dsl.get_identifier_by_value("PLAYER".to_string()) {
            Some(identifier) => {
                player_identifier = identifier;
            }
            None => {
                return Err("Should be able to get an Identifier by it's value!".to_string());
            }
        }

        player_identifier.set_value("ENEMY".to_string());
        player_identifier.set_modified_at(
            ctx.timestamp
                .checked_add(TimeDuration::from_micros(500000))
                .unwrap(),
        );

        let enemy_identifier = dsl.update_identifier_by_id(player_identifier);

        if enemy_identifier
            .get_modified_at()
            .to_system_time()
            .ne(&time)
        {
            return Err(
                "The update method should have set the modified_at column of the identifier."
                    .to_string(),
            );
        }

        let enemy = player;

        match dsl.get_identifier_by_entity_id(&enemy) {
            Some(identifier) => {
                if identifier.get_value().ne(enemy_identifier.get_value()) {
                    return Err(format!(
                        "The Identifier values should equal. Expected: {}, Actual: {}.",
                        enemy_identifier.get_value(),
                        identifier.get_value()
                    ));
                }
            }
            None => {
                return Err("Should be able to get an Identifier by it's Entity!".to_string());
            }
        }

        match dsl.create_position(&enemy, 0, 0, 0) {
            Ok(_) => {}
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to add an newly created Position.",
                    enemy
                ));
            }
        }

        let player;
        match dsl.create_entity() {
            Ok(entity) => {
                player = entity;
            }
            Err(_) => {
                return Err("Should be able to create an Entity!".to_string());
            }
        };

        match dsl.create_position(&player, 0, 0, 0) {
            Ok(_) => {}
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to add an newly created Position.",
                    player
                ));
            }
        }

        let positions_iter = dsl.get_all_positions();
        let position_count_two: usize = dsl.get_count_of_positions().try_into().unwrap();

        let mut position_count_one = 0;
        let mut positions = vec![];
        let mut position_ids = vec![];

        for position in positions_iter {
            position_count_one = position_count_one + 1;
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
            return Err("The count of Positions should equal.".to_string());
        }

        for (position_one, position_two) in zip(positions, dsl.get_positions_by_id_in(position_ids))
        {
            match position_one.as_ref() {
                Some(position_one) => {
                    if position_two.is_none() {
                        return Err(format!(
                            "Got {:?} and {:?} but they should be both Some.",
                            position_one, position_two
                        ));
                    }

                    if position_one.ne(&position_two.as_ref().unwrap()) {
                        return Err(format!(
                            "Got {:?} and {:?} but they should be equal.",
                            position_one, position_two
                        ));
                    }
                }
                None => {
                    if position_two.is_some() {
                        return Err(format!(
                            "Got {:?} and {:?} but they should be both None.",
                            position_one, position_two
                        ));
                    }
                }
            }
        }

        let world1 = handle_test_result(dsl.create_test(
            None,
            Some(player.get_id().value()),
            &player,
            &player,
            player.get_id().value(),
        ))?;
        let mut world2 = handle_test_result(dsl.create_test(
            &player,
            None,
            enemy.get_id(),
            player.get_id(),
            player.get_id().value(),
        ))?;
        let _: Option<EntityId> = world1.get_wrapped_option();
        world2.set_wrapped_option(None);
        world2.set_wrapped_option(&player);
        world2.set_wrapped_option(player.get_id());

        // TODO: Add commented lines if https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21 is added
        let _ = dsl.get_tests_by_wrapped_index(&player);
        let _ = dsl.get_tests_by_wrapped_index(player.get_id());
        let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index());
        // TODO: let _ = dsl.get_tests_by_wrapped_index(&player..);
        // TODO: let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index()..);
        let _ = dsl.delete_tests_by_wrapped_index(&player);
        let _ = dsl.delete_tests_by_wrapped_index(player.get_id());
        let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index());
        // TODO: let _ = dsl.delete_tests_by_wrapped_index(&player..);
        // TODO: let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index()..);

        let _ = dsl.get_tests_by_btree_index(world2.get_btree_index());
        // TODO: let _ = dsl.get_tests_by_btree_index(world2.get_btree_index()..);
        let _ = dsl.delete_tests_by_btree_index(world2.get_btree_index());
        // TODO: let _ = dsl.delete_tests_by_btree_index(world2.get_btree_index()..);
        info!("Test executed successfully!");

        Ok(())
    }

    fn handle_test_result(
        result: Result<Test, spacetimedb::TryInsertError<test__TableHandle>>,
    ) -> Result<Test, String> {
        match result {
            Ok(w) => {
                return Ok(w);
            }
            Err(e) => {
                return Err(format!(
                    "Should have been able to create a test. Got: {}",
                    e.to_string()
                ));
            }
        }
    }
}
