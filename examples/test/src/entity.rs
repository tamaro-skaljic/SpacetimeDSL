use crate::spacetimedsl::prelude::*;
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

pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let ctx = dsl.ctx();

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
            "The create method should have set the created_at column of the entity!".to_string(),
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
        .get_entity_relationships_by_parent_entity_id(dsl)
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
        .get_entity_relationships_by_child_entity_id(dsl)
        .len()
        .ne(&2)
    {
        return Err(
            "EntityId::get_entity_relationships_by_child_entity_id should find the 2 relationships whose child is player3!"
                .to_string(),
        );
    }

    if dsl.delete_entity_by_obj_id(&player).is_ok() {
        return Err(
            "Shouldn't be able to delete 'player' because it's a parent in a entity relationship!"
                .to_string(),
        );
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
        return Err("Shouldn't be able to get an Entity by an ID which doesn't exist!".to_string());
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

    my_view(&dsl.ctx().as_view_context()?);
    my_anonymous_view(&dsl.ctx().as_anonymous_view_context()?);

    Ok(())
}
