//! Covers `on_delete = SetZero` on a `Uuid` foreign key: deleting a referenced row resets
//! the referencing column to `Uuid::NIL`, and create and update treat `Uuid::NIL` as
//! referencing no row, the way `0` is treated for an unsigned integer.
//!
//! Identical to `on_delete_set_zero` apart from the key type, so diffing the two shows
//! exactly what `Uuid` changes.

use spacetimedb::Uuid;

#[spacetimedsl::dsl(plural_name = authors, method(update = true))]
#[spacetimedb::table(accessor = author, public)]
pub struct Author {
    #[primary_key]
    #[create_wrapper(AuthorId)]
    #[auto_gen(v7)]
    #[referenced_by(path = self, table = book)]
    id: Uuid,

    pub name: String,
}

#[spacetimedsl::dsl(plural_name = books, method(update = true))]
#[spacetimedb::table(accessor = book, public)]
pub struct Book {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(AuthorId)]
    #[foreign_key(path = self, table = author, column = id, on_delete = SetZero)]
    pub author_id: Uuid,
}
