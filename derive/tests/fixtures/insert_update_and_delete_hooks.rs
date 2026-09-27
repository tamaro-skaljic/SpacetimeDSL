//! Covers before and after hooks for insert, update and delete.

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
