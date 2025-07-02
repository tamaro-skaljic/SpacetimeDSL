/// A Entity is a unique machine-readable identifier - it contains no data other than that and has no behavior.

#[spacetimedsl::dsl(plural_name = entities)]
#[spacetimedb::table(name = entity, public)]
pub struct Entity {
    /// The unique ID of the Entity.
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    created_at: spacetimedb::Timestamp,

    modified_at: spacetimedb::Timestamp,
}

#[spacetimedb::reducer]
pub fn example(ctx: &spacetimedb::ReducerContext) -> Result<(), String> {
    let dsl: spacetimedsl::DSL<'_> = spacetimedsl::dsl(ctx);

    // Create an Entity
    let entity: Entity = dsl.create_entity()?; // Result<Entity, spacetimedsl::SpacetimeDSLError>
    let entity_id: EntityId = entity.get_id();

    // Get the count of all Entities
    let _count: u64 = dsl.count_of_all_entities();

    // Get all Entities and log their ids
    dsl.get_all_entities().for_each(|e: Entity| log::debug!("{}", e.id));

    // Get an Entity by its id
    let entity: Entity = dsl.get_entity_by_id(&entity)?; // Result<Entity, spacetimedsl::SpacetimeDSLError>
    let entity: Entity = dsl.get_entity_by_id(&entity_id)?; // Result<Entity, spacetimedsl::SpacetimeDSLError>

    // Where is the Update method?

    // Delete an Entity by its id
    let success: spacetimedsl::DeletionResult = dsl.delete_entity_by_id(&entity)?; // Result<spacetimedsl::DeletionResult, spacetimedsl::SpacetimeDSLError>
    let success: spacetimedsl::DeletionResult = dsl.delete_entity_by_id(&entity_id)?; // Result<spacetimedsl::DeletionResult, spacetimedsl::SpacetimeDSLError>

    Ok(())
}
