//! A foreign key column whose only index is declared in `#[table]`, under an accessor other
//! than the column's name. The cascade has to find the referencing rows through that
//! accessor.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = shelves, method(update = false, delete = true))]
#[spacetimedb::table(accessor = shelf)]
pub struct Shelf {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ShelfId)]
    #[referenced_by(path = crate::table_level_index_foreign_key_test, table = shelved_book)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = shelved_books, method(update = false, delete = true))]
#[spacetimedb::table(
    accessor = shelved_book,
    index(accessor = by_shelf, btree(columns = [shelf_id]))
)]
pub struct ShelvedBook {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[use_wrapper(ShelfId)]
    #[foreign_key(
        path = crate::table_level_index_foreign_key_test,
        table = shelf,
        column = id,
        on_delete = Delete
    )]
    shelf_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let shelf = dsl.create_shelf()?;
    let other_shelf = dsl.create_shelf()?;

    for shelf_id in [shelf.get_id(), shelf.get_id(), other_shelf.get_id()] {
        dsl.create_shelved_book(CreateShelvedBook { shelf_id })?;
    }

    dsl.delete_shelf_by_id(shelf.get_id())
        .map_err(|error| format!("Deleting a shelf should cascade to its books: {error}"))?;

    let books_left = dsl.get_shelved_books_by_by_shelf(shelf.get_id()).count();
    if books_left != 0 {
        return Err(format!(
            "Deleting a shelf should delete its books through the `by_shelf` index, found {books_left} left!"
        ));
    }

    let other_books = dsl
        .get_shelved_books_by_by_shelf(other_shelf.get_id())
        .count();
    if other_books != 1 {
        return Err(format!(
            "Deleting a shelf should leave the books of other shelves, found {other_books} instead of 1!"
        ));
    }

    Ok(())
}
