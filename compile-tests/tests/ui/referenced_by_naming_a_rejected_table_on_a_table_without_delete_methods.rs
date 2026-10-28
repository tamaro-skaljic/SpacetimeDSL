//! A rejected `gadget` table is named by the `#[referenced_by]` of `owner`, which neither
//! deletes nor soft-deletes rows. The attribute reports that `gadget` has to be fixed first,
//! and nothing else.
//!
//! The error after the first is that one, on the `#[referenced_by]`.

::spacetimedsl::spacetimedsl!();

pub mod owner {
    #[spacetimedsl::dsl(plural_name = owners, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = owner, public)]
    pub struct Owner {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::gadget, table = gadget)]
        id: u64,
    }
}

pub mod gadget {
    #[spacetimedsl::dsl(method(update = false, delete = true))]
    #[spacetimedb::table(accessor = gadget, public)]
    pub struct Gadget {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::owner::OwnerId)]
        #[foreign_key(path = crate::owner, table = owner, column = id)]
        owner_id: u64,
    }
}

fn main() {}
