pub mod math;

use math::DbVector2;
use spacetimedb::SpacetimeType;
use spacetimedb::rand::Rng;
use spacetimedb::{Identity, ReducerContext, TimeDuration, Timestamp, spacetimedb_lib::ScheduleAt};
use spacetimedsl::itertools::Itertools;
use spacetimedsl::{DSL, DSLContext, SpacetimeDSLError, Wrapper, dsl, hook};
use std::{collections::HashMap, time::Duration};

// TODO:
// - [x] Remove players when they are eaten on the client + death + respawn screen
// - [x] Player splitting + increased area of view
// - [x] Overlap amount should be more significant in order to eat
// - [ ] Viruses
// - [ ] Ejecting mass
// - [ ] Leaderboard

const START_PLAYER_MASS: i32 = 15;
const START_PLAYER_SPEED: i32 = 10;
const FOOD_MASS_MIN: i32 = 2;
const FOOD_MASS_MAX: i32 = 4;
const TARGET_FOOD_COUNT: usize = 600;
const MINIMUM_SAFE_MASS_RATIO: f32 = 0.85;

const MIN_MASS_TO_SPLIT: i32 = START_PLAYER_MASS * 2;
const MAX_CIRCLES_PER_PLAYER: i32 = 16;
const SPLIT_RECOMBINE_DELAY_SEC: f32 = 5.0;
const SPLIT_GRAVITY_PULL_BEFORE_RECOMBINE_SEC: f32 = 2.0;
const ALLOWED_SPLIT_CIRCLE_OVERLAP_PCT: f32 = 0.9;
const SELF_COLLISION_SPEED: f32 = 0.05; //1 == instantly separate circles. less means separation takes time

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum LoginStatus {
    LoggedIn,
    LoggedOut,
}

#[dsl(plural_name = config, method(update = false))]
#[spacetimedb::table(name = config, public)]
pub struct Config {
    #[primary_key]
    #[create_wrapper]
    id: i32,
    world_size: i64,
}

#[dsl(plural_name = entities, method(update = true))]
#[spacetimedb::table(name = entity, public)]
pub struct Entity {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = circle)]
    #[referenced_by(path = self, table = food)]
    #[referenced_by(path = self, table = consume_entity_timer)]
    id: i32,
    pub position: DbVector2, // FIXME (find out where used in dsl macro): binary operation `==` cannot be applied to type `DbVector2` consider annotating `DbVector2` with `#[derive(PartialEq)]`
    pub mass: i32,
    #[index(btree)]
    pub login_status: LoginStatus, // FIXME (find out where used in dsl macro): binary operation `==` cannot be applied to type `LoginStatus` consider annotating `LoginStatus` with `#[derive(PartialEq)]`
}

#[dsl(plural_name = circles, method(update = true))]
#[spacetimedb::table(name = circle, public)]
pub struct Circle {
    #[primary_key]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]
    entity_id: i32,
    #[index(btree)]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id, on_delete = Delete)]
    pub player_id: i32,
    pub direction: DbVector2,
    pub speed: f32,
    pub last_split_time: Timestamp,
    #[index(btree)]
    pub login_status: LoginStatus,
}

// FIXME: update = true should not have been valid here because all fields were private and no modified_at / updated_at column existed
#[dsl(plural_name = players, method(update = true), hook(after(update)))]
#[spacetimedb::table(name = player, public)]
pub struct Player {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = circle)]
    #[referenced_by(path = self, table = circle_recombine_timer)]
    id: i32,
    #[unique]
    identity: Identity,
    pub name: String,
    #[index(btree)]
    pub login_status: LoginStatus,
}

#[dsl(plural_name = food, method(update = false))]
#[spacetimedb::table(name = food, public)]
pub struct Food {
    #[primary_key]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]
    entity_id: i32,
}

#[dsl(plural_name = move_all_players_timers, method(update = false))]
#[spacetimedb::table(name = move_all_players_timer, scheduled(move_all_players))]
pub struct MoveAllPlayersTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[dsl(plural_name = spawn_food_timers, method(update = false))]
#[spacetimedb::table(name = spawn_food_timer, scheduled(spawn_food))]
pub struct SpawnFoodTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[dsl(plural_name = circle_decay_timers, method(update = false))]
#[spacetimedb::table(name = circle_decay_timer, scheduled(circle_decay))]
pub struct CircleDecayTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[dsl(plural_name = circle_recombine_timers, method(update = false))]
#[spacetimedb::table(name = circle_recombine_timer, scheduled(circle_recombine))]
pub struct CircleRecombineTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
    #[index(btree)]
    #[use_wrapper(PlayerId)]
    #[foreign_key(path = self, table = player, column = id, on_delete = Delete)]
    player_id: i32,
}

#[dsl(plural_name = consume_entity_timers, method(update = false))]
#[spacetimedb::table(name = consume_entity_timer, scheduled(consume_entity))]
pub struct ConsumeEntityTimer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
    #[index(btree)]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]
    consumed_entity_id: i32,
    #[index(btree)]
    #[use_wrapper(EntityId)]
    #[foreign_key(path = self, table = entity, column = id, on_delete = Delete)]
    consumer_entity_id: i32,
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), spacetimedsl::SpacetimeDSLError> {
    let dsl = dsl(ctx);

    log::info!("Initializing...");

    dsl.create_config(CreateConfig {
        id: 0,
        world_size: 1000,
    })?;

    dsl.create_circle_decay_timer(CreateCircleDecayTimer {
        scheduled_at: ScheduleAt::Interval(Duration::from_secs(5).into()),
    })?;

    dsl.create_spawn_food_timer(CreateSpawnFoodTimer {
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(500).into()),
    })?;

    dsl.create_move_all_players_timer(CreateMoveAllPlayersTimer {
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(50).into()),
    })?;

    Ok(())
}

#[spacetimedb::reducer(client_connected)]
pub fn connect(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    match dsl.get_player_by_identity(&dsl.ctx().sender) {
        Err(error) => match error {
            spacetimedsl::SpacetimeDSLError::NotFoundError {
                table_name: _,
                column_names_and_row_values: _,
            } => {
                dsl.create_player(CreatePlayer {
                    identity: dsl.ctx().sender,
                    name: String::new(),
                    login_status: LoginStatus::LoggedIn,
                })?;
            }
            e => return Err(e),
        },
        Ok(mut player) => {
            match player.get_login_status() {
                LoginStatus::LoggedIn => {
                    return Err(SpacetimeDSLError::Error(format!(
                        "Player (ID: {}, Identity: {}, Name: {}) is already logged in!",
                        player.get_id(),
                        player.get_identity(),
                        player.get_name()
                    )));
                }
                LoginStatus::LoggedOut => {
                    player.set_login_status(LoginStatus::LoggedIn);
                    // Any circle's and entity's login status is automatically updated via the after_player_update hook - see below.
                    dsl.update_player_by_id(player)?;
                }
            }
        }
    };

    Ok(())
}

#[hook]
fn after_player_update(
    dsl: &DSL<'_, T>,
    old_player: &Player,
    new_player: &Player,
) -> Result<(), SpacetimeDSLError> {
    if old_player.get_login_status() != new_player.get_login_status() {
        for mut circle in dsl.get_circles_by_player_id(new_player) {
            circle.set_login_status(new_player.get_login_status().clone());

            circle = dsl.update_circle_by_entity_id(circle)?;

            let mut entity = dsl.get_entity_by_id(circle.get_entity_id())?;

            entity.set_login_status(new_player.get_login_status().clone());

            dsl.update_entity_by_id(entity)?;
        }
    }
    Ok(())
}

#[spacetimedb::reducer(client_disconnected)]
pub fn disconnect(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let mut player = dsl.get_player_by_identity(&ctx.sender)?;

    match player.get_login_status() {
        LoginStatus::LoggedIn => {
            player.set_login_status(LoginStatus::LoggedOut);
            // Any circle's and entity's login status is automatically updated via the after_player_update hook - see above.
            dsl.update_player_by_id(player)?;
        }
        LoginStatus::LoggedOut => {
            return Err(SpacetimeDSLError::Error(format!(
                "Player (ID: {}, Identity: {}, Name: {}) is not logged in!",
                player.get_id(),
                player.get_identity(),
                player.get_name()
            )));
        }
    };

    Ok(())
}

#[spacetimedb::reducer]
pub fn enter_game(ctx: &ReducerContext, name: String) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    log::info!("Creating player with name {}", name);

    let mut player = dsl.get_player_by_identity(&dsl.ctx().sender)?;

    player.set_name(name);

    player = dsl.update_player_by_id(player)?;

    spawn_player_initial_circle(&dsl, &player)?;

    Ok(())
}

fn spawn_player_initial_circle(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
) -> Result<Entity, SpacetimeDSLError> {
    let mut rng = dsl.ctx().rng();

    let world_size = get_world_size(dsl)?;

    let player_start_radius = mass_to_radius(START_PLAYER_MASS);

    let x = rng.gen_range(player_start_radius..(world_size as f32 - player_start_radius));
    let y = rng.gen_range(player_start_radius..(world_size as f32 - player_start_radius));

    let entity = spawn_circle_at(
        dsl,
        player,
        START_PLAYER_MASS,
        DbVector2 { x, y },
        dsl.ctx().timestamp,
    )?;

    Ok(entity)
}

fn get_world_size(dsl: &DSL<'_, ReducerContext>) -> Result<i64, SpacetimeDSLError> {
    Ok(dsl.get_config_by_id(ConfigId::new(0))?.world_size)
}

fn spawn_circle_at(
    dsl: &DSL<'_, ReducerContext>,
    player: &Player,
    mass: i32,
    position: DbVector2,
    last_split_time: Timestamp,
) -> Result<Entity, SpacetimeDSLError> {
    let entity = dsl.create_entity(CreateEntity {
        position,
        mass,
        login_status: player.get_login_status().clone(),
    })?;

    dsl.create_circle(CreateCircle {
        entity_id: entity.get_id(),
        player_id: player.get_id(),
        direction: DbVector2 { x: 0.0, y: 1.0 },
        speed: 0.0,
        last_split_time,
        login_status: player.get_login_status().clone(),
    })?;

    Ok(entity)
}

#[spacetimedb::reducer]
pub fn respawn(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let player = dsl.get_player_by_identity(&dsl.ctx().sender)?;

    spawn_player_initial_circle(&dsl, &player)?;

    Ok(())
}

#[spacetimedb::reducer]
pub fn suicide(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let player = dsl.get_player_by_identity(&dsl.ctx().sender)?;

    for circle in dsl.get_circles_by_player_id(&player) {
        // food and circle are automatically deleted by on delete strategies of foreign keys referencing the entity table's primary key
        dsl.delete_entity_by_id(circle.get_entity_id())?;
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn update_player_input(
    ctx: &ReducerContext,
    direction: DbVector2,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let player = dsl.get_player_by_identity(&ctx.sender)?;

    for mut circle in dsl.get_circles_by_player_id(&player) {
        circle.set_direction(direction.normalized());
        circle.set_speed(direction.magnitude().clamp(0.0, 1.0));

        dsl.update_circle_by_entity_id(circle)?;
    }

    Ok(())
}

fn is_overlapping(first: &Entity, second: &Entity) -> bool {
    let dx = first.get_position().x - second.get_position().x;
    let dy = first.get_position().y - second.get_position().y;
    let squared_distance = dx * dx + dy * dy;

    let first_radius = mass_to_radius(*first.get_mass());
    let second_radius = mass_to_radius(*second.get_mass());

    // If the distance between the two circle centers is less than the
    // maximum radius, then the center of the smaller circle is inside
    // the larger circle. This gives some leeway for the circles to overlap
    // before being eaten.
    let max_radius = f32::max(first_radius, second_radius);
    squared_distance <= max_radius * max_radius
}

fn mass_to_radius(mass: i32) -> f32 {
    (mass as f32).sqrt()
}

fn mass_to_max_move_speed(mass: i32) -> f32 {
    2.0 * START_PLAYER_SPEED as f32 / (1.0 + (mass as f32 / START_PLAYER_MASS as f32).sqrt())
}

#[spacetimedb::reducer]
pub fn move_all_players(
    ctx: &ReducerContext,
    _timer: MoveAllPlayersTimer,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    // TODO identity check
    // let span = spacetimedb::log_stopwatch::LogStopwatch::new("tick");
    let world_size = get_world_size(&dsl)?;

    let circle_by_entity_id: HashMap<EntityId, Circle> = dsl
        .get_circles_by_login_status(&LoginStatus::LoggedIn)
        .map(|c| (c.get_entity_id(), c))
        .collect();

    let mut entity_by_id: HashMap<EntityId, Entity> = dsl
        .get_entities_by_login_status(&LoginStatus::LoggedIn)
        .map(|e| (e.get_id(), e))
        .collect();

    let mut direction_by_entity_id: HashMap<EntityId, DbVector2> = circle_by_entity_id
        .values()
        .map(|c| (c.get_entity_id(), c.direction * c.speed))
        .collect();

    let circles_by_player_id =
        circle_by_entity_id
            .values()
            .fold(HashMap::new(), |mut accumulator, circle| {
                accumulator
                    .entry(circle.get_player_id())
                    .or_insert_with(Vec::new)
                    .push(circle);
                accumulator
            });

    // Split circle movement
    for (_, circles) in circles_by_player_id {
        let count = circles.len();

        if count <= 1 {
            continue;
        }

        let mut entities: Vec<Entity> = Vec::with_capacity(count);

        for circle in &circles {
            entities.push(entity_by_id.remove(&circle.get_entity_id()).unwrap());
        }

        // Gravitate circles towards other circles before they recombine
        for (i, circle_i) in circles.iter().enumerate().take(count) {
            let time_since_split = ctx
                .timestamp
                .duration_since(*circle_i.get_last_split_time())
                .unwrap()
                .as_secs_f32();
            let time_before_recombining = (SPLIT_RECOMBINE_DELAY_SEC - time_since_split).max(0.0);
            if time_before_recombining > SPLIT_GRAVITY_PULL_BEFORE_RECOMBINE_SEC {
                continue;
            }

            let (slice1, slice_i) = entities.split_at_mut(i);
            let (slice_i, slice2) = slice_i.split_at_mut(1);
            let entity_i = &mut slice_i[0];

            for entity_j in slice1.iter().chain(slice2.iter()) {
                let mut diff = *entity_i.get_position() - entity_j.get_position();
                let mut squared_distance = diff.sqr_magnitude();
                if squared_distance <= 0.0001 {
                    diff = DbVector2::new(1.0, 0.0);
                    squared_distance = 1.0;
                }
                let radius_sum = mass_to_radius(entity_i.mass) + mass_to_radius(entity_j.mass);
                if squared_distance > radius_sum * radius_sum {
                    let gravity_multiplier =
                        1.0 - time_before_recombining / SPLIT_GRAVITY_PULL_BEFORE_RECOMBINE_SEC;
                    let vec = diff.normalized()
                        * (radius_sum - squared_distance.sqrt())
                        * gravity_multiplier
                        * 0.05
                        / count as f32;
                    *direction_by_entity_id.get_mut(&entity_i.get_id()).unwrap() += vec / 2.0;
                    *direction_by_entity_id.get_mut(&entity_j.get_id()).unwrap() -= vec / 2.0;
                }
            }

            // Force circles apart
            for i in 0..count {
                let (slice1, slice2) = entities.split_at_mut(i + 1);
                let entity_i = &mut slice1[i];
                for entity_j in slice2 {
                    let mut diff = *entity_i.get_position() - entity_j.get_position();
                    let mut squared_distance = diff.sqr_magnitude();
                    if squared_distance <= 0.0001 {
                        diff = DbVector2::new(1.0, 0.0);
                        squared_distance = 1.0;
                    }
                    let radius_sum = mass_to_radius(entity_i.mass) + mass_to_radius(entity_j.mass);
                    let radius_sum_multiplied = radius_sum * ALLOWED_SPLIT_CIRCLE_OVERLAP_PCT;
                    if squared_distance < radius_sum_multiplied * radius_sum_multiplied {
                        let vec = diff.normalized()
                            * (radius_sum - squared_distance.sqrt())
                            * SELF_COLLISION_SPEED;
                        *direction_by_entity_id.get_mut(&entity_i.get_id()).unwrap() += vec / 2.0;
                        *direction_by_entity_id.get_mut(&entity_j.get_id()).unwrap() -= vec / 2.0;
                    }
                }
            }
        }

        for entity in entities {
            entity_by_id.insert(entity.get_id(), entity);
        }
    }

    // Handle player input
    for circle in circle_by_entity_id.values() {
        let mut entity = match entity_by_id.remove(&circle.get_entity_id()) {
            None => {
                // FIXME: What does that mean for the foreign key relationship between circles and entities?
                // This can happen if a circle is eaten by another circle
                continue;
            }
            Some(entity) => entity,
        };

        let circle_radius = mass_to_radius(entity.mass);
        let direction = *direction_by_entity_id.get(&circle.get_entity_id()).unwrap();
        let new_pos = *entity.get_position() + direction * mass_to_max_move_speed(entity.mass);
        let min = circle_radius;
        let max = world_size as f32 - circle_radius;

        let mut position = *entity.get_position();
        position.x = new_pos.x.clamp(min, max);
        position.y = new_pos.y.clamp(min, max);
        entity.set_position(position);

        let entity = dsl.update_entity_by_id(entity)?;
        entity_by_id.insert(entity.get_id(), entity);
    }

    // Check collisions
    for circle in circle_by_entity_id.values() {
        // let span = spacetimedb::time_span::Span::start("collisions");
        let circle_entity = entity_by_id.get(&circle.get_entity_id()).unwrap();
        for other_entity in entity_by_id.values() {
            if other_entity.get_id() == circle_entity.get_id() {
                continue;
            }

            if is_overlapping(circle_entity, other_entity) {
                let other_circle: Option<&Circle> = circle_by_entity_id.get(&other_entity.get_id());
                if let Some(other_circle) = other_circle {
                    if other_circle.get_player_id() != circle.get_player_id() {
                        let mass_ratio =
                            *other_entity.get_mass() as f32 / *circle_entity.get_mass() as f32;
                        if mass_ratio < MINIMUM_SAFE_MASS_RATIO {
                            schedule_consume_entity(
                                &dsl,
                                circle_entity.get_id(),
                                other_entity.get_id(),
                            )?;
                        }
                    }
                } else {
                    schedule_consume_entity(&dsl, circle_entity.get_id(), other_entity.get_id())?;
                }
            }
        }
        // span.end();
    }

    // span.end();
    Ok(())
}

fn schedule_consume_entity(
    dsl: &DSL<'_, ReducerContext>,
    consumer_entity_id: EntityId,
    consumed_entity_id: EntityId,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_consume_entity_timer(CreateConsumeEntityTimer {
        scheduled_at: ScheduleAt::Time(dsl.ctx().timestamp),
        consumer_entity_id,
        consumed_entity_id,
    })?;

    Ok(())
}

#[spacetimedb::reducer]
pub fn consume_entity(
    ctx: &ReducerContext,
    request: ConsumeEntityTimer,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let consumed_entity = dsl.get_entity_by_id(request.get_consumed_entity_id())?;
    // food and circle are automatically deleted by on delete strategies of foreign keys referencing the entity table's primary key
    dsl.delete_entity_by_id(&consumed_entity)?;

    let mut consumer_entity = dsl.get_entity_by_id(request.get_consumer_entity_id())?;

    consumer_entity.set_mass(*consumer_entity.get_mass() + consumed_entity.get_mass());
    dsl.update_entity_by_id(consumer_entity)?;

    Ok(())
}

#[spacetimedb::reducer]
pub fn player_split(ctx: &ReducerContext) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    let player = dsl.get_player_by_identity(&dsl.ctx().sender)?;

    let circles = dsl.get_circles_by_player_id(&player).collect_vec();

    let mut circle_count = circles.len() as i32;

    if circle_count >= MAX_CIRCLES_PER_PLAYER {
        return Ok(());
    }

    for mut circle in circles {
        let mut entity = dsl.get_entity_by_id(circle.get_entity_id())?;

        if *entity.get_mass() >= MIN_MASS_TO_SPLIT * 2 {
            let half_mass = entity.get_mass() / 2;
            spawn_circle_at(
                &dsl,
                &player,
                half_mass,
                *entity.get_position() + circle.get_direction(),
                dsl.ctx().timestamp,
            )?;
            entity.set_mass(*entity.get_mass() - half_mass);
            circle.set_last_split_time(dsl.ctx().timestamp);
            dsl.update_circle_by_entity_id(circle)?;
            dsl.update_entity_by_id(entity)?;
            circle_count += 1;
            if circle_count >= MAX_CIRCLES_PER_PLAYER {
                break;
            }
        }
    }

    dsl.create_circle_recombine_timer(CreateCircleRecombineTimer {
        scheduled_at: ScheduleAt::Time(
            dsl.ctx().timestamp
                + TimeDuration::from(Duration::from_secs_f32(SPLIT_RECOMBINE_DELAY_SEC)),
        ),
        player_id: player.get_id(),
    })?;

    log::warn!("Player split!");

    Ok(())
}

#[spacetimedb::reducer]
pub fn spawn_food(ctx: &ReducerContext, _timer: SpawnFoodTimer) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    if dsl
        .get_players_by_login_status(&LoginStatus::LoggedIn)
        .count()
        == 0
    {
        //Are there no players yet?
        return Ok(());
    }

    let world_size = get_world_size(&dsl)?;

    let mut rng = ctx.rng();
    let mut food_count = dsl.count_of_all_food();
    while food_count < TARGET_FOOD_COUNT as u64 {
        let food_mass = rng.gen_range(FOOD_MASS_MIN..FOOD_MASS_MAX);
        let food_radius = mass_to_radius(food_mass);
        let x = rng.gen_range(food_radius..world_size as f32 - food_radius);
        let y = rng.gen_range(food_radius..world_size as f32 - food_radius);

        let entity = dsl.create_entity(CreateEntity {
            position: DbVector2 { x, y },
            mass: food_mass,
            login_status: LoginStatus::LoggedIn, // FIXME: This makes no sense for food
        })?;

        dsl.create_food(CreateFood {
            entity_id: entity.get_id(),
        })?;

        food_count += 1;
        log::info!("Spawned food! {}", entity.get_id());
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn circle_decay(
    ctx: &ReducerContext,
    _timer: CircleDecayTimer,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);

    for circle in dsl.get_circles_by_login_status(&LoginStatus::LoggedIn) {
        let mut entity = dsl.get_entity_by_id(circle.get_entity_id())?;
        if *entity.get_mass() <= START_PLAYER_MASS {
            continue;
        }
        entity.set_mass((*entity.get_mass() as f32 * 0.99) as i32);
        dsl.update_entity_by_id(entity)?;
    }

    Ok(())
}

pub fn calculate_center_of_mass(entities: &[Entity]) -> DbVector2 {
    let total_mass: i32 = entities.iter().map(|e| e.get_mass()).sum();
    let center_of_mass: DbVector2 = entities
        .iter()
        .map(|e| *e.get_position() * *e.get_mass() as f32)
        .sum();
    center_of_mass / total_mass as f32
}

#[spacetimedb::reducer]
pub fn circle_recombine(
    ctx: &ReducerContext,
    timer: CircleRecombineTimer,
) -> Result<(), SpacetimeDSLError> {
    let dsl = dsl(ctx);
    let circles = dsl
        .get_circles_by_player_id(timer.get_player_id())
        .collect_vec();

    let mut recombining_entities = vec![];
    for circle in circles {
        if ctx
            .timestamp
            .duration_since(*circle.get_last_split_time())
            .unwrap()
            .as_secs_f32()
            >= SPLIT_RECOMBINE_DELAY_SEC
        {
            recombining_entities.push(dsl.get_entity_by_id(circle.get_entity_id())?);
        }
    }

    if recombining_entities.len() <= 1 {
        return Ok(()); // No circles to recombine
    }

    let consumer_entity_id = recombining_entities[0].get_id();
    for consumed_entity in recombining_entities.iter().skip(1) {
        schedule_consume_entity(&dsl, consumer_entity_id.clone(), consumed_entity.get_id())?;
    }

    Ok(())
}
