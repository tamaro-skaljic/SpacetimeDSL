//! A foreign key to the `String` primary key of a rejected `country` table, whose lookup takes
//! a `&str`. It reports that `country` has to be fixed first, and nothing else.
//!
//! The error after the first is that one, on the `#[foreign_key]`.

::spacetimedsl::spacetimedsl!();

pub mod country {
    #[spacetimedsl::dsl(method(update = false, delete = true))]
    #[spacetimedb::table(accessor = country, public)]
    pub struct Country {
        #[primary_key]
        #[create_wrapper]
        #[referenced_by(path = crate::city, table = city)]
        code: String,
    }
}

pub mod city {
    #[spacetimedsl::dsl(plural_name = cities, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = city, public)]
    pub struct City {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::country::CountryCode)]
        #[foreign_key(path = crate::country, table = country, column = code, on_delete = Delete)]
        country_code: String,
    }
}

fn main() {}
