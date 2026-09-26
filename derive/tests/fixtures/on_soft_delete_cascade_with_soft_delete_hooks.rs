//! Covers the `before_soft_delete` and `after_soft_delete` hooks of a referencing table
//! inside an `on_soft_delete = SoftDelete` cascade: the cascade runs them around each
//! retirement it writes, and stops at the first error one of them returns.
//!
//! `Book` is not referenced itself, so the cascade retires its rows one at a time rather
//! than in the bulk shape `on_soft_delete_cascade` pins.

#[spacetimedsl::dsl(
    plural_name = authors,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(accessor = author, public)]
pub struct Author {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(AuthorId)]
    #[referenced_by(path = self, table = book)]
    id: u64,

    pub name: String,

    deleted: bool,
}

#[spacetimedsl::dsl(
    plural_name = books,
    method(update = true, delete = true, soft_delete = true),
    hook(before(soft_delete), after(soft_delete)),
)]
#[spacetimedb::table(accessor = book, public)]
pub struct Book {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(AuthorId)]
    #[foreign_key(path = self, table = author, column = id, on_delete = Delete, on_soft_delete = SoftDelete)]
    pub author_id: u64,

    deleted: bool,
}
