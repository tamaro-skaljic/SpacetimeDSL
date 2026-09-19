//! Covers `on_soft_delete`: retiring a referenced row retires the rows which reference it,
//! and those rows' own referencing table is consulted in turn.
//!
//! `Author` is soft-deletable and deletable, so its referencing table sets both fields.
//! `Book` is soft-deletable and is itself referenced by `Review`, which is what makes the
//! cascade recurse.
//!
//! The two markers have different shapes on purpose. `Book`'s is the `Option<Timestamp>`
//! one, and `Book` is the table whose marker a cascade writes, so this pins the one place
//! that cannot reach a timestamp with `?`: a cascade function returns an
//! `OnDeleteStrategyFailure`, not a `SpacetimeDSLError`.

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
)]
#[spacetimedb::table(accessor = book, public)]
pub struct Book {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(BookId)]
    #[referenced_by(path = self, table = review)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(AuthorId)]
    #[foreign_key(path = self, table = author, column = id, on_delete = Delete, on_soft_delete = SoftDelete)]
    pub author_id: u64,

    deleted_at: Option<spacetimedb::Timestamp>,
}

#[spacetimedsl::dsl(plural_name = reviews, method(update = true, delete = true))]
#[spacetimedb::table(accessor = review, public)]
pub struct Review {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(BookId)]
    #[foreign_key(path = self, table = book, column = id, on_delete = Delete, on_soft_delete = Ignore)]
    pub book_id: u64,
}
