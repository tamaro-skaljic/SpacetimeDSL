//! Covers `on_delete = SetZero`: deleting a referenced row resets the referencing columns
//! to the zero value of their type instead of removing the referencing rows.
//!
//! The four `on_delete_*` fixtures are identical apart from the strategy, so diffing
//! their snapshots against each other shows exactly what the strategy changes.

#[spacetimedsl::dsl(plural_name = authors, method(update = true))]
#[spacetimedb::table(accessor = author, public)]
pub struct Author {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(AuthorId)]
    #[referenced_by(path = self, table = book)]
    id: u64,

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
    pub author_id: u64,
}
