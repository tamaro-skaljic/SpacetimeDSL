//! Covers a foreign key column whose only index is declared in `#[table]`, under the
//! accessor `by_shelf` rather than the column's name. The cascade finds the referencing
//! rows through that accessor.

#[spacetimedsl::dsl(plural_name = shelves, method(update = false, delete = true))]
#[spacetimedb::table(accessor = shelf, public)]
pub struct Shelf {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ShelfId)]
    #[referenced_by(path = self, table = shelved_book)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = shelved_books, method(update = false, delete = true))]
#[spacetimedb::table(
    accessor = shelved_book,
    index(accessor = by_shelf, btree(columns = [shelf_id])),
    public
)]
pub struct ShelvedBook {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[use_wrapper(ShelfId)]
    #[foreign_key(path = self, table = shelf, column = id, on_delete = Delete)]
    shelf_id: u64,
}
