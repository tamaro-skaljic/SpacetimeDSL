# Incremental Migrations

SpacetimeDB does not provide built-in support for general schema-modifying migrations. It does, however, allow adding new tables, and changing reducers' definitions in arbitrary ways. It's possible to run general migrations using an external tool, but this is tedious, necessitates downtime, and imposes the requirement that you update all your clients at the same time as publishing your new module version.

Our friends at [Lightfox Games](https://www.lightfoxgames.com/) taught us a pattern they call "incremental migrations," which mitigates all these problems, and works perfectly with SpacetimeDB's capabilities. The short version is that, instead of altering an existing table, you add a new table with the desired new schema. Whenever your module wants to access a row from that table, it first checks the new table. If the row is present in the new table, then you've already migrated, so do whatever you want to do. If the new table doesn't have the row, instead look it up in the old table, compute and insert a row for the new table, and use that. (If the row isn't present in either the old or new table, it's just not present.) If possible, you should also update the row in the old table to match any mutations that happen in the new table, so that outdated clients can still function.

This has several advantages:

- SpacetimeDB's module hotswapping makes this a zero-downtime update. Write your new module, publish it, and watch the new table populate as it's used.
- It amortizes the cost of transforming rows or computing new columns across many transactions. Rows will only be added to the new table when they're needed.
- In many cases, old clients from before the update can coexist with new clients that use the new table. You can publish the updated module without disconnecting your clients, roll out the client update through normal channels, and allow your users to update at their own pace.

For example, imagine we have a table `player` which stores information about our players:

```
#[spacetimedb::table(accessor = character, public)]
pub struct Character {
    #[primary_key]
    player_id: Identity,
    #[unique]
    nickname: String,
    level: u32,
    class: Class,
}

#[derive(SpacetimeType, Debug, Copy, Clone)]
pub enum Class {
    Fighter,
    Caster,
    Medic,
}
```

We'll write a few helper functions and some simple reducers:

```
#[spacetimedb::reducer]
fn create_character(ctx: &ReducerContext, class: Class, nickname: String) {
    log::info!(
        "Creating new level 1 {class:?} named {nickname}",
    );
    ctx.db.character().insert(Character {
        player_id: ctx.sender(),
        nickname,
        level: 1,
        class,
    });
}

fn find_character_for_player(ctx: &ReducerContext) -> Character {
    ctx.db
        .character()
        .player_id()
        .find(ctx.sender())
        .expect("Player has not created a character")
}

fn update_character(ctx: &ReducerContext, character: Character) {
    ctx.db.character().player_id().update(character);
}

#[spacetimedb::reducer]
fn rename_character(ctx: &ReducerContext, new_name: String) {
    let character = find_character_for_player(ctx);
    log::info!(
        "Renaming {} to {}",
        character.nickname,
        new_name,
    );
    update_character(
        ctx,
        Character {
            nickname: new_name,
            ..character
        },
    );
}

#[spacetimedb::reducer]
fn level_up_character(ctx: &ReducerContext) {
    let character = find_character_for_player(ctx);
    log::info!(
        "Leveling up {} from {} to {}",
        character.nickname,
        character.level,
        character.level + 1,
    );
    update_character(
        ctx,
        Character {
            level: character.level + 1,
            ..character
        },
    );
}
```

Now we want to add a new feature: each player should be able to align themselves with the forces of good or evil, so we can get some healthy competition going between our players. We'll start each character off with `Alliance::Neutral`, and then offer them a reducer `choose_alliance` to set it to either `Alliance::Good` or `Alliance::Evil`. Our first attempt will be to add a new column to the type `Character`:

```
#[spacetimedb::table(accessor = character, public)]
struct Character {
    #[primary_key]
    player_id: Identity,
    nickname: String,
    level: u32,
    class: Class,
    alliance: Alliance,
}

#[derive(SpacetimeType, Debug, Copy, Clone)]
enum Alliance {
    Good,
    Neutral,
    Evil,
}

#[spacetimedb::reducer]
fn choose_alliance(ctx: &ReducerContext, alliance: Alliance) {
    let character = find_character_for_player(ctx);
    log::info!(
        "Setting {}'s alliance to {:?} for player {}",
        character.nickname,
        alliance,
        ctx.sender(),
    );
    update_character(
        ctx,
        Character {
            alliance,
            ..character
        },
    );
}
```

But that will fail, since SpacetimeDB doesn't know how to update our existing `character` rows with the new column:

```
Error: Database update rejected: Errors occurred:
Adding a column alliance to table character requires a manual migration
```

Instead, we'll add a new table, `character_v2`, which will coexist with our original `character` table:

```
#[spacetimedb::table(accessor = character_v2, public)]
struct CharacterV2 {
    #[primary_key]
    player_id: Identity,
    nickname: String,
    level: u32,
    class: Class,
    alliance: Alliance,
}
```

When a new player creates a character, we'll make rows in both tables for them. This way, any old clients that are still subscribing to the original `character` table will continue to work, though of course they won't know about the character's alliance.

```
#[spacetimedb::reducer]
fn create_character(ctx: &ReducerContext, class: Class, nickname: String) {
    log::info!(
        "Creating new level 1 {class:?} named {nickname} for player {}",
        ctx.sender(),
    );

    ctx.db.character().insert(Character {
        player_id: ctx.sender(),
        nickname: nickname.clone(),
        level: 1,
        class,
    });

    ctx.db.character_v2().insert(CharacterV2 {
        player_id: ctx.sender(),
        nickname,
        level: 1,
        class,
        alliance: Alliance::Neutral,
    });
}
```

We'll update our helper functions so that they operate on `character_v2` rows. In `find_character_for_player`, if we don't see the player's row in `character_v2`, we'll migrate it from `character` on the fly. In this case, we'll make the player neutral, since they haven't chosen an alliance yet.

```
fn find_character_for_player(ctx: &ReducerContext) -> CharacterV2 {
    if let Some(character) = ctx.db.character_v2().player_id().find(ctx.sender()) {
        // Already migrated; just return the new player.
        return character;
    }

    // Not yet migrated; look up an old character and update it.
    let old_character = ctx
        .db
        .character()
        .player_id()
        .find(ctx.sender())
        .expect("Player has not created a character");

    ctx.db.character_v2().insert(CharacterV2 {
        player_id: old_character.player_id,
        nickname: old_character.nickname,
        level: old_character.level,
        class: old_character.class,
        alliance: Alliance::Neutral,
    })
}
```

Just like when creating a new character, when we update a `character_v2` row, we'll also update the old `character` row, so that outdated clients can continue to function. It's very important that we perform the same translation between `character` and `character_v2` rows here as in `create_character` and `find_character_for_player`.

```
fn update_character(ctx: &ReducerContext, character: CharacterV2) {
    ctx.db.character().player_id().update(Character {
        player_id: character.player_id,
        nickname: character.nickname.clone(),
        level: character.level,
        class: character.class,
    });
    ctx.db.character_v2().player_id().update(character);
}
```

Then we can make trivial modifications to the callers of `update_character` so that they pass in `CharacterV2` instances:

```
#[spacetimedb::reducer]
fn rename_character(ctx: &ReducerContext, new_name: String) {
    let character = find_character_for_player(ctx);
    log::info!(
        "Renaming {} to {}",
        character.nickname,
        new_name,
    );
    update_character(
        ctx,
        CharacterV2 {
            nickname: new_name,
            ..character
        },
    );
}

#[spacetimedb::reducer]
fn level_up_character(ctx: &ReducerContext) {
    let character = find_character_for_player(ctx);
    log::info!(
        "Leveling up {} from {} to {}",
        character.nickname,
        character.level,
        character.level + 1,
    );
    update_character(
        ctx,
        CharacterV2 {
            level: character.level + 1,
            ..character
        },
    );
}
```

And finally, we can define our new `choose_alliance` reducer:

```
#[spacetimedb::reducer]
fn choose_alliance(ctx: &ReducerContext, alliance: Alliance) {
    let character = find_character_for_player(ctx);
    log::info!(
        "Setting alliance of {} to {:?}",
        character.nickname,
        alliance,
    );
    update_character(
        ctx,
        CharacterV2 {
            alliance,
            ..character
        },
    );
}
```

Now that we know how to define incremental migrations, we can add new features that would seem to require breaking schema changes without cumbersome external migration tools and while maintaining compatibility of outdated clients! The complete for this tutorial is on GitHub in the `clockworklabs/incr-migration-demo` repository, in branches [`v1`](https://github.com/clockworklabs/incr-migration-demo/tree/v1), [`fails-publish`](https://github.com/clockworklabs/incr-migration-demo/tree/fails-publish) and [`v2`](https://github.com/clockworklabs/incr-migration-demo/tree/v2).
