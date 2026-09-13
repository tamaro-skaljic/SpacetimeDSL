//! Covers the `String` classification path: `String` columns are passed as `&str` and
//! read by reference, unlike the `Copy` columns every other index fixture uses. Both the
//! unique and the non-unique index shape are included, because they shape their arguments
//! separately.

#[spacetimedsl::dsl(plural_name = articles, method(update = true))]
#[spacetimedb::table(accessor = article, public)]
pub struct Article {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[create_wrapper]
    #[unique]
    pub slug: String,

    #[index(btree)]
    pub category: String,

    #[create_wrapper]
    #[index(btree)]
    pub tags: Option<String>,
}
