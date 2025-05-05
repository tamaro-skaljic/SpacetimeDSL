pub mod entity {
    use spacetimedb::Timestamp;
    use spacetimedb::table;
    use spacetimedsl::dsl;

    /// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.
    #[dsl(plural_name = entities)]
    #[table(name = entity, public)]
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
        use spacetimedb::{Timestamp, table};
        use spacetimedsl::dsl;

        /// A Identifier is a developer-friendly String.
        #[dsl(plural_name = identifiers)]
        #[table(name = identifier, public)]
        pub struct Identifier {
            /// The unique ID of the Identifier.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the Identifier belongs to.
            #[unique]
            #[wrapped(path = crate::entity::EntityId)]
            entity_id: u128,

            // The unique value of the Identifier.
            #[unique]
            pub value: String,

            created_at: Timestamp,

            pub modified_at: Timestamp,
        }
    }

    pub mod position {
        use spacetimedb::{Timestamp, table};
        use spacetimedsl::dsl;

        /// A Position in the World.
        #[spacetimedsl::dsl(plural_name = positions)]
        #[spacetimedb::table(name = position, public, index(name = x_y_z, btree(columns = [x, y, z])))]
        pub struct Position {
            /// The unique ID of the Position.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the Position belongs to.
            #[unique]
            #[wrapped(path = crate::entity::EntityId)]
            entity_id: u128,

            pub x: i128,

            pub y: i128,

            pub z: i128,

            created_at: Timestamp,

            modified_at: Timestamp,
        }

        /// A unique Position in the World.
        #[dsl(plural_name = unique_positions, unique_index(name = x_y_z))]
        #[table(name = unique_position, public, index(name = x_y_z, btree(columns = [x, y, z])))]
        pub struct UniquePosition {
            /// The unique ID of the unique Position.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            /// The unique ID of the Entity the unique Position belongs to.
            #[unique]
            #[wrapped(path = crate::entity::EntityId)]
            entity_id: u128,

            pub x: i128,

            pub y: i128,

            pub z: i128,

            created_at: Timestamp,

            modified_at: Timestamp,
        }
    }

    pub mod test {
        use spacetimedb::{Timestamp, table};
        use spacetimedsl::dsl;

        /// A Position in the World.
        #[dsl(plural_name = tests)]
        #[table(name = test, public)]
        pub struct Test {
            /// The unique ID of the World.
            #[primary_key]
            #[auto_inc]
            #[wrap]
            id: u128,

            #[wrapped(path = crate::entity::EntityId)]
            pub wrapped_option: Option<u128>,

            // TODO: Add #[unique] if it's allowed by SpacetimeDB
            pub option: Option<u128>,

            #[unique]
            #[wrapped(path = crate::entity::EntityId)]
            pub wrapped_unique: u128,

            #[index(btree)]
            #[wrapped(path = crate::entity::EntityId)]
            pub wrapped_index: u128,

            #[index(btree)]
            pub btree_index: u128,

            #[unique]
            pub unique: u128,

            pub string: String,

            #[index(btree)]
            pub index_on_string: String,

            #[index(btree)]
            #[wrap]
            pub index_on_wrapped_string: String,

            #[unique]
            #[wrap]
            pub unique_on_wrapped_string: String,

            #[wrap]
            pub wrapped_string_option: Option<String>,

            #[index(direct)]
            #[unique]
            pub direct_index: u8,

            created_at: Timestamp,

            modified_at: Timestamp,
            // TODO: Vec<T> columns with index, unique and solo, with wrap and without
        }
    }
}

pub mod test {
    use crate::{
        component::{
            identifier::{
                CreateIdentifierRow, GetIdentifierRowOptionByEntityId,
                GetIdentifierRowOptionByValue, UpdateIdentifierRowById,
            },
            position::{
                CreatePositionRow, CreateUniquePositionRow, GetAllPositionRows,
                GetAllUniquePositionRows, GetCountOfPositionRows, GetCountOfUniquePositionRows,
                PositionId, UniquePositionId, UpdatePositionRowById, UpdateUniquePositionRowById,
            },
            test::{
                CreateTestRow, DeleteTestRowsByBtreeIndex, DeleteTestRowsByWrappedIndex,
                GetTestRowsByBtreeIndex, GetTestRowsByWrappedIndex, Test,
            },
        },
        entity::{CreateEntityRow, DeleteEntityRowById, EntityId, GetEntityRowOptionById},
    };
    use log::info;
    use spacetimedb::{ReducerContext, TimeDuration, reducer};
    use spacetimedsl::{Wrapper, dsl};

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
        match dsl.create_identifier(&player, "PLAYER") {
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
            match dsl.create_identifier(&player, "PLAYER") {
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
        match dsl.get_identifier_by_value("PLAYER") {
            Some(identifier) => {
                player_identifier = identifier;
            }
            None => {
                return Err("Should be able to get an Identifier by it's value!".to_string());
            }
        }

        player_identifier.set_value("ENEMY");
        player_identifier.set_modified_at(
            ctx.timestamp
                .checked_add(TimeDuration::from_micros(500000))
                .unwrap(),
        );

        let enemy_identifier = match dsl.update_identifier_by_id(player_identifier) {
            Ok(i) => i,
            Err(e) => {
                return Err(format!(
                    "Should have been able to update the identifier. Got: {}",
                    e.to_string()
                )
                .into());
            }
        };

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

        let mut player_position = match dsl.create_position(&player, 1, 1, 1) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to add an newly created Position.",
                    player
                ));
            }
        };

        player_position.set_x(0);
        player_position.set_y(0);
        player_position.set_z(0);

        _ = match dsl.update_position_by_id(player_position) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to update an Position.",
                    player
                ));
            }
        };

        let positions_iter = dsl.get_all_positions();
        let position_count_two: usize = dsl.get_count_of_positions().try_into().unwrap();

        let mut position_count_one = 0;
        let mut position_ids = vec![];
        let mut positions = vec![];

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

        let _ = match dsl.create_unique_position(&enemy, 0, 0, 0) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to add an newly created unique Position.",
                    enemy
                ));
            }
        };

        match dsl.create_unique_position(&player, 0, 0, 0) {
            Ok(_) => {
                return Err(format!(
                    "{:?}: Shouldn't be able to add an newly created unique Position which does already exist.",
                    enemy
                ));
            }
            Err(_) => {}
        }

        let mut unique_player_position = match dsl.create_unique_position(&player, 1, 1, 1) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{:?}: Should be able to add an newly created unique Position.",
                    enemy
                ));
            }
        };

        unique_player_position.set_x(0);
        unique_player_position.set_y(0);
        unique_player_position.set_z(0);

        match dsl.update_unique_position_by_id(unique_player_position) {
            Ok(_) => {
                return Err(format!(
                    "{:?}: Shouldn't be able to update an unique Position to a value in x_y_z which does already exist.",
                    enemy
                ));
            }
            Err(_) => {}
        }

        let unique_positions_iter = dsl.get_all_unique_positions();
        let unique_position_count_two: usize =
            dsl.get_count_of_unique_positions().try_into().unwrap();

        let mut unique_position_count_one = 0;
        let mut unique_position_ids = vec![];
        let mut unique_positions = vec![];

        for unique_position in unique_positions_iter {
            unique_position_count_one = unique_position_count_one + 1;
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
            return Err("The count of unique Positions should equal.".to_string());
        }

        let world1 = handle_test_result(dsl.create_test(
            None,
            Some(player.get_id().value()),
            &player,
            &player,
            player.get_id().value(),
            0,
            "string",
            "index_on_string",
            "index_on_wrapped_string",
            "unique_on_wrapped_string1",
            Some("wrapped_string_option".to_string()),
            0,
        ))?;

        let mut world2 = handle_test_result(dsl.create_test(
            &player,
            None,
            enemy.get_id(),
            player.get_id(),
            player.get_id().value(),
            1,
            "string",
            "index_on_string",
            "index_on_wrapped_string",
            "unique_on_wrapped_string2",
            Some("wrapped_string_option".to_string()),
            1,
        ))?;
        let _: Option<EntityId> = world1.get_wrapped_option();
        world2.set_wrapped_option(None);
        world2.set_wrapped_option(&player);
        world2.set_wrapped_option(player.get_id());

        // TODO: Add commented lines if https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21 is added
        let _ = dsl.get_tests_by_wrapped_index(&player);
        let _ = dsl.get_tests_by_wrapped_index(player.get_id());
        let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index());
        //let _ = dsl.get_tests_by_wrapped_index(&player..);
        //let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index()..);
        let _ = dsl.delete_tests_by_wrapped_index(&player);
        let _ = dsl.delete_tests_by_wrapped_index(player.get_id());
        let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index());
        //let _ = dsl.delete_tests_by_wrapped_index(&player..);
        //let _ = dsl.delete_tests_by_wrapped_index(&player..&player);
        //let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index()..);
        //let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index()..world2.get_wrapped_index());

        let _ = dsl.get_tests_by_btree_index(world2.get_btree_index());
        //let _ = dsl.get_tests_by_btree_index(world2.get_btree_index()..);
        let _ = dsl.delete_tests_by_btree_index(world2.get_btree_index());
        //let _ = dsl.delete_tests_by_btree_index(world2.get_btree_index()..);
        info!("Test executed successfully!");

        Ok(())
    }

    fn handle_test_result(
        result: Result<
            Test,
            spacetimedb::TryInsertError<crate::component::test::test__TableHandle>,
        >,
    ) -> Result<Test, Box<str>> {
        match result {
            Ok(w) => {
                return Ok(w);
            }
            Err(e) => {
                return Err(format!(
                    "Should have been able to create a test. Got: {}",
                    e.to_string()
                )
                .into());
            }
        }
    }
}
