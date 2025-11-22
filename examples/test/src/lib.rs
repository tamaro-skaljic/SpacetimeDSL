pub mod entity {
    use spacetimedb::{AnonymousViewContext, Timestamp, ViewContext, view};

    /// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.
    #[spacetimedsl::dsl(
        plural_name = entities,
        method(
            update = true,
        )
    )]
    #[spacetimedb::table(
        name = entity,
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
        obj_id: u128,

        created_at: Timestamp,
        modified_at: Option<Timestamp>,
    }

    #[view(name = my_view, public)]
    pub fn my_view(ctx: &ViewContext) -> Option<Entity> {
        ctx.db.entity().obj_id().find(0)
    }

    #[view(name = my_anonymous_view, public)]
    pub fn my_anonymous_view(ctx: &AnonymousViewContext) -> Vec<Entity> {
        ctx.db.entity().obj_id().find(0);
        vec![]
    }

    #[spacetimedsl::dsl(
        plural_name = tables,
        method(update = true),
        unique_index(name = id_and_name1),
        unique_index(name = id_and_name3),
    )]
    #[spacetimedb::table(
        name = table,
        index(name = id_and_name1, btree(columns = [id, name1])),
        index(name = id_and_name2, btree(columns = [id, name2])),
        index(name = id_and_name3, btree(columns = [id, name3])),
        index(name = id_and_name4, btree(columns = [id, name4])),
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
        name = entity_relationship,
        index(name = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])),
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
        name = entity_relationship2,
        index(name = parent_child_entity_id, btree(columns = [parent_entity_id, child_entity_id])),
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
        name = entity_relationship3,
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
        name = entity_relationship4,
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

pub mod component {
    pub mod identifier {
        use spacetimedb::Timestamp;

        /// A Identifier is a developer-friendly String.
        #[spacetimedsl::dsl(
            plural_name = identifiers,
            method(update = true),
        )]
        #[spacetimedb::table(
            name = identifier,
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
            name = identifier_reference,
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
            name = position,
            index(name = x_y_z, btree(columns = [x, y, z])),
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
            name = unique_position,
            index(name = x_y_z, btree(columns = [x, y, z])),
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
            name = test,
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

            #[index(direct)]
            #[unique]
            pub direct_index: u8,

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
            name = ship_object,
            index(name = id_and_sobj, btree(columns = [id, sobj_id])),
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
            name = space_ship_object,
            index(name = id_and_sobj, btree(columns = [id, sobj_id])),
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
            name = module1,
            index(name = database_and_parent_id_and_name, btree(columns = [database_id, parent_id, name])),
            public,
        )]
        #[spacetimedsl::dsl(
            plural_name = modules2,
            method(update = true),
            unique_index(name = database_and_name_and_parent_id),
        )]
        #[spacetimedb::table(
            name = module2,
            index(name = database_and_name_and_parent_id, btree(columns = [database_id, name, parent_id])),
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

    pub mod hook_test {
        use spacetimedb::Timestamp;

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
            name = attribute,
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
            name = potion,
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
            name = recipe,
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
            name = multi_column_index_with_hook_test,
            index(
                name = value_1_and_2,
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
            name = hook_call,
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
            dsl: &spacetimedsl::DSL,
            mut create_attribute_request: CreateAttribute,
        ) -> Result<CreateAttribute, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_ATTRIBUTE_INSERT".to_string(),
            })?;

            create_attribute_request.value =
                format!("{}_ATTRIBUTE", create_attribute_request.value);

            Ok(create_attribute_request)
        }

        #[spacetimedsl::hook]
        fn after_attribute_insert(
            dsl: &spacetimedsl::DSL,
            new_attribute: &Attribute,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
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
            dsl: &spacetimedsl::DSL,
            _old_attribute: &Attribute,
            mut new_attribute: Attribute,
        ) -> Result<Attribute, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_ATTRIBUTE_UPDATE".to_string(),
            })?;

            new_attribute.set_value(format!("{}_ATTRIBUTE", new_attribute.get_value()));

            Ok(new_attribute)
        }

        #[spacetimedsl::hook]
        fn after_attribute_update(
            dsl: &spacetimedsl::DSL,
            old_attribute: &Attribute,
            new_attribute: &Attribute,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
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
            dsl: &spacetimedsl::DSL,
            _old_attribute: &Attribute,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_ATTRIBUTE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_attribute_delete(
            dsl: &spacetimedsl::DSL,
            _old_attribute: &Attribute,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_ATTRIBUTE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_potion_insert(
            dsl: &spacetimedsl::DSL,
            mut create_potion_request: CreatePotion,
        ) -> Result<CreatePotion, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_POTION_INSERT".to_string(),
            })?;

            create_potion_request.value = format!("{}_POTION", create_potion_request.value);

            Ok(create_potion_request)
        }

        #[spacetimedsl::hook]
        fn after_potion_insert(
            dsl: &spacetimedsl::DSL,
            new_potion: &Potion,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
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
            dsl: &spacetimedsl::DSL,
            _old_potion: &Potion,
            mut new_potion: Potion,
        ) -> Result<Potion, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_POTION_UPDATE".to_string(),
            })?;

            new_potion.value = format!("{}_POTION", new_potion.value);

            Ok(new_potion)
        }

        #[spacetimedsl::hook]
        fn after_potion_update(
            dsl: &spacetimedsl::DSL,
            _old_potion: &Potion,
            _new_potion: &Potion,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_POTION_UPDATE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_potion_delete(
            dsl: &spacetimedsl::DSL,
            _old_potion: &Potion,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_POTION_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_potion_delete(
            dsl: &spacetimedsl::DSL,
            _old_potion: &Potion,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_POTION_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_recipe_insert(
            dsl: &spacetimedsl::DSL,
            mut create_recipe_request: CreateRecipe,
        ) -> Result<CreateRecipe, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_RECIPE_INSERT".to_string(),
            })?;

            create_recipe_request.value = format!("{}_RECIPE", create_recipe_request.value);

            Ok(create_recipe_request)
        }

        #[spacetimedsl::hook]
        fn after_recipe_insert(
            dsl: &spacetimedsl::DSL,
            _new_recipe: &Recipe,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_RECIPE_INSERT".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_recipe_update(
            dsl: &spacetimedsl::DSL,
            _old_recipe: &Recipe,
            mut new_recipe: Recipe,
        ) -> Result<Recipe, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_RECIPE_UPDATE".to_string(),
            })?;

            new_recipe.value = format!("{}_RECIPE", new_recipe.value);

            Ok(new_recipe)
        }

        #[spacetimedsl::hook]
        fn after_recipe_update(
            dsl: &spacetimedsl::DSL,
            _old_recipe: &Recipe,
            _new_recipe: &Recipe,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_RECIPE_UPDATE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_recipe_delete(
            dsl: &spacetimedsl::DSL,
            _old_recipe: &Recipe,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_RECIPE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_recipe_delete(
            dsl: &spacetimedsl::DSL,
            _old_recipe: &Recipe,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_RECIPE_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_multi_column_index_with_hook_test_insert(
            dsl: &spacetimedsl::DSL,
            create_multi_column_index_with_hook_test_request: CreateMultiColumnIndexWithHookTest,
        ) -> Result<CreateMultiColumnIndexWithHookTest, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT".to_string(),
            })?;

            Ok(create_multi_column_index_with_hook_test_request)
        }

        #[spacetimedsl::hook]
        fn after_multi_column_index_with_hook_test_insert(
            dsl: &spacetimedsl::DSL,
            _new_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_multi_column_index_with_hook_test_update(
            dsl: &spacetimedsl::DSL,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
            new_multi_column_index_with_hook_test: MultiColumnIndexWithHookTest,
        ) -> Result<MultiColumnIndexWithHookTest, spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE".to_string(),
            })?;

            Ok(new_multi_column_index_with_hook_test)
        }

        #[spacetimedsl::hook]
        fn after_multi_column_index_with_hook_test_update(
            dsl: &spacetimedsl::DSL,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
            _new_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn before_multi_column_index_with_hook_test_delete(
            dsl: &spacetimedsl::DSL,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE".to_string(),
            })?;

            Ok(())
        }

        #[spacetimedsl::hook]
        fn after_multi_column_index_with_hook_test_delete(
            dsl: &spacetimedsl::DSL,
            _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
        ) -> Result<(), spacetimedsl::SpacetimeDSLError> {
            dsl.create_hook_call(CreateHookCall {
                value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE".to_string(),
            })?;

            Ok(())
        }
    }
}

pub mod test {
    use crate::{
        component::{
            hook_test::{
                CountOfAllPotionRows, CreateAttribute, CreateAttributeRow,
                CreateMultiColumnIndexWithHookTest, CreateMultiColumnIndexWithHookTestRow,
                DeleteAttributeRowById, DeleteMultiColumnIndexWithHookTestRowById,
                GetAllHookCallRows, GetPotionRowOptionById, GetPotionRowOptionByValue,
                UpdateAttributeRowById, UpdateMultiColumnIndexWithHookTestRowById,
            },
            identifier::{
                CountOfAllIdentifierRows, CreateIdentifier, CreateIdentifierRow,
                GetIdentifierRowOptionByEntityId, GetIdentifierRowOptionByValue,
                UpdateIdentifierRowById, update_modified_at,
            },
            position::{
                CountOfAllPositionRows, CountOfAllUniquePositionRows, CreatePosition,
                CreatePositionRow, CreateUniquePosition, CreateUniquePositionRow,
                DeleteUniquePositionRowByXYZ, GetAllPositionRows, GetAllUniquePositionRows,
                GetPositionRowOptionById, PositionId, UniquePositionId, UpdatePositionRowById,
                UpdateUniquePositionRowById,
            },
            test::{
                CreateShipObject, CreateShipObjectRow, CreateTest, CreateTestRow,
                DeleteTestRowsByBtreeIndex, DeleteTestRowsByWrappedIndex,
                GetModule1RowOptionByDatabaseAndParentIdAndName,
                GetModule2RowOptionByDatabaseAndNameAndParentId, GetTestRowsByBtreeIndex,
                GetTestRowsByWrappedIndex,
            },
        },
        entity::{
            CountOfAllEntityRelationship2Rows, CountOfAllEntityRelationshipRows,
            CreateEntityRelationship, CreateEntityRelationship2, CreateEntityRelationship2Row,
            CreateEntityRelationship4, CreateEntityRelationship4Row, CreateEntityRelationshipRow,
            CreateEntityRow, DeleteEntityRelationship4RowById, DeleteEntityRowByObjId, EntityId,
            EntityRelationship4Id, GetEntityRelationship4RowOptionById, GetEntityRowOptionByObjId,
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
            direct_index: 0,
            scheduled_at: spacetimedb::ScheduleAt::Time(ctx.timestamp),
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
            direct_index: 1,
            scheduled_at: spacetimedb::ScheduleAt::Time(ctx.timestamp),
        })?;

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

        // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32Add test for SetNone strategy if it's implemented

        // This would produce a compilation error if the column order in unique multi column indices differ from the order in the table
        dsl.get_module1_by_database_and_parent_id_and_name(&0, &0, "")
            .expect_err("The module shouldn't exist");
        dsl.get_module2_by_database_and_name_and_parent_id(&0, "", &0)
            .expect_err("The module shouldn't exist");

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

        info!("Test executed successfully!");
        Ok(())
    }
}
