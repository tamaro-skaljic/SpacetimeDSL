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

pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    create_entity_sets_created_at(dsl)?;
    get_entity_by_obj_id_finds_the_created_entity(dsl)?;
    unique_multi_column_index_rejects_a_duplicate_relationship(dsl)?;
    relationships_by_parent_entity_id_are_those_of_the_parent(dsl)?;
    relationships_by_child_entity_id_are_those_of_the_child(dsl)?;
    on_delete_error_keeps_a_parent_of_relationships(dsl)?;
    on_delete_delete_removes_the_relationships_of_a_deleted_child(dsl)?;
    a_parent_without_relationships_can_be_deleted(dsl)?;
    on_delete_delete_removes_the_relationships2_of_a_deleted_entity(dsl)?;
    on_delete_ignore_keeps_the_reference_to_a_deleted_row(dsl)?;
    update_accepts_zero_instead_of_a_reference_to_a_deleted_row(dsl)?;
    update_rejects_a_new_reference_to_a_deleted_row(dsl)?;
    create_rejects_a_reference_to_a_deleted_row(dsl)?;
    views_read_through_the_read_only_dsl(dsl)?;

    Ok(())
}

fn create_entity_sets_created_at(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let entity = dsl
        .create_entity()
        .map_err(|error| format!("Should be able to create an Entity! Got:\n{error}"))?;

    if entity
        .get_created_at()
        .to_system_time()
        .ne(&dsl.ctx().timestamp.to_system_time())
    {
        return Err(
            "The create method should have set the created_at column of the entity to the time of the reducer call!"
                .to_string(),
        );
    }

    Ok(())
}

fn get_entity_by_obj_id_finds_the_created_entity(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let entity = dsl.create_entity()?;

    let found_entity = dsl
        .get_entity_by_obj_id(&entity)
        .map_err(|error| format!("Should be able to get an Entity by its ID! Got:\n{error}"))?;

    if found_entity.get_obj_id().ne(&entity.get_obj_id()) {
        return Err("get_entity_by_obj_id should return the Entity with the given ID!".to_string());
    }

    Ok(())
}

fn unique_multi_column_index_rejects_a_duplicate_relationship(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let parent = dsl.create_entity()?;
    let child = dsl.create_entity()?;
    let relationship = || CreateEntityRelationship {
        parent_entity_id: parent.get_obj_id(),
        child_entity_id: child.get_obj_id(),
    };
    dsl.create_entity_relationship(relationship())?;

    let duplicate = dsl.create_entity_relationship(relationship());

    if duplicate.is_ok() {
        return Err("Shouldn't be able to create the same entity relationship twice because of the unique multi column index `parent_child_entity_id`!".to_string());
    }

    Ok(())
}

fn relationships_by_parent_entity_id_are_those_of_the_parent(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let related_entities = create_related_entities(dsl)?;

    let relationships = related_entities
        .parent
        .get_obj_id()
        .get_entity_relationships_by_parent_entity_id(dsl);

    if relationships.len().ne(&2) {
        return Err(
            "EntityId::get_entity_relationships_by_parent_entity_id should find the 2 relationships whose parent is the Entity!"
                .to_string(),
        );
    }

    Ok(())
}

fn relationships_by_child_entity_id_are_those_of_the_child(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let related_entities = create_related_entities(dsl)?;

    let relationships = related_entities
        .child
        .get_obj_id()
        .get_entity_relationships_by_child_entity_id(dsl);

    if relationships.len().ne(&2) {
        return Err(
            "EntityId::get_entity_relationships_by_child_entity_id should find the 2 relationships whose child is the Entity!"
                .to_string(),
        );
    }

    Ok(())
}

fn on_delete_error_keeps_a_parent_of_relationships(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let related_entities = create_related_entities(dsl)?;
    let relationships_before = dsl.count_of_all_entity_relationships();

    let deletion = dsl.delete_entity_by_obj_id(&related_entities.parent);

    if deletion.is_ok() {
        return Err(
            "Shouldn't be able to delete an Entity which is the parent in an entity relationship, because parent_entity_id has on_delete = Error!"
                .to_string(),
        );
    }

    if dsl
        .count_of_all_entity_relationships()
        .ne(&relationships_before)
    {
        return Err("A refused deletion should keep every entity relationship!".to_string());
    }

    Ok(())
}

fn on_delete_delete_removes_the_relationships_of_a_deleted_child(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let related_entities = create_related_entities(dsl)?;
    let relationships_before = dsl.count_of_all_entity_relationships();

    dsl.delete_entity_by_obj_id(&related_entities.child)
        .map_err(|error| {
            format!(
                "Should be able to delete an Entity which is only a child in entity relationships! Got:\n{error}"
            )
        })?;

    if dsl
        .count_of_all_entity_relationships()
        .ne(&(relationships_before - 2))
    {
        return Err("Deleting an Entity should delete the 2 entity relationships whose child it is, through on_delete = Delete!".to_string());
    }

    Ok(())
}

fn a_parent_without_relationships_can_be_deleted(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let parent = dsl.create_entity()?;
    let child = dsl.create_entity()?;
    dsl.create_entity_relationship(CreateEntityRelationship {
        parent_entity_id: parent.get_obj_id(),
        child_entity_id: child.get_obj_id(),
    })?;
    dsl.delete_entity_by_obj_id(&child)?;

    dsl.delete_entity_by_obj_id(&parent).map_err(|error| {
        format!(
            "Should be able to delete an Entity which is not a parent in an entity relationship any more! Got:\n{error}"
        )
    })?;

    if dsl.get_entity_by_obj_id(&parent).is_ok() {
        return Err("Shouldn't be able to get an Entity after it was deleted!".to_string());
    }

    Ok(())
}

fn on_delete_delete_removes_the_relationships2_of_a_deleted_entity(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let first = dsl.create_entity()?;
    let second = dsl.create_entity()?;
    let third = dsl.create_entity()?;
    for (parent, child) in [(&first, &second), (&second, &third), (&third, &first)] {
        dsl.create_entity_relationship2(CreateEntityRelationship2 {
            parent_entity_id: parent.get_obj_id(),
            child_entity_id: child.get_obj_id(),
        })?;
    }
    let relationships2_before = dsl.count_of_all_entity_relationships2();

    dsl.delete_entity_by_obj_id(&second).map_err(|error| {
        format!(
            "Should be able to delete an Entity whose entity relationships 2 all have on_delete = Delete! Got:\n{error}"
        )
    })?;

    if dsl
        .count_of_all_entity_relationships2()
        .ne(&(relationships2_before - 2))
    {
        return Err("Deleting an Entity should delete the 2 entity relationships 2 it is part of, through on_delete = Delete!".to_string());
    }

    Ok(())
}

fn on_delete_ignore_keeps_the_reference_to_a_deleted_row(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let referenced = create_entity_relationship4(dsl, EntityRelationship4Id::new(0))?;
    let referencing = create_entity_relationship4(dsl, referenced.get_id())?;

    let deletion = dsl
        .delete_entity_relationship4_by_id(&referenced)
        .map_err(|error| {
            format!(
                "Should be able to delete an entity relationship 4 which another one references with on_delete = Ignore! Got:\n{error}"
            )
        })?;

    if deletion.entries[0]
        .row_value
        .ne(&referenced.get_id().to_string().into())
    {
        return Err(format!(
            "The deletion result should name the deleted entity relationship 4 {} first! Got:\n{deletion}",
            referenced.get_id()
        ));
    }

    let stored_referencing = dsl
        .get_entity_relationship4_by_id(&referencing)
        .map_err(|error| {
            format!(
                "The referencing entity relationship 4 should be kept, because its reference has on_delete = Ignore! Got:\n{error}"
            )
        })?;

    if stored_referencing
        .get_parent_entity_relationship4_id()
        .ne(&referenced.get_id())
    {
        return Err(
            "`parent_entity_relationship4_id` of the referencing entity relationship 4 should still name the deleted one, because it has on_delete = Ignore!"
                .to_string(),
        );
    }

    Ok(())
}

fn update_accepts_zero_instead_of_a_reference_to_a_deleted_row(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let referenced = create_entity_relationship4(dsl, EntityRelationship4Id::new(0))?;
    let mut referencing = create_entity_relationship4(dsl, referenced.get_id())?;
    dsl.delete_entity_relationship4_by_id(&referenced)?;
    referencing.set_parent_entity_relationship4_id(EntityRelationship4Id::new(0));

    dsl.update_entity_relationship4_by_id(referencing)
        .map_err(|error| {
            format!(
                "Should be able to set `parent_entity_relationship4_id` to 0, which references no row! Got:\n{error}"
            )
        })?;

    Ok(())
}

fn update_rejects_a_new_reference_to_a_deleted_row(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let deleted = create_entity_relationship4(dsl, EntityRelationship4Id::new(0))?;
    let mut referencing = create_entity_relationship4(dsl, EntityRelationship4Id::new(0))?;
    dsl.delete_entity_relationship4_by_id(&deleted)?;
    referencing.set_parent_entity_relationship4_id(&deleted);

    let update = dsl.update_entity_relationship4_by_id(referencing);

    if update.is_ok() {
        return Err("Shouldn't be able to set `parent_entity_relationship4_id` to the id of a deleted entity relationship 4!".to_string());
    }

    Ok(())
}

fn create_rejects_a_reference_to_a_deleted_row(
    dsl: &DSL<'_, ReducerContext>,
) -> Result<(), String> {
    let deleted = create_entity_relationship4(dsl, EntityRelationship4Id::new(0))?;
    dsl.delete_entity_relationship4_by_id(&deleted)?;

    let creation = create_entity_relationship4(dsl, deleted.get_id());

    if creation.is_ok() {
        return Err("Shouldn't be able to create an entity relationship 4 whose `parent_entity_relationship4_id` is the id of a deleted one!".to_string());
    }

    Ok(())
}

/// Both views look up the Entity with ID 0, which no Entity has, because `obj_id` is
/// `auto_inc` and starts at 1.
fn views_read_through_the_read_only_dsl(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let entity_of_view = my_view(&dsl.ctx().as_view_context()?);
    let entities_of_anonymous_view = my_anonymous_view(&dsl.ctx().as_anonymous_view_context()?);

    if entity_of_view.is_some() || !entities_of_anonymous_view.is_empty() {
        return Err("The views should find no Entity with ID 0!".to_string());
    }

    Ok(())
}

/// Two of the three Entities `create_related_entities` relates: `parent` is the parent of
/// two relationships and `child` the child of two.
struct RelatedEntities {
    parent: Entity,
    child: Entity,
}

/// Creates three Entities and the relationships parent → middle, parent → child and
/// middle → child.
fn create_related_entities(dsl: &DSL<'_, ReducerContext>) -> Result<RelatedEntities, String> {
    let parent = dsl.create_entity()?;
    let middle = dsl.create_entity()?;
    let child = dsl.create_entity()?;
    for (parent_entity, child_entity) in [(&parent, &middle), (&parent, &child), (&middle, &child)]
    {
        dsl.create_entity_relationship(CreateEntityRelationship {
            parent_entity_id: parent_entity.get_obj_id(),
            child_entity_id: child_entity.get_obj_id(),
        })?;
    }

    Ok(RelatedEntities { parent, child })
}

fn create_entity_relationship4(
    dsl: &DSL<'_, ReducerContext>,
    parent_entity_relationship4_id: EntityRelationship4Id,
) -> Result<EntityRelationship4, SpacetimeDSLError> {
    dsl.create_entity_relationship4(CreateEntityRelationship4 {
        parent_entity_relationship4_id,
    })
}
