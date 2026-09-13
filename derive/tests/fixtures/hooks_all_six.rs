//! Covers all six hooks - before and after each of insert, update and delete - so the
//! six hook traits and the six call sites woven into the generated methods are pinned
//! together.

#[spacetimedsl::dsl(
    plural_name = potions,
    method(update = true),
    hook(
        before(insert, update, delete),
        after(insert, update, delete),
    ),
)]
#[spacetimedb::table(accessor = potion, public)]
pub struct Potion {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    pub name: String,
}
