//! Covers a `String` primary key referenced by `String` foreign keys. A lookup by a `String`
//! key takes a `&str` rather than its wrapper, so every call passing a key along hands over a
//! `&str`, and deleting many referenced rows clones their keys out of the borrowed rows.
//!
//! Both index shapes of the referencing column are included, because their wrapper methods
//! look rows up through different DSL methods.

#[spacetimedsl::dsl(plural_name = countries, method(update = true))]
#[spacetimedb::table(accessor = country, public)]
pub struct Country {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = self, table = city)]
    #[referenced_by(path = self, table = embassy)]
    code: String,

    #[index(btree)]
    pub continent: String,
}

#[spacetimedsl::dsl(plural_name = cities, method(update = true))]
#[spacetimedb::table(accessor = city, public)]
pub struct City {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CountryCode)]
    #[foreign_key(path = self, table = country, column = code, on_delete = Delete)]
    pub country_code: String,
}

#[spacetimedsl::dsl(plural_name = embassies, method(update = true))]
#[spacetimedb::table(accessor = embassy, public)]
pub struct Embassy {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[use_wrapper(CountryCode)]
    #[foreign_key(path = self, table = country, column = code, on_delete = Error)]
    pub country_code: String,
}
