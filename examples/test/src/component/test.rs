use crate::entity::EntityId;
use crate::spacetimedsl::prelude::*;
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

pub(crate) fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    let ctx = dsl.ctx();

    let player = dsl.create_entity()?;
    let other_player = dsl.create_entity()?;

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
        wrapped_unique: other_player.get_obj_id(),
        wrapped_index: player.get_obj_id(),
        btree_index: player.get_obj_id().value(),
        unique: other_player.get_obj_id(),
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

    Ok(())
}
