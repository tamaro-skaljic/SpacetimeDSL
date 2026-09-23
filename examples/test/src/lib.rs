::spacetimedsl::spacetimedsl!();

pub mod entity {
    use spacetimedb::Timestamp;

    /// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.
    #[spacetimedsl::dsl(
        plural_name = entities,
        method(
            update = true,
        )
    )]
    #[spacetimedb::table(
        accessor = entity,
        public,
    )]
    pub struct Entity {
        /// The unique ID of the Entity.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(EntityId)]
        #[referenced_by(path = crate::entity,                table = entity_relationship)]
        #[referenced_by(path = crate::entity,                table = entity_relationship2)]
        #[referenced_by(path = crate::component::identifier, table = identifier)]
        #[referenced_by(path = crate::component::position,   table = position)]
        #[referenced_by(path = crate::component::position,   table = unique_position)]
        #[referenced_by(path = crate::component::test,       table = test)]
        #[referenced_by(path = crate::component::test,       table = ship_object)]
        #[referenced_by(path = crate::component::test,       table = space_ship_object)]
        pub obj_id: u128,

        created_at: Timestamp,
        modified_at: Option<Timestamp>,
    }

    #[spacetimedsl::dsl(
        plural_name = tables,
        method(update = true),
        unique_index(name = id_and_name1),
        unique_index(name = id_and_name3),
    )]
    #[spacetimedb::table(
        accessor = table,
        index(accessor = id_and_name1, btree(columns = [id, name1])),
        index(accessor = id_and_name2, btree(columns = [id, name2])),
        index(accessor = id_and_name3, btree(columns = [id, name3])),
        index(accessor = id_and_name4, btree(columns = [id, name4])),
        public,
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

    #[spacetimedsl::dsl(
        plural_name = entity_relationships,
        method(update = true),
        unique_index(name = parent_child_entity_id)
    )]
    #[spacetimedb::table(
        accessor = entity_relationship,
        index(accessor = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])),
        public,
    )]
    pub struct EntityRelationship {
        /// The unique ID of the Entity Relationship.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u128,

        #[index(btree)]
        #[use_wrapper(EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Error)]
        parent_entity_id: u128,

        #[index(btree)]
        #[use_wrapper(EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
        child_entity_id: u128,

        inserted_at: Timestamp,
        updated_at: Option<Timestamp>,
    }

    #[spacetimedsl::dsl(
        plural_name = entity_relationships2,
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = entity_relationship2,
        index(accessor = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])),
        public,
    )]
    pub struct EntityRelationship2 {
        /// The unique ID of the Entity Relationship2.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u128,

        #[index(btree)]
        #[use_wrapper(EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
        parent_entity_id: u128,

        #[index(btree)]
        #[use_wrapper(EntityId)]
        #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
        pub child_entity_id: u128,

        inserted_at: Timestamp,

        updated_at: Timestamp,
    }

    #[spacetimedsl::dsl(
        plural_name = entity_relationships3,
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = entity_relationship3,
        public,
    )]
    pub struct EntityRelationship3 {
        /// The unique ID of the Entity Relationship3.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::entity, table = entity_relationship3)]
        id: u128,

        #[index(btree)]
        #[use_wrapper(EntityRelationship3Id)]
        #[foreign_key(path = crate::entity, table = entity_relationship3, column = id, on_delete = SetZero)]
        pub parent_entity_relationship3_id: u128,
    }

    #[spacetimedsl::dsl(
        plural_name = entity_relationships4,
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = entity_relationship4,
        public,
    )]
    pub struct EntityRelationship4 {
        /// The unique ID of the Entity Relationship4.
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::entity, table = entity_relationship4)]
        id: u128,

        #[index(btree)]
        #[use_wrapper(EntityRelationship4Id)]
        #[foreign_key(path = crate::entity, table = entity_relationship4, column = id, on_delete = Ignore)]
        pub parent_entity_relationship4_id: u128,
    }
}

pub mod timestamp_helper_test {
    use spacetimedb::Timestamp;

    #[spacetimedsl::dsl(
        plural_name = timestamp_records,
        method(update = true),
    )]
    #[spacetimedb::table(
        accessor = timestamp_record,
        public,
    )]
    pub struct TimestampRecord {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[set_on_create]
        started_at: Timestamp,

        #[set_on_update]
        finished_at: Option<Timestamp>,

        pub value: u32,
    }
}

pub mod component {
    pub mod identifier {
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

        pub(crate) fn update_modified_at(identifier: &mut Identifier, new_value: Timestamp) {
            identifier.modified_at = Some(new_value);
        }
    }

    pub mod position {
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
    }

    pub mod test {
        use spacetimedb::{ScheduleAt, Timestamp};

        /// A Position in the World.
        #[spacetimedsl::dsl(
            plural_name = tests,
            method(update = true),
        )]
        #[spacetimedb::table(
            accessor = test,
            public,
        )]
        pub struct Test {
            /// The unique ID of the World.
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            #[use_wrapper(crate::entity::EntityId)]
            pub wrapped_option: Option<u128>,

            // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 Add #[unique] if it's allowed by SpacetimeDB
            // #[unique]
            // pub unique_option: Option<u128>,

            // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 Add unique_wrapped_option if it's allowed by SpacetimeDB
            // #[unique]
            // #[use_wrapper(crate::entity::EntityId)]
            // pub unique_wrapped_option: Option<u128>,
            #[unique]
            #[use_wrapper(crate::entity::EntityId)]
            pub wrapped_unique: u128,

            #[index(btree)]
            #[use_wrapper(crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
            pub wrapped_index: u128,

            #[index(btree)]
            pub btree_index: u128,

            #[unique]
            #[use_wrapper(crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = SetZero)]
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

            // FIXME: #[index(btree)] // Index on option is currently not supported.
            #[create_wrapper]
            pub wrapped_timestamp_option: Option<Timestamp>,

            #[index(direct)]
            #[unique]
            pub direct_index: u8,

            pub tags: Vec<String>,

            created_at: Timestamp,

            modified_at: Timestamp,
            // TODO: Vec<T> columns with index, unique and solo, with wrap and without
            scheduled_at: ScheduleAt,
        }

        #[spacetimedsl::dsl(
            plural_name = ship_objects,
            method(update = true),
            unique_index(name = id_and_sobj),
        )]
        #[spacetimedb::table(
            accessor = ship_object,
            index(accessor = id_and_sobj, btree(columns = [id, sobj_id])),
            public,
        )]
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
            #[use_wrapper(crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Error)]
            pub entity_id: u128,
        }

        #[spacetimedsl::dsl(
            plural_name = space_ship_objects,
            method(update = true),
        )]
        #[spacetimedb::table(
            accessor = space_ship_object,
            index(accessor = id_and_sobj, btree(columns = [id, sobj_id])),
            public,
        )]
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
            #[use_wrapper(crate::entity::EntityId)]
            #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Error)]
            pub entity_id: u128,
        }

        #[spacetimedsl::dsl(
            plural_name = modules1,
            method(update = true),
            unique_index(name = database_and_parent_id_and_name),
        )]
        #[spacetimedb::table(
            accessor = module1,
            index(accessor = database_and_parent_id_and_name, btree(columns = [database_id, parent_id, name])),
            public,
        )]
        #[spacetimedsl::dsl(
            plural_name = modules2,
            method(update = true),
            unique_index(name = database_and_name_and_parent_id),
        )]
        #[spacetimedb::table(
            accessor = module2,
            index(accessor = database_and_name_and_parent_id, btree(columns = [database_id, name, parent_id])),
            public,
        )]
        pub struct Module {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            database_id: u128,

            pub name: String,

            pub parent_id: u128,

            #[default(0u128)]
            pub test: u128,
        }
    }

    /// Covers `#[auto_gen(v4)]` and `#[auto_gen(v7)]` on every index shape a `Uuid` column
    /// can take part in, and on a `#[dsl(singleton)]` table.
    pub mod uuid_test {
        use spacetimedb::Uuid;

        #[spacetimedsl::dsl(
            plural_name = uuid_primary_key_records,
            method(update = true),
        )]
        #[spacetimedb::table(
            accessor = uuid_primary_key_record,
            public,
        )]
        pub struct UUIDPrimaryKeyRecord {
            #[primary_key]
            #[create_wrapper]
            #[auto_gen(v7)]
            #[referenced_by(path = crate::component::uuid_reference_test, table = uuid_reference)]
            id: Uuid,

            pub name: String,
        }

        #[spacetimedsl::dsl(
            plural_name = uuid_unique_records,
            method(update = false),
        )]
        #[spacetimedb::table(
            accessor = uuid_unique_record,
            public,
        )]
        pub struct UUIDUniqueRecord {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[unique]
            #[create_wrapper]
            #[auto_gen(v4)]
            token: spacetimedb::Uuid,
        }

        #[spacetimedsl::dsl(
            plural_name = uuid_index_records,
            method(update = false),
        )]
        #[spacetimedb::table(
            accessor = uuid_index_record,
            public,
        )]
        pub struct UUIDIndexRecord {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[index(btree)]
            #[create_wrapper]
            #[auto_gen(v4)]
            token: Uuid,
        }

        #[spacetimedsl::dsl(
            plural_name = uuid_multi_column_index_records,
            method(update = false),
        )]
        #[spacetimedb::table(
            accessor = uuid_multi_column_index_record,
            index(accessor = token_and_group, btree(columns = [token, group])),
            public,
        )]
        pub struct UUIDMultiColumnIndexRecord {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[create_wrapper]
            #[auto_gen(v4)]
            token: Uuid,

            group: u32,
        }

        #[spacetimedsl::dsl(
            plural_name = uuid_unique_multi_column_index_records,
            method(update = false),
            unique_index(name = token_and_group),
        )]
        #[spacetimedb::table(
            accessor = uuid_unique_multi_column_index_record,
            index(accessor = token_and_group, btree(columns = [token, group])),
            public,
        )]
        pub struct UUIDUniqueMultiColumnIndexRecord {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[create_wrapper]
            #[auto_gen(v7)]
            token: Uuid,

            group: u32,
        }

        #[spacetimedsl::dsl(singleton, method(update = false))]
        #[spacetimedb::table(
            accessor = uuid_singleton_record,
            public,
        )]
        pub struct UUIDSingletonRecord {
            #[create_wrapper]
            #[auto_gen(v4)]
            token: Uuid,
        }
    }

    /// References an auto-generated `Uuid` primary key from another module, where the private
    /// field of its wrapper is out of reach.
    pub mod uuid_reference_test {
        use spacetimedb::Uuid;

        #[spacetimedsl::dsl(
            plural_name = uuid_references,
            method(update = false),
        )]
        #[spacetimedb::table(
            accessor = uuid_reference,
            public,
        )]
        pub struct UUIDReference {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u64,

            #[index(btree)]
            #[use_wrapper(crate::component::uuid_test::UUIDPrimaryKeyRecordId)]
            #[foreign_key(path = crate::component::uuid_test, table = uuid_primary_key_record, column = id, on_delete = Delete)]
            record_id: Uuid,
        }
    }

    pub mod hook_test {
        use crate::spacetimedsl::prelude::*;

        #[spacetimedsl::dsl(
            plural_name = attributes,
            method(update = true),
            hook(
                before(
                    insert,
                    update,
                    delete,
                ),
                after(
                    insert,
                    update,
                    delete,
                ),
            ),
        )]
        #[spacetimedb::table(
            accessor = attribute,
            public,
        )]
        pub struct Attribute {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            #[referenced_by(path = self, table = potion)]
            id: u128,

            pub value: String,
        }

        #[spacetimedsl::dsl(
            plural_name = potions,
            method(update = true),
            hook(
                before(
                    insert,
                    update,
                    delete,
                ),
                after(
                    insert,
                    update,
                    delete,
                ),
            ),
        )]
        #[spacetimedb::table(
            accessor = potion,
            public,
        )]
        pub struct Potion {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            #[referenced_by(path = self, table = recipe)]
            id: u128,

            #[unique]
            pub value: String,

            #[index(btree)]
            #[use_wrapper(AttributeId)]
            #[foreign_key(path = self, table = attribute, column = id, on_delete = Delete)]
            attribute_id: u128,

            #[unique]
            #[use_wrapper(AttributeId)]
            #[foreign_key(path = self, table = attribute, column = id, on_delete = Delete)]
            unique_attribute_id: u128,
        }

        #[spacetimedsl::dsl(
            plural_name = recipes,
            method(update = true),
            hook(
                before(
                    insert,
                    update,
                    delete,
                ),
                after(
                    insert,
                    update,
                    delete,
                ),
            ),
        )]
        #[spacetimedb::table(
            accessor = recipe,
            public,
        )]
        pub struct Recipe {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            #[unique]
            pub value: String,

            #[index(btree)]
            #[use_wrapper(PotionId)]
            #[foreign_key(path = self, table = potion, column = id, on_delete = Delete)]
            potion_id: u128,
        }

        #[spacetimedsl::dsl(
            plural_name = multi_column_index_with_hook_tests,
            method(update = true),
            hook(
                before(
                    insert,
                    update,
                    delete,
                ),
                after(
                    insert,
                    update,
                    delete,
                ),
            ),
        )]
        #[spacetimedb::table(
            accessor = multi_column_index_with_hook_test,
            index(
                accessor = value_1_and_2,
                btree(columns = [value_1, value_2])
            ),
            public,
        )]
        pub struct MultiColumnIndexWithHookTest {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            pub value_1: bool,

            pub value_2: bool,
        }

        #[spacetimedsl::dsl(
            plural_name = hook_calls,
            method(
                update = false,
                delete = false,
            ),
        )]
        #[spacetimedb::table(
            accessor = hook_call,
            public,
        )]
        pub struct HookCall {
            #[primary_key]
            #[auto_inc]
            #[create_wrapper]
            id: u128,

            value: String,

            created_at: Timestamp,
        }

        #[spacetimedsl::hook]
        fn before_attribute_insert(
            dsl: &DSL<'_, T>,
            mut create_attribute_request: CreateAttribute,
        ) -> Result<CreateAttribute, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_ATTRIBUTE_INSERT".to_string(),
            })?;

            create_attribute_request.value =
                format!("{}_ATTRIBUTE", create_attribute_request.value);

            Ok(create_attribute_request)
        }

        #[spacetimedsl::hook]
        fn after_attribute_insert(
            dsl: &DSL<'_, T>,
            new_attribute: &Attribute,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_ATTRIBUTE_INSERT".to_string(),
            })?;

            dsl.create_potion(CreatePotion {
                value: format!("PERMANENT_{}_INCREASE", new_attribute.value),
                attribute_id: new_attribute.get_id(),
                unique_attribute_id: new_attribute.get_id(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_attribute_update(
            dsl: &DSL<'_, T>,
            _old_attribute: &Attribute,
            mut new_attribute: Attribute,
        ) -> Result<Attribute, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_ATTRIBUTE_UPDATE".to_string(),
            })?;

            new_attribute.set_value(format!("{}_ATTRIBUTE", new_attribute.get_value()));

            Ok(new_attribute)
        }

        #[spacetimedsl::hook]
        fn after_attribute_update(
            dsl: &DSL<'_, T>,
            old_attribute: &Attribute,
            new_attribute: &Attribute,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_ATTRIBUTE_UPDATE".to_string(),
            })?;

            let mut potion = dsl.get_potion_by_value(&format!(
                "PERMANENT_{}_INCREASE_POTION",
                old_attribute.get_value()
            ))?;

            potion.set_value(format!("PERMANENT_{}_INCREASE", new_attribute.get_value()));

            dsl.update_potion_by_id(potion)?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_attribute_delete(
            dsl: &DSL<'_, T>,
            _old_attribute: &Attribute,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_ATTRIBUTE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_attribute_delete(
            dsl: &DSL<'_, T>,
            _old_attribute: &Attribute,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_ATTRIBUTE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_potion_insert(
            dsl: &DSL<'_, T>,
            mut create_potion_request: CreatePotion,
        ) -> Result<CreatePotion, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_POTION_INSERT".to_string(),
            })?;

            create_potion_request.value = format!("{}_POTION", create_potion_request.value);

            Ok(create_potion_request)
        }

        #[spacetimedsl::hook]
        fn after_potion_insert(
            dsl: &DSL<'_, T>,
            new_potion: &Potion,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_POTION_INSERT".to_string(),
            })?;

            dsl.create_recipe(CreateRecipe {
                value: format!("FIRST_PERMANENT_{}_INCREASE", new_potion.get_value()),
                potion_id: new_potion.get_id(),
            })?;

            dsl.create_recipe(CreateRecipe {
                value: format!("SECOND_PERMANENT_{}_INCREASE", new_potion.get_value()),
                potion_id: new_potion.get_id(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_potion_update(
            dsl: &DSL<'_, T>,
            _old_potion: &Potion,
            mut new_potion: Potion,
        ) -> Result<Potion, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_POTION_UPDATE".to_string(),
            })?;

            new_potion.value = format!("{}_POTION", new_potion.value);

            Ok(new_potion)
        }

        #[spacetimedsl::hook]
        fn after_potion_update(
            dsl: &DSL<'_, T>,
            _old_potion: &Potion,
            _new_potion: &Potion,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_POTION_UPDATE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_potion_delete(
            dsl: &DSL<'_, T>,
            _old_potion: &Potion,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_POTION_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_potion_delete(
            dsl: &DSL<'_, T>,
            _old_potion: &Potion,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_POTION_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_recipe_insert(
            dsl: &DSL<'_, T>,
            mut create_recipe_request: CreateRecipe,
        ) -> Result<CreateRecipe, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_RECIPE_INSERT".to_string(),
            })?;

            create_recipe_request.value = format!("{}_RECIPE", create_recipe_request.value);

            Ok(create_recipe_request)
        }

        #[spacetimedsl::hook]
        fn after_recipe_insert(
            dsl: &DSL<'_, T>,
            _new_recipe: &Recipe,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_RECIPE_INSERT".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_recipe_update(
            dsl: &DSL<'_, T>,
            _old_recipe: &Recipe,
            mut new_recipe: Recipe,
        ) -> Result<Recipe, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_RECIPE_UPDATE".to_string(),
            })?;

            new_recipe.value = format!("{}_RECIPE", new_recipe.value);

            Ok(new_recipe)
        }

        #[spacetimedsl::hook]
        fn after_recipe_update(
            dsl: &DSL<'_, T>,
            _old_recipe: &Recipe,
            _new_recipe: &Recipe,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_RECIPE_UPDATE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_recipe_delete(
            dsl: &DSL<'_, T>,
            _old_recipe: &Recipe,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_RECIPE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_recipe_delete(
            dsl: &DSL<'_, T>,
            _old_recipe: &Recipe,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_RECIPE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_multi_column_index_with_hook_test_insert(
            dsl: &DSL<'_, T>,
            create_multi_column_index_with_hook_test_request: CreateMultiColumnIndexWithHookTest,
        ) -> Result<CreateMultiColumnIndexWithHookTest, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT".to_string(),
            })?;

            Ok(create_multi_column_index_with_hook_test_request)
        }

        #[spacetimedsl::hook]
        fn after_multi_column_index_with_hook_test_insert(
            dsl: &DSL<'_, T>,
            _new_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_multi_column_index_with_hook_test_update(
            dsl: &DSL<'_, T>,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
            new_multi_column_index_with_hook_test: MultiColumnIndexWithHookTest,
        ) -> Result<MultiColumnIndexWithHookTest, SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE".to_string(),
            })?;

            Ok(new_multi_column_index_with_hook_test)
        }

        #[spacetimedsl::hook]
        fn after_multi_column_index_with_hook_test_update(
            dsl: &DSL<'_, T>,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
            _new_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_multi_column_index_with_hook_test_delete(
            dsl: &DSL<'_, T>,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_multi_column_index_with_hook_test_delete(
            dsl: &DSL<'_, T>,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE".to_string(),
            })?;

            Ok(())
        }
    }
}

pub mod singleton_test {
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
}

/// A singleton table which always has a row, because it answers with a default while the
/// table is empty.
///
/// Covers the whole of `#[dsl(singleton(with_default))]`: the `DefaultSingleton`
/// implementation, the `upsert_<table>` which replaces `create_<table>` and
/// `update_<table>`, and the two timestamp columns whose two write paths differ.
pub mod singleton_with_default_test {
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
    pub const INSERTED_SUFFIX: &str = " [inserted]";
    pub const UPDATED_SUFFIX: &str = " [updated]";

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
}

/// A `#[foreign_key]` column on a singleton table, for both singleton kinds.
///
/// A singleton may carry a foreign key without an index, and the reference-integrity check
/// of its write path has to find the row by the injected primary key rather than through a
/// wrapper that key does not have. Only a module which is actually built proves that.
pub mod singleton_with_foreign_key_test {
    use crate::spacetimedsl::prelude::*;

    #[spacetimedsl::dsl(plural_name = regions, method(update = true))]
    #[spacetimedb::table(
        accessor = region,
        public,
    )]
    pub struct Region {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(RegionId)]
        #[referenced_by(path = crate::singleton_with_foreign_key_test, table = server_binding)]
        #[referenced_by(path = crate::singleton_with_foreign_key_test, table = active_tournament)]
        id: u64,

        pub name: String,
    }

    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(
        accessor = server_binding,
        public,
    )]
    pub struct ServerBinding {
        #[use_wrapper(RegionId)]
        #[foreign_key(path = crate::singleton_with_foreign_key_test, table = region, column = id, on_delete = Delete)]
        pub region_id: u64,
    }

    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(
        accessor = active_tournament,
        public,
    )]
    pub struct ActiveTournament {
        #[use_wrapper(RegionId)]
        #[foreign_key(path = crate::singleton_with_foreign_key_test, table = region, column = id, on_delete = Delete)]
        pub region_id: u64,
    }

    impl DefaultSingleton for ActiveTournament {
        fn get_default(
            _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
        ) -> Result<ActiveTournament, SpacetimeDSLError> {
            Ok(ActiveTournament {
                id: 0,
                // Zero means "no reference yet", which every reference-integrity check
                // skips, so a default is allowed to point nowhere.
                region_id: 0,
            })
        }
    }
}

/// Hash indices, which no other table here uses.
///
/// The snapshots in `spacetimedsl_derive` pin the tokens generated for `#[index(hash)]`,
/// but only a module that is actually built proves those tokens compile and run. Covers a
/// non-unique single-column hash index, a unique single-column hash index and a
/// multi-column hash index.
pub mod hash_index_test {
    #[spacetimedsl::dsl(plural_name = sessions, method(update = true, delete = true))]
    #[spacetimedb::table(
        accessor = session,
        index(accessor = region_and_shard, hash(columns = [region_id, shard_id])),
        public,
    )]
    pub struct Session {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(hash)]
        pub token: String,

        #[index(hash)]
        #[unique]
        pub device_id: u64,

        pub region_id: u64,

        pub shard_id: u64,
    }
}

pub mod spacetimedsl_cascade_delete_hook_repro {
    use crate::spacetimedsl::SpacetimeDSLError;

    #[spacetimedsl::dsl(plural_name = parent_records, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = parent_record)]
    pub struct ParentRecord {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(ParentRecordId)]
        #[referenced_by(
        path = crate::spacetimedsl_cascade_delete_hook_repro,
        table = child_marker
    )]
        id: u64,
    }

    #[spacetimedsl::dsl(
        plural_name = child_markers,
        method(update = true, delete = true),
        hook(before(insert, update, delete), after(insert, update, delete))
    )]
    #[spacetimedb::table(accessor = child_marker)]
    pub struct ChildMarker {
        #[primary_key]
        #[use_wrapper(ParentRecordId)]
        #[foreign_key(
        path = crate::spacetimedsl_cascade_delete_hook_repro,
        table = parent_record,
        column = id,
        on_delete = Delete
    )]
        parent_id: u64,
    }

    #[spacetimedsl::hook]
    fn before_child_marker_insert(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        row: CreateChildMarker,
    ) -> Result<CreateChildMarker, SpacetimeDSLError> {
        Ok(row)
    }

    #[spacetimedsl::hook]
    fn after_child_marker_insert(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _row: &ChildMarker,
    ) -> Result<(), SpacetimeDSLError> {
        Ok(())
    }

    #[spacetimedsl::hook]
    fn before_child_marker_update(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _old: &ChildMarker,
        new: ChildMarker,
    ) -> Result<ChildMarker, SpacetimeDSLError> {
        Ok(new)
    }

    #[spacetimedsl::hook]
    fn after_child_marker_update(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _old: &ChildMarker,
        _new: &ChildMarker,
    ) -> Result<(), SpacetimeDSLError> {
        Ok(())
    }

    #[spacetimedsl::hook]
    fn before_child_marker_delete(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _row: &ChildMarker,
    ) -> Result<(), SpacetimeDSLError> {
        Ok(())
    }

    #[spacetimedsl::hook]
    fn after_child_marker_delete(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        _row: &ChildMarker,
    ) -> Result<(), SpacetimeDSLError> {
        Ok(())
    }
}

pub mod cascade_hook_error_test {
    use crate::spacetimedsl::SpacetimeDSLError;

    /// What the child's before-delete hook says when it refuses.
    pub const LOCKED_MESSAGE: &str = "this lock holder is locked";

    #[spacetimedsl::dsl(plural_name = lock_groups, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = lock_group)]
    pub struct LockGroup {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(LockGroupId)]
        #[referenced_by(path = crate::cascade_hook_error_test, table = lock_holder)]
        id: u64,

        /// A non-unique index, so the many-row delete method exists.
        #[index(btree)]
        batch: u64,
    }

    #[spacetimedsl::dsl(
        plural_name = lock_holders,
        method(update = false, delete = true),
        hook(before(delete))
    )]
    #[spacetimedb::table(accessor = lock_holder)]
    pub struct LockHolder {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(LockHolderId)]
        id: u64,

        #[index(btree)]
        #[use_wrapper(LockGroupId)]
        #[foreign_key(
            path = crate::cascade_hook_error_test,
            table = lock_group,
            column = id,
            on_delete = Delete
        )]
        group_id: u64,

        locked: bool,
    }

    #[spacetimedsl::hook]
    fn before_lock_holder_delete(
        _dsl: &crate::spacetimedsl::DSL<'_, T>,
        row: &LockHolder,
    ) -> Result<(), SpacetimeDSLError> {
        match row.get_locked() {
            false => Ok(()),
            true => Err(SpacetimeDSLError::Error(LOCKED_MESSAGE.to_string())),
        }
    }
}

pub mod soft_deletion {
    use spacetimedb::Timestamp;

    /// A retired `Archive` keeps its row, so a reducer can still read what was retired.
    #[spacetimedsl::dsl(
        plural_name = archives,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(
        accessor = archive,
        public,
    )]
    pub struct Archive {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(ArchiveId)]
        #[referenced_by(path = crate::soft_deletion, table = archive_entry)]
        id: u64,

        pub label: String,

        #[set_on_update]
        modified_at: Option<Timestamp>,

        deleted: bool,
    }

    /// Retiring an `Archive` retires its entries; deleting one removes them.
    #[spacetimedsl::dsl(
        plural_name = archive_entries,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(
        accessor = archive_entry,
        public,
    )]
    pub struct ArchiveEntry {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(ArchiveId)]
        #[foreign_key(
            path = crate::soft_deletion,
            table = archive,
            column = id,
            on_delete = Delete,
            on_soft_delete = SoftDelete,
        )]
        pub archive_id: u64,

        #[set_on_soft_delete]
        retired_at: Option<Timestamp>,
    }
}

pub mod test {
    use crate::spacetimedsl::prelude::*;
    use crate::{
        component::{
            hook_test::{CreateAttribute, CreateMultiColumnIndexWithHookTest},
            identifier::{CreateIdentifier, update_modified_at},
            position::{CreatePosition, CreateUniquePosition, PositionId, UniquePositionId},
            test::{CreateShipObject, CreateTest},
            uuid_reference_test::CreateUuidReference,
            uuid_test::{
                CreateUuidMultiColumnIndexRecord, CreateUuidPrimaryKeyRecord,
                CreateUuidUniqueMultiColumnIndexRecord,
            },
        },
        entity::{
            CreateEntityRelationship, CreateEntityRelationship2, CreateEntityRelationship4, Entity,
            EntityId, EntityRelationship4Id,
        },
        hash_index_test::CreateSession,
        singleton_test::CreateGameConfig,
        singleton_with_default_test::{INSERTED_SUFFIX, UPDATED_SUFFIX, WorldSettings},
        singleton_with_foreign_key_test::{CreateRegion, CreateServerBinding, RegionId},
        soft_deletion::{CreateArchive, CreateArchiveEntry},
        timestamp_helper_test::CreateTimestampRecord,
    };

    use log::info;

    #[spacetimedb::view(accessor = my_view, public)]
    pub fn my_view(ctx: &ViewContext) -> Option<Entity> {
        let dsl = read_only_dsl(ctx);

        let _ = dsl.count_of_all_entities();
        let _ = EntityId::new(0).get_position(&dsl);
        let _ = EntityId::new(0)
            .get_entity_relationships_by_parent_entity_id(&dsl)
            .len();
        dsl.get_entity_by_obj_id(EntityId::new(0)).ok()
    }

    #[spacetimedb::view(accessor = my_anonymous_view, public)]
    pub fn my_anonymous_view(ctx: &AnonymousViewContext) -> Vec<Entity> {
        let dsl = read_only_dsl(ctx);

        dsl.get_entity_by_obj_id(EntityId::new(0))
            .into_iter()
            .collect()
    }

    // FIXME: Procedures can currently not be called through the SpacetimeDB CLI and not through reducers, only through clients. Will need to rely on successful compilation for now.
    #[spacetimedb::procedure]
    pub fn my_procedure(ctx: &mut ProcedureContext) {
        let result = ctx.try_with_tx(|ctx| {
            let dsl = dsl(ctx);

            hook_call_test(&dsl)
        });

        match result {
            Ok(_) => {}
            Err(msg) => panic!("{msg}"),
        }
    }

    fn expect_uuid_version(
        uuid: spacetimedb::Uuid,
        expected_version: spacetimedb::sats::uuid::Version,
        column: &str,
    ) -> Result<(), String> {
        if uuid.get_version() != Some(expected_version) {
            return Err(format!(
                "`{column}` should be a UUID {expected_version:?}, found {uuid}."
            ));
        }
        Ok(())
    }

    /// Each `#[auto_gen]` table gets two rows, which must receive different UUIDs of the
    /// configured version, and must be found again by the stored UUID.
    fn test_auto_generated_uuids(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
        use spacetimedb::sats::uuid::Version;

        let first_record = dsl.create_uuid_primary_key_record(CreateUuidPrimaryKeyRecord {
            name: "first".to_string(),
        })?;
        let second_record = dsl.create_uuid_primary_key_record(CreateUuidPrimaryKeyRecord {
            name: "second".to_string(),
        })?;
        expect_uuid_version(first_record.get_id().value(), Version::V7, "id")?;
        if first_record.get_id().value() >= second_record.get_id().value() {
            return Err("A UUID v7 primary key should sort in creation order.".to_string());
        }
        if dsl
            .get_uuid_primary_key_record_by_id(second_record.get_id())?
            .ne(&second_record)
        {
            return Err("The row should be found by its UUID v7 primary key.".to_string());
        }

        let reference = dsl.create_uuid_reference(CreateUuidReference {
            record_id: first_record.get_id(),
        })?;
        if reference.get_record_id().ne(&first_record.get_id()) {
            return Err("The foreign key getter should return the referenced UUID.".to_string());
        }
        dsl.delete_uuid_primary_key_record_by_id(first_record.get_id())?;
        if dsl.get_uuid_reference_by_id(reference.get_id()).is_ok() {
            return Err("Deleting the UUID row should delete the referencing row.".to_string());
        }

        let first_unique = dsl.create_uuid_unique_record()?;
        let second_unique = dsl.create_uuid_unique_record()?;
        expect_uuid_version(first_unique.get_token().value(), Version::V4, "token")?;
        if first_unique.get_token().eq(&second_unique.get_token()) {
            return Err("Two rows should get different UUIDs v4.".to_string());
        }
        if dsl
            .get_uuid_unique_record_by_token(second_unique.get_token())?
            .ne(&second_unique)
        {
            return Err("The row should be found by its unique UUID.".to_string());
        }

        let first_indexed = dsl.create_uuid_index_record()?;
        let second_indexed = dsl.create_uuid_index_record()?;
        expect_uuid_version(first_indexed.get_token().value(), Version::V4, "token")?;
        if first_indexed.get_token().eq(&second_indexed.get_token()) {
            return Err("Two rows should get different indexed UUIDs.".to_string());
        }
        if dsl
            .get_uuid_index_records_by_token(first_indexed.get_token())
            .collect_vec()
            .ne(&vec![first_indexed])
        {
            return Err("The row should be found by its indexed UUID.".to_string());
        }

        let first_multi_column_indexed = dsl
            .create_uuid_multi_column_index_record(CreateUuidMultiColumnIndexRecord { group: 1 })?;
        let second_multi_column_indexed = dsl
            .create_uuid_multi_column_index_record(CreateUuidMultiColumnIndexRecord { group: 1 })?;
        expect_uuid_version(
            first_multi_column_indexed.get_token().value(),
            Version::V4,
            "token",
        )?;
        if first_multi_column_indexed
            .get_token()
            .eq(&second_multi_column_indexed.get_token())
        {
            return Err("Two rows should get different multi-column indexed UUIDs.".to_string());
        }
        if dsl
            .get_uuid_multi_column_index_records_by_token_and_group(
                first_multi_column_indexed.get_token(),
                first_multi_column_indexed.get_group(),
            )
            .collect_vec()
            .ne(&vec![first_multi_column_indexed])
        {
            return Err("The row should be found by its multi-column indexed UUID.".to_string());
        }

        let first_unique_multi_column_indexed = dsl.create_uuid_unique_multi_column_index_record(
            CreateUuidUniqueMultiColumnIndexRecord { group: 1 },
        )?;
        let second_unique_multi_column_indexed = dsl.create_uuid_unique_multi_column_index_record(
            CreateUuidUniqueMultiColumnIndexRecord { group: 1 },
        )?;
        expect_uuid_version(
            first_unique_multi_column_indexed.get_token().value(),
            Version::V7,
            "token",
        )?;
        if first_unique_multi_column_indexed.get_token().value()
            >= second_unique_multi_column_indexed.get_token().value()
        {
            return Err("UUIDs v7 should sort in creation order.".to_string());
        }
        if dsl
            .get_uuid_unique_multi_column_index_record_by_token_and_group(
                second_unique_multi_column_indexed.get_token(),
                second_unique_multi_column_indexed.get_group(),
            )?
            .ne(&second_unique_multi_column_indexed)
        {
            return Err(
                "The row should be found by its unique multi-column indexed UUID.".to_string(),
            );
        }

        let singleton = dsl.create_uuid_singleton_record()?;
        expect_uuid_version(singleton.get_token().value(), Version::V4, "token")?;
        if dsl.get_uuid_singleton_record()?.ne(&singleton) {
            return Err("The singleton row should keep its generated UUID.".to_string());
        }

        Ok(())
    }

    #[spacetimedb::reducer]
    fn tester(ctx: &ReducerContext) -> Result<(), String> {
        let dsl = dsl(ctx);

        test_auto_generated_uuids(&dsl)?;

        let timestamp_record = dsl.create_timestamp_record(CreateTimestampRecord { value: 1 })?;
        if timestamp_record.get_started_at().ne(&ctx.timestamp)
            || timestamp_record.get_finished_at().is_some()
        {
            return Err(
                "The helper timestamp attributes should initialize create timestamps correctly."
                    .to_string(),
            );
        }

        let mut updated_timestamp_record = timestamp_record;
        updated_timestamp_record.set_value(2);
        let updated_timestamp_record =
            dsl.update_timestamp_record_by_id(updated_timestamp_record)?;
        if updated_timestamp_record
            .get_finished_at()
            .ne(&Some(ctx.timestamp))
        {
            return Err(
                "The #[set_on_update] helper attribute should refresh the timestamp on update."
                    .to_string(),
            );
        }

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

        match dsl.get_entity_by_obj_id(&player) {
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

        dsl.create_entity_relationship(CreateEntityRelationship {
            parent_entity_id: player.get_obj_id(),
            child_entity_id: player2.get_obj_id(),
        })?;
        if dsl
            .create_entity_relationship(CreateEntityRelationship {
                parent_entity_id: player.get_obj_id(),
                child_entity_id: player2.get_obj_id(),
            })
            .is_ok()
        {
            return Err("Shouldn't be able to create the same entity relationship because of the unique multi column index `parent_child_entity_id`".to_string());
        }
        let player3 = dsl.create_entity()?;
        dsl.create_entity_relationship(CreateEntityRelationship {
            parent_entity_id: player.get_obj_id(),
            child_entity_id: player3.get_obj_id(),
        })?;
        dsl.create_entity_relationship(CreateEntityRelationship {
            parent_entity_id: player2.get_obj_id(),
            child_entity_id: player3.get_obj_id(),
        })?;

        if dsl.count_of_all_entity_relationships().ne(&3) {
            return Err("Count of entity relationships should be 3!".to_string());
        }

        if player
            .get_obj_id()
            .get_entity_relationships_by_parent_entity_id(&dsl)
            .len()
            .ne(&2)
        {
            return Err(
                "EntityId::get_entity_relationships_by_parent_entity_id should find the 2 relationships whose parent is the player!"
                    .to_string(),
            );
        }

        if player3
            .get_obj_id()
            .get_entity_relationships_by_child_entity_id(&dsl)
            .len()
            .ne(&2)
        {
            return Err(
                "EntityId::get_entity_relationships_by_child_entity_id should find the 2 relationships whose child is player3!"
                    .to_string(),
            );
        }

        if dsl.delete_entity_by_obj_id(&player).is_ok() {
            return Err("Shouldn't be able to delete 'player' because it's a parent in a entity relationship!".to_string());
        }

        if dsl.count_of_all_entity_relationships().ne(&3) {
            return Err("Count of entity relationships should be 3!".to_string());
        }

        match dsl.delete_entity_by_obj_id(&player3) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "Should be able to delete 'player3' because it's only a child in entity relationships! Got:\n{error}"
                ));
            }
        };

        if dsl.count_of_all_entity_relationships().ne(&1) {
            return Err("Count of entity relationships should be 1 because 2 should be deleted through the foreign key / referenced by feature! (1)".to_string());
        }

        match dsl.delete_entity_by_obj_id(&player2) {
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
        match dsl.delete_entity_by_obj_id(&player) {
            Ok(_) => {}
            Err(error) => {
                return Err(format!(
                    "Should be able to delete 'player' because it's not a parent anymore in a entity relationship! Got:\n{error}"
                ));
            }
        };

        if dsl.get_entity_by_obj_id(&player).is_ok() {
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

        dsl.create_entity_relationship2(CreateEntityRelationship2 {
            parent_entity_id: player.get_obj_id(),
            child_entity_id: player2.get_obj_id(),
        })?;
        dsl.create_entity_relationship2(CreateEntityRelationship2 {
            parent_entity_id: player2.get_obj_id(),
            child_entity_id: player3.get_obj_id(),
        })?;
        dsl.create_entity_relationship2(CreateEntityRelationship2 {
            parent_entity_id: player3.get_obj_id(),
            child_entity_id: player.get_obj_id(),
        })?;

        if dsl.count_of_all_entity_relationships2().ne(&3) {
            return Err("Count of entity relationships 2 should be 3!".to_string());
        }

        match dsl.delete_entity_by_obj_id(&player2) {
            Ok(_) => {}
            Err(error) => return Err(format!("Should be able to delete 'player'! Got:\n{error}")),
        };

        if dsl.count_of_all_entity_relationships2().ne(&1) {
            return Err("Count of entity relationships should be 1 because 2 should be deleted through the foreign key / referenced by feature! (2)".to_string());
        }

        let er4_1 = dsl.create_entity_relationship4(CreateEntityRelationship4 {
            parent_entity_relationship4_id: EntityRelationship4Id::new(0),
        })?;
        let mut er4_2 = dsl.create_entity_relationship4(CreateEntityRelationship4 {
            parent_entity_relationship4_id: er4_1.get_parent_entity_relationship4_id(),
        })?;

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

        if dsl
            .create_entity_relationship4(CreateEntityRelationship4 {
                parent_entity_relationship4_id: er4_1.get_id(),
            })
            .is_ok()
        {
            return Err("Shouldn't be able to create a `entity_relationship4` with `er4_1` as `parent_entity_relationship4_id`".to_string());
        }

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
            .get_position(&dsl)?
            .get_id()
            .ne(&player_position.get_id())
        {
            return Err(
                "EntityId::get_position should find the Position of the player!".to_string(),
            );
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
            .get_unique_position(&dsl)?
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

        let world1 = dsl.create_test(CreateTest {
            wrapped_option: None,
            wrapped_unique: player.get_obj_id(),
            wrapped_index: player.get_obj_id(),
            btree_index: player.get_obj_id().value(),
            unique: player.get_obj_id(),
            string: "string".to_string(),
            index_on_string: "index_on_string".to_string(),
            index_on_wrapped_string: "index_on_wrapped_string".to_string(),
            unique_on_wrapped_string: "unique_on_wrapped_string".to_string(),
            wrapped_string_option: Some("wrapped_string_option".to_string()),
            wrapped_timestamp_option: Some(ctx.timestamp),
            direct_index: 0,
            tags: vec![],
            scheduled_at: ScheduleAt::Time(ctx.timestamp),
        })?;

        let mut world2 = dsl.create_test(CreateTest {
            wrapped_option: Some(player.get_obj_id()),
            wrapped_unique: player_reflection.get_obj_id(),
            wrapped_index: player.get_obj_id(),
            btree_index: player.get_obj_id().value(),
            unique: player_reflection.get_obj_id(),
            string: "string".to_string(),
            index_on_string: "index_on_string".to_string(),
            index_on_wrapped_string: "index_on_wrapped_string".to_string(),
            unique_on_wrapped_string: "unique_on_wrapped_string2".to_string(),
            wrapped_string_option: Some("wrapped_string_option".to_string()),
            wrapped_timestamp_option: Some(ctx.timestamp),
            direct_index: 1,
            tags: vec![],
            scheduled_at: ScheduleAt::Time(ctx.timestamp),
        })?;

        let tags = world2.get_tags_mut();
        tags.push("new_tag".to_string());
        world2 = dsl.update_test_by_id(world2)?;

        if world2.get_tags().len() != 1 || world2.get_tags()[0].ne("new_tag") {
            return Err("Tags should have been updated to contain 'new_tag'".to_string());
        }

        world2.get_tags_mut().clear();
        world2 = dsl.update_test_by_id(world2)?;

        if !world2.get_tags().is_empty() {
            return Err("Tags should have been updated to contain nothing".to_string());
        }

        world2 = dsl.get_test_by_id(&world2)?;

        if !world2.get_tags().is_empty() {
            return Err("Tags should have been updated to contain nothing".to_string());
        }

        let _: Option<EntityId> = world1.get_wrapped_option();
        world2.set_wrapped_option(None);
        world2.set_wrapped_option(&player);
        world2.set_wrapped_option(player.get_obj_id());
        world2.set_wrapped_option(player.get_obj_id());

        // TODO: Add commented lines if https://github.com/tamaro-skaljic/SpacetimeDSL/issues/21 is added
        let _ = dsl.get_tests_by_wrapped_index(&player);
        let _ = dsl.get_tests_by_wrapped_index(player.get_obj_id());
        let _ = dsl.get_tests_by_wrapped_index(&player.get_obj_id());
        let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index());
        //let _ = dsl.get_tests_by_wrapped_index(&player..);
        //let _ = dsl.get_tests_by_wrapped_index(world2.get_wrapped_index()..);
        let _ = dsl.delete_tests_by_wrapped_index(&player);
        let _ = dsl.delete_tests_by_wrapped_index(player.get_obj_id());
        let _ = dsl.delete_tests_by_wrapped_index(player.get_obj_id());
        let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index());
        //let _ = dsl.delete_tests_by_wrapped_index(&player..);
        //let _ = dsl.delete_tests_by_wrapped_index(&player..&player);
        //let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index()..);
        //let _ = dsl.delete_tests_by_wrapped_index(world2.get_wrapped_index()..world2.get_wrapped_index());

        let _ = dsl.get_tests_by_btree_index(world2.get_btree_index());
        //let _ = dsl.get_tests_by_btree_index(world2.get_btree_index()..);
        let _ = dsl.delete_tests_by_btree_index(world2.get_btree_index());
        //let _ = dsl.delete_tests_by_btree_index(world2.get_btree_index()..);

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
        if dsl
            .get_position_by_id(&player_reflection_position.get_id())
            .expect("should exist")
            .get_entity_id()
            .value()
            .ne(&0)
        {
            return Err("The entity_id of the position which was previously for the player_reflection entity should be 0 because the entity was deleted and the foreign key has a SetZero strategy!".to_string());
        }

        let _ = dsl.create_ship_object(CreateShipObject {
            entity_id: player.get_obj_id(),
        });

        if let Ok(success) = dsl.delete_entity_by_obj_id(&player) {
            return Err(format!(
                "The deletion of the entity player shouldn't have worked because ship_object.entity_id has a foreign key on the entity id with Error strategy Got: {success}",
            ));
        };

        // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 Add test for SetNone strategy if it's implemented

        // This would produce a compilation error if the column order in unique multi column indices differ from the order in the table
        dsl.get_module1_by_database_and_parent_id_and_name(&0, &0, "")
            .expect_err("The module shouldn't exist");
        dsl.get_module2_by_database_and_name_and_parent_id(&0, "", &0)
            .expect_err("The module shouldn't exist");

        singleton_test(&dsl)?;

        singleton_with_default_test(&dsl)?;

        singleton_with_foreign_key_test(&dsl)?;

        hash_index_test(&dsl)?;

        hook_call_test(&dsl)?;

        my_view(&dsl.ctx().as_view_context()?);
        my_anonymous_view(&dsl.ctx().as_anonymous_view_context()?);

        use crate::cascade_hook_error_test::{CreateLockGroup, CreateLockHolder, LOCKED_MESSAGE};

        let one_row_group = dsl.create_lock_group(CreateLockGroup { batch: 1 })?;
        dsl.create_lock_holder(CreateLockHolder {
            group_id: one_row_group.get_id(),
            locked: true,
        })?;

        match dsl.delete_lock_group_by_id(&one_row_group) {
            Ok(_) => {
                return Err(
                    "Deleting a lock group whose holder's before_delete hook fails should fail!"
                        .to_string(),
                );
            }
            Err(error) => {
                let error = error.to_string();
                if !error.contains(LOCKED_MESSAGE) {
                    return Err(format!(
                        "The error the before_delete hook raised should reach the caller! Got:\n{error}"
                    ));
                }
            }
        };

        let many_rows_group = dsl.create_lock_group(CreateLockGroup { batch: 2 })?;
        dsl.create_lock_holder(CreateLockHolder {
            group_id: many_rows_group.get_id(),
            locked: true,
        })?;

        match dsl.delete_lock_groups_by_batch(&2) {
            Ok(_) => {
                return Err(
                    "Deleting lock groups whose holders' before_delete hook fails should fail!"
                        .to_string(),
                );
            }
            Err(error) => {
                let error = error.to_string();
                if !error.contains(LOCKED_MESSAGE) {
                    return Err(format!(
                        "The error the before_delete hook raised should reach the caller of the many-row delete! Got:\n{error}"
                    ));
                }
            }
        };

        soft_deletion_test(&dsl)?;

        info!("Test executed successfully!");
        Ok(())
    }

    /// Exercises soft deletion end to end: the marker is written, the row stays readable,
    /// `modified_at` is left alone, the cascade retires the referencing row through
    /// `on_soft_delete = SoftDelete`, and retiring an already retired row does nothing.
    fn soft_deletion_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
        let archive = dsl.create_archive(CreateArchive {
            label: "first".to_string(),
        })?;
        let archive_id = archive.get_id();

        let entry = dsl.create_archive_entry(CreateArchiveEntry {
            archive_id: archive_id.clone(),
        })?;
        let entry_id = entry.get_id();

        let modified_at_before_soft_delete = *archive.get_modified_at();

        dsl.soft_delete_archive_by_id(&archive_id)?;

        let retired_archive = match dsl.get_archive_by_id(&archive_id) {
            Err(_) => {
                return Err("A soft-deleted Archive row should still be readable!".to_string());
            }
            Ok(retired_archive) => retired_archive,
        };

        if !retired_archive.get_deleted() {
            return Err("soft_delete_archive_by_id should have set the marker!".to_string());
        }

        if retired_archive
            .get_modified_at()
            .ne(&modified_at_before_soft_delete)
        {
            return Err("A soft deletion should leave modified_at alone!".to_string());
        }

        let retired_entry = match dsl.get_archive_entry_by_id(&entry_id) {
            Err(_) => {
                return Err(
                    "An ArchiveEntry of a soft-deleted Archive should still exist!".to_string(),
                );
            }
            Ok(retired_entry) => retired_entry,
        };

        if retired_entry.get_retired_at().is_none() {
            return Err(
                "on_soft_delete = SoftDelete should have retired the ArchiveEntry!".to_string(),
            );
        }

        let repeated = dsl.soft_delete_archive_by_id(&archive_id)?;

        if !repeated.entries.is_empty() {
            return Err(
                "Soft-deleting an already retired row should report no entries!".to_string(),
            );
        }

        Ok(())
    }

    /// Exercises every method shape a hash index produces: `filter` through the
    /// non-unique single-column index, `find` through the unique single-column one, and
    /// `filter` through the multi-column one.
    fn hash_index_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
        dsl.create_session(CreateSession {
            token: "first".to_string(),
            device_id: 11,
            region_id: 1,
            shard_id: 2,
        })
        .map_err(|e| format!("Should be able to create a Session! Got:\n{e}"))?;

        dsl.create_session(CreateSession {
            token: "first".to_string(),
            device_id: 22,
            region_id: 1,
            shard_id: 2,
        })
        .map_err(|e| format!("Should be able to create a second Session! Got:\n{e}"))?;

        let count_by_token = dsl.get_sessions_by_token("first").count();
        if count_by_token.ne(&2) {
            return Err(format!(
                "A non-unique hash index should find both sessions, got: {count_by_token}"
            ));
        }

        let count_by_region_and_shard = dsl.get_sessions_by_region_and_shard(&1, &2).count();
        if count_by_region_and_shard.ne(&2) {
            return Err(format!(
                "A multi-column hash index should find both sessions, got: {count_by_region_and_shard}"
            ));
        }

        let session = dsl
            .get_session_by_device_id(&22)
            .map_err(|e| format!("A unique hash index should find one session! Got:\n{e}"))?;

        if session.get_token().ne("first") {
            return Err(format!(
                "The session found by device_id should carry the token 'first', got: {}",
                session.get_token()
            ));
        }

        dsl.delete_session_by_device_id(&22)
            .map_err(|e| format!("Should be able to delete by a unique hash index! Got:\n{e}"))?;

        let count_after_delete = dsl.get_sessions_by_token("first").count();
        if count_after_delete.ne(&1) {
            return Err(format!(
                "One session should remain after deleting by device_id, got: {count_after_delete}"
            ));
        }

        dsl.delete_sessions_by_token("first").map_err(|e| {
            format!("Should be able to delete by a non-unique hash index! Got:\n{e}")
        })?;

        let count_at_end = dsl.get_sessions_by_token("first").count();
        if count_at_end.ne(&0) {
            return Err(format!(
                "No session should remain after deleting by token, got: {count_at_end}"
            ));
        }

        Ok(())
    }

    fn singleton_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
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

    fn singleton_with_default_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
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
                "Getting the WorldSettings a second time should still give its default!"
                    .to_string(),
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

        let updated = dsl.upsert_world_settings(changed_settings).map_err(|e| {
            format!("Upserting the WorldSettings again should update it! Got:\n{e}")
        })?;

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

        let mut settings_from_the_default =
            WorldSettings::get_default(&read_only_dsl(dsl.ctx()))
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

    fn singleton_with_foreign_key_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
        let region = dsl.create_region(CreateRegion {
            name: "Europe".to_string(),
        })?;

        // A singleton without a default: the reference-integrity check of `update_<table>`
        // has to find the row by the injected primary key.
        let mut server_binding = dsl.create_server_binding(CreateServerBinding {
            region_id: region.get_id(),
        })?;
        server_binding.set_region_id(region.get_id());

        dsl.update_server_binding(server_binding).map_err(|e| {
            format!("Updating a singleton with a foreign key should work! Got:\n{e}")
        })?;

        // A singleton with a default: its insert path and its update path each run their own
        // reference-integrity check.
        let mut tournament = dsl.get_active_tournament()?;
        tournament.set_region_id(region.get_id());

        let tournament = dsl.upsert_active_tournament(tournament).map_err(|e| {
            format!("Upserting a singleton with a foreign key should insert it! Got:\n{e}")
        })?;

        if tournament.get_region_id().ne(&region.get_id()) {
            return Err("The inserted region_id should be the one that was set!".to_string());
        }

        let mut tournament = tournament;
        tournament.set_region_id(RegionId::new(u64::MAX));

        dsl.upsert_active_tournament(tournament).expect_err(
            "Upserting a singleton whose foreign key points at no row should be rejected",
        );

        let unchanged = dsl.get_active_tournament()?;
        if unchanged.get_region_id().ne(&region.get_id()) {
            return Err("A rejected upsert should leave the stored row alone!".to_string());
        }

        Ok(())
    }

    fn hook_call_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
        let mut strength = dsl.create_attribute(CreateAttribute {
            value: "STRENGTH".to_string(),
        })?;

        if strength.get_value().ne("STRENGTH_ATTRIBUTE") {
            return Err("Attribute value should be 'STRENGTH_ATTRIBUTE'".to_string());
        }

        let permanent_strength_attribute_increase_potion =
            dsl.get_potion_by_value("PERMANENT_STRENGTH_ATTRIBUTE_INCREASE_POTION")?;

        strength.set_value("POWER".to_string());

        let power = dsl.update_attribute_by_id(strength)?;

        if power.get_value().ne("POWER_ATTRIBUTE") {
            return Err(format!(
                "Attribute value should be 'POWER_ATTRIBUTE'. Got: {}",
                power.get_value()
            ));
        }

        let permanent_power_attribute_increase_potion =
            dsl.get_potion_by_id(permanent_strength_attribute_increase_potion.get_id())?;

        if permanent_power_attribute_increase_potion
            .get_value()
            .ne("PERMANENT_POWER_ATTRIBUTE_INCREASE_POTION")
        {
            return Err(
                "Potion value should be 'PERMANENT_POWER_ATTRIBUTE_INCREASE_POTION'".to_string(),
            );
        }

        dsl.delete_attribute_by_id(&power)?;

        if dsl.count_of_all_potions().ne(&0) {
            return Err("There should be 0 potions because the attribute was deleted and the potion should be deleted in the after delete hook of the attribute table.".to_string());
        }

        let mut multi_column_index_with_hook_test =
            dsl.create_multi_column_index_with_hook_test(CreateMultiColumnIndexWithHookTest {
                value_1: false,
                value_2: false,
            })?;

        multi_column_index_with_hook_test.set_value_1(true);
        multi_column_index_with_hook_test.set_value_2(true);

        multi_column_index_with_hook_test =
            dsl.update_multi_column_index_with_hook_test_by_id(multi_column_index_with_hook_test)?;

        dsl.delete_multi_column_index_with_hook_test_by_id(&multi_column_index_with_hook_test)?;

        let hook_calls: Vec<_> = dsl
            .get_all_hook_calls()
            .map(|hc| hc.get_value().to_string())
            .collect();

        let expected_hook_call_values = vec![
            "BEFORE_ATTRIBUTE_INSERT",
            "AFTER_ATTRIBUTE_INSERT",
            "BEFORE_POTION_INSERT",
            "AFTER_POTION_INSERT",
            "BEFORE_RECIPE_INSERT",
            "AFTER_RECIPE_INSERT",
            "BEFORE_RECIPE_INSERT",
            "AFTER_RECIPE_INSERT",
            "BEFORE_ATTRIBUTE_UPDATE",
            "AFTER_ATTRIBUTE_UPDATE",
            "BEFORE_POTION_UPDATE",
            "AFTER_POTION_UPDATE",
            "BEFORE_ATTRIBUTE_DELETE",
            "AFTER_ATTRIBUTE_DELETE",
            "BEFORE_POTION_DELETE",
            "AFTER_POTION_DELETE",
            "BEFORE_RECIPE_DELETE",
            "AFTER_RECIPE_DELETE",
            "BEFORE_RECIPE_DELETE",
            "AFTER_RECIPE_DELETE",
            "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT",
            "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT",
            "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE",
            "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE",
            "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE",
            "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE",
        ];

        if hook_calls.ne(&expected_hook_call_values) {
            return Err(format!(
                "The hook calls do not match the expected ones!\n\nExpected:\n{:?}\n\nActual:\n{:?}",
                expected_hook_call_values, hook_calls
            ));
        }

        Ok(())
    }
}
