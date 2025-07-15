pub mod entity {
    use spacetimedb::Timestamp;

    /// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.
    #[spacetimedsl::dsl(plural_name = entities)]
    #[spacetimedb::table(name = entity, public)]
    pub struct Entity {
        /// The unique ID of the Entity.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::entity,                table = entity_relationship)]
        #[referenced_by(path = crate::entity,                table = entity_relationship2)]
        #[referenced_by(path = crate::component::identifier, table = identifier)]
        #[referenced_by(path = crate::component::position,   table = position)]
        #[referenced_by(path = crate::component::position,   table = unique_position)]
        #[referenced_by(path = crate::component::test,       table = test)]
        #[referenced_by(path = crate::component::test,       table = ship_object)]
        id: u128,

        created_at: Timestamp,
    }

    #[spacetimedsl::dsl(
        plural_name = tables,
        unique_index(name = id_and_name1),
        unique_index(name = id_and_name3),
    )]
    #[spacetimedb::table(
        name = table,
        public,
        index(name = id_and_name1, btree(columns = [id, name1])),
        index(name = id_and_name2, btree(columns = [id, name2])),
        index(name = id_and_name3, btree(columns = [id, name3])),
        index(name = id_and_name4, btree(columns = [id, name4])),
    )]
    pub struct Table {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u128,

        #[unique]
        pub name1: String,

        #[index(btree)]
        pub name2: String,

        #[unique]
        #[create_wrapper]
        pub name3: String,

        #[index(btree)]
        #[create_wrapper]
        pub name4: String,
    }

    #[spacetimedsl::dsl(plural_name = entity_relationships, unique_index(name = parent_child_entity_id))]
    #[spacetimedb::table(name = entity_relationship, public, index(name = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])))]
    pub struct EntityRelationship {
        /// The unique ID of the Entity Relationship.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u128,

        #[index(btree)]
        #[use_wrapper(name = EntityId)]
        #[foreign_key(path = crate::entity, table = entity, on_delete = Error)]
        parent_entity_id: u128,

        #[index(btree)]
        #[use_wrapper(name = EntityId)]
        #[foreign_key(path = crate::entity, table = entity, on_delete = Delete)]
        child_entity_id: u128,
    }

    #[spacetimedsl::dsl(plural_name = entity_relationships2)]
    #[spacetimedb::table(name = entity_relationship2, public, index(name = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])))]
    pub struct EntityRelationship2 {
        /// The unique ID of the Entity Relationship2.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u128,

        #[index(btree)]
        #[use_wrapper(name = EntityId)]
        #[foreign_key(path = crate::entity, table = entity, on_delete = Delete)]
        parent_entity_id: u128,

        #[index(btree)]
        #[use_wrapper(name = EntityId)]
        #[foreign_key(path = crate::entity, table = entity, on_delete = Delete)]
        pub child_entity_id: u128,
    }

    #[spacetimedsl::dsl(plural_name = entity_relationships3)]
    #[spacetimedb::table(name = entity_relationship3, public)]
    pub struct EntityRelationship3 {
        /// The unique ID of the Entity Relationship3.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::entity, table = entity_relationship3)]
        id: u128,

        #[index(btree)]
        #[use_wrapper(name = EntityRelationship3Id)]
        #[foreign_key(path = crate::entity, table = entity_relationship3, on_delete = SetZero)]
        parent_entity_relationship3_id: u128,
    }

    #[spacetimedsl::dsl(plural_name = entity_relationships4)]
    #[spacetimedb::table(name = entity_relationship4, public)]
    pub struct EntityRelationship4 {
        /// The unique ID of the Entity Relationship3.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::entity, table = entity_relationship4)]
        id: u128,

        #[index(btree)]
        #[use_wrapper(name = EntityRelationship4Id)]
        #[foreign_key(path = crate::entity, table = entity_relationship4, on_delete = Ignore)]
        pub parent_entity_relationship4_id: u128,
    }
}

pub mod component {
    pub mod identifier {
        use spacetimedb::Timestamp;

        /// A Identifier is a developer-friendly String.
        #[spacetimedsl::dsl(plural_name = identifiers)]
        #[spacetimedb::table(name = identifier, public)]
        pub struct Identifier {
            /// The unique ID of the Identifier.
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            #[referenced_by(path = crate::component::identifier, table = identifier_reference)]
            id: u128,

            /// The unique ID of the Entity the Identifier belongs to.
            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = Delete)]
            entity_id: u128,

            // The unique value of the Identifier.
            #[unique]
            pub value: String,

            created_at: Timestamp,

            modified_at: Option<Timestamp>,
        }

        #[spacetimedsl::dsl(plural_name = identifier_references)]
        #[spacetimedb::table(name = identifier_reference, public)]
        pub struct IdentifierReference {
            #[primary_key]
            #[use_wrapper(name = IdentifierId)]
            #[foreign_key(path = crate::component::identifier, table = identifier, on_delete = Delete)]
            id: u128,
        }

        pub(crate) fn update_modified_at(identifier: &mut Identifier, new_value: Timestamp) {
            identifier.modified_at = Some(new_value);
        }
    }

    pub mod position {
        use spacetimedb::Timestamp;

        /// A Position in the World.
        #[spacetimedsl::dsl(plural_name = positions)]
        #[spacetimedb::table(name = position, public, index(name = x_y_z, btree(columns = [x, y, z])))]
        pub struct Position {
            /// The unique ID of the Position.
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            /// The unique ID of the Entity the Position belongs to.
            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = SetZero)]
            entity_id: u128,

            pub x: i128,

            pub y: i128,

            pub z: i128,

            #[use_wrapper(path = crate::component::position::PositionId)]
            mirrored_position_id: Option<u128>,

            created_at: Timestamp,

            modified_at: Timestamp,
        }

        /// A unique Position in the World.
        #[spacetimedsl::dsl(plural_name = unique_positions, unique_index(name = x_y_z))]
        #[spacetimedb::table(name = unique_position, public, index(name = x_y_z, btree(columns = [x, y, z])))]
        pub struct UniquePosition {
            /// The unique ID of the unique Position.
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            /// The unique ID of the Entity the unique Position belongs to.
            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = Delete)]
            entity_id: u128,

            pub z: i128,

            pub y: i128,

            pub x: i128,

            created_at: Timestamp,

            modified_at: Timestamp,
        }
    }

    pub mod test {
        use spacetimedb::{ScheduleAt, Timestamp};

        /// A Position in the World.
        #[spacetimedsl::dsl(plural_name = tests)]
        #[spacetimedb::table(name = test, public)]
        pub struct Test {
            /// The unique ID of the World.
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            #[use_wrapper(path = crate::entity::EntityId)]
            pub wrapped_option: Option<u128>,

            // TODO: Add #[unique] if it's allowed by SpacetimeDB
            // #[unique]
            // pub unique_option: Option<u128>,

            // TODO: Add unique_wrapped_option if it's allowed by SpacetimeDB
            // #[unique]
            // #[use_wrapper(path = crate::entity::EntityId)]
            // pub unique_wrapped_option: Option<u128>,
            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            pub wrapped_unique: u128,

            #[index(btree)]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = Delete)]
            pub wrapped_index: u128,

            #[index(btree)]
            pub btree_index: u128,

            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = SetZero)]
            pub unique: u128,

            pub string: String,

            #[index(btree)]
            pub index_on_string: String,

            #[index(btree)]
            #[create_wrapper]
            pub index_on_wrapped_string: String,

            #[unique]
            #[create_wrapper]
            pub unique_on_wrapped_string: String,

            #[create_wrapper]
            pub wrapped_string_option: Option<String>,

            #[index(direct)]
            #[unique]
            pub direct_index: u8,

            created_at: Timestamp,

            modified_at: Timestamp,
            // TODO: Vec<T> columns with index, unique and solo, with wrap and without
            scheduled_at: ScheduleAt,
        }

        #[spacetimedsl::dsl(plural_name = ship_objects, unique_index(name = id_and_sobj))]
        #[spacetimedb::table(name = ship_object, public, index(name = id_and_sobj, btree(columns = [id, sobj_id])))]
        pub struct ShipObject {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[unique]
            #[auto_inc]
            #[create_wrapper]
            pub sobj_id: u64,

            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = Error)]
            pub entity_id: u128,
        }

        #[spacetimedsl::dsl(plural_name = space_ship_objects)]
        #[spacetimedb::table(name = space_ship_object, public, index(name = id_and_sobj, btree(columns = [id, sobj_id])))]
        pub struct SpaceShipObject {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[unique]
            #[auto_inc]
            #[create_wrapper]
            pub sobj_id: u64,

            #[unique]
            #[use_wrapper(path = crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, on_delete = Error)]
            pub entity_id: u128,
        }

        #[spacetimedsl::dsl(
            plural_name = modules1,
            unique_index(name = database_and_parent_id_and_name),
        )]
        #[spacetimedb::table(
            name = module1,
            public,
            index(name = database_and_parent_id_and_name, btree(columns = [database_id, parent_id, name])),
        )]
        #[spacetimedsl::dsl(
            plural_name = modules2,
            unique_index(name = database_and_name_and_parent_id),
        )]
        #[spacetimedb::table(
            name = module2,
            public,
            index(name = database_and_name_and_parent_id, btree(columns = [database_id, name, parent_id])),
        )]
        pub struct Module {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            database_id: u128,

            pub name: String,

            pub parent_id: u128,
        }
    }
}

pub mod test {
    use crate::{
        component::{
            identifier::{
                CountOfAllIdentifierRows, CreateIdentifierRow, GetIdentifierRowOptionByEntityId,
                GetIdentifierRowOptionByValue, UpdateIdentifierRowById, update_modified_at,
            },
            position::{
                CountOfAllPositionRows, CountOfAllUniquePositionRows, CreatePositionRow,
                CreateUniquePositionRow, DeleteUniquePositionRowByXYZ, GetAllPositionRows,
                GetAllUniquePositionRows, GetPositionRowOptionById, PositionId, UniquePositionId,
                UpdatePositionRowById, UpdateUniquePositionRowById,
            },
            test::{
                CreateShipObjectRow, CreateTestRow, DeleteTestRowsByBtreeIndex,
                DeleteTestRowsByWrappedIndex, GetModule1RowOptionByDatabaseAndParentIdAndName,
                GetModule2RowOptionByDatabaseAndNameAndParentId, GetTestRowsByBtreeIndex,
                GetTestRowsByWrappedIndex,
            },
        },
        entity::{
            CountOfAllEntityRelationship2Rows, CountOfAllEntityRelationshipRows,
            CreateEntityRelationship2Row, CreateEntityRelationship4Row,
            CreateEntityRelationshipRow, CreateEntityRow, DeleteEntityRelationship4RowById,
            DeleteEntityRowById, EntityId, EntityRelationship4Id,
            GetEntityRelationship4RowOptionById, GetEntityRowOptionById,
            UpdateEntityRelationship4RowById,
        },
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
            Err(error) => {
                return Err(format!("Should be able to create an Entity! Got:\n{error}"));
            }
        };

        let time = ctx.timestamp.to_system_time();
        if player.get_created_at().to_system_time().ne(&time) {
            return Err(
                "The create method should have set the created_at column of the entity!"
                    .to_string(),
            );
        }

        match dsl.get_entity_by_id(&player) {
            Ok(entity) => {
                player = entity;
            }
            Err(error) => {
                return Err(format!(
                    "Should be able to get an Entity by it's ID! Got:\n{error}"
                ));
            }
        };

        let player2 = match dsl.create_entity() {
            Ok(entity) => entity,
            Err(error) => {
                return Err(format!("Should be able to create an Entity! Got:\n{error}"));
            }
        };

        match dsl.create_identifier(&player, "cool") {
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

        dsl.create_entity_relationship(&player, &player2)?;
        if dsl.create_entity_relationship(&player, &player2).is_ok() {
            return Err("Shouldn't be able to create the same entity relationship because of the unique multi column index `parent_child_entity_id`".to_string());
        }
        let player3 = dsl.create_entity()?;
        dsl.create_entity_relationship(&player, &player3)?;
        dsl.create_entity_relationship(&player2, &player3)?;

        if dsl.count_of_all_entity_relationships().ne(&3) {
            return Err("Count of entity relationships should be 3!".to_string());
        }

        if dsl.delete_entity_by_id(&player).is_ok() {
            return Err("Shouldn't be able to delete 'player' because it's a parent in a entity relationship!".to_string());
        }

        if dsl.count_of_all_entity_relationships().ne(&3) {
            return Err("Count of entity relationships should be 3!".to_string());
        }

        match dsl.delete_entity_by_id(&player3) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "Should be able to delete 'player3' because it's only a child in entity relationships! Got:\n{error}"
                ));
            }
        };

        if dsl.count_of_all_entity_relationships().ne(&1) {
            return Err("Count of entity relationships should be 1 because 2 should be deleted through the foreign key / referenced by feature!".to_string());
        }

        match dsl.delete_entity_by_id(&player2) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "Should be able to delete 'player2' because it's only a child in entity relationships! Got:\n{error}"
                ));
            }
        };

        if dsl.count_of_all_entity_relationships().ne(&0) {
            return Err(
                "Count of entity relationships should be 0 because the last one should be deleted through the foreign key / referenced by feature!".to_string(),
            );
        }
        match dsl.delete_entity_by_id(&player) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "Should be able to delete 'player' because it's not a parent anymore in a entity relationship! Got:\n{error}"
                ));
            }
        };

        if dsl.get_entity_by_id(&player).is_ok() {
            return Err(
                "Shouldn't be able to get an Entity by an ID which doesn't exist!".to_string(),
            );
        }

        if dsl.count_of_all_identifiers().ne(&0) {
            return Err("Count of identifiers should be 0 because the last one should be deleted through the foreign key / referenced by feature!".to_string());
        }

        match dsl.create_entity() {
            Ok(entity) => {
                player = entity;
            }
            Err(error) => {
                return Err(format!("Should be able to create an Entity! Got:\n{error}"));
            }
        };

        let player2 = dsl.create_entity()?;
        let player3 = dsl.create_entity()?;

        dsl.create_entity_relationship2(&player, &player2)?;
        dsl.create_entity_relationship2(&player2, &player3)?;
        dsl.create_entity_relationship2(&player3, &player)?;

        if dsl.count_of_all_entity_relationships2().ne(&3) {
            return Err("Count of entity relationships 2 should be 3!".to_string());
        }

        match dsl.delete_entity_by_id(&player2) {
            Ok(_) => {}
            Err(error) => return Err(format!("Should be able to delete 'player'! Got:\n{error}")),
        };

        if dsl.count_of_all_entity_relationships2().ne(&1) {
            return Err("Count of entity relationships should be 1 because 2 should be deleted through the foreign key / referenced by feature!".to_string());
        }

        let er4_1 = dsl.create_entity_relationship4(EntityRelationship4Id::new(0))?;
        let mut er4_2 = dsl.create_entity_relationship4(&er4_1)?;

        match dsl.delete_entity_relationship4_by_id(&er4_1) {
            Ok(success) => {
                if success.entries[0]
                    .row_value
                    .ne(&er4_1.get_id().to_string().into())
                {
                    return Err(format!(
                        "Should be able to delete 'er4_1'! Got: {} and {}\n{success}",
                        success.entries[0].row_value,
                        er4_1.get_id()
                    ));
                }
            }
            Err(error) => {
                return Err(format!("Should be able to delete 'er4_1'! Got:\n{error}"));
            }
        }

        if er4_2.get_parent_entity_relationship4_id().ne(&dsl
            .get_entity_relationship4_by_id(&er4_2)
            .expect("shouldn't be deleted")
            .get_parent_entity_relationship4_id())
        {
            return Err(
                "`parent_entity_relationship4_id` of `er4_2` shouldn't have changed.".to_string(),
            );
        }

        er4_2.set_parent_entity_relationship4_id(EntityRelationship4Id::new(0));
        er4_2 = dsl.update_entity_relationship4_by_id(er4_2)?;
        er4_2.set_parent_entity_relationship4_id(&er4_1);
        if dsl.update_entity_relationship4_by_id(er4_2).is_ok() {
            return Err("Shouldn't be able to set `parent_entity_relationship4_id` of `er4_2` to id of previously deleted `er4_1`".to_string());
        }

        if dsl.create_entity_relationship4(&er4_1).is_ok() {
            return Err("Shouldn't be able to create a `entity_relationship4` with `er4_1` as `parent_entity_relationship4_id`".to_string());
        }

        let mut player_identifier;
        match dsl.create_identifier(&player, "PLAYER") {
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

        if let Ok(identifier) = dsl.create_identifier(&player, "PLAYER") {
            return Err(format!(
                "Entity {} ({}): Shouldn't be able to add an Identifier because it has already one!",
                player.get_id().value(),
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

        player_identifier.set_value("PLAYER_REFLECTION");
        update_modified_at(
            &mut player_identifier,
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

        let player_reflection_position =
            match dsl.create_position(&player_reflection, 1, 1, 1, None) {
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
                "The create method should have set the modified_at column of the identifier to the current time!"
                    .to_string(),
            );
        }

        let player = match dsl.create_entity() {
            Ok(entity) => entity,
            Err(error) => {
                return Err(format!("Should be able to create an Entity! Got:\n{error}"));
            }
        };

        let mut player_position = match dsl.create_position(
            &player,
            1,
            1,
            -1,
            player_reflection_position.get_id().clone(),
        ) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{player:?}: Should be able to add an newly created Position!"
                ));
            }
        };

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

        let _ = match dsl.create_unique_position(&player_reflection, 1, 2, 3) {
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

        let _ = match dsl.create_unique_position(&player_reflection, 3, 4, 1) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{player_reflection:?}: Should be able to add an newly created unique Position!"
                ));
            }
        };

        if dsl.create_unique_position(&player, 3, 4, 1).is_ok() {
            return Err(format!(
                "{player_reflection:?}: Shouldn't be able to add an newly created unique Position which does already exist!"
            ));
        }

        let mut unique_player_position = match dsl.create_unique_position(&player, 1, 1, 1) {
            Ok(p) => p,
            Err(_) => {
                return Err(format!(
                    "{player_reflection:?}: Should be able to add an newly created unique Position!"
                ));
            }
        };

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

        let world1 = dsl.create_test(
            None,
            &player,
            &player,
            player.get_id().value(),
            &player,
            "string",
            "index_on_string",
            "index_on_wrapped_string",
            "unique_on_wrapped_string1",
            Some("wrapped_string_option".to_string()),
            0,
            spacetimedb::ScheduleAt::Time(ctx.timestamp),
        )?;

        let mut world2 = dsl.create_test(
            &player,
            player_reflection.get_id(),
            player.get_id(),
            player.get_id().value(),
            &player_reflection,
            "string",
            "index_on_string",
            "index_on_wrapped_string",
            "unique_on_wrapped_string2",
            Some("wrapped_string_option".to_string()),
            1,
            spacetimedb::ScheduleAt::Time(ctx.timestamp),
        )?;

        let _: Option<EntityId> = world1.get_wrapped_option();
        world2.set_wrapped_option(None);
        world2.set_wrapped_option(&player);
        world2.set_wrapped_option(player.get_id());
        world2.set_wrapped_option(player.get_id());

        // TODO: Add commented lines if https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21 is added
        let _ = dsl.get_tests_by_wrapped_index(&player);
        let _ = dsl.get_tests_by_wrapped_index(player.get_id());
        let _ = dsl.get_tests_by_wrapped_index(&player.get_id());
        let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index());
        //let _ = dsl.get_tests_by_wrapped_index(&player..);
        //let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index()..);
        let _ = dsl.delete_tests_by_wrapped_index(&player);
        let _ = dsl.delete_tests_by_wrapped_index(player.get_id());
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

        match dsl.delete_entity_by_id(&player_reflection) {
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
        if dsl
            .get_position_by_id(&player_reflection_position.get_id())
            .expect("should exist")
            .get_entity_id()
            .value()
            .ne(&0)
        {
            return Err("The entity_id of the position which was previously for the player_reflection entity should be 0 because the entity was deleted and the foreign key has a SetZero strategy!".to_string());
        }

        let _ = dsl.create_ship_object(&player);

        if let Ok(success) = dsl.delete_entity_by_id(&player) {
            return Err(format!(
                "The deletion of the entity player shouldn't have worked because ship_object.entity_id has a foreign key on the entity id with Error strategy Got: {success}",
            ));
        };

        // FIXME: Add test where two foreign keys match the same primary key - is it tried to delete the same row two times and therefore it fails?
        // TODO: Add test for SetNone strategy if it's implemented

        info!("Test executed successfully!");

        // This would produce a compilation error if the column order in unique multi column indices differ from the order in the table
        dsl.get_module1_by_database_and_parent_id_and_name(&0, &0, "")
            .expect_err("The module shouldn't exist");
        dsl.get_module2_by_database_and_name_and_parent_id(&0, "", &0)
            .expect_err("The module shouldn't exist");
        Ok(())
    }
}
