//! Covers the six hooks of insert, update and delete - before and after each - so their
//! hook traits and the call sites woven into the generated methods are pinned together.
//! The soft-delete hooks are pinned by `soft_delete_hooks`.

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
