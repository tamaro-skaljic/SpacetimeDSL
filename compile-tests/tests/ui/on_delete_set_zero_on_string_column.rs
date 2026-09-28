//! `SetZero` resets a foreign key to the value that references no row: `0` for an unsigned
//! integer, `Uuid::NIL` for a `Uuid`. A `String` has no such value.

::spacetimedsl::spacetimedsl!();

pub mod owner {
    #[spacetimedsl::dsl(plural_name = owners, method(update = true))]
    #[spacetimedb::table(accessor = owner, public)]
    pub struct Owner {
        #[primary_key]
        #[create_wrapper]
        #[referenced_by(path = crate::gadget, table = gadget)]
        name: String,

        pub title: String,
    }
}

pub mod gadget {
    #[spacetimedsl::dsl(plural_name = gadgets, method(update = true))]
    #[spacetimedb::table(accessor = gadget, public)]
    pub struct Gadget {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::owner::OwnerName)]
        #[foreign_key(path = crate::owner, table = owner, column = name, on_delete = SetZero)]
        pub owner_name: String,
    }
}

fn main() {}
