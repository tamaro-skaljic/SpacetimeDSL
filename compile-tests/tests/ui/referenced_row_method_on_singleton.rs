//! A singleton's injected primary key has no wrapper type, so a singleton adds no method for
//! the referenced row, and `referenced_row_method` would switch off nothing.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so the
//! `region` table's expansion misses the trait the `server_binding` table would have declared
//! for it.

::spacetimedsl::spacetimedsl!();

pub mod region {
    #[spacetimedsl::dsl(plural_name = regions, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = region, public)]
    pub struct Region {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(RegionId)]
        #[referenced_by(path = crate::binding, table = server_binding)]
        id: u64,
    }
}

pub mod binding {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = server_binding, public)]
    pub struct ServerBinding {
        #[use_wrapper(crate::region::RegionId)]
        #[foreign_key(path = crate::region, table = region, column = id, referenced_row_method = false)]
        pub region_id: u64,
    }
}

fn main() {}
